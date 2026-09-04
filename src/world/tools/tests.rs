//! The world's tool shim (§16.7 W9): the script's exact text and the convergent
//! seeding of one shim file, plus the end-to-end proof that the seeded file
//! *runs* and forwards argv verbatim. What standing the whole roster up means —
//! the converge, the `PATH` prepend, the §8.6 control shim and [`seed`] — is
//! [`roster`], split off at §12's budget.

mod roster;

use std::os::unix::fs::PermissionsExt as _;

use super::*;
use tempfile::tempdir;

/// A host-mode `Cli` (program only, no prefix) — what `BL_BINARY` resolution
/// yields, and the recorder shape the harness tests use.
fn host(program: &str) -> Cli {
    Cli::new(program)
}

#[test]
fn shim_script_reexecs_the_clis_own_words_and_forwards_argv() {
    // Self-multiplex resolution (`yog bl …`): the shim reproduces exactly the
    // words yog itself execs, then `"$@"`.
    let cli = Cli::resolve_with(
        crate::cli_outbound::Binary::Bl,
        |_| None,
        Some(PathBuf::from("/opt/yog/bin/yog")),
    );
    let script = shim_script(BL, &cli.exec_words());
    assert!(script.starts_with("#!/bin/sh\n"), "{script}");
    assert!(
        script.ends_with("exec '/opt/yog/bin/yog' 'bl' \"$@\"\n"),
        "{script}"
    );
    // Host mode (a `BL_BINARY` override) has no namespace prefix — the shim is
    // still exactly the words that `Cli` spawns, so the two can never diverge.
    let script = shim_script(BL, &host("/usr/bin/bl").exec_words());
    assert!(script.ends_with("exec '/usr/bin/bl' \"$@\"\n"), "{script}");
}

#[test]
fn shim_script_quotes_a_path_with_spaces_and_quotes() {
    let script = shim_script(BL, &host("/we ird/yo'g").exec_words());
    assert!(
        script.ends_with("exec '/we ird/yo'\\''g' \"$@\"\n"),
        "{script}"
    );
}

#[test]
fn ensure_shim_creates_the_tree_and_marks_it_executable() {
    let dir = tempdir().unwrap();
    let tools = dir.path().join("world").join("tools");
    let path = ensure_shim(&tools, BL, &host("/usr/bin/bl")).unwrap();
    assert_eq!(path, tools.join(BL));
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        shim_script(BL, &host("/usr/bin/bl").exec_words())
    );
    // 0o755 — executable by all, writable only by the owner. The mode is the
    // child writer's now (`git_env::write_exec`), so this is what proves the
    // relocation did not lose the bit that makes a shim runnable.
    let mode = fs::metadata(&path).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o755);
}

#[test]
fn ensure_shim_leaves_an_identical_file_untouched_but_rewrites_drift() {
    let dir = tempdir().unwrap();
    let tools = dir.path().to_path_buf();
    let path = ensure_shim(&tools, BL, &host("/usr/bin/bl")).unwrap();
    let first = fs::metadata(&path).unwrap().modified().unwrap();
    // Same Cli ⇒ same content ⇒ no write at all (the common start).
    ensure_shim(&tools, BL, &host("/usr/bin/bl")).unwrap();
    assert_eq!(fs::metadata(&path).unwrap().modified().unwrap(), first);
    // yog reinstalled elsewhere ⇒ the shim converges on the new target.
    ensure_shim(&tools, BL, &host("/opt/bl")).unwrap();
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        shim_script(BL, &host("/opt/bl").exec_words())
    );
    // A hand-edit is drift too, and is overwritten.
    fs::write(&path, "#!/bin/sh\nexit 7\n").unwrap();
    ensure_shim(&tools, BL, &host("/opt/bl")).unwrap();
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        shim_script(BL, &host("/opt/bl").exec_words())
    );
}

/// bl-f558's durable half: a resolution that is not an absolute path is refused
/// rather than written, and the shim already on disk — written when yog could
/// still say which file it was — is left exactly as it stands. The bare PATH
/// *name* is what `Cli::resolve` falls back to when the self-exe reading is
/// unusable, and persisting it here is worse than persisting nothing: this
/// directory fronts the world's `PATH`, so `exec 'bl' "$@"` would re-resolve to
/// the shim itself.
#[test]
fn ensure_shim_refuses_a_target_that_is_not_an_absolute_path() {
    let dir = tempdir().unwrap();
    let tools = dir.path().join("tools");
    let good = ensure_shim(&tools, BL, &host("/opt/yog/bin/yog")).unwrap();
    let before = fs::read_to_string(&good).unwrap();

    let err = ensure_shim(&tools, BL, &host("bl")).unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::NotFound);
    assert!(err.to_string().contains("absolute path"), "{err}");
    assert_eq!(fs::read_to_string(&good).unwrap(), before);

    // Nothing on disk yet is refused the same way — an absent shim is honest
    // where a self-referential one is a loop.
    let empty = dir.path().join("empty");
    ensure_shim(&empty, BL, &Cli::new("")).unwrap_err();
    assert!(!empty.join(BL).exists());
}

/// The whole point, executed: the seeded file is a runnable program that hands
/// its argv to the target verbatim. The target is a recorder script standing in
/// for the yog binary, so the shim's own contract is what is under test.
#[test]
fn the_seeded_shim_runs_and_passes_argv_through_verbatim() {
    let dir = tempdir().unwrap();
    let target = dir.path().join("fake yog");
    let log = dir.path().join("argv");
    crate::test_support::write_exec(
        &target,
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\nexit 3\n",
            log.display()
        ),
    );
    let tools = dir.path().join("tools");
    let shim = ensure_shim(&tools, BL, &host(target.to_string_lossy().as_ref())).unwrap();
    let out = crate::git_env::output(crate::git_env::command(&shim).args([
        "close",
        "bl-1a2b",
        "-m",
        "two words",
    ]))
    .unwrap();
    // The child's exit rides back through `exec` untouched.
    assert_eq!(out.status.code(), Some(3));
    assert_eq!(
        fs::read_to_string(&log).unwrap(),
        "close\nbl-1a2b\n-m\ntwo words\n"
    );
}
