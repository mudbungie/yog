//! **The `clients` tool** (REMOTE §5, bl-c907): the one client-facing surface
//! in the model's stable prefix, and the act that makes a tool host's tools
//! callable.
//!
//! REMOTE §5: *"The model's stable prefix carries exactly one client-facing
//! surface: a **client-management tool**. Its operations: `list` — the
//! workspace's registered clients and which are live, now; `get` — one client's
//! detail and the tools it advertises. Every reply is a dated observation
//! appended to context, free to go stale, never a prefix mutation."* `load` is
//! the third, and it is the act §5's next paragraph describes; `unload` is its
//! symmetric fourth ([`edit`], REMOTE §5.2 as amended by bl-3455).
//!
//! **Only three ops need the engine.** `list`, `get` and `load` all resolve
//! against the roster read at this instant, so each asks for it where it needs
//! it. `unload` resolves against this agent's own document and nothing else, so
//! it never deposits a gesture — the set can be subtracted from on a box whose
//! engine is down, which is the same reason §5.2 gives for declaring touching
//! nothing but disk.
//!
//! **One tool, and its subject is the roster — not a multiplexer.** Loaded
//! remote tools surface as individually named definitions of their own
//! ([`super::loaded`]), because litany's `docs/DESIGN_MCP_BRIDGE.md` §6 ruling
//! binds a host too: a generic `call {client, tool, arguments}` would collapse
//! the grant gate, the tool control and every future policy into one bit. This
//! tool's own subject is *which machines are here and what they offer*, which
//! is a question no per-tool name can answer.
//!
//! **Every answer is dated and appended, and every refusal is in band.** A
//! reply carries the instant it was observed because presence is true only at
//! that instant; an unknown client, an unadvertised tool and an unreachable
//! engine are all non-zero results the model reads and steps on, never a prefix
//! change and never a hang.

use serde_json::{Value, json};
use std::sync::atomic::AtomicBool;

use super::{Site, render};
use crate::registry::roster::ClientRow;

/// The two ops that WRITE the agent's set, split from this file at §12's
/// per-file budget on the seam the module doc already draws: this file is what
/// the model may say and where it is routed, that one is what changes the
/// declared surface.
mod edit;

/// The tool's name — one word in the stable prefix, on every request.
pub const NAME: &str = "clients";

/// What the model is told the tool is for.
pub const DESCRIPTION: &str = "Registered client machines of this workspace \
and the tools they advertise. op=list: every client and which are connected \
right now. op=get with client=<name>: one client's detail and its advertised \
tools. op=load with client=<name> and tools=[<name>,…]: make those tools \
callable, by their prefixed names, from the next step on. op=unload with \
client=<name>, and tools=[…] or no tools at all: stop declaring them — the \
whole of that client's loaded tools when tools is omitted.";

/// The tool's declared input schema.
pub fn schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "op": {"type": "string", "enum": ["list", "get", "load", "unload"],
                   "description": "which operation to perform"},
            "client": {"type": "string",
                       "description": "client identity; required for get, load and unload"},
            "tools": {"type": "array", "items": {"type": "string"},
                      "description": "advertised tool names; required for load, and \
                                      optional for unload where omitting it means \
                                      that client's whole loaded set"}
        },
        "required": ["op"]
    })
}

/// One invocation of the tool, as the model spelled it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    /// The workspace's registered clients and which are live now.
    List,
    /// One client's detail and its advertised set.
    Get(String),
    /// Make named advertised tools callable from the next step on.
    Load(String, Vec<String>),
    /// Stop declaring named loaded tools from the next assembly on. `None` is
    /// the client's whole loaded set — the ordinary case, because an agent that
    /// has finished with a machine has finished with all of it, and spelling
    /// out what it loaded turns done-here into a recall exercise.
    Unload(String, Option<Vec<String>>),
}

/// Read an invocation, or the sentence that says why it is not one. Strict, and
/// naming the offending field: the model is the caller, and a decline it can
/// read is a decline it can correct on the next turn.
pub fn parse(input: &Value) -> Result<Op, String> {
    let o = input.as_object().ok_or("input is not a JSON object")?;
    let op = o
        .get("op")
        .and_then(Value::as_str)
        .ok_or("missing string field \"op\"")?;
    match op {
        "list" => Ok(Op::List),
        "get" => Ok(Op::Get(client_of(input)?)),
        "load" => Ok(Op::Load(client_of(input)?, tools_of(input)?)),
        "unload" => Ok(Op::Unload(client_of(input)?, tools_opt(input)?)),
        other => Err(format!(
            "unknown op {other:?}; expected \"list\", \"get\", \"load\" or \"unload\""
        )),
    }
}

/// The `client` field of a `get`/`load`.
fn client_of(input: &Value) -> Result<String, String> {
    input
        .get("client")
        .and_then(Value::as_str)
        .filter(|c| !c.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| "missing non-empty string field \"client\"".to_owned())
}

/// The `tools` field of a `load` — a non-empty array of strings, because a
/// load of nothing is an act with no effect and saying so is cheaper than
/// pretending it worked.
fn tools_of(input: &Value) -> Result<Vec<String>, String> {
    let rows = input
        .get("tools")
        .and_then(Value::as_array)
        .ok_or("missing array field \"tools\"")?;
    let names: Vec<String> = rows
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect();
    if names.len() != rows.len() || names.is_empty() {
        return Err("field \"tools\" must be a non-empty array of names".to_owned());
    }
    Ok(names)
}

/// The `tools` field of an `unload`, where it is optional — absent means the
/// client's whole loaded set. **Absent and empty are not the same spelling**:
/// `[]` is still an act with no effect and still declines, so a model that
/// meant *all of them* and typed the empty array is told rather than answered
/// with a no-op it will read as success.
fn tools_opt(input: &Value) -> Result<Option<Vec<String>>, String> {
    match input.get("tools") {
        None | Some(Value::Null) => Ok(None),
        Some(_) => tools_of(input).map(Some),
    }
}

/// Answer one invocation. Each arm asks for exactly what it needs: the three
/// ops whose subject is the roster read it from the engine at this moment
/// ([`super::ask::roster`]), and the one whose subject is this agent's own
/// document does not.
pub fn answer(site: &Site, input: &Value, stop: &AtomicBool) -> Result<String, String> {
    let op = parse(input)?;
    let observed = site.observed();
    match op {
        Op::List => Ok(render::list(
            &site.workspace,
            &observed,
            &asked(site, stop)?,
        )),
        Op::Get(client) => Ok(render::get(
            &site.workspace,
            &observed,
            &row_of(&asked(site, stop)?, &client)?,
        )),
        Op::Load(client, names) => {
            edit::load(site, &asked(site, stop)?, &client, &names, &observed)
        }
        Op::Unload(client, names) => edit::unload(site, &client, names.as_deref(), &observed),
    }
}

/// The workspace's roster, as the engine answers it right now.
fn asked(site: &Site, stop: &AtomicBool) -> Result<Vec<ClientRow>, String> {
    super::ask::roster(&site.state_root, &site.workspace, site.budget, stop)
}

/// One registered client of this workspace, or the refusal naming it. An
/// unregistered identity is **absent**, not forbidden (REMOTE §4): the sentence
/// says the workspace has no such client, which is what a name nobody ever
/// seated earns too.
fn row_of(rows: &[ClientRow], client: &str) -> Result<ClientRow, String> {
    rows.iter()
        .find(|r| r.client == client)
        .cloned()
        .ok_or_else(|| {
            format!(
                "no client {client:?} is registered in this workspace; \
                 op=list shows the {} there are",
                rows.len()
            )
        })
}

#[cfg(test)]
mod tests;
