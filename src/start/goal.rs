//! Goal composition + the pre-mint name preview (DESIGN §3.3, §3.4).
//!
//! Everything here is pure. The **payload prefill** ([`prefill`]) is the editable
//! text the operator sees: empty (bare), a target preamble naming the directory
//! verbatim (path), or the ball's title/body/worktree preamble (ball). What the
//! operator edits is exactly what fires (bl-6920: the goal reaches the model
//! unmutated); identity is not text at all — the minted name rides `--name`, and
//! lernie states the stored fact in its assembled context. The name is the
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
use lernie::mint::Rng;
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
    let worktree = canonical_worktree(&inputs.payload, &inputs.balls_state_root);
    Composer {
        preview: identity_preview(&inputs.conversation_names, rng),
        prefill: prefill(&inputs.payload, worktree.as_deref()),
    }
}

/// The editable payload prefill (§3.3), per rung — since bl-6920 also exactly
/// what fires: nothing is prepended. Bare is empty (the operator types); path and
/// ball carry their target preambles verbatim. `worktree` is the resolved ball
/// worktree the composer names (§3.3, threaded from the claim cross-check): the
/// canonical `<id>` leaf or the `<id>-<claimant>` variant bl actually minted, so
/// the preamble never names a nonexistent path. `None` for bare/path/new rungs.
pub(super) fn prefill(payload: &Payload, worktree: Option<&Path>) -> String {
    match payload {
        Payload::Bare => String::new(),
        Payload::Path { dir } => path_preamble(dir),
        Payload::Ball {
            project,
            ball: BallSpec::Existing {
                id, title, body, ..
            },
        } => ball_preamble(id, title, body, worktree.unwrap_or(project), project),
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
pub(super) fn canonical_worktree(payload: &Payload, balls_state_root: &Path) -> Option<PathBuf> {
    match payload {
        Payload::Ball {
            project,
            ball: BallSpec::Existing { id, .. },
        } => Some(work_worktree_path(balls_state_root, project, id, None)),
        _ => None,
    }
}

/// A workspace's name — its path leaf (§3.1): the `--as`/`YOG_NAME` stamp, for a
/// named focus, a foreign one, or a raise. **The name is a query, not a field**
/// (§3.1: the dir's existence is the registration), so this is the one place the
/// start flow answers "what is this workspace called". It never reaches the goal
/// text (§3.3, bl-df65) — the harness channel is its only one. Empty for a
/// rootless path (never a real workspace).
pub(crate) fn leaf_name(workspace: &Path) -> String {
    workspace
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// The §3.3 ball worktree preamble verbatim: the ball header, the body, and the
/// durable target-repo binding (the absolute work-worktree path rides in the
/// goal *content* because lernie has no target-repo concept, §3.3).
fn ball_preamble(id: &str, title: &str, body: &str, worktree: &Path, project: &Path) -> String {
    format!(
        "Ball {id}: {title}\n\n{body}\n\nThe project repository checkout for this work is the git worktree at:\n{worktree}   (branch work/{id} of {project})\nDo all repository work there, by absolute path. Do not rely on the current directory.",
        worktree = worktree.display(),
        project = project.display(),
    )
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

/// The per-rung driver cwd (§3.4): `~` (bare / a not-yet-created ball), the given
/// directory (path), or the resolved work worktree (an existing ball). `worktree`
/// is the claim's cross-checked path (canonical or `<id>-<claimant>`); the
/// existing-ball arm prefers it, falling back to `~` only defensively (the
/// executor always resolves one). Belt-and-suspenders beside the goal-content
/// binding (§3.3).
pub(super) fn driver_cwd(payload: &Payload, home: &Path, worktree: Option<&Path>) -> PathBuf {
    match payload {
        Payload::Path { dir } => dir.clone(),
        Payload::Ball {
            ball: BallSpec::Existing { .. },
            ..
        } => worktree.unwrap_or(home).to_path_buf(),
        Payload::Bare
        | Payload::Ball {
            ball: BallSpec::New { .. },
            ..
        } => home.to_path_buf(),
    }
}

/// The composer's fire-time parameters as a [`Prepared`](super::Prepared): the
/// resolved name, its workspace path, the per-rung driver cwd, and the editable
/// goal prefill (fired verbatim, bl-6920). `worktree` is the resolved ball worktree
/// (§3.3, addendum): the planner passes the canonical formula, the executor the
/// claim's cross-checked path. The single source both [`super::plan`]'s `Prompt`
/// step and [`super::prepare`]'s return derive from.
pub(super) fn compose_prepared(inputs: &StartInputs, worktree: Option<&Path>) -> super::Prepared {
    super::Prepared {
        name: leaf_name(&inputs.workspace),
        workspace: inputs.workspace.clone(),
        cwd: driver_cwd(&inputs.payload, &inputs.home, worktree),
        goal: prefill(&inputs.payload, worktree),
        origin: inputs.payload.origin(),
    }
}
