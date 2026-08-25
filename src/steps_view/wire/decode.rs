//! The Steps tab's **decoders** (§8.5, REMOTE §9 step 2, bl-7067) — one per
//! encoder in [`super`], across both tiers: the summary list and one step's
//! drill-in.
//!
//! The §8.3 login affordance is read back from a **pair** of keys rather than
//! a discriminant, because that is how it is written: `auth_failed` + an
//! optional `auth_row` is a bijection over its three-armed enum —
//! offered-with-a-row, offered-without, not offered — so nothing needed
//! widening for the round trip.
//!
//! The §7.3 wound was such a pair too until bl-fb87 gave it a fourth arm. A
//! `(bool, Option<reason>)` cannot address four, so it is a class token plus
//! the same optional reason now: the token picks the class and the reason
//! separates the no-response class's two arms, which is the one thing that
//! ever distinguished them.

use serde_json::Value;

use super::super::{Doc, Orphan, StepDetail, StepSummary, StepsView, Tail, ToolIo, Wound};
use crate::boundary::codec::fields::{
    bool_of, bytes_of, list_of, opt_str_of, opt_val, pick, str_of, u64_of, usize_of,
};
use crate::budgets::BudgetSpend;
use crate::git_tree::Framing;
use crate::login::auth::AuthFailure;

/// The §4.4 terminal classification, [`framing_token`](super::framing_token)'s
/// other half.
const FRAMINGS: [(&str, Framing); 3] = [
    ("complete", Framing::Complete),
    ("failed", Framing::Failed),
    ("killed", Framing::Killed),
];

/// The §7.3 wound's class, [`wound_token`](super::wound_token)'s other half.
/// A [`Wound`] of its own cannot stand in the table — [`pick`] answers `Copy`
/// values and [`Wound::Spoke`] carries a `String` — so the class is named here
/// and the reason key resolves it below.
#[derive(Clone, Copy)]
enum WoundKind {
    None,
    NoResponse,
    OutputLimit,
}

const WOUNDS: [(&str, WoundKind); 3] = [
    ("none", WoundKind::None),
    ("no_response", WoundKind::NoResponse),
    ("output_limit", WoundKind::OutputLimit),
];

/// Which [`Tail`] is orphaned, [`orphan_token`](super::orphan_token)'s other
/// half. `None` is the un-orphaned tail, which is why the table's value type
/// is an option rather than a fourth arm nothing on the wire spells.
const TAILS: [(&str, Option<Tail>); 3] = [
    ("none", None),
    ("mail", Some(Tail::Mail)),
    ("tool_window", Some(Tail::ToolWindow)),
];

/// The `steps` reply body read back: one summary per row, in sequence order,
/// and the view-level orphaned-tail class plus its optional reason (bl-ace6,
/// widened by bl-abba) — the wound's own token/reason bijection, one tier up.
pub(crate) fn steps(obj: &serde_json::Map<String, Value>) -> Result<StepsView, String> {
    let orphan = match (
        pick(obj, "orphan", &TAILS)?,
        opt_str_of(obj, "orphan_reason")?,
    ) {
        (None, _) => Orphan::None,
        (Some(tail), None) => Orphan::Mute(tail),
        (Some(tail), Some(reason)) => Orphan::Spoke(tail, reason),
    };
    Ok(StepsView {
        steps: list_of(obj, "rows", step_row)?,
        orphan,
    })
}

fn step_row(v: &Value) -> Result<StepSummary, String> {
    let o = v.as_object().ok_or("step row: not an object")?;
    let auth_row = opt_str_of(o, "auth_row")?;
    let auth_failed = match (bool_of(o, "auth_failed")?, auth_row) {
        (false, _) => AuthFailure::No,
        (true, None) => AuthFailure::Unrouted,
        (true, Some(row)) => AuthFailure::Row(row),
    };
    let wound = match (pick(o, "wound", &WOUNDS)?, opt_str_of(o, "wound_reason")?) {
        (WoundKind::None, _) => Wound::None,
        (WoundKind::NoResponse, None) => Wound::Mute,
        (WoundKind::NoResponse, Some(reason)) => Wound::Spoke(reason),
        (WoundKind::OutputLimit, _) => Wound::OutputLimit,
    };
    Ok(StepSummary {
        seq: str_of(o, "seq")?,
        framing: pick(o, "framing", &FRAMINGS)?,
        attempts: usize_of(o, "attempts")?,
        tokens: spend(o.get("tokens").ok_or("step row: missing tokens")?)?,
        commit: opt_str_of(o, "commit")?,
        started_at: opt_str_of(o, "started_at")?,
        ended_at: opt_str_of(o, "ended_at")?,
        auth_failed,
        wound,
    })
}

/// The four ARCH §6 counters. `total` is not read back — [`BudgetSpend::
/// total_tokens`](crate::budgets::BudgetSpend::total_tokens) is its one
/// authority and a second sum could only ever disagree with it.
pub(crate) fn spend(v: &Value) -> Result<BudgetSpend, String> {
    let o = v.as_object().ok_or("spend: not an object")?;
    Ok(BudgetSpend {
        input_tokens: u64_of(o, "input")?,
        output_tokens: u64_of(o, "output")?,
        cache_read_tokens: u64_of(o, "cache_read")?,
        cache_write_tokens: u64_of(o, "cache_write")?,
    })
}

/// The `step` reply body read back: one step's four record files, its
/// `response.json` events, every tool call's input and output, and each capture
/// log the encoder had bytes for.
pub(crate) fn detail(obj: &serde_json::Map<String, Value>) -> Result<StepDetail, String> {
    Ok(StepDetail {
        seq: str_of(obj, "seq")?,
        meta: doc(obj.get("meta").ok_or("step: missing meta")?)?,
        request: doc(obj.get("request").ok_or("step: missing request")?)?,
        staging: doc(obj.get("staging").ok_or("step: missing staging")?)?,
        response: list_of(obj, "response", doc)?,
        tools: list_of(obj, "tools", tool)?,
        // An absent key is a log with nothing in it — the encoder's own
        // spelling of "no seat for this", read back as exactly that.
        stderr: opt_val(obj, "stderr", crate::files_view::wire::preview_of)?,
        driver: opt_val(obj, "driver", crate::files_view::wire::preview_of)?,
    })
}

/// One record file. The `note` an unparsed doc carries is [`super::super::
/// UNPARSED`](crate::steps_view::UNPARSED) — the frame the seat renders, not a
/// fact of the file, so it is written and never read.
fn doc(v: &Value) -> Result<Doc, String> {
    let o = v.as_object().ok_or("doc: not an object")?;
    match str_of(o, "kind")?.as_str() {
        "json" => Ok(Doc::Json {
            value: o.get("value").ok_or("doc: missing value")?.clone(),
            raw: bytes_of(o, "raw")?,
        }),
        "absent" => Ok(Doc::Absent),
        "unparsed" => Ok(Doc::Unparsed(bytes_of(o, "raw")?)),
        other => Err(format!("doc: unknown kind {other:?}")),
    }
}

fn tool(v: &Value) -> Result<ToolIo, String> {
    let o = v.as_object().ok_or("tool: not an object")?;
    Ok(ToolIo {
        tool_id: str_of(o, "tool_id")?,
        input: doc(o.get("input").ok_or("tool: missing input")?)?,
        output: doc(o.get("output").ok_or("tool: missing output")?)?,
        is_error: bool_of(o, "is_error")?,
    })
}
