//! The retention policy: the entry grammar, the expiry rule, and the age read
//! against a real attempt ref.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tempfile::tempdir;

use super::{age, expired, keep};
use crate::git_tree::tests::git::{git_out, run_git};

const PROJECT: &str = "/dev/proj";

/// A `cadence.yaml` with one project's retention declared.
fn declared(minutes: &str) -> String {
    format!("cadence:\n  cheap_ms: 400\nretention:\n  {PROJECT}:\n    keep_min: {minutes}\n")
}

#[test]
fn absence_is_never_discard_at_every_shape_of_absence() {
    let project = Path::new(PROJECT);
    // No file, no block, another project's entry, a declared-but-empty value,
    // and a value that is not a number: every one of them keeps the ref.
    for text in [
        String::new(),
        "cadence:\n  cheap_ms: 400\n".to_owned(),
        "retention:\n  /dev/other:\n    keep_min: 10\n".to_owned(),
        declared(""),
        declared("soon"),
    ] {
        assert_eq!(keep(&text, project), None, "{text:?}");
    }
}

#[test]
fn a_declared_keep_is_read_in_whole_minutes() {
    assert_eq!(
        keep(&declared("1440"), Path::new(PROJECT)),
        Some(Duration::from_hours(24)),
    );
}

#[test]
fn nothing_expires_without_both_a_policy_and_an_age() {
    let day = Duration::from_hours(24);
    let second = Duration::from_secs(1);
    assert!(!expired(None, Some(day)), "no policy, no discard");
    assert!(!expired(Some(day), None), "no readable age, no discard");
    assert!(!expired(None, None));
    assert!(!expired(Some(day), day.checked_sub(second)));
    assert!(expired(Some(day), Some(day)), "the keep is inclusive");
    assert!(expired(Some(day), Some(day + second)));
}

#[test]
fn the_age_is_the_attempt_refs_own_tip_time() {
    let dir = tempdir().unwrap();
    let project: PathBuf = dir.path().join("proj");
    std::fs::create_dir_all(&project).unwrap();
    run_git(&project, &["init", "-q", "-b", "main"]);
    run_git(&project, &["config", "user.email", "t@t.local"]);
    run_git(&project, &["config", "user.name", "Tester"]);
    run_git(&project, &["config", "commit.gpgsign", "false"]);
    std::fs::write(project.join("f"), "x").unwrap();
    run_git(&project, &["add", "f"]);
    run_git(&project, &["commit", "-q", "-m", "found"]);
    run_git(&project, &["branch", "attempt/at-0badcafe"]);

    // `run_git` pins the fixture's commit time, so the age is exact.
    let committed: u64 = git_out(
        &project,
        &["log", "-1", "--format=%ct", "attempt/at-0badcafe"],
    )
    .parse()
    .unwrap();
    let now = UNIX_EPOCH + Duration::from_secs(committed + 600);
    assert_eq!(
        age(&project, "at-0badcafe", now),
        Some(Duration::from_mins(10)),
    );

    // A ref that does not resolve has no age, and neither has one whose tip is
    // in the future — yog discards on a fact or not at all.
    assert_eq!(age(&project, "at-99999999", now), None);
    assert_eq!(
        age(
            &project,
            "at-0badcafe",
            UNIX_EPOCH + Duration::from_secs(committed - 1)
        ),
        None,
    );
    // Nor does a directory git will not run in at all.
    assert_eq!(age(Path::new("/no/such/project"), "at-0badcafe", now), None);
    // And a real clock is still an answer over a real ref.
    assert!(age(&project, "at-0badcafe", SystemTime::now()).is_some());
}
