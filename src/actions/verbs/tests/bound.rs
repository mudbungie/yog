//! bl-bf79: the wall rides **every** §8.2 lernie verb, asserted from inside the
//! child.
//!
//! The bug this pins was invisible to an argv assertion and to an exit-code one
//! — `lernie message` spawned the right words and exited 0, and the driver it
//! detach-launched then died on `bz: no workspace in this environment`. So the
//! recorder here does not report what yog *passed*; it reports what the child
//! **observed** in its own environment, which is the only place `YOG_WALL` is a
//! fact.
//!
//! The sweep is over the whole verb family in one test, not one test per verb:
//! the claim is *"no workspace verb can spawn outside the sphere"*, and a
//! per-verb test proves only the verbs somebody remembered to write one for.

use super::*;

/// A `lernie` that answers with the two workspace-bound env facts it inherited,
/// in the order [`Bound`] lays them. An unwalled spawn prints an empty first
/// field — exactly what the operator's `stove` and `procedure` conversations hit.
const ENV_BODY: &str = "#!/bin/sh\nprintf '%s|%s\\n' \"$YOG_WALL\" \"$YOG_NAME\"\nexit 0\n";

/// A fire whose argv is irrelevant here — `fork`'s spawn is the subject, not its
/// flags (those are pinned by `fork`'s own tests).
fn fire(ws: &Path) -> crate::fork::Fire {
    crate::fork::Fire::at(
        ws,
        "c-parent",
        &crate::fork::Attempt {
            from: "config/default".to_owned(),
            role: "builder".to_owned(),
            skills: vec![],
        },
        "do the thing",
        Path::new("/no/yog-data"),
    )
}

#[test]
fn every_workspace_bound_verb_spawns_inside_the_sphere() {
    let (w, _g) = World::new("lernie", ENV_BODY);
    // The wall is a pure query on the anchor and the §3.1 leaf — recomputed
    // here from `world::wall`, so this asserts the same derivation the §9
    // config panes and the embedded `bz` read, not a second spelling of it.
    let env = Env::from_pairs([("XDG_DATA_HOME", w.state.path().display().to_string())]);
    let leaf = crate::naming::leaf(&w.cwd);
    let expected = format!(
        "{}|{leaf}\n",
        crate::world::wall::root_of(&env, &w.cwd).display()
    );
    assert!(!leaf.is_empty(), "the fixture workspace has a §3.1 name");

    let b = w.bound();
    let seen = [
        message(&b, w.state.path(), "TS", "a-1", "hi").unwrap(),
        stop(&b, w.state.path(), "TS", "a-1", false).unwrap(),
        scan(&b, w.state.path(), "TS").unwrap(),
        fork(&b, w.state.path(), "TS", &fire(&w.cwd)).unwrap(),
    ];
    for out in &seen {
        assert_eq!(out.stdout, expected, "an unwalled workspace-bound spawn");
    }
}

/// The two facts are layered **on top of** the world `Cli`'s standing set, never
/// in place of it: a bound spawn still nests (§16.2), which is what lets the
/// revived driver find yog's own lernie home *and* the sphere's providers.
#[test]
fn binding_extends_the_standing_world_env() {
    let (w, _g) = World::new(
        "lernie",
        "#!/bin/sh\nprintf '%s\\n' \"$LERNIE_HOME\"\nexit 0\n",
    );
    let world = Env::from_pairs([("XDG_DATA_HOME", w.state.path().display().to_string())]);
    let nested = w.cli.clone().with_env(vec![(
        "LERNIE_HOME".to_owned(),
        "/nested/lernie".to_owned(),
    )]);
    let bound = Bound::at(&nested, &world, &w.cwd);
    assert_eq!(
        scan(&bound, w.state.path(), "TS").unwrap().stdout,
        "/nested/lernie\n"
    );
    // …and the binding is the workspace's, whole: cwd and the `<ws>` argv word
    // are one fact, so no caller can pass a workspace the spawn does not run in.
    assert_eq!(bound.workspace(), w.cwd.as_path());
    assert_eq!(bound.workspace_arg(), w.cwd.display().to_string());
    assert_ne!(bound.cli(), &nested, "binding is not a no-op");
}
