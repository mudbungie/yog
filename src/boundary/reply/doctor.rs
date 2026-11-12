//! **The doctor's answer in both directions** (bl-28f4): one row per check,
//! cut off the roster on `prices`' seam.

use serde_json::{Map, Value, json};

use super::rows::decode::rows_of;
use crate::boundary::codec::fields::{bool_of, opt, str_of};
use crate::doctor::Row;

/// The reply kind, named once for both directions.
pub(super) const KIND: &str = "doctor";

/// The whole answer as one object.
pub(super) fn reply(rows: &[Row]) -> Value {
    json!({ "ok": true, "kind": KIND, "rows": rows.iter().map(row).collect::<Vec<_>>() })
}

/// The inverse of [`reply`].
pub(super) fn rows(o: &Map<String, Value>) -> Result<Vec<Row>, String> {
    rows_of(o, row_of)
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
