//! The Files tab's JSON shape (§8.5, bl-6233) — the headless serialization of
//! what the §11 Altitude-2 Files tab paints, so the two seats answer with one
//! derivation and differ only in how they say it.
//!
//! It lives beside the type rather than in [`reply`](crate::boundary::reply)
//! for the reason `workdiff::wire` gives: the shape of these rows *is* this
//! module's vocabulary. [`preview_value`] is here rather than there because
//! [`Preview`] is this module's type — the work diff's patch, the Files tab's
//! preview and any later reader of a bounded file all say it in one wording.

use serde_json::{Map, Value, json};

use super::{FileEntry, FilesView, Preview};

/// The `files` reply body: the listing, plus the asked-for file's preview when
/// one was named and the listing carried it.
pub(crate) fn reply(view: &FilesView, preview: Option<&Preview>) -> Value {
    let mut map = Map::new();
    map.insert("ok".to_owned(), json!(true));
    map.insert("kind".to_owned(), json!("files"));
    match view {
        // The disposable worktree's absence is a fact, not an empty listing
        // (§3.5): `rows` is present exactly when there is a worktree to list,
        // so a reader never has to tell "torn down" from "nothing in it".
        FilesView::AbsentWorktree => {
            map.insert("worktree".to_owned(), json!(false));
        }
        FilesView::Present { entries, truncated } => {
            map.insert("worktree".to_owned(), json!(true));
            map.insert(
                "rows".to_owned(),
                Value::Array(entries.iter().map(entry_row).collect()),
            );
            map.insert("truncated".to_owned(), json!(truncated));
        }
    }
    if let Some(preview) = preview {
        map.insert("preview".to_owned(), preview_value(preview));
    }
    Value::Object(map)
}

/// One walked entry: its identity, its size, and whether it is a directory.
fn entry_row(entry: &FileEntry) -> Value {
    json!({ "path": entry.rel_path, "size": entry.size, "dir": entry.is_dir })
}

/// A bounded preview as data — the same three classes every seat renders.
pub(crate) fn preview_value(preview: &Preview) -> Value {
    match preview {
        Preview::Text(text) => json!({ "kind": "text", "text": text }),
        Preview::Truncated { text, size } => {
            json!({ "kind": "truncated", "text": text, "size": size })
        }
        Preview::Binary { size } => json!({ "kind": "binary", "size": size }),
    }
}

/// The `files` reply body read back (bl-7067). `worktree` is the
/// discriminant the encoder chose so a reader never has to tell a torn-down
/// worktree from an empty one; it is read as exactly that.
pub(crate) fn view_of(obj: &serde_json::Map<String, Value>) -> Result<FilesView, String> {
    use crate::boundary::codec::fields::{bool_of, list_of};
    if !bool_of(obj, "worktree")? {
        return Ok(FilesView::AbsentWorktree);
    }
    Ok(FilesView::Present {
        entries: list_of(obj, "rows", entry_of)?,
        truncated: bool_of(obj, "truncated")?,
    })
}

fn entry_of(v: &Value) -> Result<FileEntry, String> {
    use crate::boundary::codec::fields::{bool_of, str_of, u64_of};
    let o = v.as_object().ok_or("file row: not an object")?;
    Ok(FileEntry {
        rel_path: str_of(o, "path")?,
        size: u64_of(o, "size")?,
        is_dir: bool_of(o, "dir")?,
    })
}

/// A bounded preview read back — the same three classes, in the same words.
pub(crate) fn preview_of(v: &Value) -> Result<Preview, String> {
    use crate::boundary::codec::fields::{str_of, u64_of};
    let o = v.as_object().ok_or("preview: not an object")?;
    match str_of(o, "kind")?.as_str() {
        "text" => Ok(Preview::Text(str_of(o, "text")?)),
        "truncated" => Ok(Preview::Truncated {
            text: str_of(o, "text")?,
            size: u64_of(o, "size")?,
        }),
        "binary" => Ok(Preview::Binary {
            size: u64_of(o, "size")?,
        }),
        other => Err(format!("preview: unknown kind {other:?}")),
    }
}
