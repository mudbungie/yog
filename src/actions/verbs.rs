//! Short-verb dispatchers + the `ops.jsonl` wiring (DESIGN §8.2, §15 Y16).
//!
//! The action surface's *short, piped* verbs — every one but the detached
//! `litany prompt` (§8.1, Y17). Per §8.2 each runs synchronously with stdout
//! and stderr piped, then appends its completed outcome to `ops.jsonl`
//! (§4.2) — replacing the legacy "stderr printed and dropped" spawn-and-drain.
//!
//! | verb | argv | cwd | origin |
//! |---|---|---|---|
//! | message | `litany message <ws> <agent> <text>` | ws | conversation |
//! | stop | `litany stop <ws> <agent>` | ws | conversation |
//! | scan | `litany scan <ws>` | ws | conversation |
//! | retarget | `litany retarget <ws> <agent>` | ws | conversation |
//! | close | `bl close <id> --as <name>` | project | balls |
//! | assign | `bl claim <id> --as <name>` | project | balls |
//! | release / unclaim | `bl unclaim <id> --as <name>` | project | balls |
//! | create | `bl create <title> --as <name> [--body B]` | project | balls |
//! | update | `bl update <id> --as <name> [--title T][--body B][-m N]` | project | balls |
//!
//! **The origin column is a constant per verb, not a parameter** (§7.3,
//! bl-48f8): a verb's §7.3 attribution is its *subject*, which it already knows
//! — a `bl` verb is about a ball, a `litany` verb about a conversation. So the
//! banner surface is decided here, where the fact is, and never by the hand that
//! fired: `close_ball` has one body reached by the composer's button, the §11
//! `c` key and the row menu ([`crate::shell`]), and forking it three ways to
//! record a pointer position would record something no operator asked about.
//!
//! Every verb runs in an explicit cwd — the `bl` verbs against the project
//! (§8.2), the `litany` verbs against the workspace (harmless — litany takes
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
//! **The four `litany` verbs take a [`Bound`], never a bare `Cli`** (bl-bf79):
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
pub use balls::{Verb, assign, close, create, edit, unclaim, update};
pub use bound::Bound;
pub use dispatch::{Outcome, log_step_done, log_step_failure, run_logged, run_logged_cwdless};
// `collect` stays crate-internal — the no-marks knob's `bl conf` seam reuses it.
pub(crate) use dispatch::collect;

// litany subcommands (pinned to `src/bin/litany.rs`, §8.2).
const MESSAGE: &str = "message";
const STOP: &str = "stop";
const SCAN: &str = "scan";
const RETARGET: &str = "retarget";
/// The §9.6 settle verb (bl-dd88) — the operator half of the learning loop.
const PROPOSAL: &str = "proposal";

/// `litany message <ws> <agent> <content>` — the resume gesture (§8.2, ARCH
/// §2.9: no resume verb exists; the deposit restarts a driver). The revived
/// driver is a **workspace-bound spawn**, which is the whole of what [`Bound`]
/// carries: its wall (§16.2) so its first `bz` finds the sphere's providers, and
/// `YOG_NAME` (§8/§3.3) so its agents' tool subprocesses stamp `--as <name>`
/// through the W9 shim, exactly as the detached `litany prompt` does (Z3).
pub fn message(
    litany: &Bound,
    state_root: &Path,
    ts: &str,
    agent: &str,
    content: &str,
) -> io::Result<Outcome> {
    let ws_s = litany.workspace_arg();
    run_logged(
        litany.cli(),
        state_root,
        ts,
        litany.workspace(),
        &[MESSAGE, &ws_s, agent, content],
        Origin::Conversation,
    )
}

/// One **attempt** (VISION V2, bl-dc0c): `litany dispatch <role> <ws> <parent>
/// --goal <goal> --from <ref> [--pin …]`, piped and logged like every other
/// short verb. The argv is composed by [`crate::fork::argv`], which is where
/// the three fire-time controls turn into the three real flags they are.
///
/// **Piped, not detached** — unlike the start flow's `litany prompt`. This verb
/// only forks the branch, writes the dispatch commit and detach-launches the
/// child's own driver; it returns immediately, and piping it is what makes a
/// refusal (an undeclared role, a ref the workspace does not have) a *rendered*
/// failure with litany's own words in it rather than a click that did nothing.
/// A cohort is this verb run N times, so a cohort with a bad candidate says
/// which one, N times over, on the §4.2 trail.
///
/// The spawn is [`Bound`] for the same reason [`message`] is: it launches the
/// **child's** driver, so the child's whole descendant tree resolves the
/// workspace's wall (§16.2) and stamps `--as` from `YOG_NAME` (§8/§3.3).
pub fn fork(
    litany: &Bound,
    state_root: &Path,
    ts: &str,
    fire: &crate::fork::Fire,
) -> io::Result<Outcome> {
    let argv = crate::fork::argv(fire);
    let args: Vec<&str> = argv.iter().map(String::as_str).collect();
    run_logged(
        litany.cli(),
        state_root,
        ts,
        litany.workspace(),
        &args,
        Origin::Conversation,
    )
}

/// `litany stop <ws> <agent>` — the §2.9 SIGTERM cascade (§8.2). It launches
/// nothing, so the [`Bound`] layer is inert here — and taken anyway, because
/// the alternative is a per-verb judgement about the wall, which is the bug
/// bl-bf79 fixed ([`bound`]).
///
/// **The `--stop-children` flag is gone from this call** (bl-6efc). litany
/// bl-3114 made `stop` walk every descendant unconditionally, and the flag
/// went on parsing while changing nothing — so yog was passing a word that
/// named a choice litany no longer offers. Passing an inert flag is a bet that
/// it stays parseable; the day litany drops it, every stop this crate makes
/// fails on an unrecognized argument. The cascade is not lost, it is
/// unconditional: a stop takes the subtree, which is the only behaviour there
/// now is.
pub fn stop(litany: &Bound, state_root: &Path, ts: &str, agent: &str) -> io::Result<Outcome> {
    let ws_s = litany.workspace_arg();
    let args = vec![STOP, ws_s.as_str(), agent];
    run_logged(
        litany.cli(),
        state_root,
        ts,
        litany.workspace(),
        &args,
        Origin::Conversation,
    )
}

/// `litany retarget <ws> <agent>` — the §9.4 **change of lineage** (bl-2d19,
/// re-scoped by bl-e654). It writes a ref mark and returns; the conversation's
/// **own** executor consumes it at its next step boundary, re-forking the
/// branch onto the marked config commit and replaying its work on top. Piped,
/// not detached, for `dispatch`'s reason: a refusal — an agent the workspace
/// has not got, a role the target config does not describe — must come back in
/// litany's own words rather than as a click that did nothing. A conversation
/// already on that lineage is litany's own clean no-op, reported the same way,
/// which is why yog models no such state.
///
/// No `--config`: litany defaults to the `default` lineage, which is the one
/// yog's picker writes and therefore the only lawful destination a seat could
/// name (§9.3, §9.4).
pub fn retarget(litany: &Bound, state_root: &Path, ts: &str, agent: &str) -> io::Result<Outcome> {
    let ws_s = litany.workspace_arg();
    run_logged(
        litany.cli(),
        state_root,
        ts,
        litany.workspace(),
        &[RETARGET, &ws_s, agent],
        Origin::Conversation,
    )
}

/// `litany proposal <ws> <id> --accept|--reject` — settle one staged proposal
/// (§9.6, bl-dd88). The learning loop's veto, and the only §9 write yog does
/// not perform itself: accepting is a compare-and-swap fast-forward of a config
/// lineage, whose expected old value is the freshness the listing showed, so
/// re-implementing it here would be a second home for a rule whose failure mode
/// is a lost race. Piped, not detached, for `retarget`'s reason exactly — a
/// stale proposal, an ambiguous one and an unknown id each refuse in litany's
/// own words, which name the tip, the lineages and the pool respectively, and
/// those sentences are the whole product an operator acts on.
///
/// Bound like every workspace verb, though it launches nothing: "which verbs
/// may skip the wall" is the per-verb decision [`bound`] exists to abolish.
pub fn proposal(
    litany: &Bound,
    state_root: &Path,
    ts: &str,
    id: &str,
    flag: &str,
) -> io::Result<Outcome> {
    let ws_s = litany.workspace_arg();
    run_logged(
        litany.cli(),
        state_root,
        ts,
        litany.workspace(),
        &[PROPOSAL, &ws_s, id, flag],
        Origin::World,
    )
}

/// `litany scan <ws>` — flush inboxes and deposit died epitaphs (§8.2, §7.3).
/// Flushing an inbox **is** the revive path [`message`] takes, so it is bound
/// for the same reason: a driver scan restarts must find the sphere's providers.
pub fn scan(litany: &Bound, state_root: &Path, ts: &str) -> io::Result<Outcome> {
    let ws_s = litany.workspace_arg();
    run_logged(
        litany.cli(),
        state_root,
        ts,
        litany.workspace(),
        &[SCAN, &ws_s],
        Origin::Conversation,
    )
}

#[cfg(test)]
mod tests;
