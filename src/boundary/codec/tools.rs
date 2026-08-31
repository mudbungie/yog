//! REMOTE §5's tool-host presentation, in the deposit envelope (bl-4e08) —
//! beside the other families' spellings for their reason exactly: a family's
//! grammar lives in its own file, and the action roster stays a roster.
//!
//! **The element spelling is not here.** It is
//! [`registry::tools`](crate::registry::tools), the same encoder the stored
//! document spends, because a presented set and a stored one written twice
//! would drift within a week (§8.5's single-source discipline).
//!
//! **No client field**, and that is the gesture (REMOTE §5): the identity a set
//! lands under is the intake's, and one on the wire would let a connection
//! write another client's set.

use serde_json::{Map, Value, json};

use super::fields::str_of;
use crate::boundary::Action;
use crate::registry::{mailbox, tools};

/// The op tokens, named once so each encoder's word and its decoder's arm are
/// one fact.
pub(super) const ADVERTISE: &str = "advertise";
pub(super) const INVOKE: &str = "invoke";
pub(super) const COMPLETE: &str = "complete";

/// One presentation as its envelope.
pub(super) fn encode(set: &[tools::Tool]) -> Value {
    json!({ "op": ADVERTISE, "tools": tools::encode(set) })
}

/// The routing leg's family as its envelopes (bl-024b): a call carries the
/// model's own `tool_use.input` verbatim, for the schema's reason (REMOTE
/// §5.1) — yog neither validates nor rewrites what a host declared it takes —
/// and a completion carries the capture in its ONE spelling
/// ([`mailbox::capture_value`]), the same bytes both replies that carry one
/// spend and the same the client-side executor writes.
pub(super) fn encode_route(verb: &mailbox::Verb) -> Value {
    match verb {
        mailbox::Verb::Invoke(call) => {
            let mut o = json!({ "op": INVOKE, "client": call.client,
                                "tool": call.tool, "input": call.input });
            if let (Some(cwd), Some(map)) = (&call.cwd, o.as_object_mut()) {
                map.insert("cwd".to_owned(), Value::String(cwd.clone()));
            }
            o
        }
        mailbox::Verb::Complete(done) => {
            json!({ "op": COMPLETE, "invocation": done.invocation,
                    "capture": mailbox::capture_value(&done.capture) })
        }
    }
}

/// Read one of the family back, by the token that named it. Every field is
/// required: a presentation with no set is not the empty set, a call with no
/// input is not a call with `{}`, and an envelope that failed to say something
/// is not a gesture.
pub(super) fn decode(op: &str, o: &Map<String, Value>) -> Result<Action, String> {
    match op {
        INVOKE => Ok(Action::Route(mailbox::Verb::Invoke(mailbox::Call {
            client: str_of(o, "client")?,
            tool: str_of(o, "tool")?,
            input: o.get("input").cloned().ok_or("invoke: missing input")?,
            cwd: mailbox::cwd_of(o).map_err(|e| format!("invoke: {e}"))?,
        }))),
        COMPLETE => Ok(Action::Route(mailbox::Verb::Complete(
            mailbox::Completion {
                invocation: str_of(o, "invocation")?,
                capture: mailbox::capture_of(o.get("capture").ok_or("complete: missing capture")?)?,
            },
        ))),
        // The caller's own table matched one of the three tokens above, so the
        // third arm IS the third token — a fallible re-check whose error arm
        // cannot be reached would be an untestable branch.
        _ => Ok(Action::Advertise {
            tools: tools::decode(o.get("tools").ok_or("advertise: missing tools")?)?,
        }),
    }
}
