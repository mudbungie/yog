//! The §11 inspector family's envelope spelling (§8.5, bl-6233) — split from
//! [`super`] at §12's per-file budget on the seam the family itself draws:
//! these six are the only queries addressed at a **conversation** rather than a
//! workspace, so the address they share is written once here and every other
//! query's spelling stays where it was.
//!
//! Both directions stay under the §4.8 compile gate: [`super::encode`] names
//! each variant by hand, and [`read`] is chained ahead of the sibling table so
//! an op it does not claim falls through unchanged.

use serde_json::{Map, Value, json};

use std::path::{Path, PathBuf};

use super::super::fields::opt_str_of;
use super::super::start::{encode_path, opt_field};
use super::super::{path_of, str_of};
use crate::boundary::Query;

/// The address every one of the six carries: the workspace, and the
/// conversation inside it.
pub(super) fn at(op: &str, workspace: &Path, agent: &str) -> Value {
    Value::Object(at_map(op, workspace, agent))
}

/// The same, still open for the two that carry one more key.
fn at_map(op: &str, workspace: &Path, agent: &str) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert("op".to_owned(), json!(op));
    map.insert("workspace".to_owned(), encode_path(workspace));
    map.insert("agent".to_owned(), json!(agent));
    map
}

/// One step's drill-in: the address, plus the sequence name that picks the step.
pub(super) fn step(workspace: &Path, agent: &str, seq: &str) -> Value {
    let mut map = at_map("step", workspace, agent);
    map.insert("seq".to_owned(), json!(seq));
    Value::Object(map)
}

/// The Files read: the address, plus the path when one file's bytes are asked
/// for. Absent is the listing — the [`WorkDiff`](Query::WorkDiff) shape.
pub(super) fn files(workspace: &Path, agent: &str, path: Option<&String>) -> Value {
    let mut map = at_map("files", workspace, agent);
    opt_field(&mut map, "path", path);
    Value::Object(map)
}

/// Decode one of the six, or `Ok(None)` when `op` names none of them — the
/// signal [`super::read`] chains on before its own table. Strict: the address
/// is required in full on every one of them, because a conversation read that
/// guessed either half would answer about a different chat entirely.
pub(super) fn read(op: &str, o: &Map<String, Value>) -> Result<Option<Query>, String> {
    Ok(Some(match op {
        "transcript" => {
            let (workspace, agent) = address(o)?;
            Query::Transcript { workspace, agent }
        }
        "steps" => {
            let (workspace, agent) = address(o)?;
            Query::Steps { workspace, agent }
        }
        "step" => {
            let (workspace, agent) = address(o)?;
            Query::Step {
                workspace,
                agent,
                seq: str_of(o, "seq")?,
            }
        }
        "files" => {
            let (workspace, agent) = address(o)?;
            Query::Files {
                workspace,
                agent,
                path: opt_str_of(o, "path")?,
            }
        }
        "rail" => {
            let (workspace, agent) = address(o)?;
            Query::Rail { workspace, agent }
        }
        "inbox" => {
            let (workspace, agent) = address(o)?;
            Query::Inbox { workspace, agent }
        }
        _ => return Ok(None),
    }))
}

/// The shared address reader — both halves required.
fn address(o: &Map<String, Value>) -> Result<(PathBuf, String), String> {
    Ok((path_of(o, "workspace")?, str_of(o, "agent")?))
}
