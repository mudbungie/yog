//! The §9 config family's halves of the [`codec`](super) (bl-3f46, bl-0164):
//! the [`ConfigFile`] destination, the §16.3 [`Mode`] and the §9.3
//! [`EditOrigin`]. Split from the top-level codec per §12's line budget; every
//! encoder here is matched by a decoder and every variant round-trips (the
//! §8.5 parity tests).
//!
//! Strict, like the rest of the codec: an unknown destination token, a missing
//! parameter, a `fork` with no source each refuse by name. A config apply
//! rewrites a file that governs every model call in a workspace — a guessed
//! destination is the last thing it may do.

use crate::boundary::Action;
use crate::boundary::config::ConfigFile;
use crate::config_edit::branch::edit::EditOrigin;
use serde_json::{Map, Value, json};

use super::str_of;

/// Encode a config destination. `file` is the discriminant; each destination
/// carries exactly its own parameters.
pub(super) fn encode_file(file: &ConfigFile) -> Value {
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

/// Read one of the family's three ops, or `None` when the token is not one —
/// which is what keeps the unknown-op refusal in a single place upstream.
/// **Write-shaped only**: [`super::query`] tries a query reading first
/// (`config`/`marks` with no `text`/`mode` field is the §8.5 read, bl-0164)
/// and this is its fallback, so by the time either op reaches here it always
/// carries the field that makes it a write.
pub(super) fn decode_action(op: &str, o: &Map<String, Value>) -> Option<Result<Action, String>> {
    match op {
        "config" => Some(decode_apply(o)),
        "marks" => Some(decode_marks(o)),
        "model" => Some(decode_pick(o)),
        _ => None,
    }
}

fn decode_apply(o: &Map<String, Value>) -> Result<Action, String> {
    Ok(Action::ApplyConfig {
        file: decode_file(o.get("target").ok_or("config: missing target")?)?,
        text: str_of(o, "text")?,
    })
}

fn decode_marks(o: &Map<String, Value>) -> Result<Action, String> {
    let branch = str_of(o, "branch")?;
    if !crate::world::marks::lawful(&branch) {
        return Err(format!("marks: {}", crate::world::marks::REFUSAL));
    }
    Ok(Action::SetMarks {
        workspace: str_of(o, "workspace")?,
        branch,
    })
}

fn decode_pick(o: &Map<String, Value>) -> Result<Action, String> {
    Ok(Action::PickModel {
        workspace: str_of(o, "workspace")?,
        role: str_of(o, "role")?,
        provider: str_of(o, "provider")?,
        model: str_of(o, "model")?,
    })
}

/// The destination a §9 gesture names — read too, by [`super::query`]'s
/// `ReadConfig` decode (bl-0164): a read and a write name the place through
/// this one reader, so they cannot disagree about a token.
pub(super) fn decode_file(v: &Value) -> Result<ConfigFile, String> {
    let obj = v.as_object().ok_or("config: target is not an object")?;
    match str_of(obj, "file")?.as_str() {
        "brazen" => Ok(ConfigFile::Brazen {
            workspace: str_of(obj, "workspace")?,
        }),
        "litany-models" => Ok(ConfigFile::LitanyModels),
        "litany-workflow" => Ok(ConfigFile::LitanyWorkflow {
            name: str_of(obj, "name")?,
        }),
        "cadence" => Ok(ConfigFile::Cadence),
        "branch" => Ok(ConfigFile::Branch {
            workspace: str_of(obj, "workspace")?,
            lineage: str_of(obj, "lineage")?,
            origin: decode_origin(obj)?,
            path: str_of(obj, "path")?,
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

fn decode_origin(obj: &Map<String, Value>) -> Result<EditOrigin, String> {
    match str_of(obj, "origin")?.as_str() {
        "advance" => Ok(EditOrigin::Advance),
        "fork" => Ok(EditOrigin::Fork {
            source: str_of(obj, "source")?,
        }),
        "orphan" => Ok(EditOrigin::Orphan),
        other => Err(format!("config: unknown origin {other:?}")),
    }
}

#[cfg(test)]
pub(crate) mod tests;
