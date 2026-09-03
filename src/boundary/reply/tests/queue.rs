//! The decision queue reply's spelling (§8.5, VISION §5 V5.2, STORIES S14).
//!
//! Two claims, and the second is the point of the shape: every §6 signal has a
//! word, and the address fields are spelled with **the same keys the gestures
//! take** — so answering a row is copying two values out of it, never
//! translating.

use super::super::*;
use crate::attention::AttentionKind;
use crate::boundary::answer::queue::QueueRow;
use crate::git_tree::AgentState;

#[test]
fn a_queue_row_spells_every_signal_and_the_address_a_gesture_takes() {
    let row = QueueRow {
        workspace: "alba".into(),
        agent: "c-1".into(),
        display: "koi".into(),
        state: AgentState::Stopped,
        uncertain: true,
        signals: vec![
            AttentionKind::Notify,
            AttentionKind::Stopped,
            AttentionKind::Budget,
            AttentionKind::Conflicted,
            AttentionKind::Mail,
            AttentionKind::Held,
            // Rule 2's other word (bl-b43b) — in the table beside `stopped`
            // even though a real row carries one or the other, because this
            // test's whole job is that no token is transposed.
            AttentionKind::Refused,
            // …and rule 7's own word (bl-6f2f), for the same reason.
            AttentionKind::Flagged,
        ],
        preview: "which branch?".into(),
        age_secs: 42,
        pending: 3,
        held: Some(crate::control::hold::Held {
            tool_use_id: "toolu_42".into(),
            tool: "bash".into(),
            reason: "bash {\"command\":\"curl x\"} classified open-world".into(),
        }),
        failure: Some("Unauthorized".into()),
        // The signal-out verb's own words (bl-6f2f), beside the class token
        // above — a flag that cannot say why costs a second read to act on.
        flag: Some(crate::monitor::Flag {
            at: "1".into(),
            reason: "please look at this one".into(),
        }),
    };
    let encoded = encode(&Reply::Attention(vec![row]));
    assert_eq!(encoded["ok"], true);
    assert_eq!(encoded["kind"], "attention");
    let out = &encoded["rows"][0];
    // The address, in the gestures' own words.
    assert_eq!(out["workspace"], "alba");
    assert_eq!(out["agent"], "c-1");
    assert_eq!(out["display"], "koi");
    assert_eq!(out["state"], "stopped");
    assert_eq!(out["uncertain"], true);
    assert_eq!(out["preview"], "which branch?");
    assert_eq!(out["age_secs"], 42);
    assert_eq!(out["pending"], 3);
    assert_eq!(
        out["signals"],
        serde_json::json!([
            "notify",
            "stopped",
            "budget",
            "conflicted",
            "mail",
            "held",
            "refused",
            "flagged"
        ]),
        "the §6 signals in the `ui.json` watermark's own vocabulary"
    );
    // The firing rules in words (bl-09ef): the announcing is a seat's, so the
    // one home for the sentence (`AttentionKind::says`) crosses beside the
    // tokens — a seat that re-worded them from the tokens would be the second
    // wording §6 forbids.
    assert_eq!(
        out["says"],
        "raised a notify mark; came to rest — your turn; exhausted its budget; has a conflicted branch; has mail queued and no driver taking it; parked a tool invocation for your answer; was refused at the provider — sign a provider in on this workspace; was flagged for a look, with a reason"
    );
    // The park rides the row rather than a query of its own (§8.6): a reader
    // sees what is waiting, why, and has the address to answer it.
    assert_eq!(out["held"]["tool_use"], "toolu_42");
    assert_eq!(out["held"]["tool"], "bash");
    // …and so does the flag's own sentence (bl-6f2f), for the same reason.
    assert_eq!(out["flag"]["at"], "1");
    assert_eq!(out["flag"]["reason"], "please look at this one");
    assert!(
        out["held"]["reason"]
            .as_str()
            .is_some_and(|r| r.contains("open-world")),
        "the control's own sentence rides the row"
    );
}

/// A row nothing is holding says so outright — a reader never has to tell an
/// absent key from a false one.
#[test]
fn a_row_with_no_park_spells_it_null() {
    let row = QueueRow {
        workspace: "alba".into(),
        agent: "c-2".into(),
        display: "elk".into(),
        state: AgentState::Quiescent,
        uncertain: false,
        signals: vec![AttentionKind::Stopped],
        preview: String::new(),
        age_secs: 0,
        pending: 0,
        held: None,
        failure: None,
        flag: None,
    };
    let encoded = encode(&Reply::Attention(vec![row]));
    assert_eq!(encoded["rows"][0]["held"], serde_json::Value::Null);
    // One firing rule is one clause, with nothing to join it to.
    assert_eq!(encoded["rows"][0]["says"], "came to rest — your turn");
}

/// Nothing waiting is an ordinary answer, not an absence of one.
#[test]
fn an_empty_queue_is_an_answer() {
    let encoded = encode(&Reply::Attention(vec![]));
    assert_eq!(encoded["ok"], true);
    assert_eq!(encoded["rows"], serde_json::json!([]));
}
