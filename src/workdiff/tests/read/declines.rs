//! **Every way the read declines** rather than showing an empty list: a ref
//! that is not there is named, a project that is no repo is unreadable, and so
//! is a repo whose objects are damaged — the §4.10 mandate that a missing
//! project renders as a named absence, never a guess. Split from the reading
//! diff at §12's budget on the seam the parent's own doc draws: above is the
//! churn and the patch, here is the sentence each failure gets instead.

use std::path::Path;

use super::super::{MAIN, Project, ball, read0, snap};
use super::{NAME, WS, only, worked};
use crate::workdiff::{Change, WorkFile, patch};

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
