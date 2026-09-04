//! **The roster, the `PATH` and the seed** — everything above one shim file:
//! that one converge materializes the WHOLE roster (bl-44a5), that the tools
//! dir goes on the front of the world's `PATH` idempotently, that the §8.6
//! capability control is a roster member like the rest, and that [`seed`]
//! *warns* rather than fails when it cannot converge. Split from the shim
//! itself at §12's budget on the seam between *what one shim is* and *what
//! standing them all up means*.

use std::os::unix::fs::PermissionsExt as _;

use super::super::*;
use tempfile::tempdir;

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
        assert_eq!(mode & 0o777, 0o755, "{namespace}");
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
/// adjudicator litany spawns cannot be a different file from the one yog
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

/// [`seed`]'s two arms over an ambient env: it converges the world's tools dir,
/// and it *warns* rather than failing when it cannot — the §8.5/§8.4 callers
/// hand the world out either way.
#[test]
fn seed_converges_the_world_tools_dir_and_warns_when_it_cannot() {
    let root = tempfile::tempdir().unwrap();
    let world_root = root.path().join("ok");
    let ambient = crate::test_support::world_under(&world_root);
    seed(&ambient);
    assert!(
        crate::world::layout(&ambient).tools.join("bl").exists(),
        "the world's `PATH` names this dir unconditionally, so seeding it is the world's own act"
    );

    // A tools *file* where the dir belongs: every shim write fails and the
    // converge cannot happen at all.
    let blocked_root = root.path().join("blocked");
    let blocked = crate::test_support::world_under(&blocked_root);
    let tools = crate::world::layout(&blocked).tools;
    std::fs::create_dir_all(tools.parent().unwrap()).unwrap();
    std::fs::write(&tools, b"not a directory").unwrap();
    seed(&blocked);
    assert!(tools.is_file(), "it said so and left the world alone");
}
