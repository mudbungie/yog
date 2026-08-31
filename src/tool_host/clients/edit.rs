//! **The two acts that change the declared surface** (REMOTE §5.2; bl-c907,
//! bl-3455): `load`, and its symmetric `unload`.
//!
//! `list` and `get` observe. These two write [`the agent's durable
//! set`](super::super::loaded), so the next assembly declares a different
//! prefix — which is what makes their shared rule the important one:
//!
//! **Whole or not at all, in both directions.** A load resolves every named
//! tool against what the client advertises *right now*, and one miss refuses
//! the whole act; an unload resolves every named tool against what the
//! document *actually holds*, and one miss refuses the whole act. REMOTE §5.2
//! gives the reason for the first — *"a partial load leaves the model
//! believing it holds a tool it does not"* — and the mirror is the reason for
//! the second: a partial unload leaves the model believing it has dropped a
//! tool it still declares. The belief desyncs in one direction or the other,
//! and neither is acceptable.
//!
//! **Each resolves against a different authority, which is the whole of the
//! asymmetry between them.** Load's is the roster, so it needs the engine and
//! carries the observation's date. Unload's is a file on this box, so it needs
//! nothing and cannot fail for being offline — an agent finished with a machine
//! can say so even when the machine, or the engine, has gone.
//!
//! **Neither costs a prefix rebuild at a moment nobody chose.** The
//! subtraction lands in the document now and is spent at the next assembly,
//! which is exactly load's own settlement (bl-c907) read backwards. Scheduling
//! such an edit against a cache miss that was going to be paid anyway is a
//! different mechanism and a different ball (bl-b6f9); nothing here defers.

use super::{Site, render, row_of};
use crate::registry::roster::ClientRow;
use crate::tool_host::loaded::{self, Entry};

/// The load act ([`Op::Load`](super::Op::Load)): resolve every named tool against what the
/// client advertises **right now**, freeze those definitions into the agent's
/// durable set, and report what became callable.
pub fn load(
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

/// The unload act ([`Op::Unload`](super::Op::Unload)): resolve every named tool against what this
/// agent's document holds, drop them, and report what stopped being declared.
/// `names` absent is that client's whole loaded set.
pub fn unload(
    site: &Site,
    client: &str,
    names: Option<&[String]>,
    observed: &str,
) -> Result<String, String> {
    let held = loaded::read(&site.state_root, &site.workspace, &site.agent);
    let gone = resolve(&held, client, names)?;
    let kept = loaded::remove(&site.state_root, &site.workspace, &site.agent, &gone)
        .map_err(|e| format!("recording the unload: {e}"))?;
    Ok(render::unload(observed, &gone, kept.len()))
}

/// Which held entries an unload names, or the sentence saying why it names
/// none.
///
/// **The client is part of every answer**, because the document is keyed on the
/// presented name and a bare tool name is not one: two machines may both have
/// contributed a `Bash`, and dropping the wrong one would silently change which
/// machine a name reaches.
///
/// A client this conversation has loaded nothing from refuses rather than
/// answering an empty success — the wholesale form's version of the same rule,
/// and the only way a model learns that its recollection of what it loaded was
/// wrong.
fn resolve(held: &[Entry], client: &str, names: Option<&[String]>) -> Result<Vec<Entry>, String> {
    let mine: Vec<Entry> = held
        .iter()
        .filter(|e| e.client == client)
        .cloned()
        .collect();
    let Some(names) = names else {
        if mine.is_empty() {
            return Err(format!(
                "this conversation has no tool loaded from client {client:?}"
            ));
        }
        return Ok(mine);
    };
    names
        .iter()
        .map(|name| {
            mine.iter()
                .find(|e| &e.tool.name == name)
                .cloned()
                .ok_or_else(|| {
                    format!("this conversation has no tool {name:?} loaded from client {client:?}")
                })
        })
        .collect()
}
