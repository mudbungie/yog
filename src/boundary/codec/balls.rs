//! The `bl` family's envelopes (§8.2): the five verbs that address a project's
//! task graph — three one-shape rows (op, project, id, `--as` name) and the
//! two with optional fields. Its own family file since
//! bl-c2bd, on the seam every other family here is cut on (`deposit`, `fan`,
//! `fork`, `config`) — the top-level codec keeps the roster, each family keeps
//! its grammar.

use serde_json::{Map, Value, json};

use crate::actions::verbs::{Verb, edit};
use crate::boundary::Action;

use super::fields::{bool_of, i64_of, list_of, opt};
use super::start::opt_field;
use super::{obj, opt_str_of, str_of};

/// The family's five spellings, off the one carrier the roster now holds
/// (bl-92d3). It lives here rather than as five rows of `super`'s match for the
/// reason the fan's `encode_verb` does: a family whose grammar is already one
/// file spells itself there, and the roster names the family once.
pub(super) fn encode(verb: &Verb) -> Value {
    match verb {
        Verb::Close { project, id, name } => ball("close", project, id, name),
        Verb::Assign { project, id, name } => ball("assign", project, id, name),
        Verb::Release { project, id, name } => ball("release", project, id, name),
        Verb::Create {
            project,
            name,
            fields,
        } => create(project, name, fields),
        Verb::Update {
            project,
            id,
            name,
            fields,
        } => update(project, id, name, fields),
    }
}

/// The three one-shape `bl` envelopes — op, project, id, `--as` name — said
/// once rather than three times, which is also what keeps the encode roster
/// inside §12's per-function budget.
fn ball(op: &str, project: &str, id: &str, name: &str) -> Value {
    json!({ "op": op, "project": project, "id": id, "name": name })
}

/// The two `bl` envelopes with **optional** fields, bodied out beside [`ball`]
/// for the same reason: a match arm that builds a map is a body, and the arm
/// roster stops reading as one once two of them are.
fn create(project: &str, name: &str, fields: &edit::Create) -> Value {
    let mut map = obj(&[("op", "create"), ("title", &fields.title), ("name", name)]);
    map.insert("project".to_owned(), Value::String(project.to_owned()));
    opt_field(&mut map, "body", fields.body.as_ref());
    encode_fields(&mut map, &fields.fields);
    Value::Object(map)
}

/// `[title, body, note]` in the order `/update`'s own flags read.
fn update(project: &str, id: &str, name: &str, fields: &edit::Update) -> Value {
    let mut map = obj(&[("op", "update"), ("id", id), ("name", name)]);
    map.insert("project".to_owned(), Value::String(project.to_owned()));
    for (key, value) in
        ["title", "body", "note"]
            .into_iter()
            .zip([&fields.title, &fields.body, &fields.note])
    {
        opt_field(&mut map, key, value.as_ref());
    }
    encode_fields(&mut map, &fields.fields);
    Value::Object(map)
}

/// The scheduling facts (bl-dbde), spelled as an **ordered array** because the
/// fold to argv applies them in order and two writes of one fact do not
/// commute. Omitted when empty, which is what the decoder reads absence as —
/// the same absent-is-a-value rule the optional string fields above take.
fn encode_fields(map: &mut Map<String, Value>, fields: &[edit::Field]) {
    if !fields.is_empty() {
        let rows: Vec<Value> = fields.iter().map(encode_field).collect();
        map.insert("fields".to_owned(), Value::Array(rows));
    }
}

/// One field application: its name, its value (`null` clears), and — for the
/// two that add or drop rather than set or clear — the direction.
fn encode_field(field: &edit::Field) -> Value {
    match field {
        edit::Field::Priority(n) => json!({ "field": "priority", "value": n }),
        edit::Field::Parent(id) => json!({ "field": "parent", "value": id }),
        edit::Field::Tag { tag, on } => json!({ "field": "tag", "value": tag, "on": on }),
        edit::Field::Needs { edge, on } => json!({ "field": "needs", "value": edge, "on": on }),
    }
}

/// The field list read back, strictly: absent is the empty list, anything
/// present must be an array of known field rows.
fn decode_fields(o: &Map<String, Value>) -> Result<Vec<edit::Field>, String> {
    if o.contains_key("fields") {
        list_of(o, "fields", decode_field)
    } else {
        Ok(Vec::new())
    }
}

fn decode_field(v: &Value) -> Result<edit::Field, String> {
    let o = v
        .as_object()
        .ok_or_else(|| "a ball field is an object".to_owned())?;
    Ok(match str_of(o, "field")?.as_str() {
        "priority" => edit::Field::Priority(opt(o, "value", i64_of)?),
        "parent" => edit::Field::Parent(opt(o, "value", str_of)?),
        "tag" => edit::Field::Tag {
            tag: str_of(o, "value")?,
            on: bool_of(o, "on")?,
        },
        "needs" => edit::Field::Needs {
            edge: str_of(o, "value")?,
            on: bool_of(o, "on")?,
        },
        other => return Err(format!("unknown ball field {other:?}")),
    })
}

/// Decode any of the five, strictly. `op` has already been matched by the
/// roster, so an unlisted one cannot arrive here — and the roster is what
/// keeps the fallthrough arm honest: `update` is the only op left.
pub(super) fn decode(op: &str, o: &Map<String, Value>) -> Result<Action, String> {
    let project = str_of(o, "project")?;
    Ok(Action::Ball(match op {
        "close" => Verb::Close {
            project,
            id: str_of(o, "id")?,
            name: str_of(o, "name")?,
        },
        "assign" => Verb::Assign {
            project,
            id: str_of(o, "id")?,
            name: str_of(o, "name")?,
        },
        "release" => Verb::Release {
            project,
            id: str_of(o, "id")?,
            name: str_of(o, "name")?,
        },
        "create" => Verb::Create {
            project,
            name: str_of(o, "name")?,
            fields: edit::Create {
                title: str_of(o, "title")?,
                body: opt_str_of(o, "body")?,
                fields: decode_fields(o)?,
            },
        },
        _ => Verb::Update {
            project,
            id: str_of(o, "id")?,
            name: str_of(o, "name")?,
            fields: edit::Update {
                title: opt_str_of(o, "title")?,
                body: opt_str_of(o, "body")?,
                note: opt_str_of(o, "note")?,
                fields: decode_fields(o)?,
            },
        },
    }))
}
