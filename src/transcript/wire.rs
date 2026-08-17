//! The conversation's JSON shape (§8.5, bl-6233) — the headless serialization
//! of the §11 Transcript tab, beside its type for the reason `workdiff::wire`
//! gives: these rows' shape *is* this module's vocabulary (the `messages/`
//! envelope and the §4.4 canonical blocks).
//!
//! **Bytes ride as text.** Every entry carries its `raw` backing bytes, decoded
//! lossily — a transcript entry is a text file, the §11 Raw toggle is the
//! window's only other door to the envelope the parsed view drops, and a
//! headless seat has none at all. A byte array would be a second spelling of
//! one content.

use serde_json::{Map, Value, json};

use super::{Block, Entry, EntryKind, Transcript, Usage};

/// The decoders, beside the encoders they undo (bl-7067, REMOTE §9 step 2).
pub(crate) mod decode;

/// The `transcript` reply body: one row per entry, in message order, the live
/// streaming tail among them when a call is in flight.
pub(crate) fn reply(transcript: &Transcript) -> Value {
    json!({
        "ok": true, "kind": "transcript",
        "rows": Value::Array(transcript.entries.iter().map(entry_row).collect()),
    })
}

/// One entry: its filename, its `kind` token, whatever that kind can say, and
/// the bytes it was read from. The tokens are the [`EntryKind`] arms — an
/// unparseable entry stays distinguishable from a parsed one on the wire
/// exactly as it does on screen (§15 Y12: surfaced, never dropped).
fn entry_row(entry: &Entry) -> Value {
    let mut map = Map::new();
    map.insert("name".to_owned(), json!(entry.name));
    map.insert("raw".to_owned(), json!(String::from_utf8_lossy(&entry.raw)));
    kind_fields(&entry.kind, &mut map);
    Value::Object(map)
}

/// The kind discriminant and its fields, written into the entry's own object.
fn kind_fields(kind: &EntryKind, map: &mut Map<String, Value>) {
    let word = match kind {
        EntryKind::Delivered {
            sender,
            epitaph,
            body,
        } => {
            map.insert("sender".to_owned(), json!(sender));
            if let Some(epitaph) = epitaph {
                map.insert("epitaph".to_owned(), json!(epitaph.label()));
            }
            map.insert("body".to_owned(), json!(body));
            "delivered"
        }
        EntryKind::Model {
            model_id,
            blocks,
            usage,
        } => {
            map.insert("model_id".to_owned(), json!(model_id));
            map.insert(
                "blocks".to_owned(),
                Value::Array(blocks.iter().map(block_value).collect()),
            );
            map.insert("usage".to_owned(), usage_value(usage));
            "model"
        }
        EntryKind::ToolResult {
            tool_use_id,
            content,
            is_error,
        } => {
            map.insert("tool_use_id".to_owned(), json!(tool_use_id));
            map.insert("content".to_owned(), json!(content));
            map.insert("is_error".to_owned(), json!(is_error));
            "tool-result"
        }
        EntryKind::Streaming { thinking, text } => {
            map.insert("thinking".to_owned(), json!(thinking));
            map.insert("text".to_owned(), json!(text));
            "streaming"
        }
        // The counter values cross, not the sentence built from them: the
        // prefix a seat paints is the row projection's, and a headless seat
        // runs that same projection over this decoded entry.
        EntryKind::Compacted {
            first,
            last,
            summary,
        } => {
            map.insert("first".to_owned(), json!(first));
            map.insert("last".to_owned(), json!(last));
            map.insert("summary".to_owned(), json!(summary));
            "compacted"
        }
        EntryKind::Raw => "raw",
    };
    map.insert("kind".to_owned(), json!(word));
}

/// One canonical content block (§4.4). A tool call carries the summary the
/// chip renders, never a second parse of the input.
fn block_value(block: &Block) -> Value {
    match block {
        Block::Text(text) => json!({ "kind": "text", "text": text }),
        Block::Thinking(text) => json!({ "kind": "thinking", "text": text }),
        Block::ToolUse {
            id,
            name,
            input_summary,
        } => json!({ "kind": "tool-use", "id": id, "name": name, "input": input_summary }),
    }
}

/// The provider's own committed counters, under their own names — no
/// vocabulary is pinned, so a counter brazen adds rides through with no edit
/// here either. Empty is the general path (a legacy entry, or a provider that
/// reported nothing), and it encodes as an empty object rather than absence:
/// "reported nothing" is what the bytes say.
fn usage_value(usage: &Usage) -> Value {
    Value::Object(
        usage
            .iter()
            .map(|(name, count)| (name.clone(), json!(count)))
            .collect(),
    )
}
