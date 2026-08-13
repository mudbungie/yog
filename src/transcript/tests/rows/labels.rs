//! What a row is **called**: the always-visible prefix seat and the hover
//! behind it. A model turn's sender label is the conversation's §3.3 display
//! name, never the model id (bl-2335) — a model is config, an agent is a
//! speaker. Folding and the auto-state live in [`super`].

use super::{default_rows, entry, model, tx};
use crate::transcript::{AutoExpand, Block, EntryKind, Tone, rows};
use std::collections::HashSet;

#[test]
fn a_delivered_message_with_no_body_says_so() {
    let t = tx(vec![entry(
        "004-kid.md",
        EntryKind::Delivered {
            sender: "kid".into(),
            epitaph: None,
            body: String::new(),
        },
    )]);
    let got = default_rows(&t);
    assert_eq!(got.len(), 1, "nothing is dropped: {got:?}");
    assert_eq!(got[0].prefix, "kid:");
    assert_eq!(got[0].preview, "(no message body)");
    assert_eq!(got[0].tone, Tone::Weak);
}

#[test]
fn a_result_deposit_names_how_the_child_ended() {
    // bl-71e8, from the operator's `energize` transcript: a stopped child's
    // deposit asserts an epitaph and nothing else, so before the prefix seat
    // said the ending this row was a blank line from a hundred-character id
    // — the operator saw nothing, and the reply it provoked had no cause.
    let t = tx(vec![entry(
        "040-kid.md",
        EntryKind::Delivered {
            sender: "kid".into(),
            epitaph: Some(crate::inboxview::Epitaph::Stopped),
            body: String::new(),
        },
    )]);
    let got = default_rows(&t);
    assert_eq!(got[0].prefix, "kid ended: stopped");
    assert_eq!(got[0].preview, "(no message body)");
}

#[test]
fn a_result_deposit_that_spoke_keeps_its_body_under_the_ending() {
    let t = tx(vec![entry(
        "042-kid.md",
        EntryKind::Delivered {
            sender: "kid".into(),
            epitaph: Some(crate::inboxview::Epitaph::FinalResponse),
            body: "the findings".into(),
        },
    )]);
    let got = default_rows(&t);
    assert_eq!(got[0].prefix, "kid ended: final-response");
    assert_eq!(got[0].preview, "the findings");
    assert_eq!(got[0].tone, Tone::Plain);
}

#[test]
fn a_model_message_with_no_blocks_still_gets_a_row() {
    let t = tx(vec![model(Vec::new())]);
    let got = default_rows(&t);
    assert_eq!(got.len(), 1, "nothing is dropped: {got:?}");
    assert_eq!(got[0].prefix, "shudder-storeroom:");
    assert_eq!(got[0].preview, "(no content blocks)");
    assert!(got[0].hover.contains("opus"), "the model id still hovers");
}

#[test]
fn a_resolved_tool_call_stops_saying_running() {
    let call = model(vec![Block::ToolUse {
        id: "toolu_1".into(),
        name: "Read".into(),
        input_summary: "{}".into(),
    }]);
    let pending = default_rows(&tx(vec![call.clone()]));
    assert_eq!(pending[0].prefix, "⚙ Read — running");
    assert_eq!(pending[0].tone, Tone::InFlight);
    let resolved = default_rows(&tx(vec![
        call,
        entry(
            "003-tool.json",
            EntryKind::ToolResult {
                tool_use_id: "toolu_1".into(),
                content: String::new(),
                is_error: false,
            },
        ),
    ]));
    assert_eq!(resolved[0].prefix, "⚙ Read");
    assert_eq!(resolved[0].tone, Tone::Plain);
}

/// bl-2335, the reported bug: a model turn was labelled `gpt-5.4:`. The sender
/// is an agent, and the agent's name is the §3.3 display name; the model id is
/// a config fact and moves to the hover, where it stays discoverable.
#[test]
fn a_model_turn_is_labelled_with_the_agent_name_and_hovers_the_model_id() {
    let t = tx(vec![model(vec![
        Block::Text("Yes".into()),
        Block::Thinking("hmm".into()),
    ])]);
    let got = default_rows(&t);
    assert_eq!(got[0].prefix, "shudder-storeroom:");
    assert!(
        !got[0].prefix.contains("opus"),
        "the model id is not a speaker: {got:?}"
    );
    assert!(
        got[0].hover.contains("opus"),
        "the model id stays discoverable: {got:?}"
    );
    // Only the model's own turns carry it: a thinking block is machinery, and
    // machinery labels say what they are, not who said them.
    assert_eq!(got[1].prefix, "thinking:");
    assert!(got[1].hover.is_empty(), "no hover to invent: {got:?}");
}

/// bl-1f75, the operator's complaint: *"I'd like tool result collapses to show
/// me the number of characters in the output."* Contracted, `✔ tool result — ok`
/// said nothing about whether the `▶` opened onto four characters or forty
/// thousand — the very decision the collapsed row exists to inform. The count
/// is of the **fold**, in characters: a row with nothing to fold has nothing to
/// size, and a byte count would over-state any payload carrying non-ASCII.
#[test]
fn a_tool_result_that_folds_says_how_big_the_fold_is_in_characters() {
    let result = |content: &str| {
        default_rows(&tx(vec![entry(
            "003-tool.json",
            EntryKind::ToolResult {
                tool_use_id: "t".into(),
                content: content.into(),
                is_error: false,
            },
        )]))
    };
    // Six characters over eleven bytes: the seat counts what a human reads.
    let folds = result("αβγ\nδε");
    assert_eq!(folds[0].prefix, "✔ tool result — ok · 6 chars");
    assert!(!folds[0].body.is_empty(), "it folds: {folds:?}");
    // Nothing hidden, nothing to size — the same fact the missing toggle is.
    let whole = result("bytes");
    assert_eq!(whole[0].prefix, "✔ tool result — ok");
    assert!(whole[0].body.is_empty(), "nothing folds: {whole:?}");
}

/// The live tail is the sibling case and it stays **bare** (bl-1f75): it is
/// in-flight, so it is already expanded on screen, and how much has landed is
/// the in-flight strip's own `N chars streamed` line (§5.1 #28a).
#[test]
fn the_live_tail_states_no_size() {
    let t = tx(vec![entry(
        "004-live.json",
        EntryKind::Streaming {
            thinking: String::new(),
            text: "half an\nanswer".into(),
        },
    )]);
    let got = default_rows(&t);
    assert_eq!(got[0].prefix, "live:");
    assert!(got[0].expanded, "in flight, so already on screen: {got:?}");
}

/// The label is whatever the §3.3 ladder handed down — including its last rung,
/// the raw agent id, for a conversation yog never named. Nothing here re-derives
/// a name, which is what keeps the one function the only spelling.
#[test]
fn the_label_is_the_callers_name_verbatim() {
    let t = tx(vec![model(vec![Block::Text("hi".into())])]);
    let unnamed = rows(
        &t,
        "20260427T120000Z-aaaa",
        AutoExpand::default(),
        &HashSet::new(),
    );
    assert_eq!(unnamed[0].prefix, "20260427T120000Z-aaaa:");
}
