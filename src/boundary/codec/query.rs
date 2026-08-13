//! The **query** half of the envelope codec (§8.5), cut from the action half
//! at §12's per-file budget on the family seam [`super::config`] already
//! established: [`super`] holds the action roster and the shared field
//! readers, this holds every populating read's spelling.
//!
//! Both directions stay exhaustive over [`Query`], so the §4.8 compile gate is
//! unchanged — a query variant added tomorrow does not build until it is
//! spelled here.
//!
//! **`config` and `marks` are shared tokens, not query-exclusive (bl-0164).**
//! Both ops answer either family — a `text`/`mode` field present is the
//! write, absent is the read — so [`read`] recognizes them only in their
//! fieldless shape and falls through (`Ok(None)`) otherwise, letting
//! [`super::config::decode_action`] answer the write. The line reads this
//! same discriminant off an empty tail, so a seat cannot spell one meaning
//! at the envelope and the other at the line.

use serde_json::{Map, Value, json};

use super::start::{encode_path, opt_field, opt_of};
use super::{obj, path_of, str_of, usize_of};
use crate::boundary::Query;

/// Encode one query to its envelope. Total over [`Query`].
pub(super) fn encode(query: &Query) -> Value {
    match query {
        Query::Workspaces => json!({ "op": "workspaces" }),
        Query::Conversations { workspace } => {
            json!({ "op": "conversations", "workspace": encode_path(workspace) })
        }
        Query::Balls => json!({ "op": "balls" }),
        Query::WorkDiff { workspace, file } => {
            let mut map = obj(&[("op", "work-diff")]);
            map.insert("workspace".to_owned(), encode_path(workspace));
            if let Some(file) = file {
                map.insert(
                    "file".to_owned(),
                    json!({ "ball": file.ball, "path": file.path }),
                );
            }
            Value::Object(map)
        }
        Query::Board => json!({ "op": "board" }),
        Query::Attention => json!({ "op": "attention" }),
        Query::Ops { max } => json!({ "op": "ops", "max": max }),
        Query::Search { text } => json!({ "op": "search", "text": text }),
        Query::Help { verb } => {
            let mut map = obj(&[("op", "help")]);
            opt_field(&mut map, "verb", verb.as_ref());
            Value::Object(map)
        }
        // The §9 config family's reads (§8.5, bl-0164) share their write's
        // own op: a `text`/`mode` field is what makes the envelope a write,
        // so a read is spelled by leaving it out, never a second op token.
        Query::ReadConfig { file } => {
            json!({ "op": "config", "target": super::config::encode_file(file) })
        }
        Query::Marks { workspace } => {
            json!({ "op": "marks", "workspace": encode_path(workspace) })
        }
        Query::Providers { workspace } => {
            json!({ "op": "providers", "workspace": encode_path(workspace) })
        }
    }
}

/// Decode `op` as a query, or `None` when it names none — the signal
/// [`super::decode`] chains on before it refuses an unknown op, exactly as it
/// chains on the config family's own reader. The two shapes are separated so
/// the reader below can `?` its field refusals: "not a query" and "a query
/// with a bad field" are different answers, and only the second is an error.
pub(super) fn decode(op: &str, o: &Map<String, Value>) -> Option<Result<Query, String>> {
    match read(op, o) {
        Ok(query) => query.map(Ok),
        Err(reason) => Some(Err(reason)),
    }
}

/// The query table itself: `Ok(None)` is "some other family's op".
fn read(op: &str, o: &Map<String, Value>) -> Result<Option<Query>, String> {
    Ok(Some(match op {
        "workspaces" => Query::Workspaces,
        "conversations" => Query::Conversations {
            workspace: path_of(o, "workspace")?,
        },
        "balls" => Query::Balls,
        "work-diff" => Query::WorkDiff {
            workspace: path_of(o, "workspace")?,
            file: work_file(o)?,
        },
        "board" => Query::Board,
        "attention" => Query::Attention,
        "ops" => Query::Ops {
            max: usize_of(o, "max")?,
        },
        "search" => Query::Search {
            text: str_of(o, "text")?,
        },
        // Strict here too: help *about* something must name a gesture, so the
        // answer is total and no seat renders an empty page.
        "help" => Query::Help {
            verb: match opt_of(o, "verb")? {
                Some(verb) if !crate::boundary::help::known(&verb) => {
                    return Err(format!("help: unknown verb {verb:?}"));
                }
                other => other,
            },
        },
        // Read-shaped only (bl-0164): present without the write's own field,
        // else `Ok(None)` falls through to `config::decode_action`'s write.
        "config" if !o.contains_key("text") => Query::ReadConfig {
            file: super::config::decode_file(o.get("target").ok_or("config: missing target")?)?,
        },
        "marks" if !o.contains_key("branch") => Query::Marks {
            workspace: path_of(o, "workspace")?,
        },
        "providers" => Query::Providers {
            workspace: path_of(o, "workspace")?,
        },
        _ => return Ok(None),
    }))
}

/// The optional `file` object of a work-diff query — both of its fields
/// required once it is present, because a patch read that guessed either half
/// would open the wrong file.
fn work_file(obj: &Map<String, Value>) -> Result<Option<crate::workdiff::WorkFile>, String> {
    let Some(value) = obj.get("file") else {
        return Ok(None);
    };
    let file = value.as_object().ok_or("file: not a JSON object")?;
    Ok(Some(crate::workdiff::WorkFile {
        ball: str_of(file, "ball")?,
        path: str_of(file, "path")?,
    }))
}
