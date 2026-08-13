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
