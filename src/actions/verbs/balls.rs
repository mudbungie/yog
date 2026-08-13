//! The `bl` half of the §8.2 verb table: the ball verbs, every one of them
//! `bl <verb> <id> --as <name>` run piped in the **project** directory and
//! logged to `ops.jsonl` like its lernie siblings.
//!
//! Split from [`super`](super) per §12's line budget, on the seam the §8.2
//! table already draws: the lernie verbs act on a *conversation* in a
//! workspace, these act on a *ball* in a project, and their §7.3 origin is
//! [`Origin::Balls`] to a verb.
//!
//! **§8.2 identity rider (Z4):** every claim/close/unclaim is stamped `--as
//! <workspace name>` — the claimant delivers its own ball (§3.2) — never the
//! operator `$USER`. Close and release stamp the ball's *bound* name; assign
//! and a move's claim the *target* name; a move's unclaim the *source* name.

use std::io;
use std::path::Path;

use super::run_logged;
use crate::actions::verbs::Outcome;
use crate::cli_outbound::Cli;
use crate::opslog::Origin;

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

/// **Move** a ball to another workspace (§8.2/§3.2): `bl unclaim <id> --as <from>`
/// then `bl claim <id> --as <to>`, in the project — the source workspace releases
/// its own ball, the target claims it, both logged (§8.2 "short, piped ×2, both
/// logged"). Returns the claim's [`Outcome`]; a spawn failure of the unclaim
/// aborts before the claim.
pub fn reassign(
    bl: &Cli,
    state_root: &Path,
    ts: &str,
    project: &Path,
    id: &str,
    from: &str,
    to: &str,
) -> io::Result<Outcome> {
    unclaim(bl, state_root, ts, project, id, from)?;
    assign(bl, state_root, ts, project, id, to)
}

/// `bl create <title> --as <name> [--body B]` in the project (§8.2). The new id
/// is [`Outcome::stdout`] (bl prints it there for `id=$(bl create …)`); `name` is
/// the authoring workspace (the start flow passes the minted/focused name).
pub fn create(
    bl: &Cli,
    state_root: &Path,
    ts: &str,
    project: &Path,
    title: &str,
    name: &str,
    body: Option<&str>,
) -> io::Result<Outcome> {
    let mut args = vec![CREATE, title, AS, name];
    if let Some(b) = body {
        args.push(BODY);
        args.push(b);
    }
    run_logged(bl, state_root, ts, project, &args, Origin::Balls)
}

/// The field edits `bl update` carries from the ball editor (§11 ball detail):
/// a retitle, a body rewrite (the living document), and/or a journal note. All
/// optional — an all-`None` update still restamps `updated` (bl's note commit).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Update {
    pub title: Option<String>,
    pub body: Option<String>,
    pub note: Option<String>,
}

impl Update {
    /// The three optional edits, owned. A constructor rather than a literal at
    /// the call site because the boundary's one caller is inside a match arm,
    /// where a five-line literal costs the chokepoint its line budget (§12).
    pub fn of(title: &Option<String>, body: &Option<String>, note: &Option<String>) -> Self {
        Self {
            title: title.clone(),
            body: body.clone(),
            note: note.clone(),
        }
    }
}

/// `bl update <id> --as <name> [--title T] [--body B] [-m NOTE]` in the project
/// (§8.2), carrying only the fields the operator changed.
pub fn update(
    bl: &Cli,
    state_root: &Path,
    ts: &str,
    project: &Path,
    id: &str,
    name: &str,
    fields: &Update,
) -> io::Result<Outcome> {
    let mut args = vec![UPDATE, id, AS, name];
    if let Some(t) = &fields.title {
        args.push(TITLE);
        args.push(t);
    }
    if let Some(b) = &fields.body {
        args.push(BODY);
        args.push(b);
    }
    if let Some(n) = &fields.note {
        args.push(NOTE);
        args.push(n);
    }
    run_logged(bl, state_root, ts, project, &args, Origin::Balls)
}
