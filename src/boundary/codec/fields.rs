//! The codec's shared field readers (§8.5) — the total helpers every family
//! module imports, beside the roster that uses them.
//!
//! The seam is the one [`line::args`](crate::boundary::line) already draws one
//! serialization over: the verb table is one thing, and *what a field is read
//! as* is another. Strictness lives here — a missing field, a mistyped value
//! and an out-of-range number each refuse **by name**, because a gesture is an
//! instruction and the forgiving-parse discipline of an `ops.jsonl` read has no
//! place in one.
//!
//! Since bl-7067 the same vocabulary serves the **reply** codec's decode side
//! (`reply::decode` and the per-type `wire` modules), which is why these are
//! `pub(crate)` rather than the gesture codec's own: an answer read back off
//! the wire is read under exactly the rules a gesture is, and two field
//! vocabularies for one JSON dialect would drift within a week.

use serde_json::{Map, Value};
use std::path::PathBuf;

use super::{Action, Gesture};

pub(super) fn act(action: Action) -> Gesture {
    Gesture::Act(action)
}

/// A required string field, or the refusal naming it.
pub(crate) fn str_of(obj: &Map<String, Value>, key: &str) -> Result<String, String> {
    obj.get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| format!("missing or non-string field {key:?}"))
}

/// A required path field — a string, read as a path.
pub(crate) fn path_of(obj: &Map<String, Value>, key: &str) -> Result<PathBuf, String> {
    Ok(PathBuf::from(str_of(obj, key)?))
}

/// A required boolean field.
pub(crate) fn bool_of(obj: &Map<String, Value>, key: &str) -> Result<bool, String> {
    obj.get(key)
        .and_then(Value::as_bool)
        .ok_or_else(|| format!("missing or non-boolean field {key:?}"))
}

/// A required signed-integer field — an age, a timestamp, an exit status.
pub(crate) fn i64_of(obj: &Map<String, Value>, key: &str) -> Result<i64, String> {
    obj.get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("missing or non-integer field {key:?}"))
}

/// A required unsigned-integer field — a token count, a byte size.
pub(crate) fn u64_of(obj: &Map<String, Value>, key: &str) -> Result<u64, String> {
    obj.get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("missing or non-integer field {key:?}"))
}

/// A required unsigned-integer field, narrowed.
pub(crate) fn usize_of(obj: &Map<String, Value>, key: &str) -> Result<usize, String> {
    let n = u64_of(obj, key)?;
    usize::try_from(n).map_err(|_| format!("field {key:?} out of range"))
}

/// A required string field read as **bytes** (§11's `raw` keys): the wire
/// carries a file's contents as text and this is the one place that choice is
/// undone. Lossless for the UTF-8 the encoder was given; a byte no `String`
/// can name was already replaced on the way out, which the encoders say.
pub(crate) fn bytes_of(obj: &Map<String, Value>, key: &str) -> Result<Vec<u8>, String> {
    Ok(str_of(obj, key)?.into_bytes())
}

/// An **optional** field of any shape: absent or `null` reads as `None`,
/// anything else is read by `read` and refuses by name on a mismatch. The one
/// field shape where "not stated" is a value rather than a malformed envelope
/// (§3.3's typed target binding, bl-6654; every absent-is-a-fact key the reply
/// surface spells), so it is a reader of its own rather than a `str_of` call
/// the caller is free to swallow the error of.
pub(crate) fn opt<T>(
    obj: &Map<String, Value>,
    key: &str,
    read: impl Fn(&Map<String, Value>, &str) -> Result<T, String>,
) -> Result<Option<T>, String> {
    match obj.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(_) => read(obj, key).map(Some),
    }
}

/// The same for a field whose value is an object or an array — the shape
/// [`opt`]'s field-readers cannot express, kept apart rather than folded so
/// neither caller has to index the map a second time (rule 4 forbids the
/// unchecked index that would take).
pub(crate) fn opt_val<T>(
    obj: &Map<String, Value>,
    key: &str,
    read: impl Fn(&Value) -> Result<T, String>,
) -> Result<Option<T>, String> {
    match obj.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => read(v).map(Some),
    }
}

/// An optional path field — [`opt`] over [`path_of`].
pub(crate) fn opt_path_of(obj: &Map<String, Value>, key: &str) -> Result<Option<PathBuf>, String> {
    opt(obj, key, path_of)
}

/// An optional string field — [`opt`] over [`str_of`]. `None` and `""` are
/// different facts (`--body ""` is an explicit empty body).
pub(crate) fn opt_str_of(obj: &Map<String, Value>, key: &str) -> Result<Option<String>, String> {
    opt(obj, key, str_of)
}

/// A required array field, each element read by `read` — the one strict list
/// reader, so a row that is not an object refuses by the list's own name
/// rather than silently becoming a shorter list.
pub(crate) fn list_of<T>(
    obj: &Map<String, Value>,
    key: &str,
    read: impl Fn(&Value) -> Result<T, String>,
) -> Result<Vec<T>, String> {
    obj.get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("missing or non-array field {key:?}"))?
        .iter()
        .map(read)
        .collect()
}

/// A required array of strings.
pub(crate) fn strings_of(obj: &Map<String, Value>, key: &str) -> Result<Vec<String>, String> {
    list_of(obj, key, |v| {
        v.as_str()
            .map(str::to_owned)
            .ok_or_else(|| format!("field {key:?} holds a non-string"))
    })
}

/// A token field read against its enum's own table — the one strictness path
/// every word-shaped field takes, so an unknown token refuses **naming the
/// offending word** in one place rather than once per enum.
pub(crate) fn pick<T: Copy>(
    obj: &Map<String, Value>,
    key: &str,
    table: &[(&str, T)],
) -> Result<T, String> {
    let token = str_of(obj, key)?;
    table
        .iter()
        .find(|(word, _)| *word == token)
        .map(|(_, value)| *value)
        .ok_or_else(|| format!("field {key:?}: unknown token {token:?}"))
}

/// A tiny string-object builder for the optional-field arms.
pub(super) fn obj(pairs: &[(&str, &str)]) -> Map<String, Value> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), Value::String((*v).to_owned())))
        .collect()
}
