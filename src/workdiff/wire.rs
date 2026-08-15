//! The work-diff's JSON shape (§8.5) — the headless serialization of what the
//! Work tab paints, so the two seats answer with one derivation and differ
//! only in how they say it.
//!
//! It lives beside the type rather than in [`reply`](crate::boundary::reply)
//! because the shape of these rows *is* this module's vocabulary; the reply
//! roster still holds the one line that names this encoder, so there remains
//! exactly one place to learn which reply encodes how.

use serde_json::{Map, Value, json};

use super::{Attempt, Change, Churn, FileChurn};
use crate::files_view::Preview;

/// The `work-diff` reply body: one row per attempt, plus the asked-for file's
/// patch when one was named.
pub(crate) fn reply(attempts: &[Attempt], patch: Option<&Preview>) -> Value {
    let mut map = Map::new();
    map.insert("ok".to_owned(), json!(true));
    map.insert("kind".to_owned(), json!("work-diff"));
    map.insert(
        "rows".to_owned(),
        Value::Array(attempts.iter().map(attempt_row).collect()),
    );
    if let Some(patch) = patch {
        map.insert(
            "patch".to_owned(),
            crate::files_view::wire::preview_value(patch),
        );
    }
    Value::Object(map)
}

/// One attempt: its identity, its `state` token, and whatever that state can
/// say. The tokens are the [`Change`] arms — an unreadable project and an
/// absent ref stay distinguishable on the wire exactly as they do on screen.
fn attempt_row(attempt: &Attempt) -> Value {
    let mut map = Map::new();
    map.insert("project".to_owned(), json!(attempt.project));
    map.insert("ball_id".to_owned(), json!(attempt.ball_id));
    match &attempt.change {
        Change::Unreadable => {
            map.insert("state".to_owned(), json!("unreadable"));
        }
        Change::Absent {
            target,
            source,
            missing,
        } => {
            map.insert("state".to_owned(), json!("absent"));
            map.insert("target".to_owned(), json!(target));
            map.insert("source".to_owned(), json!(source));
            map.insert("missing".to_owned(), json!(missing));
        }
        Change::Diff {
            target,
            source,
            target_oid,
            source_oid,
            files,
            truncated,
        } => {
            map.insert("state".to_owned(), json!("diff"));
            map.insert("target".to_owned(), json!(target));
            map.insert("source".to_owned(), json!(source));
            map.insert("target_oid".to_owned(), json!(target_oid));
            map.insert("source_oid".to_owned(), json!(source_oid));
            map.insert(
                "files".to_owned(),
                Value::Array(files.iter().map(file_row).collect()),
            );
            map.insert("truncated".to_owned(), json!(truncated));
        }
    }
    Value::Object(map)
}

/// One changed file: its path and its churn, binary said as itself.
fn file_row(file: &FileChurn) -> Value {
    match &file.churn {
        Churn::Text { added, removed } => {
            json!({ "path": file.path, "added": added, "removed": removed })
        }
        Churn::Binary => json!({ "path": file.path, "binary": true }),
    }
}

/// The `work-diff` reply body's attempts read back (bl-7067). The patch is
/// the caller's, read through [`Preview`]'s own decoder — one wording for a
/// bounded file, in both directions.
pub(crate) fn attempts_of(obj: &serde_json::Map<String, Value>) -> Result<Vec<Attempt>, String> {
    use crate::boundary::codec::fields::list_of;
    list_of(obj, "rows", attempt_of)
}

fn attempt_of(v: &Value) -> Result<Attempt, String> {
    use crate::boundary::codec::fields::{bool_of, list_of, str_of, strings_of};
    let o = v.as_object().ok_or("attempt: not an object")?;
    let change = match str_of(o, "state")?.as_str() {
        "unreadable" => Change::Unreadable,
        "absent" => Change::Absent {
            target: str_of(o, "target")?,
            source: str_of(o, "source")?,
            missing: strings_of(o, "missing")?,
        },
        "diff" => Change::Diff {
            target: str_of(o, "target")?,
            source: str_of(o, "source")?,
            target_oid: str_of(o, "target_oid")?,
            source_oid: str_of(o, "source_oid")?,
            files: list_of(o, "files", file_of)?,
            truncated: bool_of(o, "truncated")?,
        },
        other => return Err(format!("attempt: unknown state {other:?}")),
    };
    Ok(Attempt {
        project: str_of(o, "project")?,
        ball_id: str_of(o, "ball_id")?,
        change,
    })
}

/// One changed file: binary says so, and says nothing else — which is why the
/// churn is read off the shape rather than a token.
fn file_of(v: &Value) -> Result<FileChurn, String> {
    use crate::boundary::codec::fields::{str_of, u64_of};
    let o = v.as_object().ok_or("file churn: not an object")?;
    let churn = match o.get("binary") {
        Some(_) => Churn::Binary,
        None => Churn::Text {
            added: u64_of(o, "added")?,
            removed: u64_of(o, "removed")?,
        },
    };
    Ok(FileChurn {
        path: str_of(o, "path")?,
        churn,
    })
}
