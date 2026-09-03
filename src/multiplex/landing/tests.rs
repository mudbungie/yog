//! The repair's **pure error plumbing** — the two arms of [`sited`] and the
//! three of [`report`]. Everything that needs a real balls landing on disk
//! lives in `tests/multiplex_landing.rs` instead (bl-6bf5), and its OTHER
//! reason to stay there outlived the ETXTBSY one bl-fd28 dissolved: a binary
//! that founds a landing in-process must scrub its own env of
//! `git_env::INHERITED`, no spawn boundary existing to do it for a fork it does
//! not perform. Read that file's crate-root doc before moving a beat back.

use super::*;

/// The instrumentation bl-1ce0 asked for, on the failure shape it was filed
/// from: a bare `NotFound` out of any of converge's reads or forks used to
/// reach the operator as one word, naming neither the step nor the path.
#[test]
fn a_sited_error_keeps_its_kind_and_names_the_step_and_the_path() {
    let bare = io::Error::new(io::ErrorKind::NotFound, "No such file or directory");
    let err = sited(
        "read the landing schedule",
        Path::new("/home/u/w"),
        Err::<(), _>(bare),
    )
    .expect_err("the error survives");
    assert_eq!(err.kind(), io::ErrorKind::NotFound, "matchable as before");
    assert_eq!(
        err.to_string(),
        "read the landing schedule (/home/u/w): No such file or directory"
    );
}

/// The pass-through half: a site costs nothing on the path everything takes.
#[test]
fn a_sited_success_is_the_value_itself() {
    assert_eq!(
        sited("read the landing schedule", Path::new("/home/u/w"), Ok(7)).expect("ok"),
        7
    );
}

#[test]
fn every_report_arm_is_quiet_about_the_verb() {
    // Reporting never returns a verdict — the verb's exit is balls', whatever
    // the repair did. All three arms run for the branch, not for an assertion.
    report(Ok(true));
    report(Ok(false));
    report(Err(io::Error::other("boom")));
}
