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

mod config;
mod control;
mod fields;
mod fleet;
pub(crate) use fleet::{ARM as FLEET_ARM, DISARM as FLEET_DISARM};
mod fork;
mod monitor;
mod query;
mod start;
use config::encode_file;
use fields::{act, obj, opt_path_of, path_of, str_of, usize_of};
use start::{
    decode_payload, decode_prepared, encode_path, encode_payload, encode_prepared, opt_field,
    opt_of,
};
pub(crate) use start::{
    decode_prepared as prepared_from_value, encode_prepared as prepared_value, join_token,
    origin_token,
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
        } => json!({ "op": "message", "workspace": encode_path(workspace),
                     "agent": agent, "content": content }),
        Action::Stop {
            workspace,
            agent,
            children,
        } => json!({ "op": "stop", "workspace": encode_path(workspace),
                     "agent": agent, "children": children }),
        Action::Scan { workspace } => {
            json!({ "op": "scan", "workspace": encode_path(workspace) })
        }
        Action::Retarget { workspace, agent } => {
            json!({ "op": "retarget", "workspace": encode_path(workspace), "agent": agent })
        }
        Action::Close { project, id, name } => ball("close", project, id, name),
        Action::Assign { project, id, name } => ball("assign", project, id, name),
        Action::Release { project, id, name } => ball("release", project, id, name),
        Action::Move {
            project,
            id,
            from,
            to,
        } => json!({ "op": "move", "project": encode_path(project),
                     "id": id, "from": from, "to": to }),
        Action::Create {
            project,
            title,
            name,
            body,
        } => create(project, title, name, body.as_ref()),
        Action::Update {
            project,
            id,
            name,
            title,
            body,
            note,
        } => update(project, id, name, [title, body, note]),
        Action::Prepare { workspace, payload } => {
            json!({ "op": "prepare", "workspace": encode_path(workspace),
                    "payload": encode_payload(payload) })
        }
        Action::Prompt { prepared, goal } => {
            json!({ "op": "prompt", "prepared": encode_prepared(prepared), "goal": goal })
        }
        Action::DeleteWorkspace { workspace, typed } => {
            json!({ "op": "delete-workspace", "workspace": encode_path(workspace),
                    "typed": typed })
        }
        Action::DeleteAgent {
            workspace,
            agent,
            typed,
        } => json!({ "op": "delete-agent", "workspace": encode_path(workspace),
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
        Action::MarkSeen { workspace, agent } => {
            json!({ "op": "seen", "workspace": encode_path(workspace), "agent": agent })
        }
        Action::ClearTrail => json!({ "op": "clear-trail" }),
        Action::ApplyConfig { file, text } => {
            json!({ "op": "config", "target": encode_file(file), "text": text })
        }
        Action::SetMarks { workspace, branch } => {
            json!({ "op": "marks", "workspace": encode_path(workspace), "branch": branch })
        }
        Action::PickModel {
            workspace,
            role,
            provider,
            model,
        } => json!({ "op": "model", "workspace": encode_path(workspace),
                     "role": role, "provider": provider, "model": model }),
        Action::Fork {
            workspace,
            parent,
            attempt,
            goal,
        } => fork::encode(workspace, parent, attempt, goal),
    }
}

/// The three one-shape `bl` envelopes — op, project, id, `--as` name — said
/// once rather than three times, which is also what keeps [`encode_action`]
/// inside §12's per-function budget.
fn ball(op: &str, project: &std::path::Path, id: &str, name: &str) -> Value {
    json!({ "op": op, "project": encode_path(project), "id": id, "name": name })
}

/// The two `bl` envelopes with **optional** fields, bodied out beside [`ball`]
/// for the same reason: a match arm that builds a map is a body, and the arm
/// roster stops reading as one once two of them are.
fn create(project: &std::path::Path, title: &str, name: &str, body: Option<&String>) -> Value {
    let mut map = obj(&[("op", "create"), ("title", title), ("name", name)]);
    map.insert("project".to_owned(), encode_path(project));
    opt_field(&mut map, "body", body);
    Value::Object(map)
}

/// `[title, body, note]` in the order `/update`'s own flags read.
fn update(project: &std::path::Path, id: &str, name: &str, fields: [&Option<String>; 3]) -> Value {
    let mut map = obj(&[("op", "update"), ("id", id), ("name", name)]);
    map.insert("project".to_owned(), encode_path(project));
    for (key, value) in ["title", "body", "note"].into_iter().zip(fields) {
        opt_field(&mut map, key, value.as_ref());
    }
    Value::Object(map)
}

/// Decode a deposit envelope. The `op` table is the boundary's whole verb
/// roster; anything else refuses with the offending token.
pub fn decode(v: &Value) -> Result<Gesture, String> {
    let o = v.as_object().ok_or("gesture: not a JSON object")?;
    let op = str_of(o, "op")?;
    match op.as_str() {
        "message" => Ok(act(Action::Message {
            workspace: path_of(o, "workspace")?,
            agent: str_of(o, "agent")?,
            content: str_of(o, "content")?,
        })),
        "stop" => Ok(act(Action::Stop {
            workspace: path_of(o, "workspace")?,
            agent: str_of(o, "agent")?,
            children: o.get("children").and_then(Value::as_bool).unwrap_or(false),
        })),
        "scan" => Ok(act(Action::Scan {
            workspace: path_of(o, "workspace")?,
        })),
        "retarget" => Ok(act(Action::Retarget {
            workspace: path_of(o, "workspace")?,
            agent: str_of(o, "agent")?,
        })),
        "close" => Ok(act(Action::Close {
            project: path_of(o, "project")?,
            id: str_of(o, "id")?,
            name: str_of(o, "name")?,
        })),
        "assign" => Ok(act(Action::Assign {
            project: path_of(o, "project")?,
            id: str_of(o, "id")?,
            name: str_of(o, "name")?,
        })),
        "release" => Ok(act(Action::Release {
            project: path_of(o, "project")?,
            id: str_of(o, "id")?,
            name: str_of(o, "name")?,
        })),
        "move" => Ok(act(Action::Move {
            project: path_of(o, "project")?,
            id: str_of(o, "id")?,
            from: str_of(o, "from")?,
            to: str_of(o, "to")?,
        })),
        "create" => Ok(act(Action::Create {
            project: path_of(o, "project")?,
            title: str_of(o, "title")?,
            name: str_of(o, "name")?,
            body: opt_of(o, "body")?,
        })),
        "update" => Ok(act(Action::Update {
            project: path_of(o, "project")?,
            id: str_of(o, "id")?,
            name: str_of(o, "name")?,
            title: opt_of(o, "title")?,
            body: opt_of(o, "body")?,
            note: opt_of(o, "note")?,
        })),
        "prepare" => Ok(act(Action::Prepare {
            workspace: path_of(o, "workspace")?,
            payload: decode_payload(o.get("payload").ok_or("prepare: missing payload")?)?,
        })),
        "prompt" => Ok(act(Action::Prompt {
            prepared: decode_prepared(o.get("prepared").ok_or("prompt: missing prepared")?)?,
            goal: str_of(o, "goal")?,
        })),
        "delete-workspace" => Ok(act(Action::DeleteWorkspace {
            workspace: path_of(o, "workspace")?,
            typed: str_of(o, "typed")?,
        })),
        "delete-agent" => Ok(act(Action::DeleteAgent {
            workspace: path_of(o, "workspace")?,
            agent: str_of(o, "agent")?,
            typed: str_of(o, "typed")?,
        })),
        "arm" | "disarm" | "flag" => monitor::decode(op.as_str(), o),
        fleet::ARM | fleet::DISARM => fleet::decode(op.as_str(), o),
        "answer" => control::decode(o),
        // The §4.9 fifth rung over the §4.11 fold: the floor's two directions.
        "revoke" | "restore" => control::decode_floor(op.as_str(), o),
        "fork" => fork::decode(o).map(act),
        "ack" => Ok(act(Action::Ack)),
        // The §6 decision queue's answer (VISION §5 V5.2): `seen`, not `ack` —
        // the trail's alarm ack already wears that word, and these two quiet
        // different things.
        "seen" => Ok(act(Action::MarkSeen {
            workspace: path_of(o, "workspace")?,
            agent: str_of(o, "agent")?,
        })),
        "clear-trail" => Ok(act(Action::ClearTrail)),
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
