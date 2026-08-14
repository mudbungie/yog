//! The start-family halves of the [`codec`](super): the §3.4 [`Payload`] rung,
//! its [`BallSpec`], the composer's [`Prepared`], and the token spellings of
//! [`JoinState`] / [`Origin`] they carry. Split from the top-level codec per
//! §12's line budget; every encoder here is matched by a decoder and every
//! variant round-trips (the §8.5 parity tests).

use crate::opslog::Origin;
use crate::projects::join::JoinState;
use crate::start::{BallSpec, Payload, Prepared};
use serde_json::{Map, Value, json};
use std::path::Path;

use super::{opt_path_of, path_of, str_of};

/// Encode the §3.4 payload rung. `rung` is the discriminant; each rung carries
/// exactly its own inputs.
pub(super) fn encode_payload(payload: &Payload) -> Value {
    match payload {
        Payload::Bare => json!({ "rung": "bare" }),
        Payload::Path { dir } => json!({ "rung": "path", "dir": dir.to_string_lossy() }),
        Payload::Ball { project, ball } => json!({
            "rung": "ball",
            "project": project.to_string_lossy(),
            "ball": encode_ball(ball),
        }),
    }
}

/// Decode the §3.4 payload rung — strict: an unknown rung or a missing input
/// is a refusal, never a default (a guessed gesture is worse than none).
pub(super) fn decode_payload(v: &Value) -> Result<Payload, String> {
    let obj = v.as_object().ok_or("payload: not an object")?;
    match str_of(obj, "rung")?.as_str() {
        "bare" => Ok(Payload::Bare),
        "path" => Ok(Payload::Path {
            dir: path_of(obj, "dir")?,
        }),
        "ball" => Ok(Payload::Ball {
            project: path_of(obj, "project")?,
            ball: decode_ball(obj.get("ball").ok_or("payload: missing ball")?)?,
        }),
        other => Err(format!("payload: unknown rung {other:?}")),
    }
}

/// Encode the ball rung's spec: `id` present ⇒ existing (with its §3.5 join
/// state), absent ⇒ new (`bl create` mints the id).
fn encode_ball(ball: &BallSpec) -> Value {
    match ball {
        BallSpec::Existing {
            id,
            title,
            body,
            join,
        } => json!({ "id": id, "title": title, "body": body, "join": join_token(*join) }),
        BallSpec::New { title, body } => json!({ "title": title, "body": body }),
    }
}

fn decode_ball(v: &Value) -> Result<BallSpec, String> {
    let obj = v.as_object().ok_or("ball: not an object")?;
    let title = str_of(obj, "title")?;
    let body = str_of(obj, "body")?;
    match obj.get("id") {
        Some(id) => Ok(BallSpec::Existing {
            id: id.as_str().ok_or("ball: id not a string")?.to_owned(),
            title,
            body,
            join: parse_join(&str_of(obj, "join")?)?,
        }),
        None => Ok(BallSpec::New { title, body }),
    }
}

/// Encode the composer's fire-time parameters — the [`Action::Prompt`]
/// (crate::boundary::Action::Prompt) carrier and the `prepare` reply's body:
/// one spelling, so a reply deposits back verbatim as the next gesture.
pub(crate) fn encode_prepared(p: &Prepared) -> Value {
    json!({
        "name": p.name,
        "workspace": p.workspace.to_string_lossy(),
        // The §3.3 typed binding (bl-6654). `null` is the bare rung's "bind
        // nothing" — a real value of the field, not an omission, so a reply
        // deposits back as the same gesture it came from.
        "binding": p.binding.as_ref().map(|b| b.to_string_lossy()),
        "goal": p.goal,
        "origin": origin_token(p.origin),
    })
}

pub(crate) fn decode_prepared(v: &Value) -> Result<Prepared, String> {
    let obj = v.as_object().ok_or("prepared: not an object")?;
    Ok(Prepared {
        name: str_of(obj, "name")?,
        workspace: path_of(obj, "workspace")?,
        binding: opt_path_of(obj, "binding")?,
        goal: str_of(obj, "goal")?,
        origin: parse_origin(&str_of(obj, "origin")?)?,
    })
}

/// The §3.5 join-state tokens — the boundary spelling of the roster's cells.
pub(crate) fn join_token(state: JoinState) -> &'static str {
    match state {
        JoinState::ReadyStartable => "ready",
        JoinState::Blocked => "blocked",
        JoinState::Bound => "bound",
        JoinState::ClaimedElsewhere => "claimed-elsewhere",
        JoinState::Delivered => "delivered",
        JoinState::UnassignedWorkspace => "unassigned-workspace",
        JoinState::OrphanedProject => "orphaned-project",
    }
}

pub(super) fn parse_join(token: &str) -> Result<JoinState, String> {
    match token {
        "ready" => Ok(JoinState::ReadyStartable),
        "blocked" => Ok(JoinState::Blocked),
        "bound" => Ok(JoinState::Bound),
        "claimed-elsewhere" => Ok(JoinState::ClaimedElsewhere),
        "delivered" => Ok(JoinState::Delivered),
        "unassigned-workspace" => Ok(JoinState::UnassignedWorkspace),
        "orphaned-project" => Ok(JoinState::OrphanedProject),
        other => Err(format!("unknown join state {other:?}")),
    }
}

/// The §7.3 origin tokens — the same three the ops line records (§4.2).
pub(crate) fn origin_token(origin: Origin) -> &'static str {
    match origin {
        Origin::Balls => "balls",
        Origin::Conversation => "conversation",
        Origin::World => "world",
    }
}

pub(super) fn parse_origin(token: &str) -> Result<Origin, String> {
    match token {
        "balls" => Ok(Origin::Balls),
        "conversation" => Ok(Origin::Conversation),
        "world" => Ok(Origin::World),
        other => Err(format!("unknown origin {other:?}")),
    }
}

/// A `PathBuf` field's one JSON spelling (lossy UTF-8 both ways — paths cross
/// the boundary as text, exactly as they cross `ops.jsonl`).
pub(super) fn encode_path(p: &Path) -> Value {
    Value::String(p.to_string_lossy().into_owned())
}

/// An optional string field: present encodes, absent stays absent — `None`
/// and `""` are different facts (`--body ""` is an explicit empty body).
pub(super) fn opt_field(map: &mut Map<String, Value>, key: &str, v: Option<&String>) {
    if let Some(s) = v {
        map.insert(key.to_owned(), Value::String(s.clone()));
    }
}

pub(super) fn opt_of(obj: &Map<String, Value>, key: &str) -> Result<Option<String>, String> {
    match obj.get(key) {
        None => Ok(None),
        Some(v) => Ok(Some(
            v.as_str()
                .ok_or_else(|| format!("{key}: not a string"))?
                .to_owned(),
        )),
    }
}
