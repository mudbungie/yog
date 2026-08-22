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

/// The `bl` family's six envelopes (bl-c2bd) — its own family file, the seam
/// every sibling here is already cut on.
mod balls;
mod config;
mod control;
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
mod start;
mod tools;
use config::encode_file;
use deposit::{INTERRUPT, MESSAGE, deposit, deposited};
use fields::{act, obj, opt_path_of, opt_str_of, path_of, str_of, usize_of};
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
        Action::Close { project, id, name } => balls::ball("close", project, id, name),
        Action::Assign { project, id, name } => balls::ball("assign", project, id, name),
        Action::Release { project, id, name } => balls::ball("release", project, id, name),
        Action::Create {
            project,
            name,
            fields,
        } => balls::create(project, name, fields),
        Action::Update {
            project,
            id,
            name,
            fields,
        } => balls::update(project, id, name, fields),
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
            ruling,
        } => control::encode(workspace, agent, *ruling),
        Action::Floor {
            workspace,
            agent,
            raised,
        } => control::encode_floor(workspace, agent, *raised),
        Action::Ack => json!({ "op": "ack" }),
        Action::MarkSeen { workspace, agent } => at_agent("seen", workspace, agent),
        Action::ClearTrail => json!({ "op": "clear-trail" }),
        Action::ApplyConfig { file, text } => {
            json!({ "op": "config", "target": encode_file(file), "text": text })
        }
        Action::SetMarks { workspace, branch } => {
            json!({ "op": "marks", "workspace": workspace, "branch": branch })
        }
        Action::PickModel {
            workspace,
            role,
            provider,
            model,
        } => json!({ "op": "model", "workspace": workspace,
                     "role": role, "provider": provider, "model": model }),
        Action::Fork {
            workspace,
            parent,
            attempt,
            goal,
        } => fork::encode(workspace, parent, attempt, goal),
        Action::Advertise { tools } => tools::encode(tools),
        Action::Route(verb) => tools::encode_route(verb),
    }
}

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
        // REMOTE §5's tool-host family (bl-4e08, bl-024b): the presentation,
        // and the routing leg's two halves.
        tools::ADVERTISE | tools::INVOKE | tools::COMPLETE => {
            tools::decode(op.as_str(), o).map(act)
        }
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
mod tests;
