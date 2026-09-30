//! **The doctor's answer in both directions** (bl-28f4, bl-355c): the rows,
//! and the punched wire's counters as one `rendezvous` object beside them —
//! cut off the roster on `prices`' seam once the answer grew a key of its own.
//!
//! `rendezvous` is **absent, never null**, for its two readings: the answering
//! process has no listener and so no loop to speak of, or the engine predates
//! edition 21. Present, every one of its six edition-21 keys is required;
//! `accepted` (edition 22, bl-5276) is read as `0` where an older engine
//! leaves it out, and typed where it is said.

use serde_json::{Map, Value, json};

use super::rows::decode::rows_of;
use crate::boundary::codec::fields::{bool_of, opt, opt_val, str_of, u64_of};
use crate::doctor::{Report, Row};
use crate::wire::rendezvous::Standing;

/// The reply kind, named once for both directions.
pub(super) const KIND: &str = "doctor";

/// The whole answer as one object.
pub(super) fn reply(report: &Report) -> Value {
    let mut map = Map::new();
    map.insert("ok".to_owned(), json!(true));
    map.insert("kind".to_owned(), json!(KIND));
    map.insert(
        "rows".to_owned(),
        Value::Array(report.rows.iter().map(row).collect()),
    );
    if let Some(s) = &report.rendezvous {
        map.insert(
            "rendezvous".to_owned(),
            json!({ "active": s.active, "published": s.published, "calls": s.calls,
                    "punched": s.punched, "served": s.served, "accepted": s.accepted,
                    "last_poll_unix": s.last_poll_unix }),
        );
    }
    Value::Object(map)
}

/// The inverse of [`reply`].
pub(super) fn report_of(o: &Map<String, Value>) -> Result<Report, String> {
    Ok(Report {
        rows: rows_of(o, row_of)?,
        rendezvous: opt_val(o, "rendezvous", standing_of)?,
    })
}

/// One check, as every seat renders it (bl-28f4): what was examined, whether
/// this box passes it, the fact that was read — and the act, **absent on a
/// passing row**, because a remedy beside a fact that is fine is advice nobody
/// asked for.
fn row(row: &Row) -> Value {
    let mut map = Map::new();
    map.insert("check".to_owned(), json!(row.check));
    map.insert("ok".to_owned(), json!(row.ok));
    map.insert("fact".to_owned(), json!(row.fact));
    if let Some(remedy) = &row.remedy {
        map.insert("remedy".to_owned(), json!(remedy));
    }
    Value::Object(map)
}

/// One check, read back — the remedy absent exactly where the row passed,
/// which is the one thing a seat renders differently.
fn row_of(v: &Value) -> Result<Row, String> {
    let o = v.as_object().ok_or("doctor row: not an object")?;
    Ok(Row {
        check: str_of(o, "check")?,
        ok: bool_of(o, "ok")?,
        fact: str_of(o, "fact")?,
        remedy: opt(o, "remedy", str_of)?,
    })
}

/// The loop's standing, read back strictly: every key, each of its type —
/// the one post-21 key defaulted where absent.
fn standing_of(v: &Value) -> Result<Standing, String> {
    let s = v.as_object().ok_or("doctor: rendezvous is not an object")?;
    Ok(Standing {
        active: bool_of(s, "active")?,
        published: u64_of(s, "published")?,
        calls: u64_of(s, "calls")?,
        punched: u64_of(s, "punched")?,
        served: u64_of(s, "served")?,
        accepted: opt(s, "accepted", u64_of)?.unwrap_or(0),
        last_poll_unix: u64_of(s, "last_poll_unix")?,
    })
}
