//! The frame-side boundary glue (§8.5): [`AppModel::dispatch`] is the same
//! chokepoint the deposit consumer runs, over this instance's `ui.json`, and
//! [`AppModel::fire_prompt`] is the Prompt door plus the §3.4 start claim.
//! Shares [`super::world`]'s hermetic fixture; spawns a fake `lernie`, so it
//! lives in its own file (the `prepare.rs` discipline).

use super::{model_focused, world};
use crate::boundary::Action;
use crate::boundary::reply::Reply;
use crate::cli_outbound::Cli;
use crate::opslog::{self, DETACHED_EXIT};
use crate::start::Prepared;
use crate::test_support::spawn_guard;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tempfile::tempdir;

/// An everything-succeeds fake `lernie`.
fn fake_lernie(dir: &Path) -> Cli {
    let path = dir.join("lernie");
    fs::write(&path, "#!/bin/sh\nexit 0\n").unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    Cli::new(path)
}

#[test]
fn the_frames_dispatch_is_the_boundary_chokepoint_with_its_own_ui() {
    let bin = tempdir().unwrap();
    let w = world();
    let (_c, mut m) = model_focused(&w, &w.ws_cobalt);
    let _g = spawn_guard();
    let lernie = fake_lernie(bin.path());
    let deps = m.boundary_deps(&lernie, &Cli::new("/no/bl"));
    let action = Action::Scan {
        workspace: w.ws_cobalt.clone(),
    };
    let Reply::Outcome(outcome) = m.dispatch(&deps, "TS", &action).unwrap() else {
        panic!("a verb answers an outcome");
    };
    assert!(outcome.ok());
    let ops = opslog::tail(&w.roots.yog_state, 4);
    assert_eq!(ops.last().map(|e| e.ts.clone()), Some("TS".to_owned()));
}

#[test]
fn fire_prompt_launches_detached_and_holds_the_start_claim() {
    let bin = tempdir().unwrap();
    let w = world();
    let (_c, mut m) = model_focused(&w, &w.ws_cobalt);
    let _g = spawn_guard();
    let lernie = fake_lernie(bin.path());
    let prepared = Prepared {
        name: "cobalt".into(),
        workspace: w.ws_cobalt.clone(),
        binding: Some(w.ws_cobalt.clone()),
        goal: "prefill".into(),
        origin: crate::opslog::Origin::Conversation,
    };
    let minted = m
        .fire_prompt(&lernie, &Cli::new("/no/bl"), &prepared, "go", 7, "T1")
        .unwrap();
    assert!(
        !minted.is_empty(),
        "the minted conversation name rides back"
    );
    let ops = opslog::tail(&w.roots.yog_state, 4);
    assert_eq!(
        ops.last().map(|e| e.exit),
        Some(DETACHED_EXIT),
        "the handoff's §4.2 sentinel row"
    );

    // The same door refuses when the fork cannot land, error text riding back.
    let err = m
        .fire_prompt(
            &Cli::new("/no/such/lernie"),
            &Cli::new("/no/bl"),
            &prepared,
            "go",
            7,
            "T2",
        )
        .unwrap_err();
    assert!(!err.is_empty());
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
    let (_c, mut m) = model_focused(&w, &w.ws_cobalt);
    let _g = spawn_guard();
    let lernie = fake_lernie(bin.path());
    let prepared = Prepared {
        name: "cobalt".into(),
        workspace: w.ws_cobalt.clone(),
        binding: Some(w.ws_cobalt.clone()),
        goal: "prefill".into(),
        origin: crate::opslog::Origin::Conversation,
    };
    let err = m
        .fire_prompt(&lernie, &Cli::new("/no/bl"), &prepared, "go", 7, "T3")
        .unwrap_err();
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

/// **S12-T5 three-spellings** (the executor half): one attempt crosses the
/// boundary as the ordinary `lernie dispatch --from`, and the whole argv —
/// role, parent, verbatim goal, fork point, skill pin — lands on the §4.2
/// trail. A cohort is this, N times: nothing here knows how many there were.
#[test]
fn an_attempt_dispatches_the_ordinary_fork_and_logs_its_whole_argv() {
    let bin = tempdir().unwrap();
    let w = world();
    let (_c, mut m) = model_focused(&w, &w.ws_cobalt);
    let _g = spawn_guard();
    let lernie = fake_lernie(bin.path());
    let deps = m.boundary_deps(&lernie, &Cli::new("/no/bl"));
    let action = Action::Fork {
        workspace: w.ws_cobalt.clone(),
        parent: "c-1".into(),
        attempt: crate::fork::Attempt {
            from: "aaaa1111".into(),
            role: "worker".into(),
            skills: vec!["bash".into()],
        },
        goal: "try it the other way".into(),
    };
    let Reply::Outcome(outcome) = m.dispatch(&deps, "TS", &action).unwrap() else {
        panic!("a verb answers an outcome");
    };
    assert!(outcome.ok());
    let argv = opslog::tail(&w.roots.yog_state, 4)
        .last()
        .map(|e| e.argv.clone())
        .unwrap_or_default();
    assert!(argv.contains(&"dispatch".to_owned()), "{argv:?}");
    assert!(argv.contains(&"worker".to_owned()), "{argv:?}");
    assert!(argv.contains(&"c-1".to_owned()), "{argv:?}");
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
