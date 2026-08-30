//! The git-tree view-model types (§7.1 live view, §3.5 agent-state contract):
//! the error enum and the five inert structures [`GitTree::from_repo`] answers
//! with. Pure data — no git call, no egui dep, no derivation beyond the one
//! constructor that hands off to [`ProbeStack::derive`]; every field is a fact
//! read off refs or disk on the tick that built it (§3.5 stateless re-read).
//! [`super`] holds the wiring that fills them.
//!
//! The fattest of the five, [`Agent`], sits on its own file: it carries more
//! documented fields than everything here together, and §12's pre-split rule
//! wants that seam drawn before the cap does it for us.

use super::ProbeStack;
use std::path::{Path, PathBuf};

/// The one agent-branch structure, on its own file (§12): it carries more
/// documented fields than the rest of this module together.
mod agent;
pub use agent::Agent;

#[derive(Debug, thiserror::Error)]
pub enum GitTreeError {
    #[error("git invocation failed: {0}")]
    Spawn(#[from] std::io::Error),
    #[error("git {command} in {repo:?} failed: {stderr}")]
    Git {
        command: String,
        repo: PathBuf,
        stderr: String,
    },
    #[error("malformed git log line: {0:?}")]
    LogFormat(String),
    /// The governing-config derivation (§5.1 #17) declined: either no
    /// `config/*` lineage reaches the agent's branch, or two candidate
    /// ancestors are incomparable. Both mean a defective workspace, declined
    /// rather than guessed (mirrors litany `workspace.rs`).
    #[error("governing config: {0}")]
    Governing(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GitTree {
    /// The config lineage (`HEAD` → `config/default`, §2.2),
    /// first-parent, oldest to newest.
    pub commits: Vec<CommitNode>,
    /// Every agent branch (`agents/*`, §2.3), enumerated via
    /// `git for-each-ref refs/heads/agents/`. A flat authoritative set;
    /// the render tree is derived from the ids by [`descent_order`]
    /// (§2.3 hyphenated descent) — never stored (PRINCIPLES "Single
    /// source of truth").
    pub agents: Vec<Agent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitNode {
    pub oid: String,
    pub short_oid: String,
    pub timestamp_unix: i64,
    /// Commit subject — config commits are the only trunk commits
    /// (§2.2–§2.3: agents never merge anywhere), so the subject is the
    /// row's label.
    pub subject: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepCommit {
    pub oid: String,
    pub short_oid: String,
    pub timestamp_unix: i64,
    /// Commit subject. Surfaces what a branch commit is — a dispatch, a
    /// delivery commit, or a work-product-transfer commit (§2.11, §2.6,
    /// §7.1 "delivery/result-message commits surfaced").
    pub subject: String,
}

/// A single tool call surfaced to the renderer. The disk records carry
/// more metadata (timing, exit code, raw stdout) but the view-model only
/// needs identity + state to drive the indicator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCall {
    /// `tool_use.id` from the wire (e.g. `toolu_01abc…`); also the
    /// `<tool-id>/` directory name under
    /// `<workspace>/steps/<agent-id>/<NNN>/tools/`.
    pub tool_id: String,
    /// What the tool is **called** (`Read`, `Bash`, …) — `input.json`'s `name`
    /// field, read at enumerate time beside the two presence checks that
    /// decide [`state`](ToolCall::state), never from a render path (§11
    /// bl-cad5's rule). `None` when the record carries no parsable name; the
    /// §11 in-flight strip then drops the segment rather than printing the
    /// opaque `tool_id`, which names nothing to an operator.
    pub name: Option<String>,
    /// When this call **started** (§5.1 #28 elapsed, bl-9dfb): the mtime of the
    /// `input.json` whose presence already decides
    /// [`state`](ToolCall::state). The executor lands that record atomically
    /// immediately before it spawns the tool — the same instant it later records
    /// as `output.json`'s `started_at` — and never rewrites it, so its mtime is
    /// the call's start. No commit timestamp could serve instead: step records
    /// are not git-tracked (§2.3), so this file has no commit at all. `None`
    /// only when the stamp is unreadable (the record vanished between the
    /// presence check and the stat); the strip then omits elapsed.
    pub start_unix: Option<i64>,
    pub state: ToolCallState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolCallState {
    /// `input.json` has landed but `output.json` has not — the tool
    /// executor is still running. Renderer pulses this node.
    InFlight,
    /// Both `input.json` and `output.json` are present on disk. Renders
    /// statically; no repaint scheduling.
    Complete,
}

impl GitTree {
    /// Derive a workspace's tree with a fresh, throwaway [`ProbeStack`] — the
    /// one-shot path (tests, a single read). The live UI instead holds one
    /// [`ProbeStack`] across ticks (§15 Y11) so its TTL cache pays off; both
    /// route through the same [`ProbeStack::derive`] derivation.
    pub fn from_repo(workspace: &Path) -> Result<Self, GitTreeError> {
        ProbeStack::platform().derive(workspace)
    }
}
