//! The decision queue's row encoder (§8.5, VISION §5 V5.2, STORIES S14) — a
//! sibling of [`super::rows`], kept out of it for the reason [`super::board`]
//! is: a row that carries a derived list of its own.
//!
//! The address fields are spelled with the **same keys the gestures take**
//! (`workspace`, `agent`) **and in the same vocabulary** — the §3.1 workspace
//! name, never a path — so answering a row is copying two values, never
//! translating between an answer's vocabulary and a question's. That second
//! half was the defect bl-22ab closed: the key matched and the value did not,
//! so a copied address earned `unknown workspace` from the very engine that
//! had just answered it.

use crate::attention::AttentionKind;
use crate::boundary::answer::queue::{Acknowledged, QueueRow};
use serde_json::{Value, json};

/// The queue itself, and the acknowledgement's receipt — the two replies whose
/// body is this file's rows, spelled here rather than in [`super::encode`]'s
/// match for the reason the search reply's envelope is (bl-1015): one file
/// learns how a queue answer is said.
pub(super) fn attention(rows: &[QueueRow]) -> Value {
    json!({ "ok": true, "kind": "attention",
            "rows": rows.iter().map(queue_row).collect::<Vec<Value>>() })
}

/// `seen`'s receipt (bl-5cfe): the conversation the watermark landed on, in
/// the same two keys every queue row spells its address with, and the queue
/// that remains under the `rows` key every listing uses. The address is the
/// §3.1 **name**, never a path (REMOTE §8) — a receipt a seat cannot feed back
/// into the next gesture is the defect bl-22ab closed one reply over.
pub(super) fn acknowledged(ack: &Acknowledged) -> Value {
    json!({ "ok": true, "kind": ACKNOWLEDGED,
            "workspace": ack.workspace, "agent": ack.agent,
            "rows": ack.remaining.iter().map(queue_row).collect::<Vec<Value>>() })
}

/// The acknowledgement receipt's kind, named once for both directions.
pub(super) const ACKNOWLEDGED: &str = "acknowledged";

/// That receipt read back (bl-5cfe) — the queue rows through the one row
/// decoder, so a remainder and an `attention` answer are read the same way.
pub(super) fn acknowledged_of(
    o: &serde_json::Map<String, Value>,
) -> Result<crate::boundary::reply::Reply, String> {
    use crate::boundary::codec::fields::{list_of, str_of};
    Ok(crate::boundary::reply::Reply::Acknowledged(Acknowledged {
        workspace: str_of(o, "workspace")?,
        agent: str_of(o, "agent")?,
        remaining: list_of(o, "rows", queue_row_of)?,
    }))
}

fn queue_row(row: &QueueRow) -> Value {
    json!({
        "workspace": row.workspace,
        "agent": row.agent,
        "display": row.display,
        "state": super::rows::state_token(row.state),
        "uncertain": row.uncertain,
        "signals": row.signals.iter().map(|k| signal_token(*k)).collect::<Vec<&str>>(),
        // The firing rules **in words** (bl-09ef): the one home for that
        // sentence is `AttentionKind::says`, and the announcing is a seat's —
        // a desktop notification belongs on the box the operator is looking
        // at. So the sentence crosses beside the tokens rather than being
        // re-worded at each seat, which is the drift §6 forbids. Derived at
        // the encoder, never stored: the row's own home for it is the token.
        "says": row.signals.iter().map(|k| k.says()).collect::<Vec<&str>>().join("; "),
        "preview": row.preview,
        "age_secs": row.age_secs,
        "pending": row.pending,
        "held": row.held.as_ref().map(held),
        // The `refused` signal's own words (bl-9b88) — null, not absent, on the
        // one row encoder that already spells `held` that way.
        "failure": row.failure,
        // The flag's own words (§6 rule 7, bl-6f2f), null when nobody raised
        // one — `held`'s spelling on the row `held` already spells that way.
        "flag": row.flag.as_ref().map(flag),
    })
}

/// The parked invocation, when the row carries one (§8.6): what is waiting, and
/// the control's own sentence about it. A reader needs no other call to decide
/// — which is the point of the reason carrying the input summary and the class.
fn held(held: &crate::control::hold::Held) -> Value {
    json!({ "tool_use": held.tool_use_id, "tool": held.tool, "reason": held.reason })
}

/// The raised flag, when the row carries one (§6 rule 7): when it was raised
/// and why. The stamp rides because it is the acknowledgement's own evidence —
/// a reader that saw one flag and wants to know whether a later one landed
/// compares it, and `/seen` writes exactly this string.
fn flag(raised: &crate::monitor::Flag) -> Value {
    json!({ "at": raised.at, "reason": raised.reason })
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
        AttentionKind::Refused => "refused",
        AttentionKind::Flagged => "flagged",
    }
}

/// The §6 signal table — [`signal_token`]'s other half (bl-7067).
const SIGNALS: [(&str, AttentionKind); 8] = [
    ("notify", AttentionKind::Notify),
    ("stopped", AttentionKind::Stopped),
    ("budget", AttentionKind::Budget),
    ("conflicted", AttentionKind::Conflicted),
    ("mail", AttentionKind::Mail),
    ("held", AttentionKind::Held),
    // Rule 2's rest said in the word that is true of it (bl-b43b) — it stands
    // where `stopped` would, never beside it.
    ("refused", AttentionKind::Refused),
    // §6 rule 7 (bl-6f2f) — the signal-out verb's own word, beside the six
    // yog derives for itself and the one refinement.
    ("flagged", AttentionKind::Flagged),
];

/// One queue row read back. `display` is answered rather than re-derived here
/// for the reason it is answered at all: the §3.3 ladder needs the agents this
/// process cannot see, so the row's own word for itself is the only one there
/// is.
pub(super) fn queue_row_of(v: &Value) -> Result<QueueRow, String> {
    use crate::boundary::codec::fields::{bool_of, i64_of, list_of, opt_val, str_of, usize_of};
    let o = v.as_object().ok_or("queue row: not an object")?;
    Ok(QueueRow {
        workspace: str_of(o, "workspace")?,
        agent: str_of(o, "agent")?,
        display: str_of(o, "display")?,
        state: super::rows::decode::state_of(o)?,
        uncertain: bool_of(o, "uncertain")?,
        signals: list_of(o, "signals", signal_of)?,
        preview: str_of(o, "preview")?,
        age_secs: i64_of(o, "age_secs")?,
        pending: usize_of(o, "pending")?,
        held: opt_val(o, "held", held_of)?,
        failure: crate::boundary::codec::fields::opt_str_of(o, "failure")?,
        flag: opt_val(o, "flag", flag_of)?,
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

fn flag_of(v: &Value) -> Result<crate::monitor::Flag, String> {
    use crate::boundary::codec::fields::str_of;
    let o = v.as_object().ok_or("flag: not an object")?;
    Ok(crate::monitor::Flag {
        at: str_of(o, "at")?,
        reason: str_of(o, "reason")?,
    })
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
