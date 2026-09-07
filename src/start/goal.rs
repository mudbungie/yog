//! Goal composition (DESIGN §3.3, §3.4).
//!
//! Everything here is pure. The **payload prefill** ([`prefill`]) is the editable
//! text the operator sees: empty (bare), a target preamble naming the directory
//! verbatim (path), or the ball's header and body (ball). What the
//! operator edits is exactly what fires (bl-6920: the goal reaches the model
//! unmutated); identity is not text at all — the minted name rides `--name`, and
//! litany states the stored fact in its assembled context. The name is the
//! **conversation's** ([`mint_conversation`]): the workspace never enters the
//! prompt (bl-df65), because `YOG_NAME` already carries it and the world's `bl`
//! shim defaults `--as` to it (§16.7 W9, §3.3). **The pre-mint name prediction
//! and the composer view-model that paired it with the prefill are the seat's**
//! (bl-7cc8): the mint that matters is the one `execute_prompt` re-derives at
//! fire, and no §8.5 reply carries a predicted name, so a prediction composed
//! here reached nothing.
//!
//! The one stamp still composed here is read back by its inverse **in this
//! module**, one home for compose and parse (§3.3, PRINCIPLES "single source of
//! truth" — change the format here and every derivation follows):
//! [`parse_ball_stamp`] inverts [`ball_preamble`]'s header for the
//! conversation↔ball join. The identity stamp's parse survives only as
//! [`parse_identity_stamp`]'s legacy rung (`super::identity`) — its compose is
//! retired.

use super::{BallSpec, Payload, StartInputs};
use crate::binding::work_worktree_path;
use std::path::{Path, PathBuf};

/// The editable payload prefill (§3.3), per rung — since bl-6920 also exactly
/// what fires: nothing is prepended. Bare is empty (the operator types); path
/// carries its target preamble; ball carries its header and body. **Payload
/// only, since bl-6654:** the work target is no longer prose here — it rides
/// the fire's typed `--cwd` binding ([`target_binding`]) — so the prefill is
/// the model-facing content and nothing else.
pub(super) fn prefill(payload: &Payload) -> String {
    match payload {
        Payload::Bare => String::new(),
        Payload::Path { dir } => path_preamble(dir),
        Payload::Ball {
            ball: BallSpec::Existing {
                id, title, body, ..
            },
            ..
        } => ball_preamble(id, title, body),
        Payload::Ball {
            ball: BallSpec::New { title, body },
            ..
        } => format!("Ball (new): {title}\n\n{body}"),
    }
}

/// The path rung's target preamble (§3.3): the working directory named verbatim,
/// **on line one**. Every prefill yog composes leads with its headline — the
/// display ladder's second rung is the first payload line (§3.3), so a sentence
/// that buried the path on line two previewed the conversation by its own
/// boilerplate. The ball rung's `Ball <id>: <title>` header is the same invariant.
///
/// **Line two states the binding; it used to contradict it** (bl-fea6). The
/// sentence read *"Do all work there, by absolute path. Do not rely on the
/// current directory."* — and the second half was false: the same directory
/// rides the fire typed, as litany's `--cwd`
/// ([`prompt`](super::prompt)), which seeds the agent's working-directory mark
/// at creation, so every tool step of every turn starts there and a first
/// `pwd` prints the bound path. Told not to believe it, models spent their
/// orientation phase disproving it: one run's first tool call was a `cd` into
/// an invented generic home followed by `find /` for the target file, which
/// returned three sibling copies and cost five more calls to disambiguate; a
/// second opened `cd "$(pwd)" && pwd`; a third `cd`'d to that same invented
/// home and back. Two of three unrelated conversations invented the same
/// sandbox home, because *"do not rely on the current directory"* reads as
/// *"you are somewhere else"*.
///
/// bl-6654 retired the ball rung's location prose on the ground that *"location
/// stops being prose"*. The path rung keeps one line because the headline is
/// the display ladder's, and what follows it now **agrees** with the typed
/// channel instead of denying it.
fn path_preamble(dir: &Path) -> String {
    format!("{CWD_LEAD}{}\n{CWD_BINDING}", dir.display())
}

/// The preamble's first line, up to the directory itself.
const CWD_LEAD: &str = "Working directory: ";

/// Its second line, whole. Held as a constant beside the lead for the reason
/// [`parse_ball_stamp`] is held beside [`ball_preamble`]: [`strip_path_preamble`]
/// reads back exactly what this composes, so the shape has one home and the
/// inverse cannot come to expect a sentence the compose no longer writes.
const CWD_BINDING: &str = "This is already your current directory: every tool call starts there and relative paths resolve there. Keep the work inside it.";

/// The **operator's own goal**, beneath the path rung's preamble — the inverse
/// of [`path_preamble`] (bl-e2ad).
///
/// The preamble leads because the seat joins the operator's words *after* the
/// prefill it was handed (§8.5's own help: *"send the two joined as one
/// goal"*), so on the path rung — the rung every piece of coding work takes —
/// the goal's first payload line was always `Working directory: <dir>`, and the
/// §3.3 display ladder named every such conversation that. A workspace of five
/// then read as five identical absolute paths in the one column whose job is
/// telling them apart. The directory is not lost: it rides the fire typed as
/// litany's `--cwd` ([`target_binding`]), it is answered as `working_dir`
/// beside the conversation's files, and it is still line one of the goal
/// verbatim. It stops being the *name*.
///
/// The goal **verbatim** when no preamble leads it — every bare and ball rung,
/// every foreign root — and verbatim again when nothing follows one: a fire
/// with no words of the operator's own has only the directory line to show.
pub fn strip_path_preamble(goal: &str) -> String {
    let payload = strip_preamble(goal).unwrap_or(goal);
    if payload.trim().is_empty() {
        return goal.to_owned();
    }
    payload.to_owned()
}

/// The text after a well-formed preamble, or `None` when none leads `goal`.
/// Line-wise, exactly as the compose is: both lines leave, with the blank line
/// that separated them from the payload.
fn strip_preamble(goal: &str) -> Option<&str> {
    let (_dir, rest) = goal.strip_prefix(CWD_LEAD)?.split_once('\n')?;
    Some(rest.strip_prefix(CWD_BINDING)?.trim_start_matches('\n'))
}

/// The ball worktree the composer/preamble names for an **existing** ball (§3.3,
/// §3.5): the canonical `work_worktree_path` `<id>` leaf — the pure formula the
/// planner previews and the resume path falls back to. `None` for bare/path/new
/// rungs. The executor overrides this with the claim's cross-checked worktree
/// (the `<id>-<claimant>` variant when bl minted it — addendum: never a guess).
pub(super) fn canonical_worktree(inputs: &StartInputs) -> Option<PathBuf> {
    match (&inputs.payload, inputs.repo.as_deref()) {
        (
            Payload::Ball {
                ball: BallSpec::Existing { id, .. },
                ..
            },
            Some(repo),
        ) => Some(work_worktree_path(&inputs.balls_state_root, repo, id, None)),
        _ => None,
    }
}

/// The §3.3 ball payload verbatim: the `Ball <id>: <title>` header and the body.
/// The header stays because it is the §3.2 conversation→ball join, not a
/// location channel — [`parse_ball_stamp`] reads it back. The worktree
/// paragraph it used to trail ("The project repository checkout for this work
/// is the git worktree at: …") is **gone** (bl-6654, VISION §4.10 item 2): an
/// absolute path in prose was the interim channel while pinned litany had no
/// creation-time working directory, and a model had to notice and obey it. The
/// binding is typed now — [`target_binding`] rides `--cwd` — so location is a
/// parameter, and the goal is payload.
fn ball_preamble(id: &str, title: &str, body: &str) -> String {
    format!("Ball {id}: {title}\n\n{body}")
}

/// The ball id a conversation root's `goal.md` carries (§3.3): the inverse of
/// [`ball_preamble`]'s `Ball {id}: {title}` header. Pre-bl-6920 roots carry
/// the legacy identity stamp *above* the header, so the scan is line-wise — the
/// first line shaped `Ball <id>: <rest>` yields `<id>`. `None` for a bare/path
/// conversation (no header) or any goal without one. The one parse paired with
/// the one compose above: a start-flow ball is the only conversation-level
/// attribution that exists (§3.2), so a single id — never a set — is derivable.
pub fn parse_ball_stamp(goal: &str) -> Option<String> {
    goal.lines().find_map(stamp_id)
}

/// The ball id in one `Ball <id>: <title>` line, else `None`. A well-formed id
/// carries no whitespace (the compose emits a single token); that guard rejects
/// a prose line merely opening with the word `Ball` and an empty id.
fn stamp_id(line: &str) -> Option<String> {
    let (id, _title) = line.strip_prefix("Ball ")?.split_once(": ")?;
    (!id.is_empty() && !id.contains(char::is_whitespace)).then(|| id.to_owned())
}

/// The rung's **typed work target** (§3.3, bl-2b8c / VISION §4.10 item 2): what
/// the fire passes as litany's `--cwd`, seeding the agent's working-directory
/// mark at creation. The path rung binds the directory box's value; the ball
/// rung binds the claim's cross-checked `work/<id>` worktree; the bare rung and
/// a not-yet-created ball bind nothing, and an absent `--cwd` is litany's own
/// default (the agent's worktree). Not a rung table: `worktree` is already
/// `None` for every rung but an existing ball ([`super::resolve_worktree`]), so
/// it *is* the ball rung's binding and the match has two arms, not four.
pub(super) fn target_binding(payload: &Payload, worktree: Option<&Path>) -> Option<PathBuf> {
    match payload {
        Payload::Path { dir } => Some(dir.clone()),
        _ => worktree.map(Path::to_path_buf),
    }
}

/// The composer's fire-time parameters as a [`Prepared`](super::Prepared): the
/// resolved name, its workspace path, the typed target binding and the editable
/// goal prefill (fired verbatim, bl-6920). `worktree` is the resolved ball
/// worktree (§3.3, addendum): the planner passes the canonical formula, the
/// executor the claim's cross-checked path. The single source both
/// [`super::plan`]'s `Prompt` step and [`super::prepare`]'s return derive from.
///
/// **There is no per-rung driver cwd any more (bl-6654).** `Prepared` used to
/// carry one — `~`, the given directory, or the work worktree — as the initial
/// `litany prompt` process's `current_dir`. It was a second, weaker spelling of
/// the work target: it reached that one process and no tool step (every step
/// runs at the agent's own working-directory mark), which DESIGN §3.3 recorded
/// as misleading redundancy. [`target_binding`] is the one operative channel
/// now, so the field is gone rather than pinned to a constant, and the detached
/// driver simply stands in the workspace it drives.
pub(super) fn compose_prepared(inputs: &StartInputs, worktree: Option<&Path>) -> super::Prepared {
    super::Prepared {
        workspace: crate::naming::leaf(&inputs.workspace),
        binding: target_binding(&inputs.payload, worktree),
        // The §8.7 lineage is a git read, and everything here is pure: the
        // planner's preview never names one, and [`prepare`](super::prepare)
        // — the one caller that resolved it — fills it in on the way out.
        lineage: None,
        // The role is nobody's derivation (bl-9ced): a prepare answers the
        // worker default as an absence and the seat sets it on the gesture it
        // deposits back, which is where plan mode is chosen.
        role: None,
        goal: prefill(&inputs.payload),
        origin: inputs.payload.origin(),
    }
}
