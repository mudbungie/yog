//! The §9.4 workflow mark's envelopes (REMOTE §9.24, bl-b680) — its own
//! family file, on the seam every sibling here is cut along.
//!
//! **Two ops for one variant**, the pin's precedent exactly (bl-b986): the op
//! token IS the direction, so nothing is defaulted and a clear can never read
//! as a set that lost its field. The set's `config` is **required**: a mark's
//! whole point is choosing among lineages, and a set that guessed one would be
//! the boundary inventing an operator's decision.

use serde_json::{Map, Value, json};

use super::{Action, str_of};

/// The set: `{"op": "workflow", "workspace", "agent", "config"}`.
pub(crate) const WORKFLOW: &str = "workflow";
/// The clear: `{"op": "clear-workflow", "workspace", "agent"}`.
pub(crate) const CLEAR: &str = "clear-workflow";

/// One mark gesture as its envelope: the set names its lineage, the clear
/// carries no `config` key at all.
pub(super) fn encode(workspace: &str, agent: &str, config: Option<&str>) -> Value {
    match config {
        Some(config) => json!({ "op": WORKFLOW, "workspace": workspace,
                                "agent": agent, "config": config }),
        None => json!({ "op": CLEAR, "workspace": workspace, "agent": agent }),
    }
}

/// The inverse. `op` is one of the two tokens already; the set reads its
/// lineage strictly, and the clear reads none.
pub(super) fn decode(op: &str, o: &Map<String, Value>) -> Result<Action, String> {
    Ok(Action::Workflow {
        workspace: str_of(o, "workspace")?,
        agent: str_of(o, "agent")?,
        config: if op == WORKFLOW {
            Some(str_of(o, "config")?)
        } else {
            None
        },
    })
}
