//! Git-tree view-model (ARCH §7.1 live view, §3.5 agent-state contract).
//!
//! [`GitTree::from_repo`] inspects the workspace's on-disk state and
//! produces a view-model suitable for rendering. The view-model is a pure
//! function of the workspace's refs and tree content; it holds no egui
//! dependency, so a future `litany-ui-web` crate can render the same
//! structure from the web.
//!
//! Git access is via the `git` CLI (a hard dep of litany itself, per ARCH
//! §2.2) — no libgit2 native build step is required.
//!
//! # Workspace layout (ARCH §2.2–§2.3)
//!
//! A workspace holds one bare repository at `<workspace>/repo.git`: config
//! branches (`config/<name>`) and agent refs (`agents/<agent-id>`) — no
//! `main`. Callers pass the workspace path; this module resolves the git
//! dir to `<workspace>/repo.git` before issuing any git command, and reads
//! step records, inboxes, and marks directly from the workspace root.
//!
//! The trunk section is the config lineage (`HEAD`, which the workspace
//! repository points at `config/default`); the agent section enumerates
//! every `agents/*` ref and renders it as a **tree** by hyphenated descent
//! (§2.3) — agents never merge anywhere (§2.6), so every agent persists on
//! its own ref. Each agent carries its §3.5 state ([`AgentState`]), its four
//! ref-derived mark oids (conflicted, budget-exhausted, abandoned, notify —
//! §6/§4.1, projected to the [`AgentMark`] set every mark seat renders), a
//! pending-message count from its inbox, its §11 last-action timestamp (the
//! tip, the newest `messages/` entry and the live tail folded to one recency
//! fact, bl-cad5), and its branch commits with their subjects (delivery /
//! work-product-transfer commits surface by subject).

/// The live conversation-name enumeration the boundary addresses over (REMOTE
/// §8 as amended, bl-49bc) — two facts per agent, asked of disk per gesture.
mod addressing;
mod cmd;
mod descent;
mod detect;
mod enumerate;
// **Why the latest model call failed** (bl-9b88): the sentence the §3.5
// classifier reads beside the state, and the row-altitude clause of it.
mod failure;
// The Linux `/proc` probe backends (§10). Compiled only where they have a
// consumer — always on Linux (production + test), and under `cfg(test)` on
// macOS (their own unit tests) — since macOS drives liveness through `lsof`.
#[cfg(any(test, not(target_os = "macos")))]
mod fd_probe;
#[cfg(any(test, not(target_os = "macos")))]
mod lock_probe;
// The macOS `lsof` backend (§10): a pure parser + spawn shim + TTL cache. Its
// core is platform-independent, but Linux never *uses* it (its `/proc` probes
// are cheaper and always definite), so it is compiled only where it has a
// consumer — under `cfg(test)` for its coverage, and on macOS in production.
#[cfg(any(test, target_os = "macos"))]
mod lsof;
mod marks;
mod model;
mod probe;
#[cfg(any(test, target_os = "macos"))]
mod probe_cache;
mod probe_stack;
mod project;
mod state;
mod streaming;
mod terminal;
mod tools;

// The boundary's own doorway into this module (bl-49bc): which conversation a
// name addresses, read live rather than off the derivation — the same doorway
// discipline every other consumer here gets.
pub(crate) use addressing::living_agents;
pub use descent::{DescentRow, children_of, descent_order};
pub(crate) use enumerate::mtime_unix;
// The view-model types themselves (§7.1), re-exported so `git_tree::Agent` and
// friends stay the one spelling every consumer already uses.
pub use model::{Agent, CommitNode, GitTree, GitTreeError, StepCommit, ToolCall, ToolCallState};
// The §6 durable marks an agent wears — the fact behind the attention signal,
// which is why they are `pub`: every mark seat renders them (§11).
pub use marks::AgentMark;
// Config-branch browse plumbing (§9.3 / §5.1 #17–#18), consumed by
// [`crate::config_edit::branch`]. Every config-ref git call routes through
// the env-scrubbed `cmd` wrapper; these re-exports are the only doorway.
// **`merge_base`/`is_ancestor` are not only that fold's** (bl-40ab): they are
// plain reachability reads over any repo, and §3.9's science projection asks
// them of a *project* repo — the base two ends departed from, and whether a
// source has incorporated its target. One spelling of one git command.
pub(crate) use cmd::browse::{
    diff_names, for_each_ref_config, is_ancestor, ls_tree, ls_tree_long, merge_base, show_file,
};
pub use probe::Probe;
pub use probe_stack::ProbeStack;
// The **project** repo's reads (§5.1 #32), consumed by [`crate::workdiff`] —
// the same doorway discipline one repo over.
pub(crate) use project::{file_patch, head_branch, log_marker, numstat, rev_parse};
pub use state::AgentState;
// The row's first clause of a failure sentence (bl-9b88) — the §11 list's own
// bound on how much of `Agent::failure` a glance says.
pub(crate) use failure::clause;
// The live-tail fold and the file it folds, shared with the §7.2 follower
// (`app::live`) so the JSONL delta parser is never duplicated (§15 Y12:
// "reuse the streaming fold — do NOT duplicate the JSONL parser").
pub(crate) use streaming::{fold_stream, latest_response_path};
// What that fold yields (§5.1 #10, #28b) — `pub` because `Agent` carries the
// whole value and the §11 live mark, the flight strip and the transcript's
// live tail all read it off the snapshot.
pub use streaming::{Delta, Stream, stream_from_disk};
// The fold's wire spelling (bl-73e7), the follow lane's frame body.
pub use streaming::wire as stream_wire;
// The §4.4 terminal classifier, shared with the Y13 steps inspector so the
// segment-boundary parser is never duplicated (§15 Y13: "reuse
// git_tree::terminal's segment classification — do NOT duplicate the
// parser"). The live view reads it as `AgentState`; the steps inspector
// reads the raw framing (a public classification it surfaces per step) plus
// the completed-segment count. The two folds stay crate-internal.
pub use terminal::Framing;
pub(crate) use terminal::{Ending, error_text, segment_count, settled};

/// The bare workspace repository dir (ARCH §2.2). Mirrors
/// `src/workspace::REPO_DIR` in the harness; the duplicate constant keeps
/// the UI crate free of a dep on the harness binary. `pub(crate)` so the
/// config-branch browse surface ([`crate::config_edit::branch`]) resolves
/// `<workspace>/repo.git` through the one authoritative name.
pub(crate) const REPO_DIR: &str = "repo.git";

/// Top-level directory under the workspace root holding per-agent step
/// records (ARCH §2.2 / §2.3). Mirrors `src/prompt/step::STEPS_DIR`.
const STEPS_DIR: &str = "steps";

/// Top-level directory under the workspace root holding per-agent inboxes
/// (ARCH §2.11). Mirrors the harness's `inbox/<agent-id>/` layout; the
/// pending-message count and the executor-lock probe both key off it.
const INBOX_DIR: &str = "inbox";

/// Top-level directory under the workspace root holding the per-agent worktrees
/// (`agents/<agent-id>/`, ARCH §2.2): where `goal.md` and the rest of the
/// dispatch control files live on disk (§7.1 watch roots).
const AGENTS_DIR: &str = "agents";

/// The committed-transcript directory inside an agent's worktree
/// (`agents/<agent-id>/messages/`, ARCH §2.11 / §5.1 #12). Read here only for
/// its entries' mtimes — the §11 recency fact (`Agent::last_action_unix`);
/// their *content* is `crate::transcript`'s, which mirrors this name for the
/// same reason `AGENTS_DIR` is mirrored: neither module depends on the other.
const MESSAGES_DIR: &str = "messages";

// `pub(crate)` (not private) so Y6's milestone-proof test in `crate::app`
// reuses the workspace [`tests::fixture`] builder — the one place a real
// on-disk workspace is spun up (§15 Y6: "use the existing fixture builders").
#[cfg(test)]
pub(crate) mod tests;
