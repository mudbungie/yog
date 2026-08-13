//! The lernie arm's binding pieces, each driven directly: the clap parse and
//! its exit-code mapping, the `$EDITOR` hand-off, and the outcome/failure
//! plumbing ([`conclude`]/[`perform`], upstream's `perform` with `i32`).
//! [`run`]'s full verb path — preludes, the shim converge, the `Fx` build —
//! rides `tests/multiplex_lernie.rs`, a test binary of its own that owns its
//! process environment (`LERNIE_HOME`/`XDG_DATA_HOME`/`EDITOR`).

use super::*;
use crate::test_support::spawn_guard;

fn args(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| (*s).to_string()).collect()
}

#[test]
fn parse_accepts_a_verb_and_maps_help_and_errors_to_exit_codes() {
    // A well-formed verb parses into lernie's own surface.
    let cli = parse(&args(&["new", "/tmp/ws"])).unwrap();
    assert!(matches!(cli.command, cmd::Command::New(_)));
    // `--help` is clap's short-circuit: printed, exit 0 — never a verb.
    assert_eq!(parse(&args(&["--help"])).unwrap_err(), 0);
    // An unknown verb and a bare argv are clap usage errors, exit 2.
    assert_eq!(parse(&args(&["no-such-verb"])).unwrap_err(), 2);
    assert_eq!(parse(&[]).unwrap_err(), 2);
}

#[test]
fn edit_with_maps_the_editor_exit_to_the_edit_result() {
    let _guard = spawn_guard();
    let dir = tempfile::TempDir::new().unwrap();
    // The editor is handed the checkout dir as `"$1"` through `sh -c`, so a
    // multi-word $EDITOR works; a zero exit is a completed edit.
    let ok = edit_with("test -d", dir.path());
    assert!(ok.is_ok());
    // A non-zero editor exit is a failed edit, message carrying the status.
    let err = edit_with("false", dir.path()).unwrap_err();
    assert!(err.to_string().contains("editor exited"));
}

#[test]
fn conclude_performs_success_and_prints_the_uniform_failure() {
    assert_eq!(conclude(Ok(Outcome::Quiet)), 0);
    assert_eq!(conclude(Err(cmd::Error::new("prompt", "boom"))), 1);
}

#[test]
fn perform_maps_each_outcome_to_its_exit() {
    let _guard = spawn_guard();
    // The one-product line prints and succeeds; quiet succeeds silently.
    assert_eq!(perform(Outcome::Line("a-branch".to_owned())), 0);
    assert_eq!(perform(Outcome::Quiet), 0);
    // The tool verb's exit code rides through within u8.
    assert_eq!(perform(Outcome::Code(7)), 7);
    // The successor exec: a successful execve never returns, so the arm past
    // it IS the failure path — a target that cannot exec reports and fails.
    let cmd = crate::git_env::command(Path::new("/nonexistent/yog-successor"));
    assert_eq!(perform(Outcome::Exec(cmd)), 1);
}
