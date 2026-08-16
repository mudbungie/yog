//! The `bl` family's envelopes (§8.2): the six verbs that address a project's
//! task graph — three one-shape rows (op, project, id, `--as` name), the
//! four-name move, and the two with optional fields. Its own family file since
//! bl-c2bd, on the seam every other family here is cut on (`deposit`, `fan`,
//! `fork`, `config`) — the top-level codec keeps the roster, each family keeps
//! its grammar.

use serde_json::{Map, Value, json};

use crate::boundary::Action;

use super::start::opt_field;
use super::{obj, opt_str_of, str_of};

/// The three one-shape `bl` envelopes — op, project, id, `--as` name — said
/// once rather than three times, which is also what keeps the encode roster
/// inside §12's per-function budget.
pub(super) fn ball(op: &str, project: &str, id: &str, name: &str) -> Value {
    json!({ "op": op, "project": project, "id": id, "name": name })
}

/// The four-name re-home: unclaim as `from`, claim as `to`.
pub(super) fn encode_move(project: &str, id: &str, from: &str, to: &str) -> Value {
    json!({ "op": "move", "project": project, "id": id, "from": from, "to": to })
}

/// The two `bl` envelopes with **optional** fields, bodied out beside [`ball`]
/// for the same reason: a match arm that builds a map is a body, and the arm
/// roster stops reading as one once two of them are.
pub(super) fn create(project: &str, title: &str, name: &str, body: Option<&String>) -> Value {
    let mut map = obj(&[("op", "create"), ("title", title), ("name", name)]);
    map.insert("project".to_owned(), Value::String(project.to_owned()));
    opt_field(&mut map, "body", body);
    Value::Object(map)
}

/// `[title, body, note]` in the order `/update`'s own flags read.
pub(super) fn update(project: &str, id: &str, name: &str, fields: [&Option<String>; 3]) -> Value {
    let mut map = obj(&[("op", "update"), ("id", id), ("name", name)]);
    map.insert("project".to_owned(), Value::String(project.to_owned()));
    for (key, value) in ["title", "body", "note"].into_iter().zip(fields) {
        opt_field(&mut map, key, value.as_ref());
    }
    Value::Object(map)
}

/// Decode any of the six, strictly. `op` has already been matched by the
/// roster, so an unlisted one cannot arrive here — and the roster is what
/// keeps the fallthrough arm honest: `update` is the only op left.
pub(super) fn decode(op: &str, o: &Map<String, Value>) -> Result<Action, String> {
    let project = str_of(o, "project")?;
    Ok(match op {
        "close" => Action::Close {
            project,
            id: str_of(o, "id")?,
            name: str_of(o, "name")?,
        },
        "assign" => Action::Assign {
            project,
            id: str_of(o, "id")?,
            name: str_of(o, "name")?,
        },
        "release" => Action::Release {
            project,
            id: str_of(o, "id")?,
            name: str_of(o, "name")?,
        },
        "move" => Action::Move {
            project,
            id: str_of(o, "id")?,
            from: str_of(o, "from")?,
            to: str_of(o, "to")?,
        },
        "create" => Action::Create {
            project,
            title: str_of(o, "title")?,
            name: str_of(o, "name")?,
            body: opt_str_of(o, "body")?,
        },
        _ => Action::Update {
            project,
            id: str_of(o, "id")?,
            name: str_of(o, "name")?,
            title: opt_str_of(o, "title")?,
            body: opt_str_of(o, "body")?,
            note: opt_str_of(o, "note")?,
        },
    })
}
