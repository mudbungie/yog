use super::*;
use std::fs;
use tempfile::TempDir;

/// Create a directory (and parents) at `rel` under `root`; return its path.
fn mkdir(root: &Path, rel: &str) -> PathBuf {
    let p = root.join(rel);
    fs::create_dir_all(&p).unwrap();
    p
}

/// Materialize a workspace: a directory at `rel` under `root` that directly
/// contains the `repo.git` marker (as a dir). Return the workspace dir.
fn workspace(root: &Path, rel: &str) -> PathBuf {
    let ws = mkdir(root, rel);
    fs::create_dir_all(ws.join(REPO_MARK)).unwrap();
    ws
}

/// A `WorkspaceKind::Named` with the given name.
fn named(name: &str) -> WorkspaceKind {
    WorkspaceKind::Named {
        name: name.to_owned(),
    }
}

#[test]
fn names_root_and_workspace_path_place_the_name_leaf() {
    let root = Path::new("/data/yog");
    assert_eq!(names_root(root), PathBuf::from("/data/yog/workspaces"));
    // The name *is* the path leaf (§3.1): no project, no ball id in the path.
    assert_eq!(
        workspace_path(root, "cobalt-gecko"),
        PathBuf::from("/data/yog/workspaces/cobalt-gecko"),
    );
}

#[test]
fn work_worktree_path_matches_bl_claim_layout() {
    let balls = Path::new("/home/u/.local/state/balls");
    let project = Path::new("/home/u/dev/yog");
    // No claimant: leaf is the bare ball id (the observed `bl claim` layout).
    assert_eq!(
        work_worktree_path(balls, project, "bl-32d3", None),
        PathBuf::from("/home/u/.local/state/balls/plugins/bl-delivery/home/u/dev/yog/bl-32d3"),
    );
    // Claimant present: leaf is `<ball-id>-<claimant>`.
    assert_eq!(
        work_worktree_path(balls, project, "bl-32d3", Some("filtered")),
        PathBuf::from(
            "/home/u/.local/state/balls/plugins/bl-delivery/home/u/dev/yog/bl-32d3-filtered"
        ),
    );
}

#[test]
fn workspaces_classifies_across_three_roots() {
    let tmp = TempDir::new().unwrap();
    let yog = tmp.path().join("yog");
    let lernie = tmp.path().join("lernie");

    // A named workspace under yog's flat names root: the leaf is the name.
    let cobalt = workspace(&yog, "workspaces/cobalt-gecko");
    // Foreign (lernie auto-id) and a replay.
    let foreign = workspace(&lernie, "workspaces/01ABC");
    let replay = workspace(&lernie, "replays/99XYZ");
    // A names-root child without `repo.git`: skipped (not a workspace).
    mkdir(&yog, "workspaces/half-built");
    // A `repo.git` two levels deep under the names root: NOT enumerated — the
    // names root is flat, read one level (the workspace's own agents/ subtree
    // never contributes a second, bogus workspace).
    workspace(&yog, "workspaces/cobalt-gecko/agents/root/child");
    // A stray file among the entries: walked past.
    fs::write(yog.join("workspaces/NOTES"), b"x").unwrap();

    let got = workspaces(&yog, &lernie);
    assert_eq!(
        got,
        vec![
            Workspace {
                path: cobalt,
                kind: named("cobalt-gecko"),
            },
            Workspace {
                path: foreign,
                kind: WorkspaceKind::Foreign,
            },
            Workspace {
                path: replay,
                kind: WorkspaceKind::Replay,
            },
        ],
    );
}

#[test]
fn names_root_enumerates_multiple_names_sorted() {
    let tmp = TempDir::new().unwrap();
    let yog = tmp.path();
    let zephyr = workspace(yog, "workspaces/zephyr-mole");
    let amber = workspace(yog, "workspaces/amber-toad");

    let got = workspaces(yog, &yog.join("absent-lernie"));
    // Sorted by path: amber before zephyr, both Named by their leaf.
    assert_eq!(
        got,
        vec![
            Workspace {
                path: amber,
                kind: named("amber-toad"),
            },
            Workspace {
                path: zephyr,
                kind: named("zephyr-mole"),
            },
        ],
    );
}

#[test]
fn missing_roots_yield_empty() {
    let tmp = TempDir::new().unwrap();
    let yog = tmp.path().join("absent-yog");
    let lernie = tmp.path().join("absent-lernie");
    // Exercises the `read_dir` early-return in the flat enumerator.
    assert!(workspaces(&yog, &lernie).is_empty());
}
