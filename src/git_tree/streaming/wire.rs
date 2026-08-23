//! The [`Stream`](super::Stream)'s JSON spelling, both directions (REMOTE §3,
//! bl-73e7) — beside the type that owns it, exactly as `transcript::wire` and
//! `rail::wire` sit beside theirs: the shape of a fold *is* the folding
//! module's vocabulary, so the boundary names it rather than restating it.
//!
//! Three fields, each optional in the sense the type already gives them:
//! `text` and `thinking` are absent until a delta of that kind has landed —
//! which is not the same claim as an empty string — and `delta` names the kind
//! of the **last** one, absent while the stream has produced nothing at all
//! (§5.1 #9's "waiting for the API").

use serde_json::{Map, Value, json};

use super::{Delta, Stream};

/// The token for each [`Delta`] arm — one table, read by both directions, so a
/// spelling cannot drift from its reading.
const TEXT: &str = "text";
const THINKING: &str = "thinking";

/// Encode one fold.
pub fn stream_value(stream: &Stream) -> Value {
    let mut map = Map::new();
    if let Some(text) = &stream.text {
        map.insert(TEXT.to_owned(), json!(text));
    }
    if let Some(thinking) = &stream.thinking {
        map.insert(THINKING.to_owned(), json!(thinking));
    }
    if let Some(delta) = stream.last_delta {
        map.insert("delta".to_owned(), json!(token(delta)));
    }
    Value::Object(map)
}

/// Read one back. Strict on a field that is present and of the wrong shape,
/// forgiving of one that is absent — the codec's own discipline: absence is a
/// reading, and a `delta` naming no arm is a codec that has drifted.
pub fn stream_of(o: &Map<String, Value>) -> Result<Stream, String> {
    use crate::boundary::codec::fields::opt_str_of;
    Ok(Stream {
        text: opt_str_of(o, TEXT)?,
        thinking: opt_str_of(o, THINKING)?,
        last_delta: match opt_str_of(o, "delta")?.as_deref() {
            None => None,
            Some(TEXT) => Some(Delta::Text),
            Some(THINKING) => Some(Delta::Thinking),
            Some(other) => return Err(format!("stream: unknown delta kind {other:?}")),
        },
    })
}

/// The token one arm is written as.
fn token(delta: Delta) -> &'static str {
    match delta {
        Delta::Text => TEXT,
        Delta::Thinking => THINKING,
    }
}
