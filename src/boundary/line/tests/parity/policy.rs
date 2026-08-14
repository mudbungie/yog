//! The **policy** families' half of the parity table (§8.5): the gestures whose
//! subject is what an agent may do rather than what it is doing — the §4.11
//! hold answer and its §4.9 floor, the §4.9 monitor, and the §4.3 armed loop.
//!
//! Split from [`super`] at §12's cap, on the seam those four share: each is
//! aimed by the seat, each carries only what no seat could supply, and each
//! reads back as itself from the seat it was spelled at.

use super::super::ctx;
use super::rt;
use crate::boundary::line::{Context, parse};
use crate::boundary::{Action, Gesture, Query};
use crate::monitor::Verb;

/// The §8.6 capability answer: aimed by the seat like `/seen`, and its whole
/// tail is the verdict — the held `tool_use` id is derived at fire time, so
/// there is nothing else a line could carry.
#[test]
fn every_hold_answer_round_trips_and_a_bad_verdict_refuses() {
    use crate::control::judge::Ruling;
    for ruling in [Ruling::Pass, Ruling::Hold, Ruling::Refuse] {
        rt(Gesture::Act(Action::AnswerHold {
            workspace: "ws".to_owned(),
            agent: "c-1".to_owned(),
            ruling,
        }));
    }
    let bad = parse("/answer maybe", &ctx()).expect_err("only the three words");
    assert!(bad.contains("unknown verdict"), "{bad}");
    let bare = parse("/answer", &ctx()).expect_err("no default verdict exists");
    assert!(bare.contains("pass, hold or refuse"), "{bare}");
}

/// The §4.9 fifth rung over that same fold: two verbs, aimed by the seat, each
/// carrying nothing at all — the direction *is* the verb.
#[test]
fn the_capability_floor_round_trips_both_ways_and_takes_no_words() {
    for raised in [true, false] {
        rt(Gesture::Act(Action::Floor {
            workspace: "ws".to_owned(),
            agent: "c-1".to_owned(),
            raised,
        }));
    }
    let extra = parse("/revoke everything", &ctx()).expect_err("it takes no arguments");
    assert!(extra.contains("takes no arguments"), "{extra}");
    let unaimed = parse("/restore", &Context::default()).expect_err("nothing is selected");
    assert!(unaimed.contains("no workspace in context"), "{unaimed}");
    let unselected = parse(
        "/restore",
        &Context {
            workspace: Some("ws".to_owned()),
            ..Context::default()
        },
    )
    .expect_err("a floor is written for a conversation, and none is selected");
    assert!(
        unselected.contains("no conversation is selected"),
        "{unselected}"
    );
}

/// The alignment monitor's three (VISION §4.9): arming carries its model pin,
/// disarming names nothing, and a flag carries its reason verbatim.
#[test]
fn every_monitor_action_round_trips() {
    rt(Gesture::Act(Action::Monitor(Verb::Arm {
        workspace: "ws".to_owned(),
        model: "claude-haiku-4-5".to_owned(),
    })));
    rt(Gesture::Act(Action::Monitor(Verb::Disarm {
        workspace: "ws".to_owned(),
    })));
    rt(Gesture::Act(Action::Monitor(Verb::Flag {
        workspace: "ws".to_owned(),
        agent: "c-1".to_owned(),
        reason: "it is rewriting an unrelated crate".to_owned(),
    })));
    rt(Gesture::Ask(Query::Workspaces));
}

/// The armed loop's two (VISION §4.3): arming carries its cap, disbanding names
/// nothing at all. The project and the workspace are the seat's, so the line
/// elides them and the context supplies them back.
#[test]
fn every_fleet_action_round_trips() {
    rt(Gesture::Act(Action::Fleet(crate::fleet::Verb::Arm {
        workspace: "ws".to_owned(),
        project: "proj".to_owned(),
        cap: 4,
    })));
    rt(Gesture::Act(Action::Fleet(crate::fleet::Verb::Disarm {
        workspace: "ws".to_owned(),
    })));
    rt(Gesture::Ask(Query::Conversations {
        workspace: "ws".to_owned(),
    }));
    rt(Gesture::Ask(Query::Balls));
    rt(Gesture::Ask(Query::Board));
    rt(Gesture::Ask(Query::Attention));
    rt(Gesture::Ask(Query::Ops { max: 12 }));
    // Both moods of the needle: text, and the empty one that clears.
    for text in ["", "tekeli-li"] {
        rt(Gesture::Ask(Query::Search {
            text: text.to_owned(),
        }));
    }
    // Both moods of the work-diff: the listing, and one file's patch.
    for file in [
        None,
        Some(crate::workdiff::WorkFile {
            ball: "bl-1".to_owned(),
            path: "src/a.rs".to_owned(),
        }),
    ] {
        rt(Gesture::Ask(Query::WorkDiff {
            workspace: "ws".to_owned(),
            file,
        }));
    }
    rt(Gesture::Ask(Query::Help { verb: None }));
    rt(Gesture::Ask(Query::Help {
        verb: Some("prepare".to_owned()),
    }));
}
