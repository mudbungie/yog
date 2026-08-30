//! Byte-level parsing for the transcript view-model: a model `.json` into
//! canonical content blocks, and a `tool` `.json` into a `tool_result`.
//!
//! Split out of `mod.rs` for the 300-line budget (DESIGN §12); the
//! classification that calls these — filename origin token and extension —
//! stays beside the enumeration it belongs to. Every parse is forgiving:
//! unparseable bytes fall to the Raw bucket rather than erroring, so no file
//! is ever dropped (§15 Y12).

use super::{Block, EntryKind, Usage};

/// Compact-JSON cap for a `tool_use` input summary chip; longer inputs are
/// truncated with an ellipsis (the full bytes remain under the Raw toggle).
const INPUT_SUMMARY_CAP: usize = 200;

/// Parse a model `.json` into content blocks. Valid JSON always yields a
/// `Model` (possibly with no recognized blocks); only unparseable bytes fall
/// to the Raw bucket.
pub(super) fn parse_model(model_id: &str, raw: &[u8]) -> EntryKind {
    match serde_json::from_slice::<serde_json::Value>(raw) {
        Ok(value) => EntryKind::Model {
            model_id: model_id.to_string(),
            blocks: blocks_from_value(&value),
            usage: usage_from_value(&value),
        },
        Err(_) => EntryKind::Raw,
    }
}

/// The committed `usage` record's counters, verbatim (lernie ≥0.0.4 seals the
/// provider's report as `content`'s sibling). Only integer-valued counters are
/// token counts; anything else is skipped. A bare-array entry or an object
/// without `usage` yields the empty record — the general path, never an error.
fn usage_from_value(value: &serde_json::Value) -> Usage {
    value
        .get("usage")
        .and_then(serde_json::Value::as_object)
        .into_iter()
        .flatten()
        .filter_map(|(counter, v)| Some((counter.clone(), v.as_u64()?)))
        .collect()
}

/// Content blocks from a model payload — forgiving over the two canonical
/// shapes: a bare block array, or a message object with a `content` array or
/// string. Anything else yields no blocks (the Raw toggle still shows bytes).
fn blocks_from_value(value: &serde_json::Value) -> Vec<Block> {
    if let Some(arr) = block_array(value) {
        return arr.iter().filter_map(block_from_value).collect();
    }
    match value.get("content") {
        Some(serde_json::Value::String(s)) => vec![Block::Text(s.clone())],
        _ => Vec::new(),
    }
}

/// **Where a committed message's content blocks live** — the one answer, used
/// by both parsers. A `messages/NNN-*.json` file is a *bare array of canonical
/// blocks* as litany commits it (`[{"type":"tool_result",…}]`,
/// `[{"type":"tool_use",…}]`); an API-shaped message object wrapping them in
/// `content` is accepted too. `None` for any other shape.
///
/// Two answers were the bug behind the operator's stuck `⚙ bash — running`
/// (bl-47ec): the model parser accepted the bare array, the `tool` parser did
/// not, so every real `NNN-tool.json` fell to the Raw bucket — and a result
/// that is not classified as a result cannot retire the call that awaits it.
fn block_array(value: &serde_json::Value) -> Option<&Vec<serde_json::Value>> {
    value
        .as_array()
        .or_else(|| value.get("content")?.as_array())
}

/// One canonical content block, or `None` for an untyped / unknown-typed
/// element (skipped — the whole entry stays inspectable via Raw).
fn block_from_value(v: &serde_json::Value) -> Option<Block> {
    match v.get("type").and_then(|t| t.as_str())? {
        "text" => Some(Block::Text(str_field(v, "text"))),
        "thinking" => Some(Block::Thinking(str_field(v, "thinking"))),
        "tool_use" => Some(Block::ToolUse {
            id: str_field(v, "id"),
            name: str_field(v, "name"),
            input_summary: summarize_input(v.get("input")),
        }),
        _ => None,
    }
}

/// A compact one-line JSON summary of a `tool_use` input, capped.
fn summarize_input(input: Option<&serde_json::Value>) -> String {
    let Some(v) = input else {
        return String::new();
    };
    let compact = v.to_string();
    if compact.chars().count() > INPUT_SUMMARY_CAP {
        let head: String = compact.chars().take(INPUT_SUMMARY_CAP).collect();
        format!("{head}…")
    } else {
        compact
    }
}

/// Parse a `tool` `.json` into a `ToolResult`. `None` (→ Raw) when the bytes
/// don't parse or carry no `tool_use_id`-bearing result block.
pub(super) fn parse_tool_result(raw: &[u8]) -> Option<EntryKind> {
    let value: serde_json::Value = serde_json::from_slice(raw).ok()?;
    let block = find_tool_result(&value)?;
    Some(EntryKind::ToolResult {
        tool_use_id: str_field(block, "tool_use_id"),
        content: tool_result_content(block.get("content")),
        is_error: block
            .get("is_error")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
    })
}

/// Locate the result block — the value itself when it carries a
/// `tool_use_id`, else the first such element of the payload's block array
/// ([`block_array`]: the bare array litany commits, or a `content` wrapper).
/// The id is never inspected, only carried: `call_…` (OpenAI) and `toolu_…`
/// (Anthropic) are equally opaque, and the pairing is byte equality.
fn find_tool_result(value: &serde_json::Value) -> Option<&serde_json::Value> {
    if value.get("tool_use_id").is_some() {
        return Some(value);
    }
    block_array(value)?
        .iter()
        .find(|b| b.get("tool_use_id").is_some())
}

/// Flatten a `tool_result` `content` field to text: a bare string verbatim,
/// an array of blocks concatenated (text blocks' text; else compact JSON),
/// anything else compact JSON, absent/null empty.
fn tool_result_content(content: Option<&serde_json::Value>) -> String {
    match content {
        None | Some(serde_json::Value::Null) => String::new(),
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(serde_json::Value::Array(arr)) => arr.iter().map(content_piece).collect(),
        Some(other) => other.to_string(),
    }
}

/// One element of a `tool_result` content array: a text block's text, else
/// its compact JSON.
fn content_piece(v: &serde_json::Value) -> String {
    if v.get("type").and_then(|t| t.as_str()) == Some("text") {
        str_field(v, "text")
    } else {
        v.to_string()
    }
}

/// A string field of a JSON object, or empty when absent / non-string.
fn str_field(v: &serde_json::Value, key: &str) -> String {
    v.get(key)
        .and_then(|x| x.as_str())
        .unwrap_or_default()
        .to_string()
}
