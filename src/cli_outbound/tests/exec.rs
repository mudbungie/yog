//! `Cli::exec_in_world`: exit-code passthrough, the cwd branch (set / unset),
//! world-env layering, and the spawn-failure error. The scripts write their
//! observations to files and stay **silent** on stdout/stderr — the child's
//! stdio is inherited, so anything printed would leak into the test run.

use super::*;
use tempfile::tempdir;

#[test]
fn exec_in_world_passes_through_a_nonzero_exit_code() {
    let dir = tempdir().unwrap();
    let (bin, _guard) = write_script(dir.path(), "silent", "#!/bin/sh\nexit 7\n");
    let info = Cli::exec_in_world(bin.to_str().unwrap(), &[], None, &[]).unwrap();
    assert_eq!(info, ExitInfo::Code(7));
}

#[test]
fn exec_in_world_reports_success_and_forwards_args() {
    let dir = tempdir().unwrap();
    // `[ $# = 2 ]` — the two args reach the child; else exit non-zero.
    let (bin, _guard) = write_script(
        dir.path(),
        "silent",
        "#!/bin/sh\n[ $# = 2 ] || exit 3\nexit 0\n",
    );
    let info = Cli::exec_in_world(bin.to_str().unwrap(), &[], None, &["a", "b"]).unwrap();
    assert_eq!(info, ExitInfo::Code(0));
}

#[test]
fn exec_in_world_runs_in_the_given_cwd() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("cwd.txt");
    let (bin, _guard) = write_script(
        dir.path(),
        "reportcwd",
        &format!("#!/bin/sh\npwd -P > '{}'\n", out.display()),
    );
    let info = Cli::exec_in_world(bin.to_str().unwrap(), &[], Some(dir.path()), &[]).unwrap();
    assert_eq!(info, ExitInfo::Code(0));
    let reported = std::fs::read_to_string(&out).unwrap();
    assert_eq!(
        std::fs::canonicalize(reported.trim()).unwrap(),
        std::fs::canonicalize(dir.path()).unwrap(),
    );
}

#[test]
fn exec_in_world_layers_the_world_overrides_over_inherited_env() {
    let dir = tempdir().unwrap();
    let out = dir.path().join("env.txt");
    let (bin, _guard) = write_script(
        dir.path(),
        "reportenv",
        &format!(
            "#!/bin/sh\nprintf '%s' \"$XDG_STATE_HOME\" > '{}'\n",
            out.display()
        ),
    );
    let overrides = vec![("XDG_STATE_HOME".to_owned(), "/d/yog/world/state".to_owned())];
    let info = Cli::exec_in_world(bin.to_str().unwrap(), &overrides, None, &[]).unwrap();
    assert_eq!(info, ExitInfo::Code(0));
    assert_eq!(std::fs::read_to_string(&out).unwrap(), "/d/yog/world/state");
}

#[test]
fn exec_in_world_errors_when_the_command_is_missing() {
    let _guard = crate::test_support::spawn_guard();
    let err = Cli::exec_in_world("/no/such/binary-xyz", &[], None, &[]).unwrap_err();
    assert!(matches!(err, CliError::Spawn { .. }));
}
