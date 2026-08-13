//! The pinned tree: the Files tab read **as of a commit** (VISION V1.2).
//!
//! The live Files tab walks the agent's materialized worktree with `read_dir`
//! (`files_view`). Pinned, the same tab reads the same shape out of git —
//! `ls-tree -r -l` for the listing, `git show <commit>:<path>` for one file's
//! bytes — through the env-scrubbed `git_tree::cmd` doorway, and never per
//! frame: the shell memoizes it per snapshot like every other tab build
//! (§7.2 `SnapMemo`), which is the cost STORIES §S7 point 3 declined to pay
//! per-frame ("the per-frame git read this repo already removed once").
//!
//! Two differences from the live walk, both honest rather than incidental:
//! `ls-tree -r` names blobs, so the pinned listing has no directory rows; and
//! a commit whose tree cannot be read (a pin at a commit git no longer has)
//! reads as [`FilesView::AbsentWorktree`] — the same "nothing to list here"
//! value the torn-down worktree already produces, said once.

use std::path::Path;

use crate::files_view::{FileEntry, FilesView, MAX_ENTRIES, Preview, classify};
use crate::git_tree::{REPO_DIR, ls_tree_long, show_file};

/// `ls-tree -l` field count before the tab: `<mode> <type> <oid> <size>`.
const LS_FIELDS: usize = 4;
/// Names hidden at the tree root, exactly as the live walk hides them:
/// `messages/` is the Transcript tab's domain, and a bare tree has no `.git`.
const EXCLUDE_AT_ROOT: [&str; 1] = ["messages"];

/// The agent-context listing as of `commit` — the bounded, sorted shape the
/// Files tab already renders. `git ls-tree` emits path order; the live walk's
/// order is the sorted-sibling DFS pre-order, and sorting the paths gives the
/// same reading for a flat blob list (I9: two instances render identically).
pub fn files_at(workspace: &Path, commit: &str) -> FilesView {
    let Ok(out) = ls_tree_long(&workspace.join(REPO_DIR), commit) else {
        return FilesView::AbsentWorktree;
    };
    let mut entries: Vec<FileEntry> = String::from_utf8_lossy(&out)
        .lines()
        .filter_map(parse_row)
        .filter(|entry| !hidden_at_root(&entry.rel_path))
        .collect();
    entries.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
    let truncated = entries.len() > MAX_ENTRIES;
    entries.truncate(MAX_ENTRIES);
    FilesView::Present { entries, truncated }
}

/// One `<mode> <type> <oid> <size>\t<path>` row. A tree row (or any row whose
/// size is not a number — `ls-tree` writes `-` for one) contributes nothing:
/// this listing is blobs, and a size it cannot state it does not invent.
pub(super) fn parse_row(line: &str) -> Option<FileEntry> {
    let (head, path) = line.split_once('\t')?;
    let size = head.split_whitespace().nth(LS_FIELDS - 1)?.parse().ok()?;
    Some(FileEntry {
        rel_path: path.to_owned(),
        size,
        is_dir: false,
    })
}

/// Is this path under a root name the Files tab never lists?
fn hidden_at_root(rel_path: &str) -> bool {
    let root = rel_path.split('/').next().unwrap_or(rel_path);
    EXCLUDE_AT_ROOT.contains(&root)
}

/// One file's bytes as of `commit`, classified exactly as the live preview
/// classifies a file on disk — one vocabulary for "what this file is", never a
/// second. An unreadable path (deleted since, or never in this tree) reports
/// as an empty text preview rather than an error row: the listing this came
/// from is the same commit's, so a miss is a race, not a fact.
pub fn preview_at(workspace: &Path, commit: &str, path: &str) -> Preview {
    let bytes = show_file(&workspace.join(REPO_DIR), commit, path).unwrap_or_default();
    let size = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    classify(&bytes, size)
}
