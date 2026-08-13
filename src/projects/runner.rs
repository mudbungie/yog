//! The `bl` effect behind [`super::balls`] (DESIGN §5.1 #2/#4, §15 Y14, §16.7 W8).
//!
//! [`BlRunner`] is the one injected effect (arch §14, the LockProbe template):
//! the three ball reads of one project, typed. A fake replays canned balls in
//! tests; [`BlStore`] is the production impl and it is **in-process** — balls is
//! a linked crate (§16.7 W8), so `live`/`detail` load balls' own [`Catalog`]
//! straight off the nested store checkout: no `bl list/show --json` spawn, no
//! re-parse, no `--json` contract in the middle.
//!
//! **The one residual spawn.** `closed` reaches the *dead* set, which balls
//! reconstructs by walking the store's git history — not part of the read
//! surface promoted to linked consumers (`reads::history` stays crate-private).
//! So the closed listing still runs a subprocess, and under §16.7 W12 that
//! subprocess is `yog bl list -s closed --json`: the same pinned implementation,
//! out of process. It is the last JSON hop in yog ([`parse_list`]) and it dies
//! when balls promotes the dead-set walk.
//!
//! `live`/`closed` are fallible: a project whose clone is not founded is
//! **unlistable**, and that error is the §3.5 orphaned-project signal — distinct
//! from an empty listing. [`identity`] resolves the claimed-by-me axis (§4.1).

use super::balls::{Ball, parse_list};
use crate::cli_outbound::{Chunk, Cli, CliError, ExitInfo, Stream};
use balls::layout::Xdg;
use balls::reads::Catalog;
use std::io;
use std::path::{Path, PathBuf};

/// The `bl` view surface: the live set, the closed set, and one ball's detail.
/// Kept object-safe so the §7.2 derivation worker can hold a
/// `Box<dyn BlRunner>` (§15 Y16); `Send` because that worker is a thread and
/// the fetch cadence runs there, never on the frame.
pub trait BlRunner: Send {
    /// The live balls of `project` (§5.1 #2). `Err` ⇒ the project is unlistable
    /// (its clone is not founded) — the §3.5 orphaned-project signal.
    fn live(&self, project: &Path) -> io::Result<Vec<Ball>>;
    /// The closed listing, on demand (§5.1 #4) — never on the fetch cadence.
    fn closed(&self, project: &Path) -> io::Result<Vec<Ball>>;
    /// One live ball's full detail (frontmatter + body) on demand (§5.1 #4);
    /// `None` when the project is unlistable or carries no such live ball.
    fn detail(&self, project: &Path, id: &str) -> Option<Ball>;
}

/// Production [`BlRunner`] (§16.7 W8): balls' own [`Xdg`] layout over the world
/// env — [`Env::balls_layout`](crate::xdg::Env::balls_layout), so the dir yog
/// reads and the dir a spawned `bl` writes are one fact — plus the `bl` [`Cli`]
/// the one residual subprocess (the closed listing) runs on. Construct in the
/// shell with the composed world (§16.6 W2).
pub struct BlStore {
    xdg: Xdg,
    cli: Cli,
}

impl BlStore {
    pub fn new(xdg: Xdg, cli: Cli) -> Self {
        Self { xdg, cli }
    }

    /// The store checkout (`clones/<pct-enc-path>/tasks`) of a **founded**
    /// project, else `Err`. Foundedness is balls' own test — the landing
    /// checkout carries a `config/` folder (balls §2/§12, the probe
    /// `layout::Xdg::nearest_founded_ancestor` uses) — so a clone dir that is
    /// not a balls project reads as unlistable rather than as an empty store
    /// (`Catalog::load` is silent-empty on an absent `tasks/`).
    fn store(&self, project: &Path) -> io::Result<PathBuf> {
        let clone = self.xdg.clone_dir(project);
        if clone.landing().join("config").is_dir() {
            Ok(clone.store())
        } else {
            Err(io::Error::other(format!(
                "no founded balls clone for {}",
                project.display()
            )))
        }
    }

    /// The live catalog of `project`, loaded in-process.
    fn catalog(&self, project: &Path) -> io::Result<Catalog> {
        Catalog::load(&self.store(project)?)
    }
}

/// Drain a `bl` stream to its stdout string, erroring on spawn failure or a
/// non-zero/signalled exit (a forgiving parser turns an odd-but-valid body into
/// an empty set; a failed *process* is a real error worth surfacing).
fn collect_stdout(stream: Result<Stream, CliError>) -> io::Result<String> {
    let stream = stream.map_err(io::Error::other)?;
    let mut out = Vec::new();
    let mut exit = ExitInfo::Unknown;
    for chunk in stream {
        match chunk {
            Chunk::Stdout(b) => out.extend(b),
            Chunk::Stderr(_) => {}
            Chunk::Exited(e) => exit = e,
        }
    }
    match exit {
        ExitInfo::Code(0) => Ok(String::from_utf8_lossy(&out).into_owned()),
        other => Err(io::Error::other(format!("bl exited: {other:?}"))),
    }
}

impl BlRunner for BlStore {
    fn live(&self, project: &Path) -> io::Result<Vec<Ball>> {
        let catalog = self.catalog(project)?;
        Ok(catalog.entries().iter().map(Ball::from).collect())
    }

    /// The dead set, still subprocess-served (see the module doc): `yog bl list
    /// -s closed --json` with cwd = the project (§5.1 #2), re-parsed forgivingly.
    fn closed(&self, project: &Path) -> io::Result<Vec<Ball>> {
        let out = collect_stdout(
            self.cli
                .run_in(project, &["list", "-s", "closed", "--json"]),
        )?;
        Ok(parse_list(&out))
    }

    fn detail(&self, project: &Path, id: &str) -> Option<Ball> {
        self.catalog(project).ok()?.get(id).map(Ball::from)
    }
}

/// The operator's claim identity for the claimed-by-me axis (§4.1): the recorded
/// `identity_last_used`, else the invoking `$USER`, else empty (nothing is
/// "mine" when the identity is unknown — the safe default).
pub fn identity(recorded: Option<String>, user: Option<String>) -> String {
    recorded.or(user).unwrap_or_default()
}

#[cfg(test)]
mod tests;
