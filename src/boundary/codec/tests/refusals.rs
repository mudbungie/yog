//! The codec's **refusal** table (§8.5 deliverable 3, the strict edge): every
//! malformed envelope names its offence, because a gesture is an instruction
//! and a guessed default is worse than a stop. Split from the round-trip
//! tables beside it at §12's cap, along the seam the module doc already draws:
//! what re-enters as itself is one thing, what must not enter at all is
//! another.

use super::super::decode;

/// Every refusal names its offence — the depositor's only diagnostic.
#[test]
fn malformed_envelopes_refuse_with_a_reason() {
    use serde_json::json;
    let cases: Vec<(serde_json::Value, &str)> = vec![
        (json!("not an object"), "not a JSON object"),
        (json!({}), "missing or non-string field \"op\""),
        (json!({"op": "warp"}), "unknown op \"warp\""),
        (
            json!({"op": "message", "workspace": "/ws"}),
            "field \"agent\"",
        ),
        (
            json!({"op": "message", "workspace": 7, "agent": "a", "content": "c"}),
            "field \"workspace\"",
        ),
        (
            json!({"op": "nudge", "workspace": "/ws"}),
            "field \"agent\"",
        ),
        (json!({"op": "ops"}), "non-integer field \"max\""),
        (json!({"op": "seen", "workspace": "/ws"}), "field \"agent\""),
        (
            json!({"op": "ops", "max": "many"}),
            "non-integer field \"max\"",
        ),
        (
            json!({"op": "prepare", "workspace": "/ws"}),
            "missing payload",
        ),
        (
            json!({"op": "prepare", "workspace": "/ws", "payload": 3}),
            "payload: not an object",
        ),
        (
            json!({"op": "prepare", "workspace": "/ws", "payload": {"rung": "warp"}}),
            "unknown rung",
        ),
        (
            json!({"op": "prepare", "workspace": "/ws",
                   "payload": {"rung": "ball", "project": "/p"}}),
            "missing ball",
        ),
        (
            json!({"op": "prepare", "workspace": "/ws",
                   "payload": {"rung": "ball", "project": "/p", "ball": 4}}),
            "ball: not an object",
        ),
        (
            json!({"op": "prepare", "workspace": "/ws",
                   "payload": {"rung": "ball", "project": "/p",
                               "ball": {"id": 9, "title": "t", "body": "b"}}}),
            "id not a string",
        ),
        (
            json!({"op": "prepare", "workspace": "/ws",
                   "payload": {"rung": "ball", "project": "/p",
                               "ball": {"id": "x", "title": "t", "body": "b"}}}),
            "field \"join\"",
        ),
        (
            json!({"op": "prepare", "workspace": "/ws",
                   "payload": {"rung": "ball", "project": "/p",
                               "ball": {"id": "x", "title": "t", "body": "b", "join": "warp"}}}),
            "unknown join state",
        ),
        (json!({"op": "prompt", "goal": "g"}), "missing prepared"),
        (
            json!({"op": "prompt", "prepared": [], "goal": "g"}),
            "prepared: not an object",
        ),
        (
            json!({"op": "prompt", "goal": "g",
                   "prepared": {"name": "n", "workspace": "/w", "binding": null,
                                "goal": "g", "origin": "warp"}}),
            "unknown origin",
        ),
        // The §3.3 binding is a path or `null`; a number is neither, and a
        // strict edge names the field rather than reading it as unbound. The
        // wording is `str_of`'s since bl-7067 folded every optional reader onto
        // one `opt` — the promise was always that the refusal NAMES the field.
        (
            json!({"op": "prompt", "goal": "g",
                   "prepared": {"name": "n", "workspace": "/w", "binding": 7,
                                "goal": "g", "origin": "balls"}}),
            "field \"binding\"",
        ),
        (
            json!({"op": "update", "project": "/p", "id": "x", "name": "n", "title": 5}),
            "field \"title\"",
        ),
        // §8.7: an existing ball's `tags` are a required input like every
        // other. A payload that omits them names no birth policy, and reading
        // the absence as "untagged" would silently birth the drone on the
        // default lineage — the guess a strict edge exists to refuse.
        (
            json!({"op": "prepare", "workspace": "ws",
                   "payload": {"rung": "ball", "project": "p",
                               "ball": {"id": "bl-1", "title": "t", "body": "b",
                                        "join": "ready"}}}),
            "field \"tags\"",
        ),
    ];
    for (envelope, needle) in cases {
        let err = decode(&envelope).expect_err(&envelope.to_string());
        assert!(err.contains(needle), "{envelope} -> {err:?}");
    }
}
