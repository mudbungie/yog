//! STORIES **S8-T1** world-compose: the composed env overrides the nesting set
//! and nothing else, leaves the anchor ambient, and re-deriving the world
//! through the world env is a **fixed point** — no bootstrap special case
//! (STORIES S8.1, DESIGN §16.2, §16.6).
//!
//! **The row's second premise was reversed by the blast-radius
//! ruling.** It read "leaves brazen's paths ambient — one bz, one login". They
//! are not ambient any more: brazen's config, credentials and model cache
//! resolve inside the *focused workspace's wall* (§16.2 as amended, bl-c0e2),
//! so the world composes no brazen var at all and the world env — which names
//! no workspace — answers no brazen paths whatever the ambient environment
//! carries. That is what this row now asserts.
//!
//! **The row's premise drifted: there are THREE overrides, not two.** STORIES
//! says "exactly `LERNIE_HOME` and `XDG_STATE_HOME`"; §16.7 W9 added the
//! `PATH` prepend that puts `<world>/tools` in front of the ambient search
//! path, so an agent's bare `bl` is the world's own shim (§16.4). It belongs to
//! the same set for the same reason — it is a pure function of the anchor, and
//! it is idempotent under re-composition, which is what keeps the fixed point.

#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::path::Path;
use yog::world;
use yog::xdg::Env;

/// The composed overrides as a map, for name-wise assertions.
fn map(env: &Env) -> BTreeMap<String, String> {
    world::overrides(env).into_iter().collect()
}

/// STORIES **S8-T1** world-compose.
#[test]
fn s8_t1_the_world_nests_three_vars_and_re_composing_is_a_fixed_point() {
    // The ambient snapshot, read once. Everything below is path algebra over
    // its anchor — this test writes nothing and spawns nothing, so the real
    // environment is a legitimate (and more honest) input than a synthetic one.
    let ambient = Env::from_env();

    // --- The override set names exactly the nesting vars.
    let ov = map(&ambient);
    assert_eq!(
        ov.keys().cloned().collect::<Vec<_>>(),
        ["LERNIE_HOME", "PATH", "XDG_STATE_HOME"],
        "the nesting set, and nothing else"
    );
    // Each value is the layout's own path — one derivation, so the dir yog
    // watches and the dir a spawned child writes cannot be two answers.
    let layout = world::layout(&ambient);
    assert_eq!(ov["LERNIE_HOME"], layout.lernie.to_string_lossy());
    assert_eq!(ov["XDG_STATE_HOME"], layout.state.to_string_lossy());
    assert!(
        ov["PATH"].starts_with(&*layout.tools.to_string_lossy()),
        "the world's tools lead the search path: {}",
        ov["PATH"]
    );
    if let Some(path) = ambient.search_path() {
        assert!(
            ov["PATH"].ends_with(&path),
            "the ambient path follows behind"
        );
    }

    // --- What stays ambient: the ANCHOR, and only the anchor (overriding it
    // would recurse). Nothing brazen-shaped is composed here at all.
    let world_env = world::compose(&ambient);
    assert!(
        !ov.contains_key("XDG_DATA_HOME"),
        "the anchor stays ambient"
    );
    assert!(
        !ov.contains_key("BRAZEN_CONFIG") && !ov.contains_key(world::wall::YOG_WALL),
        "the world names no sphere — the wall is one layer in"
    );
    // The world alone names no workspace, so it answers no brazen paths: an
    // ambient `BRAZEN_CONFIG`/`XDG_*` in the operator's shell buys nothing.
    assert_eq!(world_env.wall(), None);
    assert_eq!(
        yog::config_edit::brazen::BrazenPaths::of(&world_env),
        None,
        "no wall, no brazen — never the machine's own"
    );
    // Lensed on a workspace, all three land inside that sphere and nowhere else.
    let corp = world::wall::env(&world_env, Path::new("/ws/corp"));
    let paths = yog::config_edit::brazen::BrazenPaths::of(&corp).expect("a focused wall");
    let wall = layout.root.join("walls/corp");
    assert_eq!(paths.config, wall.join("brazen/config.toml"));
    assert_eq!(paths.credentials_dir, wall.join("brazen/credentials"));
    assert_eq!(paths.models_cache_dir, wall.join("brazen/models"));
    // …and the spawn layer names the same wall the read lens does.
    assert_eq!(
        world::wall::pairs(&world_env, Path::new("/ws/corp")),
        vec![(
            world::wall::YOG_WALL.to_owned(),
            wall.to_string_lossy().into_owned()
        )]
    );
    assert_eq!(world_env.yog_data_root(), ambient.yog_data_root());

    // --- The world's own reads land inside it.
    assert!(world_env.lernie_data_root().starts_with(&layout.lernie));
    assert!(world_env.balls_state_root().starts_with(&layout.state));

    // The layout is pure path algebra over the anchor — no IO, no env — which
    // is what lets the start flow derive it from a plan input rather than
    // re-snapshotting the process environment.
    let under = world::layout_under(Path::new("/anchor"));
    assert_eq!(under.root, Path::new("/anchor/world"));
    assert_eq!(under.lernie, Path::new("/anchor/world/lernie"));
    assert_eq!(under.state, Path::new("/anchor/world/state"));
    assert_eq!(under.tools, Path::new("/anchor/world/tools"));

    // --- Fixed point: composing the world again reproduces it exactly. There
    // is no "first time" branch — the second application is the same function
    // over the same anchor, and the PATH prepend carries that property itself
    // rather than stacking a second tools entry.
    let twice = world::compose(&world_env);
    assert_eq!(map(&world_env), map(&twice), "re-composition is idempotent");
    assert_eq!(world_env.lernie_data_root(), twice.lernie_data_root());
    assert_eq!(world_env.balls_state_root(), twice.balls_state_root());
    assert_eq!(
        world::layout(&world_env).root,
        layout.root,
        "the anchor is stable, so the world never nests inside itself"
    );
    // Three times, for the same reason it holds twice.
    assert_eq!(map(&twice), map(&world::compose(&twice)));
}
