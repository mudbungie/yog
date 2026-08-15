//! **The advertised tool set** (REMOTE §5, bl-4e08): what a tool host presents
//! when it connects, and where the engine keeps it.
//!
//! REMOTE §5: *"A tool host presents its tool set — name, description, input
//! schema — when it connects; the engine writes it into the client's
//! registration (world, file) when it differs from what is stored."* The
//! spelling of that file is decided here and recorded in REMOTE §5:
//!
//! ```text
//! <yog-state-root>/clients/<client>/tools.json   the advertised set, one array
//! ```
//!
//! **One document per client, not one per registration.** A tool set is a fact
//! about a *machine* — what that laptop can do — and REMOTE §2 already makes
//! the registration listing the fact about which workspaces see it. Writing the
//! same array under every registration would be one fact stored N times, which
//! is the drift the single-source rule exists to prevent; the roster joins the
//! two by reading the listing it already has.
//!
//! **An element is three facts and nothing more** (bl-4e08): `name`, a single
//! path component so the later load act has an unambiguous handle;
//! `description`, one string; and `input_schema`, the JSON Schema **verbatim**
//! — yog neither validates it nor rewrites it, because it is the tool host's
//! statement to a model and any narrowing here would be yog inventing a
//! contract it does not own.
//!
//! **Names collide inside a client's set and not across clients.** Two
//! `Bash`es in one presentation is a set that cannot be addressed and it is
//! declined, loudly, naming the token; two machines both offering `Bash` is the
//! ordinary case, and disambiguating them belongs to the act that loads one.

use std::io;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::Client;
use crate::boundary::codec::fields::str_of;

/// One client's advertised set, under its own registry directory.
pub const TOOLS: &str = "tools.json";

/// One advertised tool (REMOTE §5): the three facts, and the JSON Schema
/// carried as the value it arrived as.
///
/// **[`Eq`] is written rather than derived**, because [`Value`] is not `Eq` —
/// it holds `f64`, whose `NaN` is the one value equality is not reflexive over.
/// A schema that came through a JSON decoder cannot hold one: the grammar has
/// no `NaN` literal and `serde_json` refuses to emit one. So equality here is
/// reflexive by construction, and saying so is what lets the whole gesture
/// surface keep the `Eq` its codec round-trip is asserted with.
#[derive(Debug, Clone, PartialEq)]
pub struct Tool {
    /// The tool's name, a single path component.
    pub name: String,
    /// What it does, in the tool host's own words.
    pub description: String,
    /// Its JSON Schema, verbatim.
    pub input_schema: Value,
}

impl Eq for Tool {}

/// This client's advertised set, on disk.
pub fn path(state_root: &Path, client: &Client) -> PathBuf {
    super::dir(state_root, client).join(TOOLS)
}

/// The set as JSON — the **one** spelling, spent by the boundary codec and by
/// the file alike (§8.5's single-source discipline: a stored set and a
/// presented one that were spelled twice would drift within a week).
pub fn encode(tools: &[Tool]) -> Value {
    Value::Array(
        tools
            .iter()
            .map(|t| {
                json!({ "name": t.name, "description": t.description,
                        "input_schema": t.input_schema })
            })
            .collect(),
    )
}

/// Read a set back, strictly — a missing field or a mistyped one refuses with
/// the offending key, exactly as the gesture codec's decode does. An
/// advertisement is an instruction, not an observation.
pub fn decode(v: &Value) -> Result<Vec<Tool>, String> {
    v.as_array()
        .ok_or_else(|| "tools: not an array".to_owned())?
        .iter()
        .map(|row| {
            let o = row.as_object().ok_or("tool: not a JSON object")?;
            Ok(Tool {
                name: str_of(o, "name")?,
                description: str_of(o, "description")?,
                input_schema: o
                    .get("input_schema")
                    .cloned()
                    .ok_or("tool: missing field \"input_schema\"")?,
            })
        })
        .collect()
}

/// What `client` advertises now. A client that has advertised nothing, whose
/// document cannot be read, or whose document is not a set reads as the **empty
/// set** — the same posture a fresh registration has, so no reader carries a
/// "never advertised" case beside "advertised nothing".
pub fn read(state_root: &Path, client: &Client) -> Vec<Tool> {
    let Ok(text) = std::fs::read_to_string(path(state_root, client)) else {
        return Vec::new();
    };
    serde_json::from_str(&text)
        .ok()
        .and_then(|v: Value| decode(&v).ok())
        .unwrap_or_default()
}

/// Refuse a set that cannot be addressed (REMOTE §5): a name that is not a
/// single path component, or two elements wearing one name. Both name the
/// offending token — a decline an operator can act on.
pub fn validate(tools: &[Tool]) -> Result<(), String> {
    let mut seen = std::collections::BTreeSet::new();
    for tool in tools {
        if !crate::naming::is_component(&tool.name) {
            return Err(format!("advertise: unusable tool name {:?}", tool.name));
        }
        if !seen.insert(tool.name.clone()) {
            return Err(format!("advertise: duplicate tool name {:?}", tool.name));
        }
    }
    Ok(())
}

/// Store `client`'s set, **only when it differs from what is stored** (REMOTE
/// §5). Answers whether it wrote: a re-presentation of an unchanged set is the
/// ordinary case on every reconnect, and rewriting the file each time would
/// make a mtime watcher see a change on every network blip — the very
/// connectivity-rate churn presence is RAM to avoid.
pub fn store(state_root: &Path, client: &Client, tools: &[Tool]) -> io::Result<bool> {
    if read(state_root, client) == tools {
        return Ok(false);
    }
    let file = path(state_root, client);
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(file, encode(tools).to_string())?;
    Ok(true)
}

#[cfg(test)]
mod tests;
