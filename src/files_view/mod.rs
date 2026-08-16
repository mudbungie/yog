//! Agent-worktree file view-model (DESIGN §11 Altitude-2 Files tab).
//!
//! The Files tab is the agent worktree read-only: goal.md, soul.md,
//! summary/NNN.md, descriptions/, skills/, and arbitrary work products at
//! `<workspace>/agents/<agent-id>/` (ARCH §2.3). Yog is a pure reader (§3.5):
//! [`build`] walks that tree and [`preview`] reads one file, both pure over
//! injected paths, deriving nothing they can read.
//!
//! **Two names are excluded at the worktree root** (single source of truth,
//! no redundant path): `messages/` is the Transcript tab's domain (§11), and
//! `.git` is the worktree's git plumbing, never a work product. A `messages`
//! or `.git` *deeper* in a work-product tree is ordinary and kept.
//!
//! **The walk is bounded** so a runaway work-product tree never hangs a
//! frame: at most [`MAX_ENTRIES`] entries and [`MAX_DEPTH`] path components;
//! hitting either sets the `truncated` marker. Ordering is the sorted-sibling
//! DFS pre-order — deterministic (I9), so two instances render identically.
//!
//! **The worktree is disposable materialization.** A quiescent agent's
//! worktree is torn down; its absence is [`FilesView::AbsentWorktree`], a fact
//! rendered plainly, never an error.

use std::io::Read;
use std::path::{Path, PathBuf};

mod render;
pub(crate) mod wire;
/// The bounded-bytes painter, shared with the Steps drill-in's capture-log
/// seats (§11, bl-83d6): one wording for the cap, the truncation and the binary
/// verdict, wherever a whole file is shown.
pub(crate) use render::preview_body;
pub use render::render;

/// Workspace subdir holding the per-agent worktrees (ARCH §2.3).
const AGENTS_DIR: &str = "agents";
/// Names hidden at the worktree root only: the Transcript tab owns `messages`
/// (§11), and `.git` is worktree plumbing, not a work product.
const EXCLUDE_AT_ROOT: [&str; 2] = [".git", "messages"];
/// Max path components in a listed entry; deeper dirs are not descended (a
/// non-empty one sets the truncation marker). Root children are depth 1.
const MAX_DEPTH: usize = 6;
/// Max entries in one listing; the walk stops and marks truncation past it.
/// `pub(crate)` because the pinned listing (`rail::files_at`, VISION V1.2) is
/// the same tab read out of git and takes the same bound — two caps on one
/// listing would be two answers to "how much does this tab show".
pub(crate) const MAX_ENTRIES: usize = 500;
/// Bytes read for a file preview — the NUL sniff and text both work on this
/// bounded window, so a huge work product never loads whole.
pub(crate) const PREVIEW_CAP: usize = 64 * 1024;

/// One walked worktree entry. `rel_path` is `/`-joined from the worktree root
/// (the entry's identity); `size` is the file's byte length (0 for a dir).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    pub rel_path: String,
    pub size: u64,
    pub is_dir: bool,
}

/// The Files tab's view-model. `Present` is the bounded listing; the disposable
/// worktree's absence is a first-class value (§3.5), the default.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum FilesView {
    /// The materialized worktree's sorted listing; `truncated` iff a cap was hit.
    Present {
        entries: Vec<FileEntry>,
        truncated: bool,
    },
    /// No worktree on disk — the agent is quiescent / torn down.
    #[default]
    AbsentWorktree,
}

/// A bounded file preview (DESIGN §11). Binary is detected by a NUL byte in
/// the read window; `Truncated` carries the leading [`PREVIEW_CAP`] bytes plus
/// the true size. An unopenable entry (a race against the disposable worktree's
/// teardown) reports as opaque [`Preview::Binary`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Preview {
    Text(String),
    Binary { size: u64 },
    Truncated { text: String, size: u64 },
}

/// The agent's worktree directory, `<workspace>/agents/<agent-id>/`.
pub fn agent_worktree(workspace: &Path, agent_id: &str) -> PathBuf {
    workspace.join(AGENTS_DIR).join(agent_id)
}

/// Walk `agent_id`'s worktree into a bounded, sorted [`FilesView`]. A missing
/// worktree is [`FilesView::AbsentWorktree`]; an empty one is `Present` with no
/// entries.
pub fn build(workspace: &Path, agent_id: &str) -> FilesView {
    let root = agent_worktree(workspace, agent_id);
    if !root.is_dir() {
        return FilesView::AbsentWorktree;
    }
    let mut entries = Vec::new();
    let mut truncated = false;
    walk(&root, "", 1, &mut entries, &mut truncated);
    FilesView::Present { entries, truncated }
}

/// List `dir`'s children (this call's entries are at `depth`, root children =
/// 1), sorted, appending to `entries`. Descends a dir while `depth < MAX_DEPTH`;
/// a non-empty dir at the depth floor, or a full listing, sets `truncated`.
fn walk(
    dir: &Path,
    prefix: &str,
    depth: usize,
    entries: &mut Vec<FileEntry>,
    truncated: &mut bool,
) {
    for (name, path) in sorted_children(dir, prefix.is_empty()) {
        if entries.len() >= MAX_ENTRIES {
            *truncated = true;
            return;
        }
        let (is_dir, size) = entry_meta(&path);
        let rel_path = if prefix.is_empty() {
            name
        } else {
            format!("{prefix}/{name}")
        };
        entries.push(FileEntry {
            rel_path: rel_path.clone(),
            size,
            is_dir,
        });
        if is_dir {
            if depth < MAX_DEPTH {
                walk(&path, &rel_path, depth + 1, entries, truncated);
            } else if has_children(&path) {
                *truncated = true;
            }
        }
    }
}

/// Classify one entry via `symlink_metadata` (symlinks are never followed, so
/// the walk can never escape the worktree or loop): `(is_dir, size)`, dirs
/// sized 0. An unstattable entry (a teardown race) reads as an empty file.
fn entry_meta(path: &Path) -> (bool, u64) {
    match path.symlink_metadata() {
        Ok(meta) if meta.is_dir() => (true, 0),
        Ok(meta) => (false, meta.len()),
        Err(_) => (false, 0),
    }
}

/// `dir`'s `(name, path)` children sorted by name, dropping unnameable entries
/// and — when `at_root` — the [`EXCLUDE_AT_ROOT`] names. An unreadable dir is
/// an empty listing.
fn sorted_children(dir: &Path, at_root: bool) -> Vec<(String, PathBuf)> {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut children: Vec<(String, PathBuf)> = rd
        .flatten()
        .filter_map(|e| Some((e.file_name().to_str()?.to_string(), e.path())))
        .filter(|(name, _)| !(at_root && EXCLUDE_AT_ROOT.contains(&name.as_str())))
        .collect();
    children.sort();
    children
}

/// Whether `dir` has at least one readable entry — the honest truncation test
/// for a directory sitting at the depth floor.
fn has_children(dir: &Path) -> bool {
    std::fs::read_dir(dir).is_ok_and(|mut rd| rd.next().is_some())
}

/// Preview `file`: read up to [`PREVIEW_CAP`]+1 bytes and classify them at the
/// file's stat size. An unopenable file reports as `Binary` at that size.
pub fn preview(file: &Path) -> Preview {
    let size = std::fs::symlink_metadata(file).map_or(0, |m| m.len());
    let Ok(handle) = std::fs::File::open(file) else {
        return Preview::Binary { size };
    };
    let mut buf = Vec::new();
    let _ = handle.take(PREVIEW_CAP as u64 + 1).read_to_end(&mut buf);
    classify(&buf, size)
}

/// **What this file is**, said once for every seat that asks it (§11): a NUL
/// in the window ⇒ [`Preview::Binary`]; more bytes than [`PREVIEW_CAP`] ⇒
/// [`Preview::Truncated`] (the leading cap bytes plus the true `size`); else
/// [`Preview::Text`]. `size` is the whole thing's size, which the caller knows
/// and a bounded window cannot: on disk it is the stat size, out of git the
/// blob's own length. The Files tab's live walk, the pinned tree's `git show`
/// (VISION V1.2) and the Work tab's patch all classify here — one vocabulary,
/// never three.
pub(crate) fn classify(bytes: &[u8], size: u64) -> Preview {
    if bytes.contains(&0) {
        return Preview::Binary { size };
    }
    // Under the cap `get(..CAP)` is `None`, and the whole buffer is the window.
    let window = bytes.get(..PREVIEW_CAP).unwrap_or(bytes);
    let text = String::from_utf8_lossy(window).into_owned();
    if bytes.len() > PREVIEW_CAP {
        Preview::Truncated { text, size }
    } else {
        Preview::Text(text)
    }
}

#[cfg(test)]
mod tests;
