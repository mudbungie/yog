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
