//! The Inbox tab's JSON shape (§8.5, bl-6233) — the headless serialization of
//! the §11 Inbox tab's listing, beside its type for the reason
//! `workdiff::wire` gives: the shape of these rows is this module's own
//! vocabulary (ARCH §2.11's deposit envelope).

use serde_json::{Map, Value, json};

use super::{Deposit, InboxEntry};

/// The `inbox` reply body: one row per deposit file, newest as the listing
/// orders them.
pub(crate) fn reply(entries: &[InboxEntry]) -> Value {
    json!({
        "ok": true, "kind": "inbox",
        "rows": Value::Array(entries.iter().map(entry_row).collect()),
    })
}

/// One deposit: its filename, the parsed envelope, and the file's bytes. `raw`
/// rides along because the parsed view drops the envelope and the §11 Raw
/// toggle is the window's only other door to it — a headless seat has none, so
/// dropping it here would make the bytes unreachable rather than merely
/// unrendered. It is decoded lossily: a deposit is a text file, and a byte
/// array would be a second spelling of the same content.
fn entry_row(entry: &InboxEntry) -> Value {
    let mut map = Map::new();
    map.insert("name".to_owned(), json!(entry.name));
    map.insert("raw".to_owned(), json!(String::from_utf8_lossy(&entry.raw)));
    map.insert("deposit".to_owned(), deposit_value(&entry.deposit));
    Value::Object(map)
}

/// The deposit's frontmatter and body. Absent fields are absent keys, never
/// nulls or empty strings: a forgiving parse of a hand-edited file says "this
/// was not stated", and an empty `from:` would be a different claim.
fn deposit_value(deposit: &Deposit) -> Value {
    let mut map = Map::new();
    for (key, value) in [
        ("from", deposit.sender.as_ref()),
        ("deposited_at", deposit.deposited_at.as_ref()),
        ("terminal_ref", deposit.terminal_ref.as_ref()),
    ] {
        if let Some(value) = value {
            map.insert(key.to_owned(), json!(value));
        }
    }
    // The §2.6 ending, in the one wording every seat shares — an unknown
    // forward-compat value rides through verbatim, as it does on screen.
    if let Some(epitaph) = &deposit.epitaph {
        map.insert("epitaph".to_owned(), json!(epitaph.label()));
    }
    map.insert("body".to_owned(), json!(deposit.body));
    Value::Object(map)
}

/// The `inbox` reply body read back (bl-7067): one entry per deposit row.
pub(crate) fn entries_of(obj: &serde_json::Map<String, Value>) -> Result<Vec<InboxEntry>, String> {
    use crate::boundary::codec::fields::list_of;
    list_of(obj, "rows", entry_of)
}

fn entry_of(v: &Value) -> Result<InboxEntry, String> {
    use crate::boundary::codec::fields::{bytes_of, str_of};
    let o = v.as_object().ok_or("inbox row: not an object")?;
    Ok(InboxEntry {
        name: str_of(o, "name")?,
        raw: bytes_of(o, "raw")?,
        deposit: deposit_of(o.get("deposit").ok_or("inbox row: missing deposit")?)?,
    })
}

/// The frontmatter and body. An absent key is an absent field, exactly as the
/// encoder's own note says: a forgiving parse of a hand-edited file said "this
/// was not stated", and an empty `from:` would be a different claim.
fn deposit_of(v: &Value) -> Result<Deposit, String> {
    use crate::boundary::codec::fields::{opt_str_of, str_of};
    let o = v.as_object().ok_or("deposit: not an object")?;
    Ok(Deposit {
        sender: opt_str_of(o, "from")?,
        deposited_at: opt_str_of(o, "deposited_at")?,
        epitaph: opt_str_of(o, "epitaph")?.map(|w| super::Epitaph::parse(&w)),
        terminal_ref: opt_str_of(o, "terminal_ref")?,
        body: str_of(o, "body")?,
    })
}
