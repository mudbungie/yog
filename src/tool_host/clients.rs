//! **The `clients` tool** (REMOTE §5, bl-c907): the one client-facing surface
//! in the model's stable prefix, and the act that makes a tool host's tools
//! callable.
//!
//! REMOTE §5: *"The model's stable prefix carries exactly one client-facing
//! surface: a **client-management tool**. Its operations: `list` — the
//! workspace's registered clients and which are live, now; `get` — one client's
//! detail and the tools it advertises. Every reply is a dated observation
//! appended to context, free to go stale, never a prefix mutation."* `load` is
//! the third, and it is the act §5's next paragraph describes.
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

use super::loaded::{self, Entry};
use super::{Site, render};
use crate::registry::roster::ClientRow;

/// The tool's name — one word in the stable prefix, on every request.
pub const NAME: &str = "clients";

/// What the model is told the tool is for.
pub const DESCRIPTION: &str = "Registered client machines of this workspace \
and the tools they advertise. op=list: every client and which are connected \
right now. op=get with client=<name>: one client's detail and its advertised \
tools. op=load with client=<name> and tools=[<name>,…]: make those tools \
callable, by their prefixed names, from the next step on.";

/// The tool's declared input schema.
pub fn schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "op": {"type": "string", "enum": ["list", "get", "load"],
                   "description": "which operation to perform"},
            "client": {"type": "string",
                       "description": "client identity; required for get and load"},
            "tools": {"type": "array", "items": {"type": "string"},
                      "description": "advertised tool names to load; required for load"}
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
        other => Err(format!(
            "unknown op {other:?}; expected \"list\", \"get\" or \"load\""
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

/// Answer one invocation against the workspace's roster, read from the engine
/// at this moment ([`super::ask::roster`]).
pub fn answer(site: &Site, input: &Value, stop: &AtomicBool) -> Result<String, String> {
    let op = parse(input)?;
    let observed = site.observed();
    let rows = super::ask::roster(&site.state_root, &site.workspace, site.budget, stop)?;
    match op {
        Op::List => Ok(render::list(&site.workspace, &observed, &rows)),
        Op::Get(client) => Ok(render::get(
            &site.workspace,
            &observed,
            &row_of(&rows, &client)?,
        )),
        Op::Load(client, names) => load(site, &rows, &client, &names, &observed),
    }
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

/// The load act: resolve every named tool against what the client advertises
/// **right now**, freeze those definitions into the agent's durable set, and
/// report what became callable. Every name must resolve — a partial load would
/// leave the model believing it holds a tool it does not.
fn load(
    site: &Site,
    rows: &[ClientRow],
    client: &str,
    names: &[String],
    observed: &str,
) -> Result<String, String> {
    let row = row_of(rows, client)?;
    let mut entries = Vec::new();
    for name in names {
        let tool = row
            .tools
            .iter()
            .find(|t| &t.name == name)
            .ok_or_else(|| format!("client {client:?} advertises no tool {name:?}"))?;
        let entry = Entry {
            client: client.to_owned(),
            tool: tool.clone(),
        };
        let presented = entry.presented();
        // A presented name always carries the joining underscore, so it can
        // never be this tool's own bare name — the collision that would need a
        // case is unreachable by construction, and there is none.
        if !loaded::callable(&presented) {
            return Err(format!("{presented:?} is not a usable tool name"));
        }
        entries.push(entry);
    }
    let all = loaded::add(&site.state_root, &site.workspace, &site.agent, &entries)
        .map_err(|e| format!("recording the load: {e}"))?;
    Ok(render::load(observed, &entries, all.len()))
}

#[cfg(test)]
mod tests;
