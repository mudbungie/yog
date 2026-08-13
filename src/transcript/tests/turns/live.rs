//! The turn in flight (§11): while the agent is working, its steps stay on
//! screen and the step happening RIGHT NOW is open over its class knob — the
//! other direction from [`super`], where a finished turn is one line.

use super::{call, delivered, model, prefixes};
use crate::transcript::tests::rows::{SPEAKER, default_rows, entry, tx};
use crate::transcript::{AutoExpand, Block, EntryKind, Transcript, rows};
use std::collections::HashSet;

/// The turn in flight: the agent has thought, is running a tool nothing has
/// retired, and is streaming. Nothing rolls up, and what is happening NOW is
/// open even though its class knob says contract.
fn live_turn() -> Transcript {
    tx(vec![
        delivered("001-user.md", "do the thing"),
        model(
            "002-opus.json",
            vec![
                Block::Thinking("weighing it\ndeeper still".into()),
                call("t1", "Read"),
            ],
        ),
        entry(
            "«live»",
            EntryKind::Streaming {
                thinking: String::new(),
                text: "typing\nmore".into(),
            },
        ),
    ])
}

#[test]
fn an_in_flight_turn_streams_its_steps_visibly_and_never_rolls_up() {
    let got = default_rows(&live_turn());
    assert_eq!(
        prefixes(&got),
        vec!["user:", "thinking:", "⚙ Read — running", "live:"],
        "the live turn is the show: {got:?}"
    );
    assert!(got[2].expanded, "the running tool is open: {got:?}");
    assert!(got[3].expanded, "the streaming tail is open: {got:?}");
    assert!(
        !got[1].expanded,
        "a finished step is back to its class state"
    );
}

#[test]
fn in_flight_outranks_the_class_knob_and_an_override_outranks_both() {
    let t = live_turn();
    let shut = AutoExpand {
        responses: false,
        others: false,
    };
    let got = rows(&t, SPEAKER, shut, &HashSet::new());
    assert!(
        got[2].expanded && got[3].expanded,
        "in flight wins: {got:?}"
    );
    // The operator's own flip still wins over the live auto-state (XOR).
    let folds = HashSet::from([got[2].key.clone(), got[3].key.clone()]);
    let got = rows(&t, SPEAKER, shut, &folds);
    assert!(!got[2].expanded && !got[3].expanded, "override: {got:?}");
}

/// §7.2 **the thinking ruling**: reasoning is display text, so it streams as
/// its own row beside the answer — the same two rows the committed turn above
/// already has, so nothing on screen changes shape when the step commits. Both
/// are open, because both are happening now.
#[test]
fn the_live_tail_streams_reasoning_and_answer_as_two_rows() {
    let t = tx(vec![entry(
        "«live»",
        EntryKind::Streaming {
            thinking: "weighing it".into(),
            text: "here goes".into(),
        },
    )]);
    let got = default_rows(&t);
    assert_eq!(
        prefixes(&got),
        vec!["thinking:", "live:"],
        "reasoning first, then the answer: {got:?}"
    );
    assert_eq!(got[0].preview, "weighing it");
    assert_eq!(got[1].preview, "here goes");
    assert!(
        got[0].expanded && got[1].expanded,
        "in flight, so both are open whatever the class knobs say: {got:?}"
    );
    assert_ne!(
        got[0].key, got[1].key,
        "distinct keys, so folding one does not fold the other"
    );
}
