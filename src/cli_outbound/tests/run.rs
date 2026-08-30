//! Construction, binary resolution and argv passing: `Cli::new`/`resolve`/
//! `binary`, and the self-multiplex switch every namespace rides. What running
//! one actually costs — stdout, stderr, exit code, a live pid, the child's env
//! and cwd — is [`spawned`], split off at §12's budget.

mod spawned;

use super::*;
use std::ffi::OsString;
use tempfile::tempdir;

#[test]
fn new_stores_binary_path() {
    let cli = Cli::new("/usr/local/bin/litany");
    assert_eq!(cli.binary(), Path::new("/usr/local/bin/litany"));
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
    let cli = Cli::resolve(Binary::Litany);
    assert!(!cli.binary().as_os_str().is_empty());
}

#[test]
fn resolve_litany_uses_env_var_when_set() {
    let cli = resolve_injected(Binary::Litany, Some("/opt/litany-test"));
    assert_eq!(cli.binary(), Path::new("/opt/litany-test"));
}

#[test]
fn resolve_litany_falls_back_to_default_when_env_empty() {
    let cli = resolve_injected(Binary::Litany, Some(""));
    assert_eq!(cli.binary(), Path::new("litany"));
}

#[test]
fn resolve_litany_falls_back_to_default_when_env_unset() {
    let cli = resolve_injected(Binary::Litany, None);
    assert_eq!(cli.binary(), Path::new("litany"));
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
    // W8 flipped `Bl`, W10 flipped `Bz`, W11 flipped `Litany`: with a
    // `current_exe` available and no override, each resolves to yog's own exe
    // under its own namespace prefix while the LOGICAL name stays the tool's.
    // This pins the ON arm of the one switch for all three namespaces (the
    // override/no-exe host arms are pinned above).
    let exe = PathBuf::from("/proc/self/yog");
    for (embedded, name) in [
        (Binary::Bl, "bl"),
        (Binary::Bz, "bz"),
        (Binary::Litany, "litany"),
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
    // logical/physical split: a spawn retargeted to `yog litany …` still logs
    // `["litany", …]`.
    let cli = Cli::default_target(
        "litany",
        crate::cli_outbound::resolve::Target::Namespace("litany"),
        Some(PathBuf::from("/proc/self/yog")),
    );
    assert_eq!(cli.program(), Path::new("/proc/self/yog"));
    assert_eq!(cli.prefix(), ["litany".to_string()]);
    assert_eq!(cli.binary(), Path::new("litany"));
}

#[test]
fn self_multiplex_with_no_current_exe_falls_back_to_the_host_name() {
    // Switch ON but `current_exe()` unavailable: resolve the host PATH name
    // rather than panic — a spawn that at least names the tool.
    let cli = Cli::default_target(
        "litany",
        crate::cli_outbound::resolve::Target::Namespace("litany"),
        None,
    );
    assert_eq!(cli.program(), Path::new("litany"));
    assert!(cli.prefix().is_empty());
    assert_eq!(cli.binary(), Path::new("litany"));
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
    let script = write_script(
        dir.path(),
        "echo_argv",
        "#!/bin/sh\nprintf '%s\\n' \"$@\"\n",
    );
    let cli = Cli::default_target(
        "litany",
        crate::cli_outbound::resolve::Target::Namespace("litany"),
        Some(script),
    );
    let (out, err, exit) = collect(cli.run(&["prompt", "goal"]).unwrap());
    assert_eq!(exit, ExitInfo::Code(0));
    assert!(err.is_empty());
    assert_eq!(String::from_utf8(out).unwrap(), "litany\nprompt\ngoal\n");
}
