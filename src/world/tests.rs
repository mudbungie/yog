use super::*;
use crate::cli_outbound::Cli;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::tempdir;

/// Ambient snapshot: `HOME` + the XDG anchors, with the two vars the world
/// overrides deliberately pre-set to *other* values — so a passing derivation
/// proves the world value **replaced** them, not merely filled a gap.
/// `BRAZEN_CONFIG` is pre-set to prove it is **inert** — brazen's config is the
/// wall's since the blast-radius ruling (§16.2 as amended); `USER` rides along
/// to prove a non-overridden var survives the composition.
fn ambient() -> Env {
    Env::from_pairs([
        ("HOME", "/h"),
        ("XDG_DATA_HOME", "/d"),
        ("XDG_CACHE_HOME", "/c"),
        ("LERNIE_HOME", "/ambient/lernie"),
        ("XDG_STATE_HOME", "/ambient/state"),
        ("BRAZEN_CONFIG", "/ambient/brazen.toml"),
        ("PATH", "/usr/bin:/bin"),
        ("USER", "alice"),
    ])
}

/// A fold `fn(&Env) -> PathBuf` and the nested path it must yield through the
/// composed world `Env`.
type Case = (fn(&Env) -> PathBuf, &'static str);

#[test]
fn overrides_win_and_every_substrate_fold_nests() {
    let world = compose(&ambient());
    let cases: &[Case] = &[
        // Each override seen through the fold that reads it — yielding the
        // world value, NOT the ambient one it replaced.
        (Env::lernie_config_root, "/d/yog/world/lernie"),
        (Env::lernie_data_root, "/d/yog/world/lernie"),
        (Env::balls_state_root, "/d/yog/world/state/balls"),
        (Env::balls_clones_dir, "/d/yog/world/state/balls/clones"),
        // yog's own state root moves under the world (§16.2 decision).
        (Env::yog_state_root, "/d/yog/world/state/yog"),
        (Env::yog_stage_root, "/d/yog/world/state/yog/stage"),
    ];
    for &(fold, expect) in cases {
        assert_eq!(fold(&world), PathBuf::from(expect));
    }
    // yog's two artifacts themselves, re-derived through the world Env.
    assert_eq!(
        world.yog_state_root().join("ui.json"),
        PathBuf::from("/d/yog/world/state/yog/ui.json")
    );
    assert_eq!(
        world.yog_state_root().join("ops.jsonl"),
        PathBuf::from("/d/yog/world/state/yog/ops.jsonl")
    );
}

#[test]
fn nothing_brazen_survives_the_world_and_the_anchor_is_self_consistent() {
    let amb = ambient();
    let world = compose(&amb);
    // Brazen has no ambient fold left to inherit (§16.2 as amended): the world
    // itself names no wall, so it answers no brazen paths at all. An ambient
    // `BRAZEN_CONFIG` in the operator's shell — pre-set in `ambient()` — buys
    // nothing, which is the point: one exported var must not collapse every
    // workspace onto one file.
    assert_eq!(world.wall(), None);
    assert_eq!(crate::config_edit::brazen::BrazenPaths::of(&world), None);
    // Lensed on a workspace, all three land inside that sphere's wall.
    let corp = crate::world::wall::env(&world, std::path::Path::new("/ws/corp"));
    let paths = crate::config_edit::brazen::BrazenPaths::of(&corp).expect("a wall");
    assert!(paths.config.starts_with("/d/yog/world/walls/corp"));
    assert!(paths.credentials_dir.starts_with("/d/yog/world/walls/corp"));
    assert!(
        paths
            .models_cache_dir
            .starts_with("/d/yog/world/walls/corp")
    );
    // The anchor is self-consistent: re-deriving it through the world Env yields
    // the same yog data root, so `layout(world) == layout(ambient)` (§16.2).
    assert_eq!(world.yog_data_root(), amb.yog_data_root());
    assert_eq!(layout(&world).root, layout(&amb).root);
    // A non-overridden var survives the composition untouched.
    assert_eq!(world.user(), Some("alice".to_owned()));
}

#[test]
fn layout_names_the_world_subtree() {
    let l = layout(&ambient());
    assert_eq!(l.root, PathBuf::from("/d/yog/world"));
    assert_eq!(l.lernie, PathBuf::from("/d/yog/world/lernie"));
    assert_eq!(l.state, PathBuf::from("/d/yog/world/state"));
    assert_eq!(l.tools, PathBuf::from("/d/yog/world/tools"));
}

/// The override set is exactly the §16.2 three, in order, and the `PATH` entry
/// leads with the world's tools dir (§16.7 W9) rather than replacing the ambient
/// search path. Re-deriving the set from the **composed world** `Env` — which
/// `marks`/`config_edit` do — reproduces it byte-for-byte: the composition is
/// idempotent, so no re-entry stacks a second tools entry. That fixed point is
/// also what makes [`inhabit`](super::inhabit) safe to call on an
/// already-folded process (bl-81c9), and the set being exactly these three is
/// why the fold can displace neither an agent's own space (`YOG_MARKS`) nor a
/// workspace's wall (`YOG_WALL`): both ride one layer in, and neither is here.
#[test]
fn the_override_set_nests_two_state_vars_and_fronts_the_tool_path() {
    let amb = ambient();
    let ov = overrides(&amb);
    assert_eq!(
        ov,
        vec![
            ("LERNIE_HOME".to_owned(), "/d/yog/world/lernie".to_owned()),
            ("XDG_STATE_HOME".to_owned(), "/d/yog/world/state".to_owned()),
            (
                "PATH".to_owned(),
                "/d/yog/world/tools:/usr/bin:/bin".to_owned()
            ),
        ]
    );
    assert_eq!(overrides(&compose(&amb)), ov);
}

/// §16.7 W9 end to end — **the agent's `bl` is yog's.** Seed the world's shim,
/// then spawn the *bare name* `bl` standing the world overrides, with the
/// ambient `PATH` pointed at nothing: the only thing that can resolve the name is
/// the world's own tools entry, and what runs is the shim, forwarding argv to
/// the target yog drives `bl` through. That chain — world `PATH` → shim → yog —
/// is the whole deliverable; an agent's bash inherits exactly this env (§8).
#[test]
fn a_bare_bl_spawned_in_the_world_resolves_to_the_seeded_shim() {
    let data = tempdir().unwrap();
    let bin = tempdir().unwrap();
    let log = bin.path().join("ran");
    // The shim's target: a recorder standing in for the yog binary.
    let target = bin.path().join("yog");
    fs::write(
        &target,
        format!("#!/bin/sh\nprintf '%s' \"$*\" > '{}'\n", log.display()),
    )
    .unwrap();
    fs::set_permissions(&target, fs::Permissions::from_mode(0o755)).unwrap();
    let data_s = data.path().to_string_lossy().into_owned();
    let amb = Env::from_pairs([
        ("HOME", "/h"),
        ("XDG_DATA_HOME", data_s.as_str()),
        ("PATH", "/nonexistent"),
    ]);
    tools::ensure_shim(&layout(&amb).tools, tools::BL, &Cli::new(&target)).unwrap();
    let stream = Cli::new(tools::BL)
        .with_env(overrides(&amb))
        .run(&["close", "bl-1a2b"])
        .unwrap();
    for _ in stream {}
    assert_eq!(fs::read_to_string(&log).unwrap(), "close bl-1a2b");
}

/// The §16.4 agent-correctness invariant: **reads and spawns agree.** yog
/// watches its balls clones dir through the *composed* world `Env` (a read);
/// every child spawns with the *standing* world [`overrides`] (§16.6 W2). A
/// recorder `bl` spawned in the world reports the `XDG_STATE_HOME` it received —
/// which must be the world state dir, so the clones dir it would write
/// (`$XDG_STATE_HOME/balls/clones`) is byte-for-byte the one yog watches. If the
/// two derivations ever drifted, an agent closing a ball would hit a *different*
/// clone than yog renders; this test is the guard.
#[test]
fn watched_clones_dir_equals_the_dir_a_world_spawned_bl_writes() {
    let data = tempdir().unwrap();
    let bin = tempdir().unwrap();
    let log = bin.path().join("state");
    // Ambient env anchored on a real temp data root; the world nests under it.
    let data_s = data.path().to_string_lossy().into_owned();
    let amb = Env::from_pairs([("HOME", "/h"), ("XDG_DATA_HOME", data_s.as_str())]);
    let world = compose(&amb);
    let ov = overrides(&amb);
    // A recorder `bl` that reports the XDG_STATE_HOME it was spawned with (printf
    // builtin only — no fork to race the coverage ptrace engine).
    let path = bin.path().join("bl");
    fs::write(
        &path,
        format!(
            "#!/bin/sh\nprintf '%s' \"$XDG_STATE_HOME\" > '{}'\n",
            log.display()
        ),
    )
    .unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    // Spawn it in the world (standing overrides), then drain to completion.
    let stream = Cli::new(path).with_env(ov).run(&[]).unwrap();
    for _ in stream {}
    // The child was spawned with XDG_STATE_HOME = the world state dir …
    let child_state = fs::read_to_string(&log).unwrap();
    assert_eq!(Path::new(&child_state), layout(&amb).state);
    // … so the clones dir it would write is exactly the one yog watches (derived
    // through the composed world `Env`). Reads and spawns agree.
    let spawned_clones = Path::new(&child_state).join("balls").join("clones");
    assert_eq!(spawned_clones, world.balls_clones_dir());
}
