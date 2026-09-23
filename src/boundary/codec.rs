//! The gesture codec (§8.5): one JSON envelope per [`Gesture`], `op` the
//! discriminant, every parameter a named field. This is the headless
//! serialization of the boundary — the deposit file's whole content, the
//! `yog gesture` argument, and nothing the GUI ever writes (its serialization
//! is the in-RAM variant itself).
//!
//! Encode and decode are both exhaustive over [`Gesture`], which is the §4.8
//! compile gate: a new variant does not build until it has a spelling here.
//! Decode is **strict** — an unknown `op`, a missing field, a mistyped value
//! each refuse with a reason. A gesture is an instruction, not an observation;
//! the forgiving-parse discipline of `ops.jsonl` reads does not apply.

use serde_json::{Value, json};

use super::{Action, Gesture};

/// The `bl` family's five envelopes and, since bl-92d3, its whole spelling in
/// both directions — its own family file, the seam every sibling here is
/// already cut on.
mod balls;
mod config;
mod control;
/// The two acts a REMOTE device makes on its own behalf — enrollment and the
/// sign-in — split out at §12's cap (bl-b680) on the seam `action/device`
/// already draws for their prose.
mod device;
pub(crate) use device::{ENROLL, LOGIN, grade_of};
/// The two **depositing** envelopes (bl-a33d), one family file on the seam
/// every other family here is cut on: a plain send and send-and-interrupt carry
/// the same three fields, and the only difference is what the engine does once
/// the deposit lands.
mod deposit;
mod fan;
pub(crate) mod fields;
mod fleet;
pub(crate) use fleet::{ARM as FLEET_ARM, DISARM as FLEET_DISARM};
mod fork;
mod monitor;
mod query;
/// The §3.5 spend family's two acts (bl-53d1), beside the capability family.
pub(crate) mod spend;
mod start;
mod tools;
/// The §9.4 workflow mark's two envelopes (bl-b680), one family file; the
/// line reader spells its verbs from the same two tokens.
pub(crate) mod workflow;
use config::encode_write;
use deposit::{INTERRUPT, MESSAGE, deposit, deposited};
use fields::{act, obj, opt_path_of, opt_str_of, path_of, str_of, strings_of, usize_of};
use start::{decode_payload, decode_prepared, encode_start, opt_field};
pub(crate) use start::{
    decode_prepared as prepared_from_value, encode_prepared as prepared_value, join_token,
    origin_token, parse_join, parse_origin,
};

/// Encode a gesture to its deposit envelope. Total over the surface.
pub fn encode(gesture: &Gesture) -> Value {
    match gesture {
        Gesture::Act(action) => encode_action(action),
        Gesture::Ask(query) => query::encode(query),
    }
}

fn encode_action(action: &Action) -> Value {
    match action {
        Action::Message {
            workspace,
            agent,
            content,
        } => deposit(MESSAGE, workspace, agent, content),
        Action::Interrupt {
            workspace,
            agent,
            content,
        } => deposit(INTERRUPT, workspace, agent, content),
        Action::Stop {
            workspace,
            agent,
            children,
        } => json!({ "op": "stop", "workspace": workspace,
                     "agent": agent, "children": children }),
        Action::Scan { workspace } => json!({ "op": "scan", "workspace": workspace }),
        Action::Nudge { workspace, agent } => at_agent("nudge", workspace, agent),
        Action::Retarget { workspace, agent } => at_agent("retarget", workspace, agent),
        Action::Workflow {
            workspace,
            agent,
            config,
        } => workflow::encode(workspace, agent, config.as_deref()),
        // The §8.2 `bl` family's five, each spelled in its family file
        // (bl-c2bd), one row since bl-92d3 exactly as the fan's three are.
        Action::Ball(verb) => balls::encode(verb),
        // The §8.1 start family's two, beside the `Prepared` body they share.
        Action::Prepare { .. } | Action::Prompt { .. } => encode_start(action),
        // The §4.10 fan's three, each spelled in its family file (bl-c2bd).
        Action::Fan(verb) => fan::encode_verb(verb),
        Action::DeleteWorkspace { workspace, typed } => {
            json!({ "op": "delete-workspace", "workspace": workspace,
                    "typed": typed })
        }
        Action::DeleteAgent {
            workspace,
            agent,
            typed,
        } => json!({ "op": "delete-agent", "workspace": workspace,
                     "agent": agent, "typed": typed }),
        Action::Monitor(verb) => monitor::encode(verb),
        Action::Fleet(verb) => fleet::encode(verb),
        Action::AnswerHold {
            workspace,
            agent,
            answer,
        } => control::encode(workspace, agent, *answer),
        Action::Floor {
            workspace,
            agent,
            raised,
        } => control::encode_floor(workspace, agent, *raised),
        Action::Ack => json!({ "op": "ack" }),
        Action::MarkSeen { workspace, agent } => at_agent("seen", workspace, agent),
        // The §4.1 pin's two directions (bl-b986), the floor's shape: two ops
        // for one variant, so unpinning is an instruction and not a missing
        // field.
        Action::Pin { workspace, pinned } => {
            json!({ "op": if *pinned { PIN } else { UNPIN }, "workspace": workspace })
        }
        Action::ClearTrail => json!({ "op": "clear-trail" }),
        // The §9 config write family (bl-dd88), spelled in the config family's
        // own file: one carrier here, one op per member on the wire, which is
        // what makes the fold free of a version bump (REMOTE §3).
        Action::Config(write) => encode_write(write),
        Action::Fork {
            workspace,
            parent,
            attempt,
            goal,
        } => fork::encode(workspace, parent, attempt, goal),
        Action::Advertise { tools } => tools::encode(tools),
        // The two acts a REMOTE device makes on its own behalf, spelled in the
        // device family's file (bl-b680's split, on `action/device`'s seam).
        Action::Enroll(request) => device::encode_enroll(request),
        Action::Route(verb) => tools::encode_route(verb),
        Action::Login {
            workspace,
            provider,
        } => device::encode_login(workspace, provider),
        // The §3.5 spend family (bl-53d1), spelled in its family file.
        Action::Price { .. } | Action::Ceiling { .. } => spend::encode(action),
    }
}

/// The §4.1 pin's two op tokens (bl-b986), named once for the envelope, the
/// line and the help page — which is how one act cannot be spelled three ways.
pub(crate) const PIN: &str = "pin";
pub(crate) const UNPIN: &str = "unpin";

/// The three one-shape **conversation** envelopes — op, workspace, agent — said
/// once rather than three times, for [`balls::ball`]'s reason exactly: the
/// gestures that name a conversation and carry nothing else are one shape, and
/// a match arm that rebuilds it is a body pretending to be a row.
fn at_agent(op: &str, workspace: &str, agent: &str) -> Value {
    json!({ "op": op, "workspace": workspace, "agent": agent })
}

/// Decode a deposit envelope. The `op` table is the boundary's whole verb
/// roster; anything else refuses with the offending token.
pub fn decode(v: &Value) -> Result<Gesture, String> {
    let o = v.as_object().ok_or("gesture: not a JSON object")?;
    let op = str_of(o, "op")?;
    match op.as_str() {
        MESSAGE | INTERRUPT => Ok(act(deposited(&op, o)?)),
        "stop" => Ok(act(Action::Stop {
            workspace: str_of(o, "workspace")?,
            agent: str_of(o, "agent")?,
            children: o.get("children").and_then(Value::as_bool).unwrap_or(false),
        })),
        "scan" => Ok(act(Action::Scan {
            workspace: str_of(o, "workspace")?,
        })),
        "nudge" => Ok(act(Action::Nudge {
            workspace: str_of(o, "workspace")?,
            agent: str_of(o, "agent")?,
        })),
        "retarget" => Ok(act(Action::Retarget {
            workspace: str_of(o, "workspace")?,
            agent: str_of(o, "agent")?,
        })),
        // The §9.4 workflow mark's two directions (bl-b680), in its family file.
        workflow::WORKFLOW | workflow::CLEAR => workflow::decode(op.as_str(), o).map(act),
        // The `bl` family's five (§8.2), each in its family file.
        "close" | "assign" | "release" | "create" | "update" => {
            balls::decode(op.as_str(), o).map(act)
        }
        "prepare" => Ok(act(Action::Prepare {
            workspace: str_of(o, "workspace")?,
            payload: decode_payload(o.get("payload").ok_or("prepare: missing payload")?)?,
        })),
        "prompt" => Ok(act(Action::Prompt {
            prepared: decode_prepared(o.get("prepared").ok_or("prompt: missing prepared")?)?,
            goal: str_of(o, "goal")?,
            seed: fields::opt(o, "seed", fields::u64_of)?,
        })),
        "delete-workspace" => Ok(act(Action::DeleteWorkspace {
            workspace: str_of(o, "workspace")?,
            typed: str_of(o, "typed")?,
        })),
        "delete-agent" => Ok(act(Action::DeleteAgent {
            workspace: str_of(o, "workspace")?,
            agent: str_of(o, "agent")?,
            typed: str_of(o, "typed")?,
        })),
        "arm" | "disarm" | "flag" => monitor::decode(op.as_str(), o),
        fleet::ARM | fleet::DISARM => fleet::decode(op.as_str(), o),
        "answer" => control::decode(o),
        // The §4.9 fifth rung over the §4.11 fold: the floor's two directions.
        "revoke" | "restore" => control::decode_floor(op.as_str(), o),
        "fork" => fork::decode(o).map(act),
        // The §4.10 fan's three: materialize N candidates, retire one, and
        // deliver one (V3.2's acceptance, bl-c2bd).
        fan::FAN | fan::RETIRE | fan::DELIVER => fan::decode(op.as_str(), o).map(act),
        "ack" => Ok(act(Action::Ack)),
        // The §6 decision queue's answer (VISION §5 V5.2): `seen`, not `ack` —
        // the trail's alarm ack already wears that word, and these two quiet
        // different things.
        "seen" => Ok(act(Action::MarkSeen {
            workspace: str_of(o, "workspace")?,
            agent: str_of(o, "agent")?,
        })),
        "clear-trail" => Ok(act(Action::ClearTrail)),
        // The §4.1 pin (bl-b986): the op token IS the direction, so nothing is
        // defaulted and an unpin can never read as a pin that lost a field.
        PIN | UNPIN => Ok(act(Action::Pin {
            workspace: str_of(o, "workspace")?,
            pinned: op == PIN,
        })),
        // The device's own two acts (REMOTE §8.3, §1.4), in their family file.
        LOGIN | ENROLL => device::decode(op.as_str(), o).map(act),
        // REMOTE §5's tool-host family (bl-4e08, bl-024b): the presentation,
        // and the routing leg's two halves.
        tools::ADVERTISE | tools::INVOKE | tools::COMPLETE => {
            tools::decode(op.as_str(), o).map(act)
        }
        spend::PRICE | spend::CEILING => spend::decode(op.as_str(), o).map(act),
        // The two families that read in their own modules (bl-3f46, bl-3746):
        // every query — `config`/`marks` read-shaped among them, bl-0164 —
        // then the §9 config verbs. This match stays the action roster rather
        // than growing three grammars inside it.
        other => query::decode(other, o)
            .map(|query| query.map(Gesture::Ask))
            .or_else(|| config::decode_action(other, o).map(|action| action.map(act)))
            .unwrap_or_else(|| Err(format!("unknown op {other:?}"))),
    }
}

#[cfg(test)]
pub(crate) mod tests;
