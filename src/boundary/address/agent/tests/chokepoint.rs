//! **The receipt, used** (bl-49bc): the handle a `/prompt` hands back — the
//! minted §3.3 name — driven through both §8.5 chokepoints against a
//! conversation whose id it is not.
//!
//! These are the beats an operator cares about, and the second is the one the
//! defect was dangerous for: `Floor` writes yog's **own** standing policy row,
//! keyed by agent id and matched by hyphenated prefix, so a row landed under a
//! display name would read as policy, log as policy, and govern nothing. The
//! test is therefore not "it succeeds" but *what the row is keyed on*.

use crate::boundary::dispatch::{Caller, Deps, dispatch};
use crate::boundary::reply::Reply;
use crate::boundary::{Action, Query, answer};
use crate::cli_outbound::Cli;
use crate::git_tree::{Agent, AgentState};
use crate::ui_state::UiState;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::{TempDir, tempdir};

/// The root the fire minted, and the name its receipt handed back.
const ROOT: &str = "20260101T000000Z-aaaa";
const MINTED: &str = "pale-otter";

/// A world one named conversation deep, published exactly as the §7.2 worker
/// publishes what it derived.
fn world() -> (TempDir, PathBuf, Deps) {
    let dir = tempdir().expect("tempdir");
    let ws = dir.path().join("names").join("alba");
    let named = Agent {
        name: Some(MINTED.to_owned()),
        ..crate::boundary::tests::agent(ROOT, AgentState::Quiescent, 1)
    };
    let deps = Deps {
        // Exits 0 on every platform the suite runs on: enough to prove which
        // argv a resolved gesture spawned without driving a conversation.
        lernie: Cli::new("/usr/bin/true"),
        bl: Cli::new("/no/such/bl"),
        state_root: dir.path().join("state"),
        home: dir.path().join("home"),
        yog_data_root: dir.path().join("data"),
        balls_state_root: dir.path().join("balls"),
        yog_binary: PathBuf::from("/no/such/yog"),
        world: crate::test_support::no_world(),
        snapshot: Arc::new(crate::boundary::tests::snapshot(
            &ws,
            "alba",
            vec![named],
            vec![],
        )),
        caller: Caller::default(),
    };
    (dir, ws, deps)
}

fn ui() -> UiState {
    UiState::open(PathBuf::from("/nonexistent/ui.json"))
}

/// **The read half**: `/agent` asked by the minted name answers about the
/// root — `present:false` over an empty derivation was the symptom, and the
/// reply now identifies itself by the id it resolved to.
#[test]
fn a_read_asked_by_the_minted_name_answers_about_the_root() {
    let (_dir, ws, deps) = world();
    let asked = Query::Agent {
        workspace: crate::naming::leaf(&ws),
        agent: MINTED.to_owned(),
    };
    match answer::answer(&asked, &deps, &ui(), 0).expect("answered") {
        Reply::Agent(view) => {
            assert_eq!(view.agent_id, ROOT, "the id the name resolved to");
            assert_eq!(view.name, MINTED, "and the §3.3 ladder still names it");
        }
        other => panic!("a seat read answers an agent, not {other:?}"),
    }
}

/// **The write half, and the dangerous one**: the standing floor row is keyed on
/// the resolved **id**, so a gesture spelling the display name cannot leave
/// policy that governs nothing.
#[test]
fn a_floor_written_by_name_is_keyed_on_the_resolved_id() {
    let (_dir, ws, deps) = world();
    let landed = dispatch(
        &deps,
        &mut ui(),
        "1000",
        &Action::Floor {
            workspace: crate::naming::leaf(&ws),
            agent: MINTED.to_owned(),
            raised: true,
        },
    );
    assert_eq!(landed.expect("written"), Reply::Floored { standing: true });
    let rows = crate::opslog::tail(&deps.state_root, usize::MAX);
    assert_eq!(rows.len(), 1, "one row, nothing else");
    assert_eq!(rows[0].argv, vec!["yog-control", "floor", ROOT, "raise"]);
}

/// And the refusal, at the same door: a name nothing wears reaches no executor
/// at all — no ops row, no spawn, no policy.
#[test]
fn an_unknown_name_refuses_before_any_executor_runs() {
    let (_dir, ws, deps) = world();
    let why = dispatch(
        &deps,
        &mut ui(),
        "1000",
        &Action::Floor {
            workspace: crate::naming::leaf(&ws),
            agent: "grey-heron".to_owned(),
            raised: true,
        },
    )
    .expect_err("refused");
    assert!(why.contains("unknown conversation"), "{why}");
    assert!(
        crate::opslog::tail(&deps.state_root, usize::MAX).is_empty(),
        "nothing ran"
    );
}
