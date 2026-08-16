//! The two **depositing** envelopes (§8.2, bl-a33d): `{"op":"message", …}` and
//! `{"op":"interrupt", …}`.
//!
//! Its own module on the seam every other family here is cut on, and for
//! [`super::ball`]'s reasoning — three identical fields, and the only
//! difference between the two gestures is what the engine does once the deposit
//! has landed. Neither carries a `children` flag, because a deposit's subject is
//! the conversation being talked to (`stop children` is the subtree's verb), and
//! the two op words are named once here so the directions cannot drift.

use serde_json::{Map, Value, json};

use crate::boundary::Action;

use super::str_of;

/// The plain §8.2 send.
pub(super) const MESSAGE: &str = "message";
/// Send-and-interrupt: the same deposit with a stop ahead of it.
pub(super) const INTERRUPT: &str = "interrupt";

/// Encode either deposit — the op word is the whole difference.
pub(super) fn deposit(op: &str, workspace: &str, agent: &str, content: &str) -> Value {
    json!({ "op": op, "workspace": workspace, "agent": agent, "content": content })
}

/// [`deposit`] read back, strictly: the same three fields either way, and the
/// `op` says which gesture they make.
pub(super) fn deposited(op: &str, o: &Map<String, Value>) -> Result<Action, String> {
    let workspace = str_of(o, "workspace")?;
    let agent = str_of(o, "agent")?;
    let content = str_of(o, "content")?;
    Ok(match op {
        INTERRUPT => Action::Interrupt {
            workspace,
            agent,
            content,
        },
        _ => Action::Message {
            workspace,
            agent,
            content,
        },
    })
}
