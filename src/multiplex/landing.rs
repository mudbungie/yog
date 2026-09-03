//! **Landing convergence** (DESIGN §16.3, bl-7e54): the repair for a landing
//! yog's world founded *before* balls' config home was nested.
//!
//! bl-e47b nested balls' config home at the `bl` seam ([`super::bl::edge`]), so
//! `Xdg::default_config()` now resolves inside the world and falls through to
//! balls' EMBEDDED default. That fixed **founding**, and founding only: a
//! landing's `config/plugins.toml` is seeded ONCE (`checkout::prime` seeds only
//! when `substrate::is_landing` is false, and `seed::rebind` "never prunes or
//! rewrites the committed schedule"), and balls' own rename convergence cannot
//! reach this damage either — it rewrites a RETIRED name to its current
//! spelling, and here there is no old name left to rewrite, just an absent one.
//! So every landing founded against the operator's stale seed template stayed
//! silently local: no `bl-tracker` at any phase, so the store never fetched and
//! never pushed, and no `show` hook, so `bl show <id>` printed no worktree line.
//!
//! **The reframe that makes this yog's business at all.** balls draws its
//! convergence boundary at the landing on purpose — *"an old name in the XDG
//! layer is the user's file"*. Inside yog's world there IS no user file: yog
//! supplies balls' state home, its config home and its `exe_dir`, so a landing
//! under the world is yog's own generated artifact, exactly like the
//! `world/tools/` shims (§5.2). The shims converge on the way into every verb
//! ([`crate::world::tools::ensure_tools`]); this is the same rule one layer
//! down, for the same reason, and it lands at the same seam.
//!
//! **yog never restates balls' schedule.** The repair re-runs
//! [`balls::seed::seed_landing`] — balls' own embedded default, bound and
//! pruned against the world's tools dir by balls' own rule — so no phase name,
//! plugin name or hook order is ever written here. That is what separates this
//! from the rejected candidate 3 (a second implementation of the seed default
//! living in yog); the single source of truth stays balls'. The correct
//! long-term home is still balls itself — `bl prime` converging a schedule
//! missing first-party entries would repair every box, not just yog's — and
//! that ask is unaffected by this landing.
//!
//! **What the repair preserves.** Only `plugins.toml` is balls' to re-derive.
//! `balls.toml` holds the scalar knobs an operator may have set through `bl
//! conf` (`task-remote`, `log_level`, a landing-scoped `tasks_branch`), so it is
//! read before the re-seed and written back after — the repair restores the
//! capability schedule without spending anybody's configuration.

use std::fs;
use std::io;
use std::path::Path;

use balls::edge::Edge;
use balls::hooks::Hooks;
use balls::message::Message;
use balls::verb::Verb;
use balls::{seed, substrate};

use crate::git_env;
use crate::world::tools;

/// The balls plugin binaries **yog's world provides** as `bl` siblings — the
/// world's own roster fact ([`tools::ROSTER`]), not a reading of balls'
/// schedule. `ensure_tools` seeds both before any verb reaches here, so balls'
/// seed can never prune either one, and a landing that names one of them
/// nowhere was therefore seeded against a template that is not the one this
/// binary carries.
const PROVIDED: [&str; 2] = [tools::BL_DELIVERY, tools::BL_TRACKER];

/// The landing commit's subject when the repair rewrites a schedule.
const SUBJECT: &str = "balls: converge landing schedule";

/// Converge the landing this invocation addresses, returning whether it was
/// repaired. Idempotent, and cheap on the overwhelmingly common converged
/// landing: one `starts_with`, one `rev-parse` and one parse of `plugins.toml`,
/// then out — no re-seed, no git write, no commit.
///
/// Four ways out, in cost order: the landing is not yog's to converge; the clone
/// was never founded (a `prime` is about to seed it correctly, so there is
/// nothing to repair); its schedule already names every plugin the world
/// provides; or it does not, and balls' own seed re-derives it.
///
/// **The containment gate is the reframe's precondition, not a safety belt.**
/// This module may rewrite a committed schedule *because* a landing inside yog's
/// world is yog's own generated artifact — and that is true only of landings
/// inside yog's world. The world env is handed DOWN to a spawn rather than
/// re-composed here (see [`super::bl::edge`]), so a `yog bl` invoked from a
/// shell that never entered the world addresses the operator's **ambient** balls
/// state, where balls' own boundary rules and *"an old name in the XDG layer is
/// the user's file"* is exactly right. Converging there would be yog reaching
/// outside itself to rewrite a file it does not own — the §16.2 severability
/// promise inverted — so a landing that is not under `<yog-data-root>/world` is
/// left alone, however tracker-less it looks.
pub fn converge(edge: &Edge, world: &Path) -> io::Result<bool> {
    let landing = edge.xdg.clone_dir(&edge.invocation_path).landing();
    if !landing.starts_with(world) {
        return Ok(false);
    }
    if !substrate::is_landing(&landing) {
        return Ok(false);
    }
    // Sited like every other step, but it is the one that can be ruled OUT of a
    // `NotFound`: `Hooks::load` answers the empty schedule for an absent
    // `plugins.toml` rather than erring, so an ENOENT reaching the operator
    // came from the re-seed or one of the forks below, never from here.
    let referenced =
        sited("read the landing schedule", &landing, Hooks::load(&landing))?.referenced();
    if PROVIDED.iter().all(|name| referenced.contains_key(*name)) {
        return Ok(false);
    }
    reseed(edge, &landing)?;
    Ok(true)
}

/// Name the site an [`io::Error`] came out of, keeping its `kind` so a caller
/// can still match on it and its own words so nothing is lost.
///
/// Every fallible step of this convergence is a read or a fork against a path,
/// and each of them can answer the *same* bare `NotFound` — which is exactly
/// what a rare macOS failure did answer (bl-1ce0), naming neither the step nor
/// the path, so a one-line warning out of [`report`] was undiagnosable. This
/// takes an already-evaluated `Result` rather than a closure on purpose: one
/// arm, one test, and a site label at each call instead of a per-site error
/// type nobody would match on.
///
/// **The label locates the fork; it does not promise the path is the fault.**
/// A `NotFound` off one of these forks has been the *program* rather than the
/// cwd — a peer thread's returning `exec` freeing the environment this fork's
/// `PATH` was read out of, so `git` itself could not be found while the
/// checkout named here existed the whole time (bl-2f8b; the mechanism and its
/// placement rule live at [`crate::git_env::exec`]). Rule that out before
/// reading the path as the complaint.
fn sited<T>(site: &str, path: &Path, result: io::Result<T>) -> io::Result<T> {
    result.map_err(|e| io::Error::new(e.kind(), format!("{site} ({}): {e}", path.display())))
}

/// Re-derive the landing's capability schedule from balls' embedded default,
/// preserving the scalar config beside it. `seed_landing` writes BOTH files, so
/// `balls.toml` is carried across the call rather than protected from it — one
/// read and one write, no branch on which keys an operator might have set.
fn reseed(edge: &Edge, landing: &Path) -> io::Result<()> {
    let scalars = landing.join("config").join("balls.toml");
    let keep = fs::read(&scalars).ok();
    sited(
        "re-seed the landing",
        landing,
        seed::seed_landing(&edge.xdg, landing, edge.exe_dir.as_deref()),
    )?;
    if let Some(body) = keep {
        sited(
            "restore the scalar config",
            &scalars,
            fs::write(&scalars, body),
        )?;
    }
    commit(landing, &edge.default_actor)
}

/// Seal the rewrite as one ordinary landing commit, in balls' own message
/// shape. Gated on a dirty tree, so a re-seed that reproduced the bytes already
/// there costs a `status` and stops — which is what makes the whole convergence
/// idempotent independently of the [`converge`] gate above.
pub fn commit(landing: &Path, actor: &str) -> io::Result<()> {
    if git(landing, &["status", "--porcelain"])?.trim().is_empty() {
        return Ok(());
    }
    git(landing, &["add", "-A"])?;
    let rendered = Message::checkout(Verb::Prime, actor, SUBJECT.to_owned()).render();
    let message = sited("render the landing commit message", landing, rendered)?;
    git(landing, &["commit", "-q", "-m", &message])?;
    Ok(())
}

/// Report a convergence outcome and carry on. A repair is **announced** — it
/// rewrote a committed file, so it must not be silent — and a failure is a
/// warning, never the verb's exit: the landing was usable enough to reach here,
/// and a repair that cannot run must not take the op it rode in on with it.
pub(super) fn report(outcome: io::Result<bool>) {
    use std::io::Write as _;
    let mut err = io::stderr();
    match outcome {
        Ok(true) => {
            let _ = writeln!(
                err,
                "yog bl: converged this landing's plugin schedule — it was seeded before \
                 balls' config home was nested, so it wired no tracker and no show hook"
            );
        }
        Ok(false) => {}
        Err(e) => {
            let _ = writeln!(err, "yog bl: converge landing: {e}");
        }
    }
}

/// Run one `git` in `cwd`, answering its stdout under a site naming the
/// subcommand — so a failure to *spawn* (an ENOENT off the `PATH` lookup or an
/// absent `cwd`, indistinguishable from a failed read in a bare error) reads
/// the same way as a failure to *succeed*, and both say which of the three
/// forks it was.
pub fn git(cwd: &Path, args: &[&str]) -> io::Result<String> {
    // The subcommand alone, never the whole argument vector: one of these forks
    // carries the rendered commit message, and a site label is for locating a
    // failure, not for reprinting its input.
    let site = format!("git {}", args.first().copied().unwrap_or_default());
    sited(&site, cwd, run(cwd, args))
}

/// The fork itself, through the crate's scrubbing constructor. A non-zero exit
/// becomes the error carrying git's own stderr — the caller reports it and the
/// verb proceeds, since a repair that cannot run must never fail the op it rode
/// in on.
fn run(cwd: &Path, args: &[&str]) -> io::Result<String> {
    let out = git_env::output(git_env::git().current_dir(cwd).args(args))?;
    if !out.status.success() {
        return Err(io::Error::other(
            String::from_utf8_lossy(&out.stderr).into_owned(),
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

#[cfg(test)]
mod tests;
