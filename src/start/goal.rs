//! Goal composition + the pre-mint name preview (DESIGN §3.3, §3.4).
//!
//! Everything here is pure. The **payload prefill** ([`prefill`]) is the editable
//! text the operator sees: empty (bare), a target preamble naming the directory
//! verbatim (path), or the ball's header and body (ball). What the
//! operator edits is exactly what fires (bl-6920: the goal reaches the model
//! unmutated); identity is not text at all — the minted name rides `--name`, and
//! litany states the stored fact in its assembled context. The name is the
//! **conversation's** ([`mint_conversation`]): the workspace never enters the
//! prompt (bl-df65), because `YOG_NAME` already carries it and the world's `bl`
//! shim defaults `--as` to it (§16.7 W9, §3.3). [`preview`] mints the predicted
//! conversation name from a pure read (the target workspace's already-derived
//! name facts + the injected RNG) and pairs it with the prefill; the mint is
//! re-derived at fire — the preview is a prediction, the fire's mint is the
//! truth.
//!
//! The one stamp still composed here is read back by its inverse **in this
//! module**, one home for compose and parse (§3.3, PRINCIPLES "single source of
//! truth" — change the format here and every derivation follows):
//! [`parse_ball_stamp`] inverts [`ball_preamble`]'s header for the
//! conversation↔ball join. The identity stamp's parse survives only as
//! [`parse_identity_stamp`]'s legacy rung (`super::identity`) — its compose is
//! retired.

use super::identity::identity_preview;
use super::{BallSpec, Payload, StartInputs};
use crate::binding::work_worktree_path;
use litany::mint::Rng;
use std::path::{Path, PathBuf};

/// The composer view-model (§3.3): the greyed name-prediction `preview` line and
/// the editable payload `prefill`. Both are pure reads — nothing spawns (I7).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Composer {
    pub preview: String,
    pub prefill: String,
}

/// The pre-submit composer view-model (§3.3): the predicted name paired with
/// the payload prefill. Nothing spawns (I7).
pub fn preview(inputs: &StartInputs, rng: &dyn Rng) -> Composer {
    Composer {
        preview: identity_preview(&inputs.conversation_names, rng),
        prefill: prefill(&inputs.payload),
    }
}

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
fn path_preamble(dir: &Path) -> String {
    format!(
        "Working directory: {}\nDo all work there, by absolute path. Do not rely on the current directory.",
        dir.display(),
    )
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
        goal: prefill(&inputs.payload),
        origin: inputs.payload.origin(),
    }
}
