//! Turn rollup (§11): a finished turn's machinery is ONE aggregate line before
//! the answer, opening onto step rows that each still fold — and a run that is
//! not a finished turn is left exactly as it was. The other direction, a turn
//! still running, is [`live`]; the fixtures both read are here.

use super::render::painted_with;
use super::rows::{SPEAKER, default_rows, entry, tx};
use crate::transcript::{
    AutoExpand, Block, EntryKind, Fold, Row, RowClass, Tone, Transcript, rows,
};
use std::collections::HashSet;

mod live;
mod tokens;

/// The aggregate line the [`finished_turn`] fixture earns.
const AGGREGATE: &str = "⚙ 2 inference calls · 2 tool calls · 1 thinking block";
/// That turn's aggregate key: its first entry, with the turn suffix.
const TURN_KEY: &str = "tx/002-opus.json#turn";

pub(super) fn model(name: &str, blocks: Vec<Block>) -> crate::transcript::Entry {
    model_with_usage(name, blocks, &[])
}

/// A model entry whose bytes committed `counters` as its `usage` record —
/// empty is the legacy shape (no record at all).
pub(super) fn model_with_usage(
    name: &str,
    blocks: Vec<Block>,
    counters: &[(&str, u64)],
) -> crate::transcript::Entry {
    entry(
        name,
        EntryKind::Model {
            model_id: "opus".into(),
            blocks,
            usage: counters.iter().map(|(k, n)| (k.to_string(), *n)).collect(),
        },
    )
}

pub(super) fn call(id: &str, name: &str) -> Block {
    Block::ToolUse {
        id: id.into(),
        name: name.into(),
        input_summary: "{}".into(),
    }
}

pub(super) fn result(name: &str, id: &str) -> crate::transcript::Entry {
    entry(
        name,
        EntryKind::ToolResult {
            tool_use_id: id.into(),
            content: "bytes\nmore bytes".into(),
            is_error: false,
        },
    )
}

pub(super) fn delivered(name: &str, body: &str) -> crate::transcript::Entry {
    entry(
        name,
        EntryKind::Delivered {
            sender: "user".into(),
            epitaph: None,
            body: body.into(),
        },
    )
}

/// A turn the agent finished: two inferences, two tool calls, one thinking
/// block and an intermediate remark, then the answer it left the operator.
fn finished_turn() -> Transcript {
    tx(vec![
        delivered("001-user.md", "do the thing"),
        model(
            "002-opus.json",
            vec![
                Block::Thinking("weighing it\ndeeper still".into()),
                call("t1", "Read"),
            ],
        ),
        result("003-tool.json", "t1"),
        model(
            "004-opus.json",
            vec![Block::Text("on it".into()), call("t2", "Bash")],
        ),
        result("005-tool.json", "t2"),
        model("006-opus.json", vec![Block::Text("done".into())]),
    ])
}

pub(super) fn prefixes(got: &[Row]) -> Vec<String> {
    got.iter().map(|r| r.prefix.clone()).collect()
}

#[test]
fn a_finished_turn_is_one_aggregate_line_before_the_answer() {
    let got = default_rows(&finished_turn());
    assert_eq!(
        prefixes(&got),
        vec!["user:", AGGREGATE, "shudder-storeroom:"],
        "the machinery is one line: {got:?}"
    );
    assert_eq!(got[2].preview, "done", "the answer stands alone: {got:?}");
    let turn = &got[1];
    assert_eq!(turn.key, TURN_KEY, "keyed on the turn's first entry");
    assert_eq!(turn.class, RowClass::Other);
    assert_eq!(turn.tone, Tone::Weak);
    assert_eq!(turn.fold, Fold::Steps);
    assert_eq!(turn.role, None, "rolled-up machinery: nobody is speaking");
    assert!(!turn.expanded, "machinery arrives folded: {got:?}");
}

#[test]
fn expanding_the_turn_reveals_each_step_and_every_step_still_folds() {
    let t = finished_turn();
    let open = HashSet::from([TURN_KEY.to_string()]);
    let got = rows(&t, SPEAKER, AutoExpand::default(), &open);
    assert_eq!(
        prefixes(&got),
        vec![
            "user:",
            AGGREGATE,
            "thinking:",
            "⚙ Read",
            "✔ tool result — ok · 16 chars",
            "shudder-storeroom:",
            "⚙ Bash",
            "✔ tool result — ok · 16 chars",
            "shudder-storeroom:",
        ],
        "every step is its own row again: {got:?}"
    );
    assert!(got[1].expanded);
    // Folds all the way down: a revealed step keeps its own auto-state, and
    // its own override flips just it.
    let thinking = got[2].clone();
    assert_eq!(thinking.key, "tx/002-opus.json#0");
    assert!(!thinking.expanded, "a step keeps its class auto-state");
    let deeper = HashSet::from([TURN_KEY.to_string(), thinking.key.clone()]);
    let got = rows(&t, SPEAKER, AutoExpand::default(), &deeper);
    assert!(got[2].expanded, "the step's own fold opens it: {got:?}");
    assert!(got[1].expanded, "its parent is untouched");
}

#[test]
fn the_machinery_knob_opens_every_finished_turn_at_once() {
    // No new knob: the aggregate is machinery, so the `others` knob rules it —
    // ON expands the turn and every step inside it.
    let auto = AutoExpand {
        responses: true,
        others: true,
    };
    let got = rows(&finished_turn(), SPEAKER, auto, &HashSet::new());
    assert_eq!(got.len(), 9, "the steps are back: {got:?}");
    assert!(got.iter().all(|r| r.expanded), "all open: {got:?}");
}

#[test]
fn a_turn_the_model_never_answered_keeps_its_steps_on_screen() {
    // Nothing in flight, but the model has not spoken since the tool landed:
    // an unfinished turn has no answer to be the one line before.
    let t = tx(vec![
        delivered("001-user.md", "do the thing"),
        model("002-opus.json", vec![call("t1", "Read")]),
        result("003-tool.json", "t1"),
    ]);
    assert_eq!(
        prefixes(&default_rows(&t)),
        vec!["user:", "⚙ Read", "✔ tool result — ok · 16 chars"]
    );
}

#[test]
fn a_run_with_no_inference_call_is_not_a_turn() {
    // Stray unparseable bytes before an answer: nothing the model produced, so
    // there is no machinery run to aggregate and the rows stand as they are.
    let t = tx(vec![
        entry("junk", EntryKind::Raw),
        model("002-opus.json", vec![Block::Text("done".into())]),
    ]);
    assert_eq!(
        prefixes(&default_rows(&t)),
        vec!["junk", "shudder-storeroom:"]
    );
    // And an empty transcript projects nothing at all.
    assert!(default_rows(&tx(Vec::new())).is_empty());
}

#[test]
fn each_delivered_message_starts_a_turn_of_its_own() {
    let mut entries = finished_turn().entries;
    entries.push(delivered("007-user.md", "again"));
    entries.push(model(
        "008-opus.json",
        vec![Block::Thinking("once more".into())],
    ));
    entries.push(model(
        "009-opus.json",
        vec![Block::Text("done again".into())],
    ));
    let got = default_rows(&tx(entries));
    assert_eq!(
        prefixes(&got),
        vec![
            "user:",
            AGGREGATE,
            "shudder-storeroom:",
            "user:",
            "⚙ 1 inference call · 1 thinking block",
            "shudder-storeroom:",
        ],
        "one aggregate per turn, singular terms and no zero term: {got:?}"
    );
    assert_eq!(got[4].key, "tx/008-opus.json#turn", "its own key: {got:?}");
}

#[test]
fn a_finished_turn_paints_one_line_that_opens_onto_its_steps() {
    let t = finished_turn();
    let shut = painted_with(&t, false, AutoExpand::default(), &mut HashSet::new());
    assert!(shut.contains(AGGREGATE), "the aggregate paints:\n{shut}");
    assert!(shut.contains('▶'), "and it folds:\n{shut}");
    for hidden in ["thinking:", "weighing it", "⚙ Read", "on it"] {
        assert!(!shut.contains(hidden), "{hidden} is folded away:\n{shut}");
    }
    assert!(shut.contains("done"), "the answer is not folded:\n{shut}");

    let mut open = HashSet::from([TURN_KEY.to_string()]);
    let painted = painted_with(&t, false, AutoExpand::default(), &mut open);
    assert!(painted.contains('▼'), "the aggregate is open:\n{painted}");
    for step in ["thinking:", "⚙ Read", "on it", "✔ tool result — ok"] {
        assert!(painted.contains(step), "{step} is revealed:\n{painted}");
    }
    assert!(
        !painted.contains("deeper still"),
        "each step is still folded shut on its own:\n{painted}"
    );
}
