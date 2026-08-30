//! **The tool host's own document** (REMOTE §5.2, bl-024b): what this machine
//! can run, and — by dropping half of it — what it says it can run.
//!
//! ```json
//! [{"name": "Bash",
//!   "description": "run a command in a shell",
//!   "input_schema": {"type": "object",
//!                    "properties": {"command": {"type": "string"}},
//!                    "required": ["command"]},
//!   "command": ["/usr/local/libexec/yog-tools/bash-tool"],
//!   "cwd": "/srv/work"}]
//! ```
//!
//! **The advertisement is a projection of this file, not a second list.** The
//! first three keys *are* REMOTE §5.1's advertised element, verbatim, and
//! [`advertisement`] is the whole of the derivation: drop `command` and `cwd`.
//! One document, two readings — so what a host offers and what it can actually
//! run cannot drift, which is the entire reason the config is not a pair of
//! lists an operator has to keep in step.
//!
//! **`command` is an argv, spawned directly.** There is no shell and no
//! interpolation of the invocation's input into it: a shell would make the
//! declared schema advisory and turn an operator's config into a
//! command-injection surface for anything a model can type. The input reaches
//! the command exactly as litany's own tool contract delivers one (its ARCH
//! §3.3) — the JSON on stdin, bytes on stdout, the exit code the verdict.
//!
//! **It sits beside the wire material, not inside the world**
//! (`<yog-data-root>/tools.json`, the sibling of `wire/` and `world/`), for
//! `wire/`'s reason exactly: it describes *this machine*, it is written by the
//! operator, and nothing yog generates may sit where a reseed would take it.
//!
//! JSON rather than TOML for one reason (REMOTE §5.2): `input_schema` is JSON
//! Schema carried verbatim, and any other syntax would make the operator
//! transcribe it.

use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::boundary::codec::fields::{opt_str_of, strings_of};
use crate::registry::tools::{self, Tool};
use crate::xdg::Env;

/// The document's leaf under the yog data root.
pub const TOOLS: &str = "tools.json";

/// One tool this machine offers: the advertised half, and the local half that
/// is never presented to anyone.
#[derive(Debug, Clone, PartialEq)]
pub struct Local {
    /// The three facts a client presents (REMOTE §5.1).
    pub tool: Tool,
    /// The argv, spawned directly — never a shell line.
    pub command: Vec<String>,
    /// The working directory to run it in, when the operator named one.
    pub cwd: Option<PathBuf>,
}

impl Eq for Local {}

/// This machine's document.
pub fn path(world: &Env) -> PathBuf {
    world.yog_data_root().join(TOOLS)
}

/// Read it, or say why it is not one. An absent document is a **refusal**
/// rather than the empty set: a tool host with nothing to offer has nothing to
/// do, and starting one is an explicit act that deserves an explicit answer —
/// the same posture `yog seat` takes to absent wire material.
pub fn read(file: &Path) -> Result<Vec<Local>, String> {
    let text = std::fs::read_to_string(file).map_err(|e| {
        format!(
            "{}: {e} — this machine has no tool-host config",
            file.display()
        )
    })?;
    let doc: Value = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", file.display()))?;
    let rows = doc
        .as_array()
        .ok_or_else(|| format!("{}: not a JSON array", file.display()))?;
    let set: Vec<Local> = rows
        .iter()
        .map(one)
        .collect::<Result<_, String>>()
        .map_err(|e| format!("{}: {e}", file.display()))?;
    tools::validate(&advertisement(&set)).map_err(|e| format!("{}: {e}", file.display()))?;
    Ok(set)
}

/// One element: the advertised three read by the **same** decoder the wire and
/// the stored document spend, then the local two.
fn one(row: &Value) -> Result<Local, String> {
    let o = row.as_object().ok_or("tool: not a JSON object")?;
    let command = strings_of(o, "command")?;
    if command.is_empty() {
        return Err("tool: field \"command\" is an empty argv".to_owned());
    }
    Ok(Local {
        tool: tools::of_one(row)?,
        command,
        cwd: opt_str_of(o, "cwd")?.map(PathBuf::from),
    })
}

/// **The advertisement, derived** (REMOTE §5.2): the same document with the
/// local half dropped. The one derivation, so a host cannot offer what it
/// cannot run.
pub fn advertisement(set: &[Local]) -> Vec<Tool> {
    set.iter().map(|local| local.tool.clone()).collect()
}

/// Which element an invocation names, by position — an index rather than a
/// borrow, so the caller resolves it against the very list it passed in.
pub fn position(set: &[Local], name: &str) -> Option<usize> {
    set.iter().position(|local| local.tool.name == name)
}

#[cfg(test)]
mod tests;
