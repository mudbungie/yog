//! bl-54c1 — **a blank goal never sends** (DESIGN §8.1 step 2, bl-6191),
//! asserted once per spelling of the fire.
//!
//! The invariant was stated in the doc and enforced nowhere: the predicate
//! that carried it had no caller, so every spelling of `Prompt` could spawn a
//! detached driver on an empty payload — spend for nothing, and a conversation
//! whose first entry is blank. It is seated in
//! [`dispatch::prompt`](crate::boundary::dispatch::prompt) for the reason the
//! §3.5 ceiling is: that door is the one place every spelling passes, so one
//! test covers the line, the deposit and the loop's own re-prompt at once.
//!
//! Each beat asserts the refusal **and** that nothing was paid for it: the
//! trail file is never created, so the gate stands ahead of the §4.11
//! confinement read and of the ceiling's own `["yog-step","ceiling"]` row.

use super::snapshot;
use crate::boundary::dispatch::{Deps, dispatch, prompt};
use crate::boundary::line::Context;
use crate::boundary::{Action, Gesture, codec, line};
use crate::cli_outbound::Cli;
use crate::opslog::Origin;
use crate::start::Prepared;
use crate::test_support::world::no_world;
use crate::ui_state::UiState;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const WS: &str = "alba";

/// The word every spelling is refused with — stated once here so a reworded
/// refusal fails loudly rather than letting one spelling drift off the others.
const REFUSAL: &str = "the goal is blank";

/// A `Prepared` for the bare rung, whose prefill is empty by construction
/// (§8.1 step 2) — the rung that can actually reach the door with nothing.
fn prepared() -> Prepared {
    Prepared {
        workspace: WS.to_owned(),
        binding: None,
        lineage: None,
        goal: String::new(),
        origin: Origin::Conversation,
    }
}

/// A `Deps` over one named workspace, with substrate binaries that do not
/// exist: a beat here that reached a spawn would fail loudly instead of
/// passing on a mock.
fn deps(state: &Path, yog: &Path) -> Deps {
    let ws = crate::binding::workspace_path(yog, WS);
    Deps {
        litany: Cli::new("/no/such/litany"),
        bl: Cli::new("/no/such/bl"),
        state_root: state.to_path_buf(),
        yog_binary: PathBuf::from("/no/such/yog"),
        world: no_world(),
        home: yog.to_path_buf(),
        yog_data_root: yog.to_path_buf(),
        balls_state_root: yog.to_path_buf(),
        snapshot: Arc::new(snapshot(&ws, WS, vec![], vec![])),
        caller: crate::boundary::dispatch::Caller::default(),
    }
}

/// The typed door itself — the spelling a seat's start glue and the §4.3
/// loop's re-prompt both enter through, holding a `Prepared` and no line.
#[test]
fn the_typed_door_refuses_an_empty_goal_and_pays_nothing() {
    let state = tempfile::tempdir().unwrap();
    let yog = tempfile::tempdir().unwrap();
    let deps = deps(state.path(), yog.path());
    let ui = UiState::open(PathBuf::from("/nonexistent/ui.json"));
    let ws = crate::binding::workspace_path(yog.path(), WS);
    let refusal = prompt(&deps, &ui, "T1", &ws, &prepared(), "", None).unwrap_err();
    assert!(refusal.contains(REFUSAL), "{refusal}");
    assert!(
        !state.path().join("ops.jsonl").exists(),
        "a blank goal costs no trail row: the gate stands ahead of the ceiling's"
    );
}

/// Whitespace is not a payload: the refusal is on the trimmed goal, so a
/// space, a tab and a newline are as blank as nothing at all.
#[test]
fn whitespace_only_is_blank() {
    let state = tempfile::tempdir().unwrap();
    let yog = tempfile::tempdir().unwrap();
    let deps = deps(state.path(), yog.path());
    let ui = UiState::open(PathBuf::from("/nonexistent/ui.json"));
    let ws = crate::binding::workspace_path(yog.path(), WS);
    for goal in [" ", "\t", "\n", " \t\r\n "] {
        let refusal = prompt(&deps, &ui, "T1", &ws, &prepared(), goal, None).unwrap_err();
        assert!(refusal.contains(REFUSAL), "{goal:?}: {refusal}");
    }
    assert!(!state.path().join("ops.jsonl").exists());
}

/// The deposit spelling: a `prompt` envelope as a headless caller writes it,
/// read back by the same codec and run through the §8.5 table — the arm that
/// stands between a wire gesture and the door.
#[test]
fn the_deposited_envelope_refuses_a_whitespace_goal() {
    let state = tempfile::tempdir().unwrap();
    let yog = tempfile::tempdir().unwrap();
    let deps = deps(state.path(), yog.path());
    let mut ui = UiState::open(PathBuf::from("/nonexistent/ui.json"));
    let wire = codec::encode(&Gesture::Act(Action::Prompt {
        prepared: prepared(),
        goal: "   ".to_owned(),
        seed: None,
    }));
    let Ok(Gesture::Act(action)) = codec::decode(&wire) else {
        unreachable!("the prompt envelope round-trips")
    };
    let refusal = dispatch(&deps, &mut ui, "T1", &action).unwrap_err();
    assert!(refusal.contains(REFUSAL), "{refusal}");
    assert!(!state.path().join("ops.jsonl").exists());
}

/// The line spelling: `/prompt` over a bare rung's prepared. It refuses at the
/// reader — the tail is empty and the prefill it falls to is empty too — so a
/// blank goal never even becomes a gesture. The door's refusal is what covers
/// every seat the reader does not stand in front of; both are the one
/// invariant, and this beat is that the line has no way past it.
#[test]
fn the_line_spelling_never_produces_a_blank_gesture() {
    let ctx = Context {
        prepared: Some(prepared()),
        ..Context::default()
    };
    for draft in ["/prompt", "/prompt   ", "/prompt \t"] {
        let refusal = line::parse(draft, &ctx).unwrap_err();
        assert!(refusal.contains("/prompt"), "{draft:?}: {refusal}");
    }
}
