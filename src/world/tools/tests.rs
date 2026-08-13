//! The world's tool shim (§16.7 W9): the script's exact text, the convergent
//! seeding, and the `PATH` prepend — plus the end-to-end proof that the seeded
//! file *runs* and forwards argv verbatim.

use super::*;
use crate::test_support::spawn_guard;
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
    let mode = fs::metadata(&path).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, SHIM_MODE);
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

/// The whole point, executed: the seeded file is a runnable program that hands
/// its argv to the target verbatim. The target is a recorder script standing in
/// for the yog binary, so the shim's own contract is what is under test.
#[test]
fn the_seeded_shim_runs_and_passes_argv_through_verbatim() {
    let g = spawn_guard();
    let dir = tempdir().unwrap();
    let target = dir.path().join("fake yog");
    let log = dir.path().join("argv");
    fs::write(
        &target,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\nexit 3\n",
            log.display()
        ),
    )
    .unwrap();
    fs::set_permissions(&target, fs::Permissions::from_mode(SHIM_MODE)).unwrap();
    let tools = dir.path().join("tools");
    let shim = ensure_shim(&tools, BL, &host(target.to_string_lossy().as_ref())).unwrap();
    let out = crate::git_env::command(&shim)
        .args(["close", "bl-1a2b", "-m", "two words"])
        .output()
        .unwrap();
    drop(g);
    // The child's exit rides back through `exec` untouched.
    assert_eq!(out.status.code(), Some(3));
    assert_eq!(
        fs::read_to_string(&log).unwrap(),
        "close\nbl-1a2b\n-m\ntwo words\n"
    );
}

/// The bl-44a5 hatch converge: one call materializes the WHOLE roster, each
/// shim's body exactly what [`ensure_shim`] with that namespace's own
/// resolution writes — so a pre-first-Start `yog env`/`yog exec` hands out a
/// `PATH` whose head is real. Expectations are computed through the same
/// [`Cli::resolve`] the function uses, so the test holds under any ambient
/// `*_BINARY` seam.
#[test]
fn ensure_tools_converges_every_roster_shim() {
    let dir = tempdir().unwrap();
    let tools = dir.path().join("world").join("tools");
    ensure_tools(&tools).unwrap();
    for (namespace, binary) in ROSTER {
        let path = tools.join(namespace);
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            shim_script(namespace, &Cli::resolve(binary).exec_words()),
            "{namespace}"
        );
        let mode = fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, SHIM_MODE, "{namespace}");
    }
    // The roster carries yog itself (bl-3ff4), and its shim is the one that
    // names NO verb word — `exec <yog> "$@"`, not `exec <yog> yog "$@"` — so an
    // agent's `yog gesture …` reaches the argv surface rather than a namespace
    // that does not exist.
    let yog_shim = fs::read_to_string(tools.join(YOG)).unwrap();
    assert!(
        !yog_shim.contains("' yog \"$@\""),
        "no verb word in yog's own shim: {yog_shim}"
    );
    assert!(
        yog_shim.contains("\"$@\""),
        "argv passes through: {yog_shim}"
    );
    // Idempotent: the steady state is one read and no write per shim.
    let stamp = fs::metadata(tools.join(BL)).unwrap().modified().unwrap();
    ensure_tools(&tools).unwrap();
    assert_eq!(
        fs::metadata(tools.join(BL)).unwrap().modified().unwrap(),
        stamp
    );
}

#[test]
fn prepend_path_puts_the_tools_dir_first_and_is_idempotent() {
    let tools = Path::new("/d/yog/world/tools");
    assert_eq!(
        prepend_path(tools, Some("/usr/bin:/bin".to_owned())),
        "/d/yog/world/tools:/usr/bin:/bin"
    );
    // Re-composing over an already-world PATH is a no-op — no stacked entries.
    let once = prepend_path(tools, Some("/usr/bin".to_owned()));
    assert_eq!(prepend_path(tools, Some(once.clone())), once);
    // An absent or empty ambient PATH leaves the tools dir alone.
    assert_eq!(prepend_path(tools, None), "/d/yog/world/tools");
    assert_eq!(
        prepend_path(tools, Some(String::new())),
        "/d/yog/world/tools"
    );
    // A tools dir that merely *contains* the ambient head is still prepended
    // (the guard compares whole entries, not prefixes).
    assert_eq!(
        prepend_path(tools, Some("/d/yog/world/tools-old:/bin".to_owned())),
        "/d/yog/world/tools:/d/yog/world/tools-old:/bin"
    );
}

/// §8.6: the capability control's shim is a roster member like the rest — the
/// path the authored `tool_control:` block names is exactly what
/// [`ensure_control`] writes, and both halves come from one place so the
/// adjudicator lernie spawns cannot be a different file from the one yog
/// authored.
#[test]
fn the_capability_control_shim_is_seeded_where_the_authored_block_names_it() {
    let dir = tempdir().unwrap();
    let tools = dir.path().join("tools");
    let seeded = ensure_control(&tools).unwrap();
    assert_eq!(seeded, control_path(&tools));
    assert!(seeded.is_absolute(), "the block must not be PATH-resolved");
    assert_eq!(
        fs::read_to_string(&seeded).unwrap(),
        shim_script(
            TOOL_CONTROL,
            &Cli::resolve(crate::cli_outbound::Binary::ToolControl).exec_words()
        )
    );
}
