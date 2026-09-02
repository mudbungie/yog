//! The `bl` half of the §8.2 verb table: the ball verbs, every one of them
//! `bl <verb> <id> --as <name>` run piped in the **project** directory and
//! logged to `ops.jsonl` like its litany siblings.
//!
//! Split from [`super`](super) per §12's line budget, on the seam the §8.2
//! table already draws: the litany verbs act on a *conversation* in a
//! workspace, these act on a *ball* in a project, and their §7.3 origin is
//! [`Origin::Balls`] to a verb.
//!
//! **§8.2 identity rider (Z4):** every claim/close/unclaim is stamped `--as
//! <workspace name>` — the claimant delivers its own ball (§3.2) — never the
//! operator `$USER`. Close and release stamp the ball's *bound* name; assign
//! stamps the *target* name.

use std::io;
use std::path::Path;

use super::run_logged;
use crate::actions::verbs::Outcome;
use crate::cli_outbound::Cli;
use crate::opslog::Origin;

pub mod edit;
pub mod verb;

pub use verb::Verb;

// bl subcommands (pinned to `bl <verb> --skill`, §8.2).
const CLOSE: &str = "close";
const CLAIM: &str = "claim";
const UNCLAIM: &str = "unclaim";
const CREATE: &str = "create";
const UPDATE: &str = "update";
const AS: &str = "--as";
const TITLE: &str = "--title";
const BODY: &str = "--body";
const NOTE: &str = "-m";

/// `bl close <id> --as <name>` in the project (§8.2): fold/gate/squash; bl's own
/// pre-commit-gate failures ride back verbatim in the returned [`Outcome`] (the
/// claim and the bl-delivery worktree stay up — bl's own semantics). `name` is the ball's
/// **bound workspace name** — the claimant delivers its own ball (§3.2 rider),
/// never the operator `$USER`.
pub fn close(
    bl: &Cli,
    state_root: &Path,
    ts: &str,
    project: &Path,
    id: &str,
    name: &str,
) -> io::Result<Outcome> {
    run_logged(
        bl,
        state_root,
        ts,
        project,
        &[CLOSE, id, AS, name],
        Origin::Balls,
    )
}

/// `bl claim <id> --as <name>` in the project — **assign** a ready ball to a
/// workspace (§8.2/§3.2): the late-mutable binding as a first-class verb, stamped
/// with the *target* workspace name. Distinct from the start flow's claim
/// ([`crate::start::execute_claim`]): assign only binds, it starts no conversation
/// and needs no worktree cross-check.
pub fn assign(
    bl: &Cli,
    state_root: &Path,
    ts: &str,
    project: &Path,
    id: &str,
    name: &str,
) -> io::Result<Outcome> {
    run_logged(
        bl,
        state_root,
        ts,
        project,
        &[CLAIM, id, AS, name],
        Origin::Balls,
    )
}

/// `bl unclaim <id> --as <name>` in the project — **release** (§8.2/§3.2),
/// stamped with the ball's bound workspace name.
pub fn unclaim(
    bl: &Cli,
    state_root: &Path,
    ts: &str,
    project: &Path,
    id: &str,
    name: &str,
) -> io::Result<Outcome> {
    run_logged(
        bl,
        state_root,
        ts,
        project,
        &[UNCLAIM, id, AS, name],
        Origin::Balls,
    )
}

/// `bl create <title> --as <name> [--body B] [fields…]` in the project (§8.2).
/// The new id is [`Outcome::stdout`] (bl prints it there for `id=$(bl create
/// …)`); `name` is the authoring workspace (the start flow passes the resolved
/// or focused name). The payload spells its own argv ([`edit::Create`]).
pub fn create(
    bl: &Cli,
    state_root: &Path,
    ts: &str,
    project: &Path,
    name: &str,
    fields: &edit::Create,
) -> io::Result<Outcome> {
    spend(bl, state_root, ts, project, CREATE, fields.argv(name))
}

/// `bl update <id> --as <name> [--title T] [--body B] [-m NOTE] [fields…]` in
/// the project (§8.2), carrying only what the operator changed.
pub fn update(
    bl: &Cli,
    state_root: &Path,
    ts: &str,
    project: &Path,
    id: &str,
    name: &str,
    fields: &edit::Update,
) -> io::Result<Outcome> {
    let mut argv = vec![id.to_owned()];
    argv.extend(fields.argv(name));
    spend(bl, state_root, ts, project, UPDATE, argv)
}

/// Run `bl <verb> <argv…>`, the one place an owned argv is narrowed to the
/// borrowed slice [`run_logged`] takes.
fn spend(
    bl: &Cli,
    state_root: &Path,
    ts: &str,
    project: &Path,
    verb: &str,
    argv: Vec<String>,
) -> io::Result<Outcome> {
    let mut args = vec![verb];
    args.extend(argv.iter().map(String::as_str));
    run_logged(bl, state_root, ts, project, &args, Origin::Balls)
}
