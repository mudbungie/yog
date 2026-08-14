//! The §8.5 round trip (REMOTE §9 step 2, bl-7067): **encode → decode is the
//! identity over the whole reply surface**, plus the refusal that rides beside
//! it and the envelope rules that tell the two apart.
//!
//! This is the deliverable's own test. A reply that survives the trip is a
//! reply a seat holding no world can render exactly as the window renders it;
//! one that does not is an answer the wire quietly narrowed, which is the
//! failure mode §3's "one dispatch surface, N serializations" exists to
//! prevent.

mod strict;
mod surface;

use super::super::{decode, encode, refusal};

#[test]
fn every_reply_variant_survives_the_round_trip() {
    for reply in surface::surface() {
        let wire = encode(&reply);
        assert_eq!(
            decode(&wire),
            Ok(Ok(reply.clone())),
            "{} did not survive: {wire}",
            kind_of(&wire),
        );
    }
}

/// A refusal is the envelope with **no `kind`** — `ok` cannot be the
/// discriminant, because a captured run spells its own exit verdict there.
#[test]
fn a_refusal_round_trips_as_the_err_side() {
    assert_eq!(decode(&refusal("unknown op")), Ok(Err("unknown op".into())));
}

/// The one envelope that reads `ok: false` and is not a refusal: a `bl close`
/// that failed its gate. It must come back as the outcome it is.
#[test]
fn a_failed_outcome_is_not_read_as_a_refusal() {
    let failed = crate::actions::verbs::Outcome {
        exit: 1,
        stdout: String::new(),
        stderr: "gate".into(),
    };
    let wire = encode(&super::super::Reply::Outcome(failed.clone()));
    assert_eq!(
        wire["ok"], false,
        "the run's own verdict, not the envelope's"
    );
    assert_eq!(
        decode(&wire),
        Ok(Ok(super::super::Reply::Outcome(failed))),
        "a non-zero exit is an answer, not a refused gesture"
    );
}

/// Every listing's rows are read strictly enough that a bad row fails the
/// listing rather than shortening it — the promise `list_of` makes.
#[test]
fn a_malformed_row_fails_its_listing_rather_than_vanishing() {
    let wire = serde_json::json!({ "ok": true, "kind": "ops", "rows": [1] });
    assert!(
        decode(&wire).is_err(),
        "a non-object row is not a shorter list"
    );
}

fn kind_of(wire: &serde_json::Value) -> String {
    wire.get("kind")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("refusal")
        .to_owned()
}

/// The token tables are written twice — the `match` that spells a value and
/// the table that parses one — for the reason `join_token`/`parse_join`
/// already are: a match is the compile gate and a table is the parser. This
/// is what holds the two halves together, over **every arm** of the three
/// vocabularies the conversation row carries.
#[test]
fn every_conversation_row_token_survives_both_halves_of_its_table() {
    use crate::git_tree::AgentState;
    use crate::nav::convs::{ConvRow, Flight};
    use crate::transcript::Tone;
    let base = ConvRow {
        root_id: "c".into(),
        state: AgentState::Live,
        uncertain: false,
        preview: String::new(),
        age_secs: 0,
        flight: None,
        attention: 0,
        members: 1,
        depth: 0,
        direct: 0,
        stoppable: false,
        stop_children: false,
        ball: None,
        name: None,
        name_display_only: false,
        verdict: None,
        tone: Tone::Plain,
    };
    let mut rows = Vec::new();
    for tone in [
        Tone::Plain,
        Tone::Weak,
        Tone::Good,
        Tone::Bad,
        Tone::Live,
        Tone::InFlight,
    ] {
        rows.push(ConvRow {
            tone,
            ..base.clone()
        });
    }
    for state in [
        AgentState::Live,
        AgentState::InFlight,
        AgentState::Quiescent,
        AgentState::Stopped,
    ] {
        rows.push(ConvRow {
            state,
            ..base.clone()
        });
    }
    for flight in [Flight::Inference, Flight::Tools, Flight::Subagents] {
        rows.push(ConvRow {
            flight: Some(flight),
            ..base.clone()
        });
    }
    let reply = super::super::Reply::Conversations(rows);
    assert_eq!(decode(&encode(&reply)), Ok(Ok(reply)));
}
