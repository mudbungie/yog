//! The world-level client row read back off its own keys (bl-4e08) — split
//! off [`super`] at §12's cap (bl-53d1). The doctor's check that sat beside it
//! moved to `reply::doctor` with the rest of that answer (bl-355c).

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
