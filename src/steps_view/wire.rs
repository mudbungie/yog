//! The Steps tab's JSON shape (§8.5, bl-6233) — the headless serialization of
//! both tiers: the cheap per-step summary list and one step's drill-in. Beside
//! its type for the reason `workdiff::wire` gives — framings, wounds and
//! jsonview docs are this module's own vocabulary — and cut along the same
//! two-tier seam the module itself is cut along.

use serde_json::{Map, Value, json};

use super::{Doc, Orphan, StepDetail, StepSummary, StepsView, Tail, ToolIo, Wound};
use crate::budgets::BudgetSpend;
use crate::git_tree::Framing;

/// The decoders, beside the encoders they undo (bl-7067, REMOTE §9 step 2).
pub(crate) mod decode;

/// The `steps` reply body: one row per step, in sequence order, and the
/// view-level orphaned-tail state (bl-ace6) — the wound's class-token shape,
/// at the top because it is not any one step's fact.
pub(crate) fn steps(view: &StepsView) -> Value {
    let mut map = Map::new();
    map.insert("ok".to_owned(), json!(true));
    map.insert("kind".to_owned(), json!("steps"));
    map.insert(
        "rows".to_owned(),
        Value::Array(view.steps.iter().map(step_row).collect()),
    );
    map.insert("orphan".to_owned(), json!(orphan_token(&view.orphan)));
    if let Orphan::Spoke(_, reason) = &view.orphan {
        map.insert("orphan_reason".to_owned(), json!(reason));
    }
    Value::Object(map)
}

/// Which [`Tail`] is orphaned, or none — a class token rather than the
/// `orphaned` boolean the pair carried until bl-abba, for exactly the reason
/// [`wound_token`] is one: with a second shape the (bool, Option<reason>)
/// pair stopped being a bijection, and the honest fix is a discriminant, not
/// a second boolean beside the first. The reason key still separates mute
/// from spoken, which is the one thing it ever did.
fn orphan_token(orphan: &Orphan) -> &'static str {
    match orphan {
        Orphan::None => "none",
        Orphan::Mute(tail) | Orphan::Spoke(tail, _) => match tail {
            Tail::Mail => "mail",
            Tail::ToolWindow => "tool_window",
        },
    }
}

/// One step's summary. The timestamps and the read-state commit are absent
/// keys when `meta.json` did not carry them — the same absence the list paints,
/// never a zero or an empty string standing in for a fact nobody recorded.
fn step_row(step: &StepSummary) -> Value {
    let mut map = Map::new();
    map.insert("seq".to_owned(), json!(step.seq));
    map.insert("framing".to_owned(), json!(framing_token(step.framing)));
    map.insert("attempts".to_owned(), json!(step.attempts));
    map.insert("tokens".to_owned(), spend_value(&step.tokens));
    for (key, value) in [
        ("commit", step.commit.as_ref()),
        ("started_at", step.started_at.as_ref()),
        ("ended_at", step.ended_at.as_ref()),
    ] {
        if let Some(value) = value {
            map.insert(key.to_owned(), json!(value));
        }
    }
    // The §7.3 wound, which since bl-015b carries the §8.3 login affordance
    // too: the `refused` class IS the affordance being offered, and
    // `auth_row` is the provider row it points at. The `auth_failed` boolean
    // that used to ride beside them was that same fact a third time, so it is
    // gone — a decoder reads the affordance back off the class.
    wound_fields(&step.wound, &mut map);
    Value::Object(map)
}

/// The §7.3 wound's whole wire spelling — its class token, the adapter's own
/// reason when the no-response class left words behind, and the provider row
/// when the refusal routed to one.
///
/// `pub(crate)` for the `spend_value` reason one screen down: the §8.5
/// transcript's settled-failure notice (`transcript::wire`, bl-015b) carries
/// this same vocabulary, and one spelling in one place is what keeps the two
/// shapes from drifting into two dialects of one enum.
pub(crate) fn wound_fields(wound: &Wound, map: &mut Map<String, Value>) {
    map.insert("wound".to_owned(), json!(wound_token(wound)));
    match wound {
        Wound::Spoke(reason) => {
            map.insert("wound_reason".to_owned(), json!(reason));
        }
        Wound::Refused(auth) => {
            if let Some(row) = auth.row() {
                map.insert("auth_row".to_owned(), json!(row));
            }
        }
        Wound::None | Wound::Mute | Wound::OutputLimit => {}
    }
}

/// The §7.3 wound's **class**, beside the optional reason (bl-fb87). A class
/// token rather than the `wounded` boolean the pair used to carry: with a
/// fourth arm the (bool, Option<reason>) pair stopped being a bijection, and
/// the honest fix is a discriminant, not a second boolean beside the first.
/// The two no-response arms share one token because the reason key is exactly
/// what tells them apart; `refused` carries its row in `auth_row` for the same
/// reason, which is the key the affordance already spelled it under.
fn wound_token(wound: &Wound) -> &'static str {
    match wound {
        Wound::None => "none",
        Wound::Mute | Wound::Spoke(_) => "no_response",
        Wound::OutputLimit => "output_limit",
        Wound::Refused(_) => "refused",
    }
}

/// The §4.4 terminal classification, in the three words the seat renders.
fn framing_token(framing: Framing) -> &'static str {
    match framing {
        Framing::Complete => "complete",
        Framing::Failed => "failed",
        Framing::Killed => "killed",
    }
}

/// The four ARCH §6 counters and their total — the total is derived, and it is
/// carried because every seat that reads a step reads it against a ceiling.
///
/// `pub(crate)` since bl-7067: the §3.5 board figure spells its token half in
/// exactly this shape, the way `files_view::wire::preview_value` already serves
/// the work diff's patch. One spelling of one thing, in one place.
pub(crate) fn spend_value(spend: &BudgetSpend) -> Value {
    json!({
        "input": spend.input_tokens, "output": spend.output_tokens,
        "cache_read": spend.cache_read_tokens, "cache_write": spend.cache_write_tokens,
        "total": spend.total_tokens(),
    })
}

/// The `step` reply body: one step's four record files, its `response.json`
/// events, every tool call's input and output, and each capture log that has
/// bytes (bl-83d6) — an absent key for a log with nothing in it, the same way
/// the `files` reply omits a preview nobody asked for. A key carrying an empty
/// text would be a second encoding of "there is nothing there".
pub(crate) fn detail(detail: &StepDetail) -> Value {
    let mut map = Map::new();
    for (key, value) in [
        ("ok", json!(true)),
        ("kind", json!("step")),
        ("seq", json!(detail.seq)),
        ("meta", doc_value(&detail.meta)),
        ("request", doc_value(&detail.request)),
        ("staging", doc_value(&detail.staging)),
        (
            "response",
            Value::Array(detail.response.iter().map(doc_value).collect()),
        ),
        (
            "tools",
            Value::Array(detail.tools.iter().map(tool_value).collect()),
        ),
    ] {
        map.insert(key.to_owned(), value);
    }
    for (key, log) in [("stderr", &detail.stderr), ("driver", &detail.driver)] {
        if let Some(preview) = log {
            map.insert(
                key.to_owned(),
                crate::files_view::wire::preview_value(preview),
            );
        }
    }
    Value::Object(map)
}

/// One record file as data: parsed, absent, or bytes that are not JSON. The
/// three stay distinct on the wire exactly as they do on screen — rendered
/// bare, malformed content is indistinguishable from a file whose content
/// happens to be that text, which is why [`super::UNPARSED`] exists at all.
fn doc_value(doc: &Doc) -> Value {
    match doc {
        // `raw` rides beside the tree because a `serde_json::Value` is not a
        // lossless record of its source (key order, spacing and number spelling
        // all go), so the tree alone could never answer "what does the file
        // say" (S7-T1).
        Doc::Json { value, raw } => json!({
            "kind": "json", "value": value,
            "raw": String::from_utf8_lossy(raw),
        }),
        Doc::Absent => json!({ "kind": "absent" }),
        Doc::Unparsed(raw) => json!({
            "kind": "unparsed", "note": super::UNPARSED,
            "raw": String::from_utf8_lossy(raw),
        }),
    }
}

/// One tool call's records (ARCH §3.3), with litany's own `is_error` reading.
fn tool_value(tool: &ToolIo) -> Value {
    json!({
        "tool_id": tool.tool_id, "is_error": tool.is_error,
        "input": doc_value(&tool.input), "output": doc_value(&tool.output),
    })
}
