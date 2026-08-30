//! **The §3.5 spend ceiling at the frame's own door** (bl-56d5, bl-a80a) —
//! split off [`super`] at the cap on the seam it already had: every other beat
//! there asserts what a gesture *routed to*, and these two assert what one was
//! **refused** by, and that the refusal is a §4.2 trail row rather than a spawn.
//!
//! Two beats because the gate has two claims and the first cannot carry the
//! second: a `0` ceiling refuses over an empty roster too, so only a fire into
//! an idle workspace refused by a *sibling's* spend proves the door enumerates
//! the §3.1 roster at all.

use super::{fake_litany, model_focused, prepared, world};
use crate::boundary::Action;
use crate::cli_outbound::Cli;
use crate::opslog;
use crate::test_support::engine;
use std::fs;
use tempfile::tempdir;

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
    let litany = fake_litany(bin.path());
    let deps = m.boundary_deps(&litany, &Cli::new("/no/bl"));
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
    let deps = m.boundary_deps(&fake_litany(bin.path()), &Cli::new("/no/bl"));
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
