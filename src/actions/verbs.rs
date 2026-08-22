//! Short-verb dispatchers + the `ops.jsonl` wiring (DESIGN §8.2, §15 Y16).
//!
//! The action surface's *short, piped* verbs — every one but the detached
//! `lernie prompt` (§8.1, Y17). Per §8.2 each runs synchronously with stdout
//! and stderr piped, then appends its completed outcome to `ops.jsonl`
//! (§4.2) — replacing the legacy "stderr printed and dropped" spawn-and-drain.
//!
//! | verb | argv | cwd | origin |
//! |---|---|---|---|
//! | message | `lernie message <ws> <agent> <text>` | ws | conversation |
//! | stop | `lernie stop <ws> <agent> [--stop-children]` | ws | conversation |
//! | scan | `lernie scan <ws>` | ws | conversation |
//! | retarget | `lernie retarget <ws> <agent>` | ws | conversation |
//! | close | `bl close <id> --as <name>` | project | balls |
//! | assign | `bl claim <id> --as <name>` | project | balls |
//! | release / unclaim | `bl unclaim <id> --as <name>` | project | balls |
//! | create | `bl create <title> --as <name> [--body B]` | project | balls |
//! | update | `bl update <id> --as <name> [--title T][--body B][-m N]` | project | balls |
//!
//! **The origin column is a constant per verb, not a parameter** (§7.3,
//! bl-48f8): a verb's §7.3 attribution is its *subject*, which it already knows
//! — a `bl` verb is about a ball, a `lernie` verb about a conversation. So the
//! banner surface is decided here, where the fact is, and never by the hand that
//! fired: `close_ball` has one body reached by the composer's button, the §11
//! `c` key and the row menu ([`crate::shell`]), and forking it three ways to
//! record a pointer position would record something no operator asked about.
//!
//! Every verb runs in an explicit cwd — the `bl` verbs against the project
//! (§8.2), the `lernie` verbs against the workspace (harmless — lernie takes
//! the ws as argv — and a truthful `cwd` field). One invariant, no per-verb
//! special-case. `create`'s captured id is just its [`Outcome::stdout`] (bl
//! prints the new id there). The `ts` stamp is minted at the shell boundary and
//! injected, keeping this path pure-otherwise and deterministic in tests.
//!
//! **§8.2 identity rider (Z4):** every `bl` claim/close/unclaim is stamped `--as
//! <workspace name>`, **not** the operator `$USER` — the claimant delivers its own
//! ball (§3.2). Close/release stamp the ball's *bound* name; assign stamps the
//! *target* name. Enablement predicates live in [`super`](crate::actions).
//!
//! **The four `lernie` verbs take a [`Bound`], never a bare `Cli`** (bl-bf79):
//! the workspace's wall (`YOG_WALL`, §16.2) and its name (`YOG_NAME`, §3.3) are
//! laid once where the workspace is known, so no verb — including one written
//! later — can spawn a workspace-bound child outside the sphere. See
//! [`bound`] for the failure that fold retired.
//!
//! **No verb is gated (§16.7 W13).** Phase 1 consulted a host-tool capability
//! gate here; the substrates are exact-pinned crates now, so the verbs yog
//! drives *are* the verbs it ships — the version is the lockfile (§16.4/§16.5)
//! and there is no host binary left to be skewed against. Every dispatcher goes
//! straight to its spawn; a failure is still the durable `ops.jsonl` line it
//! always was.

use std::io;
use std::path::Path;

use crate::opslog::Origin;

mod balls;
mod bound;
mod dispatch;
pub use balls::{assign, close, create, edit, unclaim, update};
pub use bound::Bound;
pub use dispatch::{Outcome, log_step_done, log_step_failure, run_logged, run_logged_cwdless};
// `collect` stays crate-internal — the no-marks knob's `bl conf` seam reuses it.
pub(crate) use dispatch::collect;

// lernie subcommands (pinned to `src/bin/lernie.rs`, §8.2).
const MESSAGE: &str = "message";
const STOP: &str = "stop";
const SCAN: &str = "scan";
const RETARGET: &str = "retarget";
const STOP_CHILDREN: &str = "--stop-children";

/// `lernie message <ws> <agent> <content>` — the resume gesture (§8.2, ARCH
/// §2.9: no resume verb exists; the deposit restarts a driver). The revived
/// driver is a **workspace-bound spawn**, which is the whole of what [`Bound`]
/// carries: its wall (§16.2) so its first `bz` finds the sphere's providers, and
/// `YOG_NAME` (§8/§3.3) so its agents' tool subprocesses stamp `--as <name>`
/// through the W9 shim, exactly as the detached `lernie prompt` does (Z3).
pub fn message(
    lernie: &Bound,
    state_root: &Path,
    ts: &str,
    agent: &str,
    content: &str,
) -> io::Result<Outcome> {
    let ws_s = lernie.workspace_arg();
    run_logged(
        lernie.cli(),
        state_root,
        ts,
        lernie.workspace(),
        &[MESSAGE, &ws_s, agent, content],
        Origin::Conversation,
    )
}

/// One **attempt** (VISION V2, bl-dc0c): `lernie dispatch <role> <ws> <parent>
/// --goal <goal> --from <ref> [--pin …]`, piped and logged like every other
/// short verb. The argv is composed by [`crate::fork::argv`], which is where
/// the three fire-time controls turn into the three real flags they are.
///
/// **Piped, not detached** — unlike the start flow's `lernie prompt`. This verb
/// only forks the branch, writes the dispatch commit and detach-launches the
/// child's own driver; it returns immediately, and piping it is what makes a
/// refusal (an undeclared role, a ref the workspace does not have) a *rendered*
/// failure with lernie's own words in it rather than a click that did nothing.
/// A cohort is this verb run N times, so a cohort with a bad candidate says
/// which one, N times over, on the §4.2 trail.
///
/// The spawn is [`Bound`] for the same reason [`message`] is: it launches the
/// **child's** driver, so the child's whole descendant tree resolves the
/// workspace's wall (§16.2) and stamps `--as` from `YOG_NAME` (§8/§3.3).
pub fn fork(
    lernie: &Bound,
    state_root: &Path,
    ts: &str,
    fire: &crate::fork::Fire,
) -> io::Result<Outcome> {
    let argv = crate::fork::argv(fire);
    let args: Vec<&str> = argv.iter().map(String::as_str).collect();
    run_logged(
        lernie.cli(),
        state_root,
        ts,
        lernie.workspace(),
        &args,
        Origin::Conversation,
    )
}

/// `lernie stop <ws> <agent> [--stop-children]` — the §2.9 SIGTERM cascade,
/// optionally to the agent's descendants (§8.2). It launches nothing, so the
/// [`Bound`] layer is inert here — and taken anyway, because the alternative is
/// a per-verb judgement about the wall, which is the bug bl-bf79 fixed
/// ([`bound`]).
pub fn stop(
    lernie: &Bound,
    state_root: &Path,
    ts: &str,
    agent: &str,
    stop_children: bool,
) -> io::Result<Outcome> {
    let ws_s = lernie.workspace_arg();
    let mut args = vec![STOP, ws_s.as_str(), agent];
    if stop_children {
        args.push(STOP_CHILDREN);
    }
    run_logged(
        lernie.cli(),
        state_root,
        ts,
        lernie.workspace(),
        &args,
        Origin::Conversation,
    )
}

/// `lernie retarget <ws> <agent>` — the §9.4 exit from the config freeze
/// (bl-2d19). It writes a ref mark and returns; the conversation's **own**
/// executor consumes it at its next step boundary, re-forking the branch onto
/// the marked config commit and replaying its work on top. Piped, not detached,
/// for `dispatch`'s reason: a refusal — an agent the workspace has not got, a
/// role the target config does not describe — must come back in lernie's own
/// words rather than as a click that did nothing.
///
/// No `--config`: lernie defaults to the `default` lineage, which is the one
/// yog's picker writes and the one the drift this verb answers is measured
/// against (§9.3, §9.4).
pub fn retarget(lernie: &Bound, state_root: &Path, ts: &str, agent: &str) -> io::Result<Outcome> {
    let ws_s = lernie.workspace_arg();
    run_logged(
        lernie.cli(),
        state_root,
        ts,
        lernie.workspace(),
        &[RETARGET, &ws_s, agent],
        Origin::Conversation,
    )
}

/// `lernie scan <ws>` — flush inboxes and deposit died epitaphs (§8.2, §7.3).
/// Flushing an inbox **is** the revive path [`message`] takes, so it is bound
/// for the same reason: a driver scan restarts must find the sphere's providers.
pub fn scan(lernie: &Bound, state_root: &Path, ts: &str) -> io::Result<Outcome> {
    let ws_s = lernie.workspace_arg();
    run_logged(
        lernie.cli(),
        state_root,
        ts,
        lernie.workspace(),
        &[SCAN, &ws_s],
        Origin::Conversation,
    )
}

#[cfg(test)]
mod tests;
