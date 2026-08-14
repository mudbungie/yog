//! Workspace enumeration across the three roots (DESIGN §3.1). Everything here
//! is a pure function of injected roots plus a bounded directory walk — no env
//! reads (roots come from `crate::xdg`), no writes.
//!
//! §3.1: a workspace is exactly a directory holding `repo.git`. Under yog's own
//! **flat names root** its leaf *is* the chosen name; under lernie's roots it is
//! foreign (`workspaces/`, auto-id) or a read-only replay (`replays/`). Three
//! roots, one shape, classification by path alone. The names root is flat by
//! construction — names are direct children — so each root is one readdir.
//!
//! The binding itself is **not** here: a ball is bound to a workspace iff its
//! claimant equals the workspace name (§3.2), joined in
//! [`crate::projects::join`]. The superseded mirrored-path convention
//! (`<yog_data>/balls/<project>/<ball_id>/`) is dead — a location cannot be
//! reassigned but a claim can, and assignment is late-mutable (§3.1/§3.2).

use std::path::{Path, PathBuf};

/// The marker a workspace directory directly contains (§3.1); a workspace is
/// exactly a directory holding this. Matches `git_tree`'s `REPO_DIR`.
const REPO_MARK: &str = "repo.git";

/// A discovered workspace directory and its classification (§3.1). Derived from
/// the tree, never stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    /// The workspace directory — the dir directly containing `repo.git`.
    pub path: PathBuf,
    pub kind: WorkspaceKind,
}

/// How a workspace is classified, decided by the root it sits under (§3.1).
/// Three roots, one shape, classification by path alone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceKind {
    /// Under yog's flat names root (`<yog_data>/workspaces/`): the dir leaf is
    /// the chosen name — the workspace's identity **and** the `--as` identity of
    /// every ball claim it makes (§3.1/§3.2).
    Named { name: String },
    /// Under `<lernie-data>/workspaces/` — lernie's auto-id territory, rendered
    /// unnamed, never created by yog (§3.1).
    Foreign,
    /// Under `<lernie-data>/replays/` — a read-only replay workspace (§3.1).
    Replay,
}

/// yog's flat names root (§3.1): `<yog_data>/workspaces/`, the directory whose
/// leaf-named children are yog's own named workspaces. The single home of the
/// `workspaces` path segment, shared by [`workspace_path`] and the app's
/// NamesRoot watch (§7.1).
pub fn names_root(yog_data_root: &Path) -> PathBuf {
    yog_data_root.join("workspaces")
}

/// Forward derivation (§3.1/§3.2): the workspace directory a `name` binds to —
/// `<yog_data>/workspaces/<name>/`. The name *is* the path leaf; there is no
/// stored home. A ball bound to this workspace is any ball whose claimant equals
/// `name` (§3.2, joined in [`crate::projects::join`]).
pub fn workspace_path(yog_data_root: &Path, name: &str) -> PathBuf {
    names_root(yog_data_root).join(name)
}

/// Place a leaf under a territory root by the verbatim project mirror
/// (balls arch §11): `<territory>/<project, leading '/' stripped>/<leaf>/`. The
/// convention the bl-delivery worktree root is built on. A relative project
/// mirrors through unchanged (its leading `/` is absent).
fn under(territory: &Path, project: &Path, leaf: &str) -> PathBuf {
    let mirror = project.strip_prefix("/").unwrap_or(project);
    territory.join(mirror).join(leaf)
}

/// The bl-delivery work-worktree path for a claim (§3.3, balls arch §11):
/// `<balls_state_root>/plugins/bl-delivery/<project verbatim>/<leaf>/`, where
/// the leaf is `<ball_id>` or, when a claimant disambiguates, `<ball_id>-<claimant>`.
/// Pure — the path is computed, never stored; cross-checked against `bl claim`
/// stdout at claim time.
pub fn work_worktree_path(
    balls_state_root: &Path,
    project: &Path,
    ball_id: &str,
    claimant: Option<&str>,
) -> PathBuf {
    let leaf = match claimant {
        Some(c) => format!("{ball_id}-{c}"),
        None => ball_id.to_owned(),
    };
    let delivery = balls_state_root.join("plugins").join("bl-delivery");
    under(&delivery, project, &leaf)
}

/// How each of [`roots`]'s three roots classifies its leaves (§3.1), positional
/// with that list — one root, one kind, no third place naming either.
const KINDS: [fn(&str) -> WorkspaceKind; 3] = [
    |leaf| WorkspaceKind::Named {
        name: leaf.to_owned(),
    },
    |_| WorkspaceKind::Foreign,
    |_| WorkspaceKind::Replay,
];

/// The three workspace roots (§3.1), in classification order: yog's flat names
/// root, then lernie's `workspaces/` (foreign) and `replays/`. The single home
/// of that list — [`workspaces`] enumerates it, and
/// [`names::validate`](crate::names::validate) refuses a name equal to any leaf
/// under it, so creation and enumeration can never disagree about where names
/// live.
pub fn roots(yog_data_root: &Path, lernie_data_root: &Path) -> Vec<PathBuf> {
    vec![
        names_root(yog_data_root),
        lernie_data_root.join("workspaces"),
        lernie_data_root.join("replays"),
    ]
}

/// Every workspace across the three roots (§3.1), tagged with its kind: named
/// (yog's flat names root, leaf = name), then foreign and replay (lernie's flat
/// `workspaces/` and `replays/`). Each root is one readdir, sorted by path; a
/// missing root contributes nothing — the general path with no inputs, not a
/// bootstrap special case.
pub fn workspaces(yog_data_root: &Path, lernie_data_root: &Path) -> Vec<Workspace> {
    let mut out = Vec::new();
    for (root, classify) in roots(yog_data_root, lernie_data_root).iter().zip(KINDS) {
        enumerate_flat(root, classify, &mut out);
    }
    out
}

/// The name of the workspace at `path`, iff it is one of yog's own
/// (§3.1: the leaf *is* the name). `None` for a foreign, replay, or unknown
/// path — none of which can carry a claimant identity, so none of which can
/// bind a ball (§3.2). The one home of that question: the §3.6 delete gate,
/// the §3.5 join's own binding and the Work tab's attempt lookup all ask it
/// here rather than each re-matching the enum.
pub fn named_of(workspaces: &[Workspace], path: &Path) -> Option<String> {
    workspaces
        .iter()
        .find(|w| w.path == path)
        .and_then(|w| match &w.kind {
            WorkspaceKind::Named { name } => Some(name.clone()),
            WorkspaceKind::Foreign | WorkspaceKind::Replay => None,
        })
}

/// True when `dir` directly contains the `repo.git` marker.
fn is_workspace(dir: &Path) -> bool {
    dir.join(REPO_MARK).exists()
}

/// One-level (flat) enumeration: direct children of `dir` that are workspaces,
/// each tagged by `classify` over its leaf name. An absent `dir` contributes
/// nothing. Sorted by path for a stable, determinism-derived roster (I9).
/// `classify` is a bare `fn` (the call sites capture nothing) so the enumerator
/// has one instantiation — no monomorphized-per-closure llvm-cov phantom (§12.1).
fn enumerate_flat(dir: &Path, classify: fn(&str) -> WorkspaceKind, out: &mut Vec<Workspace>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut found: Vec<Workspace> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir() && is_workspace(p))
        .map(|path| {
            // A readdir entry always has a leaf; `_lossy` keeps a non-UTF-8 name
            // (foreign/replay ids are ASCII, chosen names UTF-8 by construction).
            let leaf = path.file_name().unwrap_or_default().to_string_lossy();
            Workspace {
                kind: classify(leaf.as_ref()),
                path,
            }
        })
        .collect();
    found.sort_by(|a, b| a.path.cmp(&b.path));
    out.extend(found);
}

#[cfg(test)]
mod tests;
