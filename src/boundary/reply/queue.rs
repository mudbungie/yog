//! The decision queue's row encoder (§8.5, VISION §5 V5.2, STORIES S14) — a
//! sibling of [`super::rows`], kept out of it for the reason [`super::board`]
//! is: a row that carries a derived list of its own.
//!
//! The address fields are spelled with the **same keys the gestures take**
//! (`workspace`, `agent`), so answering a row is copying two values, never
//! translating between an answer's vocabulary and a question's.

use crate::attention::AttentionKind;
use crate::boundary::answer::queue::QueueRow;
use serde_json::{Value, json};

pub(super) fn queue_row(row: &QueueRow) -> Value {
    json!({
        "workspace": row.workspace.to_string_lossy(),
        "agent": row.agent,
        "display": row.display,
        "state": super::rows::state_token(row.state),
        "uncertain": row.uncertain,
        "signals": row.signals.iter().map(|k| signal_token(*k)).collect::<Vec<&str>>(),
        "preview": row.preview,
        "age_secs": row.age_secs,
        "pending": row.pending,
        "held": row.held.as_ref().map(held),
    })
}

/// The parked invocation, when the row carries one (§8.6): what is waiting, and
/// the control's own sentence about it. A reader needs no other call to decide
/// — which is the point of the reason carrying the input summary and the class.
fn held(held: &crate::control::hold::Held) -> Value {
    json!({ "tool_use": held.tool_use_id, "tool": held.tool, "reason": held.reason })
}

/// The §6 signal tokens — the `ui.json` `seen` keys' own words for four of
/// them, so a reader of the queue and a reader of the watermark document are
/// reading one vocabulary.
fn signal_token(kind: AttentionKind) -> &'static str {
    match kind {
        AttentionKind::Notify => "notify",
        AttentionKind::Stopped => "stopped",
        AttentionKind::Budget => "budget",
        AttentionKind::Conflicted => "conflicted",
        AttentionKind::Mail => "mail",
        AttentionKind::Held => "held",
    }
}

/// The §6 signal table — [`signal_token`]'s other half (bl-7067).
const SIGNALS: [(&str, AttentionKind); 6] = [
    ("notify", AttentionKind::Notify),
    ("stopped", AttentionKind::Stopped),
    ("budget", AttentionKind::Budget),
    ("conflicted", AttentionKind::Conflicted),
    ("mail", AttentionKind::Mail),
    ("held", AttentionKind::Held),
];

/// One queue row read back. `display` is answered rather than re-derived here
/// for the reason it is answered at all: the §3.3 ladder needs the agents this
/// process cannot see, so the row's own word for itself is the only one there
/// is.
pub(super) fn queue_row_of(v: &Value) -> Result<QueueRow, String> {
    use crate::boundary::codec::fields::{bool_of, i64_of, list_of, opt_val, str_of, usize_of};
    let o = v.as_object().ok_or("queue row: not an object")?;
    Ok(QueueRow {
        workspace: crate::boundary::codec::fields::path_of(o, "workspace")?,
        agent: str_of(o, "agent")?,
        display: str_of(o, "display")?,
        state: super::rows::decode::state_of(o)?,
        uncertain: bool_of(o, "uncertain")?,
        signals: list_of(o, "signals", signal_of)?,
        preview: str_of(o, "preview")?,
        age_secs: i64_of(o, "age_secs")?,
        pending: usize_of(o, "pending")?,
        held: opt_val(o, "held", held_of)?,
    })
}

fn signal_of(v: &Value) -> Result<AttentionKind, String> {
    let token = v.as_str().ok_or("signal: not a string")?;
    SIGNALS
        .iter()
        .find(|(word, _)| *word == token)
        .map(|(_, kind)| *kind)
        .ok_or_else(|| format!("unknown signal {token:?}"))
}

fn held_of(v: &Value) -> Result<crate::control::hold::Held, String> {
    use crate::boundary::codec::fields::str_of;
    let o = v.as_object().ok_or("held: not an object")?;
    Ok(crate::control::hold::Held {
        tool_use_id: str_of(o, "tool_use")?,
        tool: str_of(o, "tool")?,
        reason: str_of(o, "reason")?,
    })
}
