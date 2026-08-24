//! The boundary glue a frame's gestures land in (§8.5), driven from where the
//! **engine** stands ([`crate::test_support::engine::act`]) — since bl-1747 the
//! window posts every act over the wire and holds no dispatch of its own, so
//! these read the chokepoint the wire's listener reaches, over a `ui.json`
//! opened fresh per gesture. Shares [`super::world`]'s hermetic fixture; spawns
//! a fake `lernie`, so it lives in its own file (the `prepare.rs` discipline).

use super::{model_focused, world};
use crate::boundary::Action;
use crate::boundary::reply::Reply;
use crate::cli_outbound::Cli;
use crate::opslog::{self, DETACHED_EXIT};
use crate::start::Prepared;
use crate::test_support::engine;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tempfile::tempdir;

/// The conversation the retarget and the fork address. **Id-shaped** (ARCH
/// §2.3's compact stamp): the §8.5 chokepoint resolves the conversation a
/// gesture names (bl-49bc), and an id reads as one on its own.
const AGENT: &str = "20260101T000000Z-c1";

/// An everything-succeeds fake `lernie`.
fn fake_lernie(dir: &Path) -> Cli {
    let path = dir.join("lernie");
    fs::write(&path, "#!/bin/sh\nexit 0\n").unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    Cli::new(path)
}

/// The composer's own hand-off, as the two `Prompt` drives below spend it.
fn prepared(w: &super::World) -> Prepared {
    Prepared {
        workspace: crate::naming::leaf(&w.ws_cobalt),
        binding: Some(w.ws_cobalt.clone()),
        goal: "prefill".into(),
        origin: crate::opslog::Origin::Conversation,
    }
}

#[test]
fn the_engines_dispatch_is_the_one_chokepoint_a_posted_act_reaches() {
    let bin = tempdir().unwrap();
    let w = world();
    let (_c, m) = model_focused(&w, &w.ws_cobalt);
    let lernie = fake_lernie(bin.path());
    let deps = m.boundary_deps(&lernie, &Cli::new("/no/bl"));
    let action = Action::Scan {
        workspace: crate::naming::leaf(&(w.ws_cobalt.clone())),
    };
    let Reply::Outcome(outcome) = engine::act(&m, &deps, "TS", &action).unwrap() else {
        panic!("a verb answers an outcome");
    };
    assert!(outcome.ok());
    let ops = opslog::tail(&w.roots.yog_state, 4);
    assert_eq!(ops.last().map(|e| e.ts.clone()), Some("TS".to_owned()));
}

/// The §9.4 exit rides the same chokepoint (bl-2d19), and what reaches the
/// substrate is the workspace-bound `lernie retarget <ws> <agent>` — asserted
/// off the §4.2 trail, which records the argv actually spawned.
#[test]
fn the_retarget_exit_spawns_the_bound_lernie_verb() {
    let bin = tempdir().unwrap();
    let w = world();
    let (_c, m) = model_focused(&w, &w.ws_cobalt);
    let lernie = fake_lernie(bin.path());
    let deps = m.boundary_deps(&lernie, &Cli::new("/no/bl"));
    let action = Action::Retarget {
        workspace: crate::naming::leaf(&(w.ws_cobalt.clone())),
        agent: AGENT.into(),
    };
    let Reply::Outcome(outcome) = engine::act(&m, &deps, "TR", &action).unwrap() else {
        panic!("a verb answers an outcome");
    };
    assert!(outcome.ok());
    let ops = opslog::tail(&w.roots.yog_state, 4);
    let last = ops.last().expect("the verb is on the trail");
    assert_eq!(
        last.argv[1..],
        [
            "retarget".to_owned(),
            w.ws_cobalt.display().to_string(),
            AGENT.to_owned()
        ]
    );
    assert_eq!(last.cwd, w.ws_cobalt.display().to_string());
}

#[test]
fn the_prompt_door_launches_detached_and_mints_off_the_seat_s_own_seed() {
    let bin = tempdir().unwrap();
    let w = world();
    let (_c, m) = model_focused(&w, &w.ws_cobalt);
    let lernie = fake_lernie(bin.path());
    let deps = m.boundary_deps(&lernie, &Cli::new("/no/bl"));
    let action = Action::Prompt {
        prepared: prepared(&w),
        goal: "go".into(),
        // The §3.3 seed is the firing seat's, carried on the gesture since
        // bl-1747 rather than reached into `Deps` — this is a window's own
        // preview seed crossing with the act it predicted.
        seed: Some(7),
    };
    let Reply::Started { conversation } = engine::act(&m, &deps, "T1", &action).unwrap() else {
        panic!("the prompt door answers the minted name");
    };
    assert!(
        !conversation.is_empty(),
        "the minted conversation name rides back"
    );
    let ops = opslog::tail(&w.roots.yog_state, 4);
    assert_eq!(
        ops.last().map(|e| e.exit),
        Some(DETACHED_EXIT),
        "the handoff's §4.2 sentinel row"
    );

    // The same door refuses when the fork cannot land, error text riding back.
    let dead = m.boundary_deps(&Cli::new("/no/such/lernie"), &Cli::new("/no/bl"));
    let err = engine::act(&m, &dead, "T2", &action).unwrap_err();
    assert!(!err.is_empty());
}

/// **A caller that predicted no name still mints one** (bl-1747): `seed: None`
/// is the deposited line and the §4.3 loop, and the door draws off this
/// moment's stamp instead. The default lives at the door, so no intake carries
/// a copy of it.
#[test]
fn a_seedless_prompt_mints_off_the_stamp() {
    let bin = tempdir().unwrap();
    let w = world();
    let (_c, m) = model_focused(&w, &w.ws_cobalt);
    let lernie = fake_lernie(bin.path());
    let deps = m.boundary_deps(&lernie, &Cli::new("/no/bl"));
    let action = Action::Prompt {
        prepared: prepared(&w),
        goal: "go".into(),
        seed: None,
    };
    let Reply::Started { conversation } = engine::act(&m, &deps, "T9", &action).unwrap() else {
        panic!("the prompt door answers the minted name");
    };
    assert!(!conversation.is_empty(), "a name was minted regardless");
}

/// The §3.5 spend ceiling gates the frame's own door, not just the dispatch
/// match: one gate, every spawn path (bl-56d5). A `ceiling` of 0 beside a
/// priced table is the hard stop, so this needs no spend fixture.
#[test]
fn the_spend_ceiling_refuses_the_fire_and_says_so_on_the_trail() {
    let bin = tempdir().unwrap();
    let w = world();
    fs::create_dir_all(&w.roots.yog_state).unwrap();
    fs::write(
        w.roots.ui_json(),
        r#"{"v":1,"prices":{"opus":{"input":1}},"ceiling":0}"#,
    )
    .unwrap();
    let (_c, m) = model_focused(&w, &w.ws_cobalt);
    let lernie = fake_lernie(bin.path());
    let deps = m.boundary_deps(&lernie, &Cli::new("/no/bl"));
    let action = Action::Prompt {
        prepared: prepared(&w),
        goal: "go".into(),
        seed: Some(7),
    };
    let err = engine::act(&m, &deps, "T3", &action).unwrap_err();
    assert!(err.contains("spend ceiling reached"), "{err}");
    let ops = opslog::tail(&w.roots.yog_state, 4);
    let last = ops.last().expect("the refusal is on the trail");
    assert_eq!(last.ts, "T3");
    assert_eq!(last.argv.first().map(String::as_str), Some("yog-step"));
    assert!(
        last.argv.iter().any(|a| a == "ceiling"),
        "the row names the gate: {:?}",
        last.argv
    );
    assert!(
        !last.argv.iter().any(|a| a == "prompt"),
        "nothing was spawned: {:?}",
        last.argv
    );
}

/// **bl-a80a end to end.** The gate's scope is the world, so a fire into
/// `cobalt` — which has spent nothing — is refused by what `spare` spent. This
/// is the one drive that proves the door actually enumerates the §3.1 roster:
/// the test above uses a `0` ceiling, which refuses over an empty roster too.
#[test]
fn spend_in_another_workspace_refuses_a_fire_into_an_idle_one() {
    let bin = tempdir().unwrap();
    let w = world();
    let step = w
        .ws_spare
        .join("steps")
        .join("20260101T000000Z-x")
        .join("001");
    fs::create_dir_all(&step).unwrap();
    fs::write(
        step.join("response.json"),
        r#"{"type":"usage","input_tokens":3000000}"#,
    )
    .unwrap();
    fs::write(step.join("request.json"), r#"{"model":"opus"}"#).unwrap();
    fs::write(
        w.roots.ui_json(),
        r#"{"v":1,"prices":{"opus":{"input":1}},"ceiling":2}"#,
    )
    .unwrap();
    let (_c, m) = model_focused(&w, &w.ws_cobalt);
    let deps = m.boundary_deps(&fake_lernie(bin.path()), &Cli::new("/no/bl"));
    let action = Action::Prompt {
        prepared: prepared(&w),
        goal: "go".into(),
        seed: Some(7),
    };
    let err = engine::act(&m, &deps, "T3", &action).unwrap_err();
    assert!(
        err.contains("$3.00"),
        "the sibling's spend is the figure: {err}"
    );
}

/// **S12-T5 three-spellings** (the executor half): one attempt crosses the
/// boundary as the ordinary `lernie dispatch --from`, and the whole argv —
/// role, parent, verbatim goal, fork point, skill pin — lands on the §4.2
/// trail. A cohort is this, N times: nothing here knows how many there were.
#[test]
fn an_attempt_dispatches_the_ordinary_fork_and_logs_its_whole_argv() {
    let bin = tempdir().unwrap();
    let w = world();
    let (_c, m) = model_focused(&w, &w.ws_cobalt);
    let lernie = fake_lernie(bin.path());
    let deps = m.boundary_deps(&lernie, &Cli::new("/no/bl"));
    let action = Action::Fork {
        workspace: crate::naming::leaf(&(w.ws_cobalt.clone())),
        parent: AGENT.into(),
        attempt: crate::fork::Attempt {
            from: "aaaa1111".into(),
            role: "worker".into(),
            skills: vec!["bash".into()],
        },
        goal: "try it the other way".into(),
    };
    let Reply::Outcome(outcome) = engine::act(&m, &deps, "TS", &action).unwrap() else {
        panic!("a verb answers an outcome");
    };
    assert!(outcome.ok());
    let argv = opslog::tail(&w.roots.yog_state, 4)
        .last()
        .map(|e| e.argv.clone())
        .unwrap_or_default();
    assert!(argv.contains(&"dispatch".to_owned()), "{argv:?}");
    assert!(argv.contains(&"worker".to_owned()), "{argv:?}");
    assert!(argv.contains(&AGENT.to_owned()), "{argv:?}");
    assert!(argv.contains(&"--from".to_owned()), "{argv:?}");
    assert!(argv.contains(&"aaaa1111".to_owned()), "{argv:?}");
    assert!(
        argv.contains(&"try it the other way".to_owned()),
        "{argv:?}"
    );
    // The pin's source is the **world's** pool, never an ambient lernie's.
    let pin = argv.iter().find(|a| a.starts_with("skills/bash/SKILL.md="));
    let pin = pin.expect("the skill rides as a pin");
    assert!(pin.contains("world/lernie/skills/bash/SKILL.md"), "{pin}");
}
