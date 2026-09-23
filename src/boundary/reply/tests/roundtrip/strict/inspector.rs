//! The §11 inspector families' own strictness: every token a transcript entry,
//! a step record, a preview, a work-diff attempt or an inbox row is read under
//! refuses by name. Its own file at §12's cap, on the seam the decoder is cut
//! along — `decode`'s receipt/listing/inspector chain, which `surface` splits
//! the fixture by too.

use serde_json::{Value, json};

use super::super::super::super::decode;
use super::{refuses, rows};

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

/// The §9.4 workflow mark beside the governing commit (bl-b680): absent and
/// null both read as the general path, and a present body is read by name.
#[test]
fn the_workflow_mark_reads_absent_as_null_and_a_body_strictly() {
    let governing = |mark: Value| {
        let mut o = json!({ "ok": true, "kind": "governing", "oid": "b", "short_oid": "b",
                            "follows": "default", "diverged_lineages": 0, "files": [] });
        if !mark.is_null() {
            o["workflow_mark"] = mark;
        }
        o
    };
    assert!(matches!(
        decode(&governing(Value::Null)),
        Ok(Ok(crate::boundary::reply::Reply::Governing {
            workflow_mark: None,
            ..
        }))
    ));
    refuses(&governing(json!(7)), "workflow_mark: not an object");
    refuses(&governing(json!({ "holder": "r" })), "field \"oid\"");
    refuses(
        &governing(json!({ "holder": "r", "oid": "d", "short_oid": "d", "lineage": 7 })),
        "lineage",
    );
}
