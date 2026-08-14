//! Table-driven allowlist tests, one case list per `RootKind` (DESIGN §7.1).
//! The Workspace cases preserve the original hardcoded-allowlist assertions.

use super::*;
use std::path::Path;

const ROOT: &str = "/r";

/// Assert every path in `admit` is watched and every path in `reject` is not,
/// each resolved as a child of `ROOT`.
fn check(kind: RootKind, admit: &[&str], reject: &[&str]) {
    let r = Path::new(ROOT);
    for a in admit {
        assert!(is_watched(kind, r, &r.join(a)), "{kind:?} should admit {a}");
    }
    for x in reject {
        assert!(
            !is_watched(kind, r, &r.join(x)),
            "{kind:?} should reject {x}"
        );
    }
}

#[test]
fn workspace_allowlist() {
    check(
        RootKind::Workspace,
        &[
            "steps",
            "steps/abc-1/001/request.json",
            "inbox",
            "inbox/aa-bb/user-001.md",
            "repo.git/HEAD",
            "repo.git/refs",
            "repo.git/refs/heads/agents/aa-bb",
            "agents/aa-bb/goal.md",
            "agents/aa-bb/summary/001.md",
            "agents/20260424T120000Z-deadbeef/messages",
            "agents/20260424T120000Z-deadbeef/skills/child",
        ],
        &[
            "README.md",
            "random/x.txt",
            "aa-bb/random.txt",
            "agents/aa-bb/notes/x.md",
            // A bare `agents/<id>` file (no per-worktree tail) is not watched.
            "agents/aa-bb",
        ],
    );
}

#[test]
fn workspace_admits_every_worktree_prefix_under_any_agent_id() {
    let r = Path::new(ROOT);
    for prefix in WORKTREE_PREFIXES {
        for id in ["aa-bb", "20260424T120000Z-deadbeef"] {
            let path = r.join("agents").join(id).join(prefix);
            assert!(is_watched(RootKind::Workspace, r, &path), "{id}/{prefix}");
            assert!(is_watched(RootKind::Workspace, r, &path.join("child")));
        }
    }
}

#[test]
fn coarse_roots_admit_any_descendant_but_not_the_root() {
    for kind in [RootKind::NamesRoot, RootKind::WorkspacesRoot] {
        check(
            kind,
            &[
                "someproject",
                "acme/webapp/bl-1234",
                "replays/foo",
                "a/b/c/d",
            ],
            &[],
        );
        // The root itself (empty relative path) is never a change of interest.
        let r = Path::new(ROOT);
        assert!(!is_watched(kind, r, r), "{kind:?} admitted its own root");
    }
}

#[test]
fn balls_clones_allowlist() {
    check(
        RootKind::BallsClones,
        &[
            // Clone dir create/remove (single segment).
            "clone-a",
            // Task files: tasks/tasks/*.md.
            "clone-a/tasks/tasks/bl-1234.md",
            // Landing config subtree: config/config/**.
            "clone-a/config/config",
            "clone-a/config/config/hooks/pre-commit",
        ],
        &[
            // The multi-MB unrotated per-clone log is filtered out.
            "clone-a/log",
            "clone-a/log/rotated.1",
            // tasks/tasks/*.md is one level of .md files: nested or
            // non-.md names are rejected.
            "clone-a/tasks/tasks/sub/nested.md",
            "clone-a/tasks/tasks/notes.txt",
            // The intermediate task dir itself is not a task file.
            "clone-a/tasks/tasks",
            "clone-a/tasks",
            // The outer config dir alone is not the config/config subtree.
            "clone-a/config",
            "clone-a/random.txt",
        ],
    );
}

#[test]
fn yog_state_allowlist() {
    check(
        RootKind::YogState,
        &["ui.json", "ops.jsonl", "cadence.yaml"],
        &[
            "ui.json.tmp",
            ".ui.json.yog-tmp-1234",
            "ops.jsonl.1",
            "other.json",
            "sub/ui.json",
        ],
    );
}

#[test]
fn paths_outside_the_root_are_rejected_for_every_kind() {
    let r = Path::new(ROOT);
    let outside = Path::new("/other/manifest.yaml");
    for kind in [
        RootKind::Workspace,
        RootKind::NamesRoot,
        RootKind::WorkspacesRoot,
        RootKind::BallsClones,
        RootKind::YogState,
    ] {
        assert!(
            !is_watched(kind, r, outside),
            "{kind:?} admitted {outside:?}"
        );
    }
}
