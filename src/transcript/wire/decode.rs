//! The conversation's **decoders** (§8.5, REMOTE §9 step 2, bl-7067) — one per
//! encoder in [`super`], undoing the entry, its kind's own fields, the §4.4
//! canonical blocks and the provider's committed counters.
//!
//! **Bytes come back from text.** `raw` was written with
//! `String::from_utf8_lossy`, which is exact for every entry whose file is
//! UTF-8 — every entry lernie writes — and lossy for one that is not, in which
//! case the replacement happened on the way OUT and this reads back what the
//! wire actually carries. That is the ruling [`super`] states, read from the
//! other side: a transcript entry is a text file, and a byte array beside the
//! text would be a second spelling of one content.

use serde_json::Value;

use super::super::{Block, Entry, EntryKind, Transcript, Usage};
use crate::boundary::codec::fields::{bool_of, bytes_of, list_of, opt_val, str_of, usize_of};
use crate::inboxview::Epitaph;

/// The `transcript` reply body read back: one entry per row, in message order.
pub(crate) fn transcript(obj: &serde_json::Map<String, Value>) -> Result<Transcript, String> {
    Ok(Transcript {
        entries: list_of(obj, "rows", entry_row)?,
    })
}

fn entry_row(v: &Value) -> Result<Entry, String> {
    let o = v.as_object().ok_or("transcript row: not an object")?;
    let kind = match str_of(o, "kind")?.as_str() {
        "delivered" => EntryKind::Delivered {
            sender: str_of(o, "sender")?,
            epitaph: opt_val(o, "epitaph", epitaph)?,
            body: str_of(o, "body")?,
        },
        "model" => EntryKind::Model {
            model_id: str_of(o, "model_id")?,
            blocks: list_of(o, "blocks", block)?,
            usage: usage(o.get("usage").ok_or("transcript row: missing usage")?)?,
        },
        "tool-result" => EntryKind::ToolResult {
            tool_use_id: str_of(o, "tool_use_id")?,
            content: str_of(o, "content")?,
            is_error: bool_of(o, "is_error")?,
        },
        "streaming" => EntryKind::Streaming {
            thinking: str_of(o, "thinking")?,
            text: str_of(o, "text")?,
        },
        "compacted" => EntryKind::Compacted {
            first: usize_of(o, "first")?,
            last: usize_of(o, "last")?,
            summary: str_of(o, "summary")?,
        },
        "raw" => EntryKind::Raw,
        other => return Err(format!("transcript row: unknown kind {other:?}")),
    };
    Ok(Entry {
        name: str_of(o, "name")?,
        raw: bytes_of(o, "raw")?,
        kind,
    })
}

/// The §2.6 ending. [`Epitaph::parse`] is total — an unrecognized value is
/// [`Epitaph::Unknown`], which is exactly what the label it was written from
/// says — so this is the one token on the reply surface with no refusal, and
/// deliberately: forward-compat pass-through is that arm's whole point.
fn epitaph(v: &Value) -> Result<Epitaph, String> {
    Ok(Epitaph::parse(v.as_str().ok_or("epitaph: not a string")?))
}

fn block(v: &Value) -> Result<Block, String> {
    let o = v.as_object().ok_or("block: not an object")?;
    match str_of(o, "kind")?.as_str() {
        "text" => Ok(Block::Text(str_of(o, "text")?)),
        "thinking" => Ok(Block::Thinking(str_of(o, "text")?)),
        "tool-use" => Ok(Block::ToolUse {
            id: str_of(o, "id")?,
            name: str_of(o, "name")?,
            input_summary: str_of(o, "input")?,
        }),
        other => Err(format!("block: unknown kind {other:?}")),
    }
}

/// The provider's own counters under their own names — no vocabulary is
/// pinned here either, so a counter brazen adds rides back with no edit.
fn usage(v: &Value) -> Result<Usage, String> {
    v.as_object()
        .ok_or("usage: not an object")?
        .iter()
        .map(|(name, count)| {
            count
                .as_u64()
                .map(|n| (name.clone(), n))
                .ok_or_else(|| format!("usage {name:?}: not a count"))
        })
        .collect()
}
