//! The codec's shared field readers (§8.5) — the total helpers every family
//! module imports, beside the roster that uses them.
//!
//! The seam is the one [`line::args`](crate::boundary::line) already draws one
//! serialization over: the verb table is one thing, and *what a field is read
//! as* is another. Strictness lives here — a missing field, a mistyped value
//! and an out-of-range number each refuse **by name**, because a gesture is an
//! instruction and the forgiving-parse discipline of an `ops.jsonl` read has no
//! place in one.

use serde_json::{Map, Value};
use std::path::PathBuf;

use super::{Action, Gesture};

pub(super) fn act(action: Action) -> Gesture {
    Gesture::Act(action)
}

/// A required string field, or the refusal naming it.
pub(super) fn str_of(obj: &Map<String, Value>, key: &str) -> Result<String, String> {
    obj.get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| format!("missing or non-string field {key:?}"))
}

/// A required path field — a string, read as a path.
pub(super) fn path_of(obj: &Map<String, Value>, key: &str) -> Result<PathBuf, String> {
    Ok(PathBuf::from(str_of(obj, key)?))
}

/// A required unsigned-integer field.
pub(super) fn usize_of(obj: &Map<String, Value>, key: &str) -> Result<usize, String> {
    let n = obj
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("missing or non-integer field {key:?}"))?;
    usize::try_from(n).map_err(|_| format!("field {key:?} out of range"))
}

/// A tiny string-object builder for the optional-field arms.
pub(super) fn obj(pairs: &[(&str, &str)]) -> Map<String, Value> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), Value::String((*v).to_owned())))
        .collect()
}
