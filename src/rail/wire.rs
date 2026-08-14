//! The step spine's JSON shape (§8.5, bl-6233) — the headless serialization of
//! VISION V1's rail, beside its type for the reason `workdiff::wire` gives:
//! notches, seats and cards are this module's own vocabulary.

use serde_json::{Map, Value, json};

use super::{ChildCard, Notch, Rail};

/// The `rail` reply body: the notches, then the cards hanging off them. Two
/// lists rather than a nesting, because a card names its notch by index and a
/// notch with no card is still a place a gesture can reach.
pub(crate) fn reply(rail: &Rail) -> Value {
    json!({
        "ok": true, "kind": "rail",
        "rows": Value::Array(rail.notches.iter().map(notch_row).collect()),
        "cards": Value::Array(rail.cards.iter().map(card_row).collect()),
    })
}

/// One notch: its step, the read-state commit it pins to, its spend, and its
/// seat in the chat. `commit` and `row`/`cut` are absent — not empty — when the
/// step recorded no `meta.json` or the chat gave the call no seat: both are
/// exactly what makes a notch unpinnable, and a reader must not have to tell
/// that from a notch pinned at the empty string.
fn notch_row(notch: &Notch) -> Value {
    let mut map = Map::new();
    map.insert("seq".to_owned(), json!(notch.seq));
    map.insert("tokens".to_owned(), json!(notch.tokens));
    if let Some(commit) = &notch.commit {
        map.insert("commit".to_owned(), json!(commit));
        map.insert("short".to_owned(), json!(notch.short()));
    }
    if let Some(place) = &notch.place {
        map.insert("row".to_owned(), json!(place.row));
        map.insert("cut".to_owned(), json!(place.cut));
    }
    Value::Object(map)
}

/// One child card (VISION V1.4): who the child is, where it forked from, what
/// it is doing, what it spent, the last of its inference text, and the notch it
/// was born at.
fn card_row(card: &ChildCard) -> Value {
    let mut map = Map::new();
    map.insert("agent".to_owned(), json!(card.agent_id));
    map.insert("name".to_owned(), json!(card.name));
    map.insert("fork".to_owned(), json!(card.fork));
    // The §5.1 state token, in the roster's own words — the conversation rows
    // already spell it, and two tables for one vocabulary would drift.
    map.insert(
        "state".to_owned(),
        json!(crate::boundary::reply::rows::state_token(card.state)),
    );
    map.insert("tokens".to_owned(), json!(card.tokens));
    map.insert("notch".to_owned(), json!(card.provenance_notch));
    // Absent when the child has produced no inference text at all — which is a
    // different statement from having said the empty string.
    if let Some(tail) = &card.tail {
        map.insert("tail".to_owned(), json!(tail));
    }
    Value::Object(map)
}
