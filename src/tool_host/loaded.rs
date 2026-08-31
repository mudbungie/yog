//! **The loaded set** (REMOTE §5, bl-c907): which of a tool host's advertised
//! tools an agent has made callable, and the file that survives the step.
//!
//! REMOTE §5: *"Loading is the agent's own point-in-time act. From a `get`, the
//! agent loads a client's tools; loaded definitions are callable from that turn
//! on … the invariant is that nothing but an explicit load ever changes the
//! tool surface."*
//!
//! ```text
//! <yog-state-root>/loaded/<workspace>/<agent>.json
//! ```
//!
//! **It is durable because a driver is not.** Each step is a fresh process
//! (§2.11's exec baton), so a set held in RAM would be unloaded by the next
//! hop — the load act would last exactly one turn, which is not what "callable
//! from that turn on" says. It sits under yog's own state root rather than in
//! the workspace, because the workspace is the conversation's git repository
//! and a yog document does not belong in an agent's worktree.
//!
//! **The definition is frozen at the load act, not re-read at assembly.** The
//! file carries the whole advertised element — name, description, JSON Schema —
//! as it stood when the agent loaded it, so [`crate::tool_host::Injection::tools`]
//! is a pure local file read that needs no engine and cannot fail. That is the
//! REMOTE §5 rule (bl-bc7c) rather than a shortcut: *"definitions frozen in the
//! prefix, presence answered at invocation"*. A prefix that changed when a
//! client reconnected would put a connectivity-rate fact inside the model's
//! cached context, which is the whole defect §5 was amended to remove; and the
//! staleness that freezing admits is corrected where §5 already corrects it —
//! *"a client refuses a tool it no longer carries"*, in band, at the call.
//!
//! **The key is the agent, and there is no inheritance.** The set belongs to
//! the agent that loaded it, so a fresh conversation — and a freshly dispatched
//! subagent — starts clean and loads what it needs through the same `clients`
//! tool every agent always has.
//!
//! **Two writers, and they are symmetric** (REMOTE §5.2, bl-3455). [`add`] is
//! the load act's, [`remove`] the unload act's, and neither resolves a name:
//! each takes entries the act already resolved — load's against what the client
//! advertises right now, unload's against what this document actually holds —
//! so the whole-or-not-at-all rule lives once, in the act, and this module only
//! ever seals a set it was handed.

use std::io;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::boundary::codec::fields::str_of;
use crate::registry::tools::{self, Tool};

/// The loaded-set root's leaf under yog's state root.
pub const LOADED: &str = "loaded";

/// One loaded remote tool: the client that advertises it, and the definition
/// frozen as it read at the load act.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    /// The tool host's identity (REMOTE §2).
    pub client: String,
    /// Its advertised element, verbatim.
    pub tool: Tool,
}

/// [`Eq`] for [`Tool`]'s own reason: a schema that came through a JSON decoder
/// cannot hold a `NaN`, so equality here is reflexive by construction.
impl Eq for Entry {}

impl Entry {
    /// The name the model spells, and the name [`route`](crate::tool_host)
    /// keys on: the client's identity, an underscore, the advertised name.
    ///
    /// **Prefixed always, never only when ambiguous.** REMOTE §5.1 leaves the
    /// cross-client collision — two laptops both advertising `Bash` — to *"the
    /// act that loads one"*, and a rule that prefixes conditionally would make
    /// a tool's own name depend on what some other machine advertises, so the
    /// prefix a model learned would change under it. One rule, no case.
    pub fn presented(&self) -> String {
        format!("{}_{}", self.client, self.tool.name)
    }
}

/// True iff `name` is a name a provider will accept for a tool: ASCII letters,
/// digits, `_` and `-`, one to sixty-four of them. The advertised half is
/// already a path component (§5.1) and a client identity already is one, but
/// neither rules out the characters a tool block refuses — so the composed
/// name is checked once, here, and a load that cannot produce a callable name
/// declines naming it.
pub fn callable(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// This agent's document. A workspace or agent that is not a plain path
/// component has no document — the same emptiness a fresh agent reads, rather
/// than a name that addresses the filesystem.
pub fn path(state_root: &Path, workspace: &str, agent: &str) -> Option<PathBuf> {
    if !crate::naming::is_component(workspace) || !crate::naming::is_component(agent) {
        return None;
    }
    Some(
        state_root
            .join(LOADED)
            .join(workspace)
            .join(format!("{agent}.json")),
    )
}

/// The set as JSON — one array, each element the advertised three facts with
/// the client beside them. The tool half is spelled by
/// [`tools::one`](crate::registry::tools::one), the same encoder the
/// advertisement and the boundary codec spend, so a stored definition and a
/// presented one cannot drift.
pub fn encode(entries: &[Entry]) -> Value {
    Value::Array(
        entries
            .iter()
            .map(|e| json!({ "client": e.client, "tool": tools::one(&e.tool) }))
            .collect(),
    )
}

/// Read a set back strictly, naming the offending key — the advertisement's
/// own decode discipline, applied to yog's own document.
pub fn decode(v: &Value) -> Result<Vec<Entry>, String> {
    v.as_array()
        .ok_or_else(|| "loaded: not an array".to_owned())?
        .iter()
        .map(|row| {
            let o = row.as_object().ok_or("loaded: not a JSON object")?;
            Ok(Entry {
                client: str_of(o, "client")?,
                tool: tools::of_one(o.get("tool").ok_or("loaded: missing field \"tool\"")?)?,
            })
        })
        .collect()
}

/// What this agent has loaded. A document that is absent, unreadable or
/// undecodable reads as the **empty set** — which is also what every agent
/// reads before its first load, so no reader carries two cases.
pub fn read(state_root: &Path, workspace: &str, agent: &str) -> Vec<Entry> {
    let Some(file) = path(state_root, workspace, agent) else {
        return Vec::new();
    };
    let Ok(text) = std::fs::read_to_string(file) else {
        return Vec::new();
    };
    serde_json::from_str(&text)
        .ok()
        .and_then(|v: Value| decode(&v).ok())
        .unwrap_or_default()
}

/// Add `entries` to this agent's set and answer the whole of it. Union by
/// presented name, later wins: re-loading a tool the client has re-advertised
/// refreshes the frozen definition, which is the only way a definition ever
/// changes in place.
pub fn add(
    state_root: &Path,
    workspace: &str,
    agent: &str,
    entries: &[Entry],
) -> io::Result<Vec<Entry>> {
    let mut kept = without(state_root, workspace, agent, entries);
    kept.extend(entries.iter().cloned());
    seal(state_root, workspace, agent, kept)
}

/// Drop `gone` from this agent's set and answer what is left (REMOTE §5.2,
/// bl-3455). The caller resolved those entries against this very document, so
/// a name it does not hold refused the act before this was reached — which is
/// why there is no miss to report here and why the last unload leaves an empty
/// array rather than a special case: [`read`] already answers an empty set for
/// a document that is absent, unreadable or empty, so a set emptied and a set
/// never written read alike.
pub fn remove(
    state_root: &Path,
    workspace: &str,
    agent: &str,
    gone: &[Entry],
) -> io::Result<Vec<Entry>> {
    let kept = without(state_root, workspace, agent, gone);
    seal(state_root, workspace, agent, kept)
}

/// This agent's set with every entry `named` presents dropped — the half both
/// writers share, because a load replaces by presented name exactly as an
/// unload deletes by it.
fn without(state_root: &Path, workspace: &str, agent: &str, named: &[Entry]) -> Vec<Entry> {
    read(state_root, workspace, agent)
        .into_iter()
        .filter(|old| !named.iter().any(|one| one.presented() == old.presented()))
        .collect()
}

/// Write `kept` as this agent's whole set, sorted, and answer it back.
fn seal(
    state_root: &Path,
    workspace: &str,
    agent: &str,
    mut kept: Vec<Entry>,
) -> io::Result<Vec<Entry>> {
    let file = path(state_root, workspace, agent)
        .ok_or_else(|| io::Error::other(format!("unusable address {workspace:?}/{agent:?}")))?;
    kept.sort_by_key(Entry::presented);
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(file, encode(&kept).to_string())?;
    Ok(kept)
}

#[cfg(test)]
mod tests;
