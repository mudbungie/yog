//! The two world-level rows read back off their own keys (bl-4e08, bl-28f4)
//! — a registered client and a doctor's check — split off [`super`] at §12's
//! cap (bl-53d1) on the seam the encoder already draws: `encode` keeps the
//! rows nothing else spells, and these two are its.

use serde_json::Value;

use crate::boundary::codec::fields::{bool_of, i64_of, opt, str_of};

/// One registered client, read back (REMOTE §5, bl-4e08) — the tools through
/// `registry::tools`, the same decoder the gesture and the document spend.
pub(super) fn client_row(v: &Value) -> Result<crate::registry::roster::ClientRow, String> {
    let o = v.as_object().ok_or("client row: not an object")?;
    Ok(crate::registry::roster::ClientRow {
        client: str_of(o, "client")?,
        present: bool_of(o, "present")?,
        tools: crate::registry::tools::decode(o.get("tools").ok_or("client row: missing tools")?)?,
        // Absent is never (bl-d542); a present field must be a number, so a
        // stamp that is not one refuses here rather than reading as never.
        last_seen: opt(o, "last_seen", i64_of)?,
    })
}

/// One check, read back (bl-28f4) — the remedy absent exactly where the row
/// passed, which is the one thing a seat renders differently.
pub(super) fn doctor_row(v: &Value) -> Result<crate::doctor::Row, String> {
    let o = v.as_object().ok_or("doctor row: not an object")?;
    Ok(crate::doctor::Row {
        check: str_of(o, "check")?,
        ok: bool_of(o, "ok")?,
        fact: str_of(o, "fact")?,
        remedy: opt(o, "remedy", str_of)?,
    })
}
