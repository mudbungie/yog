//! Per-root-kind path allowlists (DESIGN §7.1).
//!
//! One `fs_watcher::Watcher` runs per watched root, and each root has a *kind*
//! that fixes which paths under it are worth surfacing. [`is_watched`] is the
//! single pure predicate generalizing the previously-hardcoded Workspace
//! allowlist over every kind in the §7.1 table. It is path-only (no stat, no
//! I/O), so it is exhaustively testable and shared unchanged by a future
//! `litany-ui-web`. Event *kind* (create vs remove) and recursion *scope*
//! (top vs recursive) are the watcher's concern, never the allowlist's.

use std::path::Path;

/// Workspace-root paths (ARCH §2.2, §3.5): the shared `steps/` and `inbox/`
/// trees, outside every worktree, namespaced by agent id. Control files are
/// no longer loose here — they live in the config commit (§2.2), observed
/// through the refs below.
const ROOT_CONTROL_PREFIXES: &[&str] = &["steps", "inbox"];

/// Per-agent-worktree paths (ARCH §2.2 layout). Each agent occupies a worktree
/// at `agents/<agent-id>/`, with this set of files inside.
const WORKTREE_PREFIXES: &[&str] = &[
    "goal.md",
    "soul.md",
    "summary",
    "messages",
    "descriptions",
    "skills",
];

/// Refs and HEAD live in the bare workspace repository at `repo.git`
/// (ARCH §2.2). Branch existence is read from refs/ — no sidecar state file
/// (PRINCIPLES.md "Single source of truth").
///
/// `packed-refs` is load-bearing, not decoration: yog reads refs through `git
/// for-each-ref`, which reads the loose tree **and** the packed file. After a
/// `git pack-refs` (which `git gc` runs) the loose tree is empty, and deleting a
/// packed ref then rewrites `packed-refs` alone — touching nothing under
/// `repo.git/refs/`. Without this entry that deletion is invisible to the
/// watcher and reaches yog only via the 15 s sweep: a reproducible dropped
/// event, proven in `drift_tests`.
const REFS_PREFIXES: &[&str] = &["repo.git/HEAD", "repo.git/refs", "repo.git/packed-refs"];

/// The kind of root a [`Watcher`](super::Watcher) guards, selecting its
/// allowlist (DESIGN §7.1). `Hash` so `(root, kind)` keys a
/// [`WatchSet`](crate::watch::WatchSet) map (Y6).
///
/// The set is exactly what `desired_watches` arms — there is no kind here that
/// nothing watches. `BrazenConfig` and `LitanyConfig` used to be, and were
/// retired unarmed at bl-9130: config is operator-authored draft state that
/// feeds no re-derivation, its concurrency answer is §9's hash guard, and the
/// litany config root *is* the litany data root under the world's `LITANY_HOME`
/// collapse (§16.2), so arming it recursively is the watcher §7.1 rejects. The
/// §9 editors re-read on pane open instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RootKind {
    /// A litany workspace dir: `steps/`, `inbox/`, per-agent worktree files,
    /// `repo.git/{HEAD,refs}` — the original hardcoded allowlist.
    Workspace,
    /// `$XDG_DATA_HOME/yog/workspaces/` (flat by construction, §3.1/§7.1): named
    /// workspaces appearing and being removed.
    NamesRoot,
    /// `<litany-data>/workspaces/`, `replays/`: foreign workspaces and replays
    /// appearing and being removed.
    WorkspacesRoot,
    /// `$XDG_STATE_HOME/balls/clones/`: clone dirs, per-clone `tasks/tasks/*.md`
    /// and `config/config/**`. The churny per-clone `log` is filtered out.
    BallsClones,
    /// `$XDG_STATE_HOME/yog/`: `ui.json`, `ops.jsonl` and `cadence.yaml`
    /// (§4.1, §4.2, §7.2 bl-3381).
    YogState,
}

/// True when `path` (expected under `root`) falls in `kind`'s allowlist. Pure
/// and path-only: a rejected path is never surfaced by the watcher.
pub fn is_watched(kind: RootKind, root: &Path, path: &Path) -> bool {
    let Ok(rel) = path.strip_prefix(root) else {
        return false;
    };
    let rel = rel.to_string_lossy();
    match kind {
        RootKind::Workspace => workspace(&rel),
        // Coarse structural roots (§7.1: "dir create/remove"): any descendant
        // is a candidate — a named workspace's own git churn re-enumerates
        // idempotently, coalesced by the frame's debounce (§7.2).
        RootKind::NamesRoot | RootKind::WorkspacesRoot => !rel.is_empty(),
        RootKind::BallsClones => balls_clones(&rel),
        RootKind::YogState => {
            rel == "ui.json" || rel == "ops.jsonl" || rel == crate::app::cadence::CADENCE_YAML
        }
    }
}

/// The original Workspace allowlist (ARCH §3.5), byte-for-byte.
fn workspace(rel: &str) -> bool {
    if matches_any(rel, ROOT_CONTROL_PREFIXES) || matches_any(rel, REFS_PREFIXES) {
        return true;
    }
    // Per-agent-worktree paths live under `agents/<agent-id>/…`. The watcher
    // does not enumerate agent ids — any id segment is admissible; the tail
    // determines the hit.
    matches!(
        rel.strip_prefix("agents/").and_then(|t| t.split_once('/')),
        Some((_agent_id, tail)) if matches_any(tail, WORKTREE_PREFIXES)
    )
}

/// The `$XDG_STATE_HOME/balls/clones/` allowlist (§7.1): each clone dir itself,
/// its `tasks/tasks/*.md` task files and `config/config/**` landing subtree;
/// the multi-MB unrotated per-clone `log` is filtered to avoid event storms.
fn balls_clones(rel: &str) -> bool {
    let Some((_clone, tail)) = rel.split_once('/') else {
        // A single non-empty segment is a clone dir itself (create/remove).
        return !rel.is_empty();
    };
    if matches_any(tail, &["log"]) {
        return false;
    }
    if let Some(name) = tail.strip_prefix("tasks/tasks/") {
        return !name.is_empty() && !name.contains('/') && name.ends_with(".md");
    }
    matches_any(tail, &["config/config"])
}

fn matches_any(rel: &str, prefixes: &[&str]) -> bool {
    prefixes
        .iter()
        .any(|prefix| rel == *prefix || rel.starts_with(&format!("{prefix}/")))
}

#[cfg(test)]
mod tests;
