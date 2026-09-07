//! The staged proposals' JSON spelling, both directions (§9.6; REMOTE §9.22,
//! bl-dd88) — beside the type that owns it, exactly as `transcript::wire` and
//! `rail::wire` sit beside theirs: the shape of a derived row is the deriving
//! module's vocabulary, and the boundary names it rather than restating it.
//!
//! **`whole` is absent, never null, for the bare listing.** Absence is the
//! fact — the read named no proposal — and a null would be a second spelling
//! of it, which the codec's own discipline refuses.
//!
//! **`fresh` crosses even though it is derived from `lineages`.** A seat could
//! read emptiness as staleness, but that is a rule, and the wire states a
//! derivation so a seat need not own one (REMOTE §9.4). Only the engine can be
//! sure the two were read in one pass — which is what makes the pair honest —
//! so the decoder reads it back strictly rather than recomputing it.

use serde_json::{Map, Value, json};

use super::{ProposalRow, ProposalView};
use crate::boundary::codec::fields::{bool_of, list_of, opt_str_of, str_of, strings_of};

/// The reply `kind` a proposals read answers under, named once for both
/// directions.
pub const KIND: &str = "proposals";

/// The whole answer.
pub fn reply(view: &ProposalView) -> Value {
    let mut map = Map::new();
    map.insert("ok".to_owned(), json!(true));
    map.insert("kind".to_owned(), json!(KIND));
    map.insert(
        "rows".to_owned(),
        Value::Array(view.rows.iter().map(row).collect()),
    );
    if let Some(whole) = &view.whole {
        map.insert("whole".to_owned(), json!(whole));
    }
    Value::Object(map)
}

/// One row.
fn row(row: &ProposalRow) -> Value {
    json!({
        "id": row.id,
        "lineages": row.lineages,
        "parent": row.parent,
        "fresh": row.fresh,
        "diffstat": row.diffstat,
        "subject": row.subject,
    })
}

/// Read the answer back. Strict on every row field, since each is a derivation
/// the engine made and a seat cannot re-make; forgiving of `whole`'s absence,
/// which is a reading.
pub fn view_of(o: &Map<String, Value>) -> Result<ProposalView, String> {
    Ok(ProposalView {
        rows: list_of(o, "rows", row_of)?,
        whole: opt_str_of(o, "whole")?,
    })
}

/// One row read back.
fn row_of(v: &Value) -> Result<ProposalRow, String> {
    let o = v.as_object().ok_or("proposal row: not an object")?;
    Ok(ProposalRow {
        id: str_of(o, "id")?,
        lineages: strings_of(o, "lineages")?,
        parent: str_of(o, "parent")?,
        fresh: bool_of(o, "fresh")?,
        diffstat: str_of(o, "diffstat")?,
        subject: str_of(o, "subject")?,
    })
}
