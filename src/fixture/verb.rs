//! `yog fixture` — lay a named world state and print what a harness needs to
//! dial it.
//!
//! The shape is [`wire-certs`](crate::wire::provision::verb)'s deliberately:
//! the settings are environment readings folded into a pure [`Plan`], the
//! process edge does the reading, and the one thing on argv is the **subject**
//! — a state name, which is what the verb is about rather than how it behaves.
//! Bare, it lists the roster; an unknown name is refused with the roster beside
//! it, because a refusal that does not say what *would* have worked costs a
//! second run.
//!
//! **A lay is destructive and therefore guarded.** Determinism means starting
//! from nothing, so the root is removed before it is written — and a root that
//! overlapped the ambient yog data root would delete the operator's own
//! workspaces, conversations and world without a prompt. The check is the
//! two-directional path-prefix test `scripts/drive/drive.sh` already applies
//! for the same reason: containment either way is the same accident.

use super::places::Places;
use super::{Laid, roster};
use crate::wire::material::{ANCHORS, Role};
use crate::wire::provision::LOOPBACK;
use crate::xdg::Env;
use std::path::PathBuf;

/// This verb's own word.
pub const SUBCMD: &str = "fixture";

/// The three environment readings, named once so the process edge that reads
/// them and the plan that folds them cannot disagree. `WIRE_HOST`/`WIRE_PORT`
/// are `wire-certs`' own spellings on purpose: one address has one vocabulary.
pub const READS: [&str; 3] = ["FIXTURE_ROOT", "WIRE_HOST", "WIRE_PORT"];

/// What one invocation was asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Plan {
    /// No state named: print the roster.
    List,
    /// Lay `state` under `root`, minting material aimed at `address`.
    Lay {
        state: String,
        root: PathBuf,
        address: String,
    },
    /// A refusal folded at plan time, so the decision is testable without a
    /// filesystem: an unknown name, or a root that overlaps the live world.
    Refuse(String),
}

/// Fold a plan from the subject and the three readings. Pure — it reads no
/// disk and no process environment.
pub fn plan(
    ambient: &Env,
    state: Option<String>,
    root: Option<String>,
    host: Option<String>,
    port: Option<String>,
) -> Plan {
    let Some(state) = stated(state) else {
        return Plan::List;
    };
    if roster::resolve(&state).is_none() {
        return Plan::Refuse(format!(
            "yog {SUBCMD}: {state:?} is not a state — try one of: {}",
            roster::names().join(", ")
        ));
    }
    let root = stated(root).map_or_else(|| default_root(ambient, &state), PathBuf::from);
    if let Some(refusal) = overlaps(&root, &ambient.yog_data_root()) {
        return Plan::Refuse(refusal);
    }
    Plan::Lay {
        address: format!(
            "{}:{}",
            stated(host).unwrap_or_else(|| LOOPBACK.to_owned()),
            stated(port).unwrap_or_else(free_port)
        ),
        state,
        root,
    }
}

/// Where a state goes when the caller names nowhere: a stable path under the
/// **cache** root, which is where a throwaway tree belongs and is what
/// `scripts/drive/drive.sh` already defaults its own scratch to.
///
/// Never under the data root — that is the world's anchor, and a scratch tree
/// inside it is one `rm` away from being mistaken for the operator's own (and
/// would be refused by [`overlaps`] anyway). Folded off the injected `Env`
/// rather than off `TMPDIR`, so this stays the pure decision its caller says it
/// is: `Env::from_env` is the crate's one environment read.
fn default_root(ambient: &Env, state: &str) -> PathBuf {
    ambient.yog_cache_root().join("fixture").join(state)
}

/// The live-world refusal. Two directions, because a scratch root containing
/// the world and a world containing the scratch root are the same accident.
fn overlaps(root: &std::path::Path, live: &std::path::Path) -> Option<String> {
    let (a, b) = (with_slash(root), with_slash(live));
    if !a.starts_with(&b) && !b.starts_with(&a) {
        return None;
    }
    Some(format!(
        "yog {SUBCMD}: refusing to lay a fixture over the LIVE world.\n  \
         fixture: {}\n  live:    {}\n  \
         A lay wipes its root before it writes, so the two may never overlap. \
         Point {} somewhere else; the live world is the engine's.",
        root.display(),
        live.display(),
        READS[0]
    ))
}

/// A path as a prefix-comparable string: exactly one trailing separator.
///
/// The separator is what stops `/tmp/yog-a` from reading as a parent of
/// `/tmp/yog-ab`. Trimming first is what stops the **root** from reading as a
/// parent of nothing: `/` would otherwise become `//`, which no real path is a
/// prefix of, so a fixture root of `/` — a root that contains every world there
/// is — passed the guard.
fn with_slash(path: &std::path::Path) -> String {
    format!("{}/", path.display().to_string().trim_end_matches('/'))
}

/// A port nothing is listening on, asked of the kernel and handed straight
/// back. The engine cannot answer this question for us: `127.0.0.1:0` in the
/// material is a *request*, and only the listener ever learns what it became —
/// which is in its RAM and not on any surface a harness can read.
///
/// The syscall and the reading of it are split for the reason the whole crate
/// splits them: a bind that fails is not reproducible on demand, so the arm
/// that answers for it would be code nothing could ever run. [`port_of`] is the
/// decision and takes the answer as a value.
fn free_port() -> String {
    port_of(std::net::TcpListener::bind((LOOPBACK, 0)).and_then(|l| l.local_addr()))
}

/// What a bind answered, as a port. A kernel that would not give one falls back
/// to the stated default — the same port `wire-certs` mints for an operator who
/// names none, so there is one number and not two.
fn port_of(bound: std::io::Result<std::net::SocketAddr>) -> String {
    bound.map_or_else(
        |_| crate::wire::provision::PORT.to_owned(),
        |a| a.port().to_string(),
    )
}

/// One environment reading, as a statement. An empty value is an unset one — a
/// `make` variable that expanded to nothing must not become an empty host.
fn stated(value: Option<String>) -> Option<String> {
    value.filter(|v| !v.is_empty())
}

/// Perform `plan`. Exit `0` acted or listed, `1` refused — and every refusal is
/// its own sentence, on stderr, naming what would have worked.
pub fn perform(plan: &Plan) -> i32 {
    match plan {
        Plan::List => {
            for (name, recipe) in roster::ROSTER {
                println!("{name:<12} {}", recipe.summary);
            }
            0
        }
        Plan::Refuse(sentence) => {
            eprintln!("{sentence}");
            1
        }
        Plan::Lay {
            state,
            root,
            address,
        } => match perform_lay(state, root, address) {
            Ok(laid) => {
                println!("{}", laid.json());
                0
            }
            Err(e) => {
                eprintln!("yog {SUBCMD}: {e}");
                1
            }
        },
    }
}

/// The act: wipe, lay, mint, answer. `pub` because it is the whole of what the
/// verb does and a test should drive it without capturing stdout.
pub fn perform_lay(state: &str, root: &PathBuf, address: &str) -> Result<Laid, String> {
    let recipe = roster::resolve(state).ok_or_else(|| format!("{state:?} is not a state"))?;
    if let Err(e) = std::fs::remove_dir_all(root)
        && e.kind() != std::io::ErrorKind::NotFound
    {
        return Err(format!("clear {}: {e}", root.display()));
    }
    let origin = now_unix();
    let hold = super::lay::lay(root, recipe, origin)?;
    let places = Places::under(root);
    crate::wire::provision::mint(&places.wire, address, &[], false)?;
    let leaf = Role::Client.leaf();
    Ok(Laid {
        state: state.to_owned(),
        root: root.clone(),
        address: address.to_owned(),
        anchors: places.wire.join(ANCHORS),
        chain: places.wire.join(format!("{leaf}.pem")),
        key: places.wire.join(format!("{leaf}.key")),
        origin,
        hold,
    })
}

/// The second every offset in a recipe is measured back from. The one clock
/// reading this module makes, reported in the answer so a harness can compute
/// exactly what the engine will derive.
fn now_unix() -> i64 {
    unix_of(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH))
}

/// A duration since the epoch as a signed second count — split from the clock
/// read above for [`port_of`]'s reason: a clock before the epoch and a clock
/// past `i64` are both real answers and neither is one a test can ask the
/// machine for.
fn unix_of(since: Result<std::time::Duration, std::time::SystemTimeError>) -> i64 {
    since.map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
}

/// The refusal a second word on the command line earns. One state per lay, and
/// every setting is an environment reading — a word here is one of those put in
/// the wrong place, which must refuse rather than vanish into a default.
pub fn stray(tail: &[String]) -> Option<String> {
    let word = tail.get(1)?;
    Some(format!(
        "yog {SUBCMD}: {word:?} is a second word — one state per lay, and {} are \
         environment readings, so they go BEFORE the verb: \
         `{}=<dir> yog {SUBCMD} <state>`",
        READS.join(", "),
        READS[0]
    ))
}

#[cfg(test)]
mod tests;
