//! The decode side's **strictness** (bl-7067), the gesture codec's own
//! discipline applied to answers: an unknown token, a missing field and a
//! mistyped value each refuse with a reason **naming the offender**, so a seat
//! that cannot render an answer can say which word it choked on.
//!
//! Every case here is a refusal the decoder spells out loud. The refusals it
//! inherits from the field readers (`missing or non-string field "x"`) are the
//! gesture codec's, tested there once rather than once per field here.

use serde_json::{Value, json};

use super::super::super::decode;

/// Refuse `wire`, and say so naming `needle`.
#[track_caller]
fn refuses(wire: &Value, needle: &str) {
    let err = decode(wire).expect_err(&wire.to_string());
    assert!(err.contains(needle), "{wire} -> {err:?}");
}

/// A listing envelope with one row, for the row-level refusals.
fn rows(kind: &str, row: Value) -> Value {
    json!({ "ok": true, "kind": kind, "rows": [row] })
}

#[test]
fn the_envelope_itself_is_read_strictly() {
    refuses(&json!("a string"), "not a JSON object");
    refuses(&json!({ "ok": true, "kind": 7 }), "non-string field");
    refuses(&json!({ "ok": true, "kind": "sideways" }), "sideways");
    // A body with no kind must be a refusal, and a refusal states its error.
    refuses(&json!({ "ok": true }), "an answer with no kind");
    refuses(&json!({ "ok": false }), "field \"error\"");
    refuses(&json!({}), "field \"ok\"");
}

#[test]
fn receipt_fields_are_read_strictly() {
    refuses(&json!({ "ok": true, "kind": "outcome" }), "\"exit\"");
    refuses(
        &json!({ "ok": true, "kind": "outcome", "exit": 1e300, "stdout": "", "stderr": "" }),
        "\"exit\"",
    );
    refuses(
        &json!({ "ok": true, "kind": "prepared" }),
        "missing prepared",
    );
    refuses(
        &json!({ "ok": true, "kind": "answered", "tool_use": "t", "tool": "Bash",
                 "verdict": "maybe", "advanced": false }),
        "unknown verdict \"maybe\"",
    );
}

#[test]
fn every_row_level_token_refuses_by_name() {
    refuses(
        &rows(
            "workspaces",
            json!({ "workspace": "/w", "kind": "sideways", "attention": 0,
                    "agents": 0, "running": false }),
        ),
        "unknown kind \"sideways\"",
    );
    refuses(
        &rows(
            "attention",
            json!({ "workspace": "/w", "agent": "c", "display": "c", "state": "live",
                    "uncertain": false, "signals": ["sideways"], "preview": "",
                    "age_secs": 0, "pending": 0 }),
        ),
        "unknown signal \"sideways\"",
    );
    refuses(&rows("help", json!({ "verb": "sideways" })), "unknown verb");
    refuses(&rows("help", json!(7)), "not an object");
}

#[test]
fn the_conversation_rows_alignment_verdict_is_a_known_word() {
    refuses(
        &rows(
            "conversations",
            json!({ "root_id": "c", "display": "c", "display_only": false,
                    "state": "live", "uncertain": false, "preview": "",
                    "age_secs": 0, "attention": 0, "members": 1, "depth": 0,
                    "direct": 0, "tone": "plain", "stoppable": false,
                    "stop_children": false,
                    "alignment": { "workspace": "/w", "agent": "c",
                                   "verdict": "sideways", "sha": "a",
                                   "reason": "r", "model": "m" } }),
        ),
        "unknown verdict \"sideways\"",
    );
    refuses(
        &rows(
            "conversations",
            json!({ "root_id": "c", "display": "c", "display_only": false,
                    "state": "sideways", "uncertain": false, "preview": "",
                    "age_secs": 0, "attention": 0, "members": 1, "depth": 0,
                    "direct": 0, "tone": "plain" }),
        ),
        "unknown token \"sideways\"",
    );
}

/// **A follow frame with no fold at all is a codec that has drifted, not an
/// empty tail** (bl-73e7). An empty tail is an empty object; a missing `stream`
/// key is a frame that says nothing about the thing it exists to say, and a
/// `delta` naming no arm of the fold is the same class of drift.
#[test]
fn the_follow_frame_is_read_strictly() {
    let follow = |stream: Value| json!({ "ok": true, "kind": "follow", "stream": stream });
    refuses(&json!({ "ok": true, "kind": "follow" }), "missing stream");
    refuses(&follow(json!("said")), "not an object");
    refuses(
        &follow(json!({ "delta": "sideways" })),
        "unknown delta kind",
    );
    refuses(&follow(json!({ "text": 7 })), "\"text\"");
}

#[test]
fn the_board_figure_is_read_strictly() {
    let row = |spend: Value| {
        rows(
            "board",
            json!({ "project": "/p", "id": "bl-1", "title": "t", "priority": 0,
                    "column": "ready", "state": "ready", "gates": [], "drones": [],
                    "spend": spend }),
        )
    };
    refuses(&row(json!({})), "missing tokens");
    refuses(
        &row(json!({ "tokens": { "input": 0, "output": 0, "cache_read": 0, "cache_write": 0 } })),
        "missing attribution",
    );
    refuses(
        &row(
            json!({ "tokens": { "input": 0, "output": 0, "cache_read": 0, "cache_write": 0 },
                     "attribution": { "kind": "sideways" } }),
        ),
        "unknown kind \"sideways\"",
    );
}

#[test]
fn the_search_hit_addresses_and_fields_are_known_words() {
    let hit = |at: &str, field: &str| {
        json!({ "ok": true, "kind": "search", "needle": "n", "unreadable": [],
                "rows": [{ "at": at, "field": field, "offset": 0, "excerpt": "e",
                           "project": "/p", "id": "bl-1" }] })
    };
    refuses(&hit("sideways", "name"), "unknown address \"sideways\"");
    refuses(&hit("ball", "sideways"), "unknown field \"sideways\"");
}

/// The §6 mark table is the one vocabulary the conversation seat owns, so an
/// unknown mark refuses by name here rather than degrading to an unmarked
/// conversation — a reply that dropped a mark would tell a seat nothing is
/// waiting on the operator.
#[test]
fn an_unknown_agent_mark_refuses_by_name() {
    let seat = |marks: Value| {
        json!({ "ok": true, "kind": "agent", "agent": "c", "root": "c",
                "display": "c", "display_only": false, "tip": "",
                "state": "live", "marks": marks,
                "stoppable": false, "stop_children": false })
    };
    refuses(&seat(json!(["sideways"])), "unknown mark \"sideways\"");
    refuses(&seat(json!([7])), "non-string");
    refuses(&seat(json!("notified")), "non-array");
}

#[test]
fn the_inspector_families_own_tokens_refuse_by_name() {
    refuses(
        &rows(
            "transcript",
            json!({ "name": "n", "raw": "", "kind": "sideways" }),
        ),
        "unknown kind \"sideways\"",
    );
    refuses(
        &rows(
            "transcript",
            json!({ "name": "n", "raw": "", "kind": "model", "model_id": "m",
                    "blocks": [{ "kind": "sideways" }], "usage": {} }),
        ),
        "block: unknown kind \"sideways\"",
    );
    refuses(
        &rows(
            "transcript",
            json!({ "name": "n", "raw": "", "kind": "model", "model_id": "m",
                    "blocks": [], "usage": { "input_tokens": "lots" } }),
        ),
        "usage \"input_tokens\"",
    );
    refuses(
        &rows(
            "transcript",
            json!({ "name": "n", "raw": "", "kind": "model", "model_id": "m",
                    "blocks": [], "usage": 7 }),
        ),
        "usage: not an object",
    );
    refuses(
        &rows(
            "transcript",
            json!({ "name": "n", "raw": "", "kind": "model",
                                    "model_id": "m", "blocks": [] }),
        ),
        "missing usage",
    );
    refuses(
        &rows(
            "transcript",
            json!({ "name": "n", "raw": "", "kind": "delivered", "sender": "u",
                    "body": "", "epitaph": 7 }),
        ),
        "epitaph: not a string",
    );
}

#[test]
fn the_step_records_and_previews_refuse_by_name() {
    let detail = |meta: Value| {
        json!({ "ok": true, "kind": "step", "seq": "001", "meta": meta,
                "request": { "kind": "absent" }, "staging": { "kind": "absent" },
                "response": [], "tools": [] })
    };
    refuses(&detail(json!({ "kind": "sideways" })), "doc: unknown kind");
    refuses(&detail(json!({ "kind": "json" })), "missing value");
    refuses(
        &json!({ "ok": true, "kind": "step", "seq": "001", "response": [], "tools": [] }),
        "missing meta",
    );
    refuses(
        &json!({ "ok": true, "kind": "step", "seq": "001", "meta": { "kind": "absent" },
                 "request": { "kind": "absent" }, "staging": { "kind": "absent" },
                 "response": [], "tools": [{ "tool_id": "t", "is_error": false }] }),
        "tool: missing input",
    );
    refuses(
        &json!({ "ok": true, "kind": "files", "worktree": false,
                 "preview": { "kind": "sideways" } }),
        "preview: unknown kind",
    );
}

#[test]
fn a_work_diff_attempt_states_its_state_in_a_known_word() {
    refuses(
        &rows(
            "work-diff",
            json!({ "project": "/p", "ball_id": "bl-1", "state": "sideways" }),
        ),
        "unknown state \"sideways\"",
    );
}

#[test]
fn an_inbox_row_must_carry_its_deposit() {
    refuses(
        &rows("inbox", json!({ "name": "n", "raw": "" })),
        "missing deposit",
    );
}
