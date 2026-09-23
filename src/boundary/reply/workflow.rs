//! **The workflow mark on the wire** (REMOTE §9.24, bl-b680): how the
//! `governing` reply spells the §9.4 mark beside the commit it rides with, and
//! how a seat reads it back.
//!
//! Its own file on the seam every type-owning `wire` module takes — the
//! encoder and the decoder of one body beside each other, so the two cannot
//! drift — and beside the reply rather than inside `config_edit`, because the
//! derivation there is git and this is JSON.
//!
//! **`null` is the general path.** No mark anywhere on the descent is what
//! every unmarked conversation answers, and it is spelled as `null` rather
//! than an absent key on `follows`' own precedent: the encoder always writes
//! the key, and a reader that has never heard of it reads the answer it
//! always read. The decoder accepts absence too (REMOTE §3.2: a post-floor
//! key an older engine never wrote defaults), and reads a present object
//! strictly — every token named, a mistyped one refused by name.

use serde_json::{Map, Value, json};

use crate::boundary::codec::fields::{opt_str_of, str_of};
use crate::config_edit::branch::workflow_mark::WorkflowMark;

/// The mark as its body, or `null`.
pub(super) fn value(mark: Option<&WorkflowMark>) -> Value {
    match mark {
        Some(mark) => json!({
            "holder": mark.holder, "oid": mark.oid, "short_oid": mark.short_oid,
            "lineage": mark.lineage,
        }),
        None => Value::Null,
    }
}

/// The inverse of [`value`] for a present, non-null value. `lineage` is
/// `null` — never absent — when no `config/*` ref stands on the marked
/// commit, and it is read as such.
pub(super) fn mark_of(v: &Value) -> Result<WorkflowMark, String> {
    let o: &Map<String, Value> = v.as_object().ok_or("workflow_mark: not an object")?;
    Ok(WorkflowMark {
        holder: str_of(o, "holder")?,
        oid: str_of(o, "oid")?,
        short_oid: str_of(o, "short_oid")?,
        lineage: opt_str_of(o, "lineage")?,
    })
}
