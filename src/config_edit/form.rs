//! The typed config pane's view-model (DESIGN §9.5) — **controls over facts**,
//! not an editor over a file.
//!
//! The §9.1/§9.2 surfaces bound a whole file's text to a `TextEdit`, so every
//! setting was edited blind and judged afterwards, at Apply. Here each setting a
//! file declares is a [`Row`] carrying the [`Control`] it is edited through, the
//! value the file currently spells, and — for a provider reference — the same
//! judgement the §9.2 Apply gate and the §9.4 pick gate make
//! ([`is_unknown_row`]). Validation happens at input, not after it.
//!
//! **The file stays the single fact.** A control does not write to disk and
//! holds no copy of anything: [`write`] rewrites the *draft text* through the
//! same anchored line edit §9.4's picker uses, and that draft Applies through
//! the unchanged §9 pipeline (stage → gate → hash-guard → atomic rename). There
//! is no second store to drift, and no second authority on the file's shape.
//!
//! **A new setting is a row, not a rebuild.** Which settings exist is
//! [`schema`]'s table; this file is only how one is read and written. A file
//! with no schema has none of its settings typed and keeps the raw editor —
//! the general path with empty input, not a branch.

use crate::model_pick::grammar::{
    GrammarError, entry_field, entry_names, flow_members, flow_value, is_unknown_row, set_field,
};

mod schema;

pub use schema::{CADENCE_SCHEMA, Control, FieldSpec, ROLES_SCHEMA, Schema, schema_for};

/// One rendered setting: which entry declares it, what it edits, what the file
/// currently spells, and why that value is not usable.
///
/// **Owned, not `&'static`, since bl-dc3f.** The rows cross the §8.5 boundary
/// on `Query::ReadConfig`'s answer, so they are read back off a wire as well as
/// built from [`FieldSpec`]'s tables, and a decoded row has no static text to
/// borrow. One type in both directions, as everywhere else on this boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// The block entry this setting hangs under — a model id, a role name. A
    /// seat groups the flat list by it; the engine keeps no second shape for
    /// that, because a grouping IS this field read twice.
    pub entry: String,
    pub name: String,
    pub control: Control,
    pub help: String,
    /// The value as its control shows it — a flow sequence as its members.
    pub value: String,
    /// Why this value cannot be used as it stands, or `None`.
    pub fault: Option<String>,
}

/// Read a file's text as typed settings (§9.5) — **one flat list, entry order
/// then field order**. A field the entry does not declare yields no row: the
/// answer carries the settings that exist, and litany's own loader stays the
/// authority on a half-written entry.
///
/// `provider_rows` is brazen's effective table; an empty one is no answer and
/// faults nothing ([`is_unknown_row`]).
pub fn read(schema: &Schema, text: &str, provider_rows: &[String]) -> Vec<Row> {
    entry_names(text, schema.block)
        .into_iter()
        .flat_map(|entry| {
            schema
                .fields
                .iter()
                .filter_map(|spec| row(schema, text, &entry, spec, provider_rows))
                .collect::<Vec<Row>>()
        })
        .collect()
}

fn row(
    schema: &Schema,
    text: &str,
    entry: &str,
    spec: &FieldSpec,
    provider_rows: &[String],
) -> Option<Row> {
    let raw = entry_field(text, schema.block, entry, spec.name)?;
    let (value, fault) = present(spec.control, &raw, provider_rows);
    Some(Row {
        entry: entry.to_owned(),
        name: spec.name.to_owned(),
        control: spec.control,
        help: spec.help.to_owned(),
        value,
        fault,
    })
}

/// The stored value as its control shows it, plus why it is not usable.
fn present(control: Control, raw: &str, provider_rows: &[String]) -> (String, Option<String>) {
    match control {
        Control::Provider => (
            raw.to_owned(),
            is_unknown_row(raw, provider_rows).then(|| {
                format!(
                    "brazen's table has no provider row `{raw}` — every dispatch \
                     through it dies with `unknown provider`; pick a live row, or \
                     add the row in the brazen config editor"
                )
            }),
        ),
        Control::List => match flow_members(raw) {
            Some(members) => (members.join(", "), None),
            None => (
                raw.to_owned(),
                Some(format!(
                    "not the inline `[a, b]` form yog edits — fix `{raw}` in the \
                     raw text below"
                )),
            ),
        },
        Control::Number { min, max } => (
            raw.to_owned(),
            match raw.parse::<u64>() {
                Ok(n) if (min..=max).contains(&n) => None,
                _ => Some(format!("expected a whole number from {min} to {max}")),
            },
        ),
        Control::Text => (raw.to_owned(), None),
    }
}

/// Write one control's value back into the draft text (§9.5) — the same
/// anchored line edit the picker writes through, so an off-grammar file
/// declines here exactly as it declines there. The caller Applies the returned
/// text through the unchanged §9 pipeline; nothing here touches disk.
///
/// It takes the [`FieldSpec`] rather than a [`Row`] (bl-dc3f): a row is now an
/// **answered** thing a peer may hand back, so the field being written and the
/// control normalizing it must come from the schema — the one authority on what
/// this file declares — and not from the bytes asking for the write.
pub fn write(
    schema: &Schema,
    text: &str,
    entry: &str,
    spec: &FieldSpec,
    value: &str,
) -> Result<String, GrammarError> {
    set_field(
        schema.file,
        text,
        schema.block,
        entry,
        spec.name,
        &normalize(spec.control, value),
    )
}

/// The value as the file spells it: a bounded number clamped into range, a list
/// re-emitted as litany's flow sequence, anything else trimmed. A number that
/// does not parse falls to `min` — the control cannot produce one, and a
/// keystroke mid-edit must never write a line the grammar cannot read back.
fn normalize(control: Control, value: &str) -> String {
    match control {
        Control::Number { min, max } => value
            .trim()
            .parse::<u64>()
            .unwrap_or(min)
            .clamp(min, max)
            .to_string(),
        Control::List => flow_value(
            &value
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_owned)
                .collect::<Vec<String>>(),
        ),
        Control::Provider | Control::Text => value.trim().to_owned(),
    }
}

#[cfg(test)]
mod tests;
