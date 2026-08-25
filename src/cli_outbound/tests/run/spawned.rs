//! **The happy-path run**: a real child forked off a script on disk, its stdout
//! and stderr streamed, its exit code propagated, its pid live while it runs,
//! and the two things a caller may set on it — the child's environment and its
//! working directory. Split from resolution at §12's budget on the seam the
//! parent's own doc draws: above is *which binary*, here is *running it*.

use super::super::super::{Cli, ExitInfo};
use super::super::{collect, write_script};
use tempfile::tempdir;

#[test]
fn run_streams_stdout_and_reports_exit_zero() {
    let dir = tempdir().unwrap();
    let bin = write_script(
        dir.path(),
        "fake_lernie",
        "#!/bin/sh\nprintf 'hello out\\n'\nexit 0\n",
    );
    let cli = Cli::new(bin);
    let stream = cli.run(&[]).unwrap();
    let (out, err, exit) = collect(stream);
    assert_eq!(out, b"hello out\n");
    assert!(err.is_empty());
    assert_eq!(exit, ExitInfo::Code(0));
}

#[test]
fn run_streams_stderr_and_propagates_nonzero_exit() {
    let dir = tempdir().unwrap();
    let bin = write_script(
        dir.path(),
        "fake_lernie",
        "#!/bin/sh\nprintf 'boom\\n' 1>&2\nexit 7\n",
    );
    let cli = Cli::new(bin);
    let (out, err, exit) = collect(cli.run(&["some", "args"]).unwrap());
    assert!(out.is_empty());
    assert_eq!(err, b"boom\n");
    assert_eq!(exit, ExitInfo::Code(7));
}

#[test]
fn pid_is_available_while_running() {
    let dir = tempdir().unwrap();
    let bin = write_script(dir.path(), "fake_lernie", "#!/bin/sh\nexit 0\n");
    let cli = Cli::new(bin);
    let stream = cli.run(&[]).unwrap();
    let pid = stream.pid();
    assert!(pid.is_some());
    drop(stream);
}

#[test]
fn run_env_sets_child_environment_variables() {
    let dir = tempdir().unwrap();
    // Echo the two vars the config-edit drive sets (§9.3), one per line, so
    // the child's environment is observable in the stream.
    let bin = write_script(
        dir.path(),
        "fake_lernie",
        "#!/bin/sh\nprintf '%s\\n%s\\n' \"$EDITOR\" \"$YOG_EDIT_SRC\"\n",
    );
    let cli = Cli::new(bin);
    let env = [
        ("EDITOR", "/x/yog --editor-apply"),
        ("YOG_EDIT_SRC", "/s/n"),
    ];
    let (out, err, exit) = collect(cli.run_env(&env, &["config"]).unwrap());
    assert_eq!(exit, ExitInfo::Code(0));
    assert!(err.is_empty());
    assert_eq!(
        String::from_utf8(out).unwrap(),
        "/x/yog --editor-apply\n/s/n\n"
    );
}

#[test]
fn run_in_sets_child_working_directory() {
    let dir = tempdir().unwrap();
    // Report the physical cwd so the child's `current_dir` is observable
    // in the stream.
    let bin = write_script(dir.path(), "fake_lernie", "#!/bin/sh\npwd -P\n");
    let cli = Cli::new(bin);
    let (out, err, exit) = collect(cli.run_in(dir.path(), &[]).unwrap());
    assert_eq!(exit, ExitInfo::Code(0));
    assert!(err.is_empty());
    let reported = String::from_utf8(out).unwrap();
    assert_eq!(
        std::fs::canonicalize(reported.trim()).unwrap(),
        std::fs::canonicalize(dir.path()).unwrap(),
    );
}
