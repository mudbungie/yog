//! **What a §9 config read answers** (§8.5, §9.5; bl-dc3f): [`ConfigView`] —
//! one destination's raw bytes and the same bytes read through the file's own
//! schema — and both directions of its spelling.
//!
//! Its own type rather than two fields on the answer enum, on the seam §9.5
//! itself draws: a config file is *one fact with two views*, and the pair
//! travels together everywhere — the read composes it, the codec spells it, a
//! seat renders both halves of it. `reply/model` stays the roster of what an
//! answer IS, at §12's pre-split band, and this is what one of them is made of.
//!
//! §9.5 rules that *"every setting the files declare is answered as the typed
//! thing it is"*, judged at input rather than at Apply. The whole enumeration
//! that says so — which block an entry hangs under, what fields it carries,
//! what control each is edited through, the bounds a number is legal in, and
//! the provider judgement `grammar::is_unknown_row` makes — lived in
//! `config_edit::form` and reached no seat. This is its spelling.
//!
//! **The bounds ride the control, because the control is a shape and not a
//! word.** A seat that knows a field is a number but not its range cannot judge
//! at input, which is the ruling's own verb; and yog's bounds are the worker's
//! own consts (`app::cadence`), so a seat re-deriving them would be a second
//! authority on the clock's limits. `min`/`max` are therefore present exactly
//! on the `number` kind and absent everywhere else — a shape, not four
//! optional siblings on a flat object.

use serde_json::{Map, Value, json};

use crate::boundary::codec::fields::{list_of, opt_str_of, str_of, u64_of};
use crate::config_edit::form::{Control, Row};

/// One §9 destination, **both views** (§8.5 bl-0164, §9.5 bl-dc3f):
/// [`ReadConfig`](crate::boundary::Query::ReadConfig)'s answer — the file
/// editors' Reload spelled headless, and the controls pane's own read beside
/// it.
///
/// One answer rather than two queries, because they are one file: a
/// `Query::ConfigSchema` would answer a *description* a second read then had to
/// be joined against, and the two could be of different bytes. A file yog has
/// no grammar for carries an **empty** `settings` — §9.5's three justified
/// raw-text fallbacks are the general path with empty input, not a branch, and
/// a seat with nothing typed to show is already showing the raw editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigView {
    pub text: String,
    pub settings: Vec<Row>,
}

/// The answer's frame: the bytes, and the settings read out of them.
pub(super) fn config(view: &ConfigView) -> Value {
    json!({
        "ok": true, "kind": "config", "text": view.text,
        "settings": view.settings.iter().map(setting).collect::<Vec<Value>>(),
    })
}

/// The frame read back (REMOTE §9 step 2).
pub(crate) fn config_of(o: &Map<String, Value>) -> Result<ConfigView, String> {
    Ok(ConfigView {
        text: str_of(o, "text")?,
        settings: list_of(o, "settings", decode)?,
    })
}

/// One setting: the entry it hangs under, its field name, the control it is
/// edited through, the words beside it, the value the file currently spells,
/// and — **absent, never null** — why that value cannot be used as it stands.
///
/// `entry` rides every row rather than the answer carrying groups: a grouping
/// is this field read twice, and a seat that wants one makes it while it
/// paints. `fault` is absent for a usable value on the roster's own discipline
/// — a reader must never have to tell *no fault* from *a fault with nothing to
/// say*.
fn setting(row: &Row) -> Value {
    let mut map = Map::new();
    map.insert("entry".to_owned(), json!(row.entry));
    map.insert("name".to_owned(), json!(row.name));
    map.insert("control".to_owned(), control(row.control));
    map.insert("help".to_owned(), json!(row.help));
    map.insert("value".to_owned(), json!(row.value));
    if let Some(fault) = &row.fault {
        map.insert("fault".to_owned(), json!(fault));
    }
    Value::Object(map)
}

/// The control as its own object: a `kind` word always, and the bounds the
/// `number` kind carries.
fn control(control: Control) -> Value {
    match control {
        Control::Provider => json!({ "kind": "provider" }),
        Control::List => json!({ "kind": "list" }),
        Control::Text => json!({ "kind": "text" }),
        Control::Number { min, max } => json!({ "kind": "number", "min": min, "max": max }),
    }
}

/// The same setting read back — strict, like every decoder here: a missing
/// field, a mistyped value and an unknown `kind` each refuse naming the
/// offender. A `number` whose bounds are absent refuses too rather than
/// defaulting: a control that invented its own range would judge input by a
/// rule the engine never stated, which is the drift this answer exists to end.
fn decode(v: &Value) -> Result<Row, String> {
    let o = v.as_object().ok_or("setting: not an object")?;
    Ok(Row {
        entry: str_of(o, "entry")?,
        name: str_of(o, "name")?,
        control: control_of(
            o.get("control")
                .ok_or("setting: missing field \"control\"")?,
        )?,
        help: str_of(o, "help")?,
        value: str_of(o, "value")?,
        fault: opt_str_of(o, "fault")?,
    })
}

fn control_of(v: &Value) -> Result<Control, String> {
    let o = v.as_object().ok_or("control: not an object")?;
    match str_of(o, "kind")?.as_str() {
        "provider" => Ok(Control::Provider),
        "list" => Ok(Control::List),
        "text" => Ok(Control::Text),
        "number" => Ok(Control::Number {
            min: u64_of(o, "min")?,
            max: u64_of(o, "max")?,
        }),
        other => Err(format!("control: unknown kind {other:?}")),
    }
}

#[cfg(test)]
mod tests;
