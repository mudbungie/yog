//! The answer a harness parses, and the two vocabulary types under it.

use super::recipe::{Conv, Recipe, Step, Wsp};
use super::{Laid, roster};
use std::path::PathBuf;

fn laid() -> Laid {
    Laid {
        state: "busy".to_owned(),
        root: PathBuf::from("/w"),
        address: "127.0.0.1:9".to_owned(),
        anchors: PathBuf::from("/w/ca.pem"),
        chain: PathBuf::from("/w/client.pem"),
        key: PathBuf::from("/w/client.key"),
        origin: 7,
        hold: vec![PathBuf::from("/w/inbox")],
    }
}

/// The whole consumer contract is this object, so every key a harness reads is
/// asserted by name — a renamed key is a break in another repository.
#[test]
fn the_answer_names_every_path_a_harness_needs() {
    let value: serde_json::Value = serde_json::from_str(&laid().json()).expect("json");
    assert_eq!(value["state"], "busy");
    assert_eq!(value["root"], "/w");
    assert_eq!(value["address"], "127.0.0.1:9");
    assert_eq!(value["anchors"], "/w/ca.pem");
    assert_eq!(value["chain"], "/w/client.pem");
    assert_eq!(value["key"], "/w/client.key");
    assert_eq!(value["origin"], 7);
    assert_eq!(value["hold"][0], "/w/inbox");
}

/// The derived reads a caller of this type makes.
#[test]
fn an_answer_is_comparable_and_printable() {
    let (a, b) = (laid(), laid().clone());
    assert_eq!(a, b);
    assert!(format!("{a:?}").contains("busy"));
}

/// Every name in the roster resolves, and an unknown one does not — the two
/// directions of the one lookup a consumer's contract rests on.
#[test]
fn every_listed_name_resolves_and_nothing_else_does() {
    assert!(!roster::names().is_empty());
    for name in roster::names() {
        assert!(roster::resolve(&name).is_some(), "{name}");
    }
    assert!(roster::resolve("no-such-state").is_none());
}

/// Two states may not share a name, and every one must say what it is: a bare
/// `yog fixture` is the roster, and a nameless row in it is a state nobody can
/// choose on purpose.
#[test]
fn the_roster_is_unique_and_every_row_speaks() {
    let mut seen = std::collections::BTreeSet::new();
    for (name, recipe) in roster::ROSTER {
        assert!(seen.insert(*name), "duplicate state {name}");
        assert!(!recipe.summary.is_empty(), "{name} has no summary");
    }
}

/// The `empty` state is the general shape with no inputs — asserted, because
/// the module's whole claim is that a first run is not a special case.
#[test]
fn the_empty_state_is_the_shape_with_nothing_in_it() {
    let empty = roster::resolve("empty").expect("empty");
    assert!(empty.workspaces.is_empty());
    assert!(empty.cadence.is_none());
    assert!(empty.brazen.is_none());
}

/// The vocabulary's two constructors, run rather than const-evaluated.
#[test]
fn a_bare_conversation_carries_a_goal_and_nothing_else() {
    let conv = Conv::new("c-1", "goal\n");
    assert_eq!(conv.id, "c-1");
    assert_eq!(conv.goal, "goal\n");
    assert!(conv.marks.is_empty());
    assert!(conv.messages.is_empty());
    assert!(conv.summaries.is_empty());
    assert!(conv.deposits.is_empty());
    assert_eq!(conv.age_secs, 0);
    assert!(conv.driver_log.is_empty());
    assert!(conv.step == Step::Absent);
    let recipe = Recipe::empty("nothing");
    assert_eq!(recipe.summary, "nothing");
    assert!(recipe.workspaces.is_empty());
}

/// A workspace is a name and its conversations, and the roster's non-empty
/// states all stand in the §3.1 bootstrap name.
#[test]
fn every_laid_workspace_is_the_bootstrap_name() {
    let wsp = Wsp {
        name: roster::WORKSPACE,
        convs: &[],
    };
    assert_eq!(wsp.name, "home");
    for (_, recipe) in roster::ROSTER {
        for w in recipe.workspaces {
            assert_eq!(w.name, roster::WORKSPACE);
            assert!(!w.convs.is_empty());
        }
    }
}

/// Every `Step` arm is spent by some state — a recipe vocabulary with an arm
/// nothing uses is an arm nobody has ever seen render.
#[test]
fn every_step_shape_is_spent_by_some_state() {
    let mut seen = [false; 6];
    for (_, recipe) in roster::ROSTER {
        for wsp in recipe.workspaces {
            for conv in wsp.convs {
                let slot = match conv.step {
                    Step::Absent => 0,
                    Step::Settled => 1,
                    Step::Failed => 2,
                    Step::OutputLimit => 3,
                    Step::Streaming => 4,
                    Step::Wound(_) => 5,
                };
                seen[slot] = true;
            }
        }
    }
    assert_eq!(seen, [true; 6], "an unspent Step arm");
}
