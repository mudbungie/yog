//! Construction, binary resolution, argv passing, happy-path streaming,
//! and `run_in` cwd propagation: `Cli::new`/`resolve`/`binary` and
//! `run`/`run_in` returning stdout, stderr, exit code, and a live pid.

use super::*;
use std::ffi::OsString;
use tempfile::tempdir;

#[test]
fn new_stores_binary_path() {
    let cli = Cli::new("/usr/local/bin/lernie");
    assert_eq!(cli.binary(), Path::new("/usr/local/bin/lernie"));
}

/// Resolve `binary` against an injected env lookup — no ambient-env
/// mutation, so no `unsafe std::env::set_var`. `set = Some(v)` models the
/// override var present with value `v` (an empty `v` exercises the "set but
/// empty → default" branch); `None` models the var absent.
fn resolve_injected(binary: Binary, set: Option<&str>) -> Cli {
    let value = set.map(OsString::from);
    // `None` current_exe: these cases pin the host-mode branches (override /
    // PATH name), where the self-multiplex exe is never consulted (§16.7 W12);
    // the self-mode branches are pinned by `resolve`'s own unit tests.
    Cli::resolve_with(binary, move |_| value.clone(), None)
}

#[test]
fn resolve_reads_ambient_env() {
    // Covers the production `std::env`-backed lookup that `resolve` wires
    // into `resolve_with`. The resolved path depends on the ambient env, so
    // we assert only that a non-empty binary is produced; the injected-
    // lookup cases below pin each resolution branch deterministically.
    let cli = Cli::resolve(Binary::Lernie);
    assert!(!cli.binary().as_os_str().is_empty());
}

#[test]
fn resolve_lernie_uses_env_var_when_set() {
    let cli = resolve_injected(Binary::Lernie, Some("/opt/lernie-test"));
    assert_eq!(cli.binary(), Path::new("/opt/lernie-test"));
}

#[test]
fn resolve_lernie_falls_back_to_default_when_env_empty() {
    let cli = resolve_injected(Binary::Lernie, Some(""));
    assert_eq!(cli.binary(), Path::new("lernie"));
}

#[test]
fn resolve_lernie_falls_back_to_default_when_env_unset() {
    let cli = resolve_injected(Binary::Lernie, None);
    assert_eq!(cli.binary(), Path::new("lernie"));
}

#[test]
fn resolve_bl_uses_env_var_when_set() {
    let cli = resolve_injected(Binary::Bl, Some("/opt/bl-test"));
    assert_eq!(cli.binary(), Path::new("/opt/bl-test"));
}

#[test]
fn resolve_bl_falls_back_to_default_when_env_unset() {
    let cli = resolve_injected(Binary::Bl, None);
    assert_eq!(cli.binary(), Path::new("bl"));
}

#[test]
fn resolve_bz_uses_env_var_when_set() {
    let cli = resolve_injected(Binary::Bz, Some("/opt/bz-test"));
    assert_eq!(cli.binary(), Path::new("/opt/bz-test"));
}

#[test]
fn resolve_bz_falls_back_to_default_when_env_unset() {
    let cli = resolve_injected(Binary::Bz, None);
    assert_eq!(cli.binary(), Path::new("bz"));
}

#[test]
fn every_namespace_is_self_multiplexed() {
    // W8 flipped `Bl`, W10 flipped `Bz`, W11 flipped `Lernie`: with a
    // `current_exe` available and no override, each resolves to yog's own exe
    // under its own namespace prefix while the LOGICAL name stays the tool's.
    // This pins the ON arm of the one switch for all three namespaces (the
    // override/no-exe host arms are pinned above).
    let exe = PathBuf::from("/proc/self/yog");
    for (embedded, name) in [
        (Binary::Bl, "bl"),
        (Binary::Bz, "bz"),
        (Binary::Lernie, "lernie"),
    ] {
        let cli = Cli::resolve_with(embedded, |_| None, Some(exe.clone()));
        assert_eq!(cli.program(), exe, "{name} must exec yog itself");
        assert_eq!(cli.prefix(), [name.to_string()]);
        assert_eq!(cli.binary(), Path::new(name));
    }
}

#[test]
fn self_multiplex_execs_current_exe_under_the_namespace_prefix() {
    // §16.7 W12 self-mode: the physical program is the injected exe and the
    // namespace is the leading argv, while `binary()` — the ops-log argv[0]
    // (§8.2) — stays the logical namespace, not the exe path. This is the whole
    // logical/physical split: a spawn retargeted to `yog lernie …` still logs
    // `["lernie", …]`.
    let cli = Cli::default_target(
        "lernie",
        crate::cli_outbound::resolve::Target::Namespace("lernie"),
        Some(PathBuf::from("/proc/self/yog")),
    );
    assert_eq!(cli.program(), Path::new("/proc/self/yog"));
    assert_eq!(cli.prefix(), ["lernie".to_string()]);
    assert_eq!(cli.binary(), Path::new("lernie"));
}

#[test]
fn self_multiplex_with_no_current_exe_falls_back_to_the_host_name() {
    // Switch ON but `current_exe()` unavailable: resolve the host PATH name
    // rather than panic — a spawn that at least names the tool.
    let cli = Cli::default_target(
        "lernie",
        crate::cli_outbound::resolve::Target::Namespace("lernie"),
        None,
    );
    assert_eq!(cli.program(), Path::new("lernie"));
    assert!(cli.prefix().is_empty());
    assert_eq!(cli.binary(), Path::new("lernie"));
}

/// The switch's third shape (bl-3ff4): yog's own shim resolves the running
/// executable with **no** leading verb word. A prefix here would spawn `yog yog
/// …`, which routes nowhere — which is why the switch is an enum rather than a
/// second bool beside the first.
#[test]
fn yogs_own_shim_execs_the_running_exe_with_no_verb_word() {
    let exe = PathBuf::from("/proc/self/yog");
    let cli = Cli::default_target(
        "yog",
        crate::cli_outbound::resolve::Target::SelfBare,
        Some(exe.clone()),
    );
    assert_eq!(cli.program(), exe);
    assert!(
        cli.prefix().is_empty(),
        "no namespace word: {:?}",
        cli.prefix()
    );
    // Resolved the ordinary way too — through `Binary::Yog`, not just the
    // injected target — so the roster entry and this shape cannot drift apart.
    let resolved = Cli::resolve_with(Binary::Yog, |_| None, Some(exe.clone()));
    assert_eq!(resolved.program(), exe);
    assert!(resolved.prefix().is_empty());
}

/// The `*_BINARY` escape hatch reaches yog's shim like every other entry.
#[test]
fn yogs_shim_honors_its_binary_override() {
    let cli = Cli::resolve_with(
        Binary::Yog,
        |k| (k == "YOG_BINARY").then(|| std::ffi::OsString::from("/opt/yog")),
        Some(PathBuf::from("/proc/self/yog")),
    );
    assert_eq!(cli.program(), Path::new("/opt/yog"));
}

#[test]
fn override_wins_over_the_self_multiplex_switch() {
    // The `*_BINARY` override is the escape hatch: set and non-empty it wins
    // outright — even with a `current_exe` present (were the namespace flipped).
    let cli = Cli::resolve_with(
        Binary::Bl,
        |_| Some(OsString::from("/opt/bl-test")),
        Some(PathBuf::from("/proc/self/yog")),
    );
    assert_eq!(cli.program(), Path::new("/opt/bl-test"));
    assert!(cli.prefix().is_empty());
}

#[test]
fn self_multiplex_spawn_prepends_the_namespace_to_the_childs_argv() {
    // The physical spawn is `program` + `prefix` + args (§16.7 W12): a
    // self-mode `Cli` over an argv-echoing script sees the namespace prepended
    // before the caller's args — the exact `yog <namespace> <args…>` shape.
    let dir = tempdir().unwrap();
    let (script, _spawn_guard) = write_script(
        dir.path(),
        "echo_argv",
        "#!/bin/sh\nprintf '%s\\n' \"$@\"\n",
    );
    let cli = Cli::default_target(
        "lernie",
        crate::cli_outbound::resolve::Target::Namespace("lernie"),
        Some(script),
    );
    let (out, err, exit) = collect(cli.run(&["prompt", "goal"]).unwrap());
    assert_eq!(exit, ExitInfo::Code(0));
    assert!(err.is_empty());
    assert_eq!(String::from_utf8(out).unwrap(), "lernie\nprompt\ngoal\n");
}

#[test]
fn run_streams_stdout_and_reports_exit_zero() {
    let dir = tempdir().unwrap();
    let (bin, _spawn_guard) = write_script(
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
    let (bin, _spawn_guard) = write_script(
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
    let (bin, _spawn_guard) = write_script(dir.path(), "fake_lernie", "#!/bin/sh\nexit 0\n");
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
    let (bin, _spawn_guard) = write_script(
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
    let (bin, _spawn_guard) = write_script(dir.path(), "fake_lernie", "#!/bin/sh\npwd -P\n");
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
