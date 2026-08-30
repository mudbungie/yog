//! STORIES **S8-T2** every-spawn-nested: every dispatched verb — the start
//! steps, the per-agent verbs, the detached prompt — carries the world
//! overrides, asserted **through the world `Cli`** so a new call site inherits
//! them by construction; and the dir yog watches is the dir a spawned `bl`
//! writes (STORIES S8.1/S8.2, DESIGN §16.2, §16.6 W2).
//!
//! The "by construction" claim is structural: the standing env rides on the
//! `Cli` itself, not on each call, so a verb cannot forget it — there is no
//! per-call-site argument to omit. This test spends one world `Cli` across
//! several unrelated verbs and asserts the nesting on **every** recorded spawn,
//! which is what makes the structural claim observable: a verb that built its
//! own `Cli` would show up here as a bare child.

#![allow(clippy::unwrap_used)]

use crate::support::Recorder;
use tempfile::tempdir;
use yog::actions::verbs;
use yog::binding::{work_worktree_path, workspace_path};
use yog::cli_outbound::Cli;
use yog::projects::join::JoinState;
use yog::start::{self, BallSpec, Deps, Payload, StartInputs};
use yog::world;
use yog::xdg::Env;

/// One recorded spawn observed `key` = `want` in its own environment. Named
/// per-invocation so a failure says which argv carried the un-nested value.
fn nests(inv: &crate::support::Invocation, key: &str, want: &std::path::Path) {
    assert_eq!(
        inv.env.get(key).map(String::as_str),
        Some(want.to_string_lossy().as_ref()),
        "un-nested {key} on {:?}",
        inv.argv
    );
}

/// STORIES **S8-T2** every-spawn-nested.
#[test]
fn s8_t2_one_world_cli_nests_every_verb_it_spawns() {
    let (bin, state) = (tempdir().unwrap(), tempdir().unwrap());
    let (yog, balls, home, project) = (
        tempdir().unwrap(),
        tempdir().unwrap(),
        tempdir().unwrap(),
        tempdir().unwrap(),
    );
    // The world composed off a yog data root of our choosing — the same
    // derivation production uses, so the values asserted below are the real
    // ones and not a fixture's idea of them.
    let layout = world::layout_under(yog.path());
    let ambient = Env::from_env();
    let mut overrides = world::overrides(&ambient);
    // Re-point the two nesting vars at this test's own anchor; PATH's prepend
    // is asserted by S8-T1 and left as composed.
    for (key, value) in &mut overrides {
        if key == "LITANY_HOME" {
            *value = layout.litany.to_string_lossy().into_owned();
        } else if key == "XDG_STATE_HOME" {
            *value = layout.state.to_string_lossy().into_owned();
        }
    }

    let canonical = work_worktree_path(balls.path(), project.path(), "bl-7", None);
    let litany_rec = Recorder::new(bin.path(), "litany").authoring_workspaces();
    let bl_rec = Recorder::new(bin.path(), "bl").on("claim", &canonical.to_string_lossy(), 0);
    // The ONE construction point. Every verb below takes these by reference.
    let litany = Cli::new(litany_rec.path()).with_env(overrides.clone());
    let bl = Cli::new(bl_rec.path()).with_env(overrides.clone());

    let workspace = workspace_path(yog.path(), "cobalt-gecko");
    let deps = Deps {
        bl: bl.clone(),
        litany: litany.clone(),
        state_root: state.path().to_path_buf(),
        yog_binary: std::path::PathBuf::from("/no/yog"),
    };
    let inputs = StartInputs {
        workspace: workspace.clone(),
        repo: Some(project.path().to_path_buf()),
        payload: Payload::Ball {
            project: yog::naming::leaf(project.path()),
            ball: BallSpec::Existing {
                id: "bl-7".to_owned(),
                title: "Wire it".to_owned(),
                body: "Do the thing.".to_owned(),
                join: JoinState::ReadyStartable,
                tags: Vec::new(),
            },
        },
        home: home.path().to_path_buf(),
        yog_data_root: yog.path().to_path_buf(),
        balls_state_root: balls.path().to_path_buf(),
        conversation_names: Vec::new(),
    };

    // --- The start steps (litany prime, litany new, bl claim).
    start::prepare(&deps, &inputs, "T0").unwrap();
    // --- A per-agent verb, through the same `Cli`, bound to the workspace it is
    // about (§8.2, bl-bf79) — the binding LAYERS onto the world set, so these
    // spawns must show both nestings and the wall.
    let world_env = world::compose(&Env::from_env());
    let bound = verbs::Bound::at(&litany, &world_env, &workspace);
    verbs::scan(&bound, state.path(), "T1").unwrap();
    verbs::message(&bound, state.path(), "T2", "c-001", "hello").unwrap();
    // --- And a `bl` verb that is not part of a start at all.
    verbs::close(
        &bl,
        state.path(),
        "T3",
        project.path(),
        "bl-7",
        "cobalt-gecko",
    )
    .unwrap();

    // Every recorded spawn — whichever binary, whichever verb — observed the
    // world's nesting set. Not "the ones we remembered to check": all of them.
    let mut spawns = litany_rec.invocations();
    spawns.extend(bl_rec.invocations());
    assert!(
        spawns.len() >= 6,
        "the flow really spawned: {}",
        spawns.len()
    );
    for inv in &spawns {
        nests(inv, "LITANY_HOME", &layout.litany);
        nests(inv, "XDG_STATE_HOME", &layout.state);
    }

    // A workspace-BOUND spawn layers `YOG_NAME` and the workspace's `YOG_WALL`
    // ON TOP of the standing set — it extends the world env, it does not replace
    // it (which is how the shim's `--as <name>` and the nesting coexist).
    //
    // The wall half is bl-bf79's: without it the revived driver's first `bz`
    // dies "no workspace in this environment", which is a spawn that reached the
    // wire and produced nothing — a failure a `YOG_NAME`-only assertion missed
    // for as long as this test has existed.
    let named = litany_rec
        .invocations()
        .into_iter()
        .find(|i| i.env.contains_key("YOG_NAME"))
        .expect("a workspace-bound verb ran");
    assert_eq!(
        named.env.get("YOG_NAME").map(String::as_str),
        Some("cobalt-gecko")
    );
    assert!(named.env.contains_key("LITANY_HOME"), "and still nests");
    nests(
        &named,
        "YOG_WALL",
        &world::wall::root_of(&world_env, &workspace),
    );

    // --- One path, two readers. The dir a spawned `bl` writes its state into
    // is the dir yog watches: both are `layout.state`, from the one derivation
    // in `world::overrides` — not two agreeing constants.
    let watched = world::layout_under(yog.path()).state;
    let written = overrides
        .iter()
        .find(|(k, _)| k == "XDG_STATE_HOME")
        .map(|(_, v)| v.clone())
        .unwrap();
    assert_eq!(written, watched.to_string_lossy());

    // The bare `Cli` is a different value from the world one — the standing env
    // is what distinguishes them, so `resolve_in_world` cannot silently degrade
    // to `resolve` without this failing.
    assert_ne!(Cli::new(bl_rec.path()), bl, "a world Cli is not a bare one");
}
