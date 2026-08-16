//! **S11-T2 project-read**: `target..source` against a real project repo —
//! the churn, the patch, and every way the read declines rather than showing
//! an empty list.

use std::path::{Path, PathBuf};

use super::{MAIN, Project, ball, close_gate, read0, snap};
use crate::files_view::Preview;
use crate::workdiff::{Attempt, Change, Churn, WorkFile, patch};

const WS: &str = "/data/workspaces/storeroom";
const NAME: &str = "storeroom";

/// The one attempt of a snapshot, or the panic that says there wasn't one.
fn only(attempts: Vec<Attempt>) -> Attempt {
    assert_eq!(attempts.len(), 1, "{attempts:?}");
    attempts.into_iter().next().unwrap()
}

/// A repo with `work/bl-1` carrying two commits off `main`.
fn worked() -> Project {
    let project = Project::new();
    project.switch("work/bl-1");
    project.commit("src/a.rs", "fn a() {}\n");
    project.commit("docs/b.md", "b\n");
    // A project repo sits on its integration branch; the work happens in the
    // claim's own worktree, which is a checkout of `work/<id>` elsewhere.
    project.checkout(MAIN);
    project
}

/// The read is exactly the ruling: everything on the claim's branch that the
/// delivery target does not have, per file, with the two commits named.
#[test]
fn the_diff_is_target_dot_dot_source_of_the_bound_claim() {
    let project = worked();
    let snap = snap(
        Path::new(WS),
        NAME,
        &project.path,
        vec![ball("bl-1", Some(NAME), None)],
    );
    let attempt = only(read0(&snap, Path::new(WS)));
    assert_eq!(attempt.ball_id, "bl-1");
    // **It names the project, it does not locate it** (REMOTE §8, bl-ccf7):
    // the §5.1 #1 wire name, which is the word `--project` takes and which
    // resolves back to the repository the patch read runs in — and never the
    // absolute path, which a client on another machine could not use.
    assert_eq!(attempt.project, snap.project_name(&project.path));
    assert!(!attempt.project.contains(std::path::MAIN_SEPARATOR));
    assert_eq!(
        snap.project_path(&attempt.project),
        Ok(project.path.clone())
    );
    assert_eq!(attempt.range().as_deref(), Some("main..work/bl-1"));
    let Change::Diff {
        target,
        source,
        target_oid,
        source_oid,
        files,
        truncated,
    } = &attempt.change
    else {
        panic!("a worked branch diffs: {attempt:?}");
    };
    assert_eq!((target.as_str(), source.as_str()), (MAIN, "work/bl-1"));
    assert_ne!(target_oid, source_oid);
    assert_eq!(target_oid.len(), 40, "a full oid: {target_oid}");
    assert!(!truncated);
    let paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(paths, vec!["docs/b.md", "src/a.rs"]);
    assert_eq!(
        files[1].churn,
        Churn::Text {
            added: 1,
            removed: 0
        }
    );
}

/// A close-gating child reads against its parent's branch, not against the
/// integration branch — the same target `bl close` would deliver to.
#[test]
fn a_close_gated_child_reads_against_its_parents_branch() {
    let project = Project::new();
    project.switch("work/bl-parent");
    project.commit("parent.txt", "parent work\n");
    project.switch("work/bl-kid");
    project.commit("kid.txt", "kid work\n");
    let snap = snap(
        Path::new(WS),
        NAME,
        &project.path,
        vec![
            close_gate(ball("bl-parent", None, None), "bl-kid"),
            ball("bl-kid", Some(NAME), Some("bl-parent")),
        ],
    );
    let attempt = only(read0(&snap, Path::new(WS)));
    assert_eq!(
        attempt.range().as_deref(),
        Some("work/bl-parent..work/bl-kid")
    );
    let Change::Diff { files, .. } = &attempt.change else {
        panic!("{attempt:?}");
    };
    let paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(
        paths,
        vec!["kid.txt"],
        "the parent's own work is not the kid's"
    );
}

/// A claim whose branch was never worked resolves both ends and finds nothing
/// between them — which is a different sentence from every failure below.
#[test]
fn a_branch_with_no_work_diffs_to_nothing() {
    let project = Project::new();
    project.switch("work/bl-1");
    project.checkout(MAIN);
    let snap = snap(
        Path::new(WS),
        NAME,
        &project.path,
        vec![ball("bl-1", Some(NAME), None)],
    );
    let Change::Diff { files, .. } = only(read0(&snap, Path::new(WS))).change else {
        panic!("both ends resolve");
    };
    assert!(files.is_empty());
}

/// A ref that is not there is named, never guessed at: no work branch yet, and
/// no such target branch either.
#[test]
fn a_ref_that_is_not_there_is_named() {
    let project = Project::new();
    let snap = snap(
        Path::new(WS),
        NAME,
        &project.path,
        vec![ball("bl-1", Some(NAME), None)],
    );
    let attempt = only(read0(&snap, Path::new(WS)));
    let Change::Absent {
        target,
        source,
        missing,
    } = &attempt.change
    else {
        panic!("an unminted work branch is absent: {attempt:?}");
    };
    assert_eq!((target.as_str(), source.as_str()), (MAIN, "work/bl-1"));
    assert_eq!(missing, &vec!["work/bl-1".to_owned()]);
    assert_eq!(attempt.range().as_deref(), Some("main..work/bl-1"));
    // No patch can be read at a range that does not resolve.
    assert!(
        patch(
            &snap,
            std::slice::from_ref(&attempt),
            &WorkFile {
                ball: "bl-1".to_owned(),
                handle: None,
                path: "src/a.rs".to_owned(),
            }
        )
        .is_none()
    );
}

/// A project whose repo cannot be read says so — the §4.10 mandate that a
/// missing project renders as a named absence, never a guess.
#[test]
fn a_project_that_is_not_a_repo_is_unreadable() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().to_path_buf();
    let snap = snap(
        Path::new(WS),
        NAME,
        &project,
        vec![ball("bl-1", Some(NAME), None)],
    );
    let attempt = only(read0(&snap, Path::new(WS)));
    assert_eq!(attempt.change, Change::Unreadable);
    assert_eq!(attempt.range(), None, "there is no range to state");
    assert!(
        patch(
            &snap,
            &[attempt],
            &WorkFile {
                ball: "bl-1".to_owned(),
                handle: None,
                path: "x".to_owned(),
            }
        )
        .is_none()
    );
}

/// A repo whose refs still resolve but whose objects are damaged is unreadable
/// too: the diff cannot be stated, so it is not stated — the same sentence as
/// a missing repo, because the operator's position is the same.
#[test]
fn a_repo_that_cannot_diff_is_unreadable() {
    let project = worked();
    let snap = snap(
        Path::new(WS),
        NAME,
        &project.path,
        vec![ball("bl-1", Some(NAME), None)],
    );
    let Change::Diff {
        target_oid,
        source_oid,
        ..
    } = only(read0(&snap, Path::new(WS))).change
    else {
        panic!("the repo reads before it is damaged");
    };
    // Keep the two commits the refs name and remove every other object: both
    // ends still resolve, and there are no trees left to compare.
    keep_only(&project.path, &[target_oid, source_oid]);
    assert_eq!(only(read0(&snap, Path::new(WS))).change, Change::Unreadable);
}

/// Delete every loose object in `repo` except the named oids.
fn keep_only(repo: &Path, keep: &[String]) {
    let objects = repo.join(".git/objects");
    for shard in std::fs::read_dir(&objects).unwrap().flatten() {
        let name = shard.file_name().to_string_lossy().into_owned();
        if name.len() != 2 {
            continue;
        }
        for object in std::fs::read_dir(shard.path()).unwrap().flatten() {
            let oid = format!("{name}{}", object.file_name().to_string_lossy());
            if !keep.contains(&oid) {
                std::fs::remove_file(object.path()).unwrap();
            }
        }
    }
}

/// One file's patch comes back through the Files tab's own vocabulary; a ball
/// this workspace does not hold has no patch to read.
#[test]
fn a_picked_file_reads_its_patch() {
    let project = worked();
    let snap = snap(
        Path::new(WS),
        NAME,
        &project.path,
        vec![ball("bl-1", Some(NAME), None)],
    );
    let attempts = read0(&snap, Path::new(WS));
    let file = |ball: &str, path: &str| WorkFile {
        ball: ball.to_owned(),
        handle: None,
        path: path.to_owned(),
    };
    let Some(Preview::Text(text)) = patch(&snap, &attempts, &file("bl-1", "src/a.rs")) else {
        panic!("a changed file has a patch");
    };
    assert!(text.contains("+fn a() {}"), "{text}");
    assert!(text.contains("src/a.rs"), "{text}");
    assert!(patch(&snap, &attempts, &file("bl-other", "src/a.rs")).is_none());
}

/// A workspace that holds no ball owes no work anywhere, and neither does a
/// foreign or replay one — an empty answer, and the same empty answer.
#[test]
fn a_workspace_with_no_claim_has_no_attempt() {
    let project = Project::new();
    let ws = PathBuf::from(WS);
    let unclaimed = snap(&ws, NAME, &project.path, vec![ball("bl-1", None, None)]);
    assert!(read0(&unclaimed, &ws).is_empty());
    // A path the snapshot does not carry as one of yog's own named workspaces
    // cannot claim anything, so it has nothing to compare.
    assert!(read0(&unclaimed, Path::new("/data/workspaces/other")).is_empty());
}
