//! Project (balls-clone) enumeration and nested-delivery detection
//! (DESIGN §5.1 #1, §15 Y14).
//!
//! A **project** is one balls invocation path — the `bl` control plane keys its
//! per-project state at `$XDG_STATE_HOME/balls/clones/<percent-encoded-path>/`
//! (balls arch §1). The project's *identity* is the decoded path, and it is the
//! cwd every `bl … --json` runs in ([`balls`], DESIGN §5.1 #2). Enumeration is
//! therefore `readdir + percent-decode basename` — one query, nothing stored.
//!
//! **Nested-delivery detection** (§5.1 #1): a decoded path that itself lies
//! under the balls `plugins/bl-delivery/` tree is a ball's own work-worktree
//! that became a balls project — an *internal* clone. [`visible`] drops them,
//! unconditionally.
//!
//! **There is no toggle (bl-e3e7).** There was: an "internal clones" checkbox
//! at the top of the §11 balls section, backed by a `ui.json` boolean and a
//! re-read in the derivation worker. It was deleted, not renamed, because the
//! set it revealed is never a thing to work in. Such a store exists only
//! because something ran `bl` with its cwd inside a `work/<id>` worktree, and
//! balls' own guide says that addresses "a *different* (usually empty) store";
//! the clone dir lives under `clones/`, *outside* the worktree, so it outlives
//! the `bl close` that tears the worktree down. Revealing them therefore put
//! phantom projects on the roster — each with a new-ball form that would file
//! a ball into a throwaway store, and each becoming an orphaned-project row
//! once its worktree was gone. No yog verb ever acted on one. A label cannot
//! rescue a view whose ON state is a trap, so the toggle went with it.

pub mod balls;
pub mod join;
pub mod runner;

use crate::xdg::percent_decode;
use std::path::{Path, PathBuf};

/// One enumerated balls project (§5.1 #1). `path` is the decoded invocation
/// path — the identity and the `bl` cwd; `internal` flags a nested-delivery
/// clone (hidden by default, [`visible`]). Both are derived from the clone
/// basename alone — nothing about the project is stored by yog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub path: PathBuf,
    pub internal: bool,
}

/// The bl-delivery territory a nested-delivery clone lives under, derived from
/// the clones dir (its parent is the balls state root; balls arch §1). `None`
/// only when `clones_dir` has no parent — then no path can be internal.
fn delivery_root(clones_dir: &Path) -> Option<PathBuf> {
    clones_dir
        .parent()
        .map(|state| state.join("plugins").join("bl-delivery"))
}

/// True iff `project` (a decoded invocation path) lies under the bl-delivery
/// tree — a ball's own worktree that became a balls project (§5.1 #1).
fn is_internal(project: &Path, delivery_root: Option<&Path>) -> bool {
    delivery_root.is_some_and(|d| project.starts_with(d))
}

/// Enumerate every balls project under `clones_dir` (§5.1 #1): each child dir's
/// basename percent-decodes to the project path, flagged `internal` when it
/// falls under bl-delivery. Sorted by path for a stable roster. A missing or
/// unreadable clones dir yields an empty Vec — the general path with no inputs,
/// not a bootstrap special case (the [`crate::binding`] discipline).
pub fn enumerate(clones_dir: &Path) -> Vec<Project> {
    let delivery = delivery_root(clones_dir);
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(clones_dir) else {
        return out;
    };
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        let path = PathBuf::from(percent_decode(&name));
        let internal = is_internal(&path, delivery.as_deref());
        out.push(Project { path, internal });
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}

/// The longest a project's roster label may run before it elides (§11): the
/// left panel is a column of names, and a name this long has stopped naming.
const LABEL_MAX: usize = 32;

/// The roster label for each of `paths`, in order (§11, bl-ac3d): the shortest
/// trailing run of components that tells a project apart from every other one
/// enumerated — its basename wherever that is already unique — elided at
/// [`LABEL_MAX`] characters.
///
/// The label is **cosmetic**: the path is the project's identity (§5.1 #1) and
/// stays one hover away, so two labels that elide alike cost nothing. Rendering
/// the path itself is what sized the whole left panel to the widest project a
/// scratch directory could name.
pub fn labels(paths: &[PathBuf]) -> Vec<String> {
    paths
        .iter()
        .map(|path| elide(&shortest_unique(path, paths)))
        .collect()
}

/// The shortest trailing suffix of `path` that no *other* path in `paths`
/// shares, falling back to the whole path when the set holds it twice — then
/// nothing shorter could separate them, and nothing needs to.
fn shortest_unique(path: &Path, paths: &[PathBuf]) -> String {
    let depth = path.components().count();
    (1..=depth)
        .map(|k| (suffix(path, k), k))
        .find(|(cand, k)| paths.iter().filter(|o| &suffix(o, *k) == cand).count() == 1)
        .map_or_else(|| suffix(path, depth), |(cand, _)| cand)
}

/// `path`'s last `k` components, rendered — the whole path when it is shorter.
fn suffix(path: &Path, k: usize) -> String {
    let total = path.components().count();
    path.components()
        .skip(total.saturating_sub(k))
        .collect::<PathBuf>()
        .display()
        .to_string()
}

/// `s` capped at [`LABEL_MAX`] characters, head kept and the cut marked — the
/// house spelling of a clipped preview (`git_tree::detect::truncate_preview`).
fn elide(s: &str) -> String {
    if s.chars().count() <= LABEL_MAX {
        return s.to_owned();
    }
    let head: String = s.chars().take(LABEL_MAX - 1).collect();
    format!("{head}…")
}

/// The projects a surface shows (§5.1 #1): the non-internal clones. A
/// nested-delivery clone is never one of them — see the module doc for why the
/// operator toggle that used to reveal them was deleted (bl-e3e7).
pub fn visible(projects: &[Project]) -> Vec<&Project> {
    projects.iter().filter(|p| !p.internal).collect()
}

#[cfg(test)]
mod tests;
