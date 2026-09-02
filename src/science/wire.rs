//! The science projection's JSON shape (§8.5, §3.9) — the headless
//! serialization, beside its type for the reason [`crate::workdiff::wire`]
//! gives: these rows' shape *is* this module's vocabulary, and the reply roster
//! still holds the one line naming this encoder.
//!
//! **The diff column is spelled by the diff module, both ways.** A science row
//! carries a [`workdiff::Attempt`](crate::workdiff::Attempt), so its `diff`
//! object is `workdiff::wire`'s own row — one spelling of an attempt's identity
//! and churn, wherever it is said.
//!
//! Absent rather than null throughout, the roster's own discipline: an attempt
//! with no conversation, no frozen goal and no readable config has absences,
//! not empty strings.

use serde_json::{Map, Value, json};

use super::{Attempt, Outcome, Verdict};
use crate::boundary::codec::fields::{list_of, opt_str_of, str_of, u64_of, usize_of};

/// The `science` reply body: one row per attempt.
pub(crate) fn reply(rows: &[Attempt]) -> Value {
    json!({
        "ok": true, "kind": "science",
        "rows": Value::Array(rows.iter().map(row).collect()),
    })
}

/// One attempt: the composed diff, the frozen inputs, the step-record figures,
/// what it said, what was said to it, and the derived outcome.
fn row(attempt: &Attempt) -> Value {
    let mut map = Map::new();
    map.insert(
        "diff".to_owned(),
        crate::workdiff::wire::attempt_row(&attempt.diff),
    );
    for (key, value) in [
        ("base", &attempt.base),
        ("conversation", &attempt.conversation),
        ("goal", &attempt.goal),
        ("governing", &attempt.governing),
        ("response", &attempt.response),
    ] {
        if let Some(value) = value {
            map.insert(key.to_owned(), json!(value));
        }
    }
    map.insert("pins".to_owned(), json!(attempt.pins));
    map.insert("usage".to_owned(), usage(&attempt.usage));
    map.insert("wall_secs".to_owned(), json!(attempt.wall_secs));
    map.insert("steps".to_owned(), json!(attempt.steps));
    map.insert(
        "verdicts".to_owned(),
        Value::Array(attempt.verdicts.iter().map(verdict).collect()),
    );
    // Absent on an intact record, like every other column with nothing to say:
    // nonzero states how many entries compaction deleted out from under the
    // verdicts and the response (§5.1 #12, bl-fde5).
    if attempt.compacted > 0 {
        map.insert("compacted".to_owned(), json!(attempt.compacted));
    }
    map.insert("outcome".to_owned(), outcome(&attempt.outcome));
    Value::Object(map)
}

/// The four ARCH §6 counters, by their own names.
fn usage(spend: &crate::budgets::BudgetSpend) -> Value {
    json!({
        "input_tokens": spend.input_tokens, "output_tokens": spend.output_tokens,
        "cache_read_tokens": spend.cache_read_tokens,
        "cache_write_tokens": spend.cache_write_tokens,
    })
}

fn verdict(v: &Verdict) -> Value {
    json!({ "sender": v.sender, "body": v.body })
}

/// The outcome as a token plus whatever that token can say: the acceptance's
/// commit, the rejection's winner when there was one. `reworked` and `pending`
/// say nothing else, because there is nothing else to say.
fn outcome(outcome: &Outcome) -> Value {
    match outcome {
        Outcome::Accepted { commit } => json!({ "state": ACCEPTED, "commit": commit }),
        Outcome::Rejected { by } => {
            let mut map = Map::new();
            map.insert("state".to_owned(), json!(REJECTED));
            if let Some(by) = by {
                map.insert("by".to_owned(), json!(by));
            }
            Value::Object(map)
        }
        Outcome::Reworked => json!({ "state": REWORKED }),
        Outcome::Pending => json!({ "state": PENDING }),
    }
}

/// The outcome tokens, named once for both directions.
const ACCEPTED: &str = "accepted";
const REJECTED: &str = "rejected";
const REWORKED: &str = "reworked";
const PENDING: &str = "pending";

/// The `science` reply body's rows read back (REMOTE §9 step 2) — strict, like
/// every other decoder here: an unknown outcome token refuses naming it.
pub(crate) fn rows_of(obj: &Map<String, Value>) -> Result<Vec<Attempt>, String> {
    list_of(obj, "rows", row_of)
}

fn row_of(v: &Value) -> Result<Attempt, String> {
    let o = v.as_object().ok_or("science row: not an object")?;
    let diff = o.get("diff").ok_or("science row: missing diff")?;
    Ok(Attempt {
        diff: crate::workdiff::wire::attempt_of(diff)?,
        base: opt_str_of(o, "base")?,
        conversation: opt_str_of(o, "conversation")?,
        goal: opt_str_of(o, "goal")?,
        pins: crate::boundary::codec::fields::strings_of(o, "pins")?,
        governing: opt_str_of(o, "governing")?,
        usage: usage_of(o.get("usage").ok_or("science row: missing usage")?)?,
        wall_secs: u64_of(o, "wall_secs")?,
        steps: usize_of(o, "steps")?,
        response: opt_str_of(o, "response")?,
        verdicts: list_of(o, "verdicts", verdict_of)?,
        compacted: crate::boundary::codec::fields::opt(o, "compacted", usize_of)?.unwrap_or(0),
        outcome: outcome_of(o.get("outcome").ok_or("science row: missing outcome")?)?,
    })
}

fn usage_of(v: &Value) -> Result<crate::budgets::BudgetSpend, String> {
    let o = v.as_object().ok_or("usage: not an object")?;
    Ok(crate::budgets::BudgetSpend {
        input_tokens: u64_of(o, "input_tokens")?,
        output_tokens: u64_of(o, "output_tokens")?,
        cache_read_tokens: u64_of(o, "cache_read_tokens")?,
        cache_write_tokens: u64_of(o, "cache_write_tokens")?,
    })
}

fn verdict_of(v: &Value) -> Result<Verdict, String> {
    let o = v.as_object().ok_or("verdict: not an object")?;
    Ok(Verdict {
        sender: str_of(o, "sender")?,
        body: str_of(o, "body")?,
    })
}

fn outcome_of(v: &Value) -> Result<Outcome, String> {
    let o = v.as_object().ok_or("outcome: not an object")?;
    let state = str_of(o, "state")?;
    Ok(match state.as_str() {
        ACCEPTED => Outcome::Accepted {
            commit: str_of(o, "commit")?,
        },
        REJECTED => Outcome::Rejected {
            by: opt_str_of(o, "by")?,
        },
        REWORKED => Outcome::Reworked,
        PENDING => Outcome::Pending,
        other => return Err(format!("outcome: unknown state {other:?}")),
    })
}
