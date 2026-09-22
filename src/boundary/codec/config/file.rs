//! The §9 config family's **destination**: the object a gesture carries in
//! `target` — both directions of [`ConfigFile`], the §9.3 [`EditOrigin`]
//! flattened onto it, and the one reader that names the place a parameter was
//! looked for. Split off [`super`] at §12's cap (bl-edfc), on the seam the
//! nesting itself draws: every other op in the family carries its parameters
//! beside `op`, and this one carries them a level down, read by the write
//! (`decode_apply`) and by the §8.5 read (`codec::query`) through this single
//! reader so the two cannot disagree about a token.

use crate::boundary::codec::fields::str_of;
use crate::boundary::config::ConfigFile;
use crate::config_edit::branch::edit::EditOrigin;
use serde_json::{Map, Value, json};

/// Encode a config destination. `file` is the discriminant; each destination
/// carries exactly its own parameters.
pub(crate) fn encode_file(file: &ConfigFile) -> Value {
    match file {
        ConfigFile::Brazen { workspace } => {
            json!({ "file": "brazen", "workspace": workspace })
        }
        ConfigFile::LitanyModels => json!({ "file": "litany-models" }),
        ConfigFile::LitanyWorkflow { name } => {
            json!({ "file": "litany-workflow", "name": name })
        }
        ConfigFile::Cadence => json!({ "file": "cadence" }),
        ConfigFile::Branch {
            workspace,
            lineage,
            origin,
            path,
        } => {
            let mut map = Map::new();
            map.insert("file".to_owned(), json!("branch"));
            map.insert("workspace".to_owned(), Value::String(workspace.clone()));
            map.insert("lineage".to_owned(), json!(lineage));
            map.insert("path".to_owned(), json!(path));
            for (k, v) in origin_fields(origin) {
                map.insert(k, v);
            }
            Value::Object(map)
        }
    }
}

/// The tail every destination refusal carries — the fact the field name alone
/// cannot state.
const PLACE: &str = "the config family carries its parameters INSIDE target";

/// A destination parameter, read **naming the object it was looked in**
/// (bl-edfc). [`str_of`]'s own sentence names a key and cannot name the place;
/// everywhere else in the codec that is enough, because a field sits beside
/// `op` where the writer put it. Here it is a defect: an operator who writes
/// `{"op":"config","workspace":"ops","target":{"file":"brazen"}}` HAS stated a
/// workspace, as a string, at the top level, and `missing or non-string field
/// "workspace"` reads as a flat contradiction of the envelope in front of
/// them. One wrapper and not a sentence per field: the place is the same fact
/// for all nine parameters the five destinations take, and nine hand-written
/// sentences drift. The two refusals beside it — `config: missing target` and
/// `config: target is not an object` — stay as they were written; they already
/// name the place, because the place is all they are about.
fn field_of(obj: &Map<String, Value>, file: &str, key: &str) -> Result<String, String> {
    str_of(obj, key).map_err(|why| format!("config: target for file {file:?}: {why} — {PLACE}"))
}

/// The destination a §9 gesture names — read too, by [`super::super::query`]'s
/// `ReadConfig` decode (bl-0164): a read and a write name the place through
/// this one reader, so they cannot disagree about a token.
pub(crate) fn decode_file(v: &Value) -> Result<ConfigFile, String> {
    let obj = v.as_object().ok_or("config: target is not an object")?;
    let file = str_of(obj, "file")?;
    match file.as_str() {
        "brazen" => Ok(ConfigFile::Brazen {
            workspace: field_of(obj, &file, "workspace")?,
        }),
        "litany-models" => Ok(ConfigFile::LitanyModels),
        "litany-workflow" => Ok(ConfigFile::LitanyWorkflow {
            name: field_of(obj, &file, "name")?,
        }),
        "cadence" => Ok(ConfigFile::Cadence),
        "branch" => Ok(ConfigFile::Branch {
            workspace: field_of(obj, &file, "workspace")?,
            lineage: field_of(obj, &file, "lineage")?,
            origin: decode_origin(obj, &file)?,
            path: field_of(obj, &file, "path")?,
        }),
        other => Err(format!("config: unknown target file {other:?}")),
    }
}

/// The §9.3 lineage mode, flattened onto its destination: `origin` names it and
/// `source` accompanies exactly the fork.
fn origin_fields(origin: &EditOrigin) -> Vec<(String, Value)> {
    match origin {
        EditOrigin::Advance => vec![("origin".to_owned(), json!("advance"))],
        EditOrigin::Fork { source } => vec![
            ("origin".to_owned(), json!("fork")),
            ("source".to_owned(), json!(source)),
        ],
        EditOrigin::Orphan => vec![("origin".to_owned(), json!("orphan"))],
    }
}

fn decode_origin(obj: &Map<String, Value>, file: &str) -> Result<EditOrigin, String> {
    match field_of(obj, file, "origin")?.as_str() {
        "advance" => Ok(EditOrigin::Advance),
        "fork" => Ok(EditOrigin::Fork {
            source: field_of(obj, file, "source")?,
        }),
        "orphan" => Ok(EditOrigin::Orphan),
        other => Err(format!("config: unknown origin {other:?}")),
    }
}

#[cfg(test)]
mod tests;
