//! **The corpus's standing record** (bl-32cb): per shape, the field signature
//! its fixtures spell and the protocol version at which that signature last
//! moved.
//!
//! It exists to make one REMOTE rule mechanical rather than remembered — *a
//! change to a wire-visible shape bumps the protocol version*. The fixtures
//! alone cannot enforce it: regenerating them makes any diff vanish, so a
//! shape could change meaning at a standing version and nothing would notice.
//! The record remembers what the shapes *were*, so a signature that moved
//! while the version stood still is refusable at the moment it is regenerated.
//!
//! **The version the comparison is against is neither the shape's nor the
//! record's: it is the newest version that has been PUBLISHED**
//! ([`crate::wire::hello::PROTOCOL_PUBLISHED`], bl-9ced amending bl-00de).
//! The per-shape `since` is a *stamp* — the version at which that shape last
//! moved, which is what a client reads — and never a term in the test. It used
//! to be one, and the reasoning was wrong in a way that only showed later: a
//! shape edited at a version that was then found spent, and raised past, kept
//! the pre-bump number, because the regeneration after the bump saw an
//! unchanged signature and had nothing to restamp. bl-00de replaced it with
//! the record's own top-level `protocol` — the version the record was last
//! *generated* at — which fixed the stamp and left one thing wrong: that
//! number is a **proxy** for "a version a peer may be speaking", and it
//! advances at every bump whether or not the bump ever shipped. So the second
//! lane of one unreleased wave was refused and had to take another integer,
//! which §9.11 accepted as cheap. Under bl-bca2's release hold it is not
//! cheap — each number is three consumer repositories re-vendoring a constant
//! and one more window in which no published suite composes — so the term is
//! now the thing itself. `protocol` stays in the record as the generation
//! stamp a reader wants; it is no longer a term in the rule.
//!
//! A regeneration at an unchanged version with unchanged signatures is still a
//! byte-identical no-op.
//!
//! **A signature is field paths and their JSON types, not bytes.** Adding a
//! sample to a shape leaves the signature alone, which is right: a new fixture
//! is not a wire change. Renaming a field, changing its type, gaining one,
//! losing one or losing the whole shape all move it, which is also right. A new
//! shape moves nothing — REMOTE is explicit that a new verb is not a bump,
//! because strict decode already refuses an unknown one in band.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value, json};

use super::Shape;

/// One shape's record.
pub(super) struct Entry {
    pub(super) since: u32,
    pub(super) signature: Vec<String>,
}

/// Every shape's record, plus the protocol the corpus as a whole is for.
pub(super) struct Ledger {
    pub(super) protocol: u32,
    pub(super) shapes: BTreeMap<String, Entry>,
}

impl Ledger {
    /// Read a committed record. Anything unreadable is an empty record — the
    /// same answer a corpus that does not exist yet gives, and the gate below
    /// then asks for a regeneration rather than for a version bump.
    pub(super) fn read(text: &str) -> Self {
        let value = serde_json::from_str::<Value>(text).unwrap_or(Value::Null);
        let mut shapes = BTreeMap::new();
        for (name, entry) in value
            .get("shapes")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
        {
            shapes.insert(
                name.clone(),
                Entry {
                    since: number(entry.get("since")),
                    signature: strings(entry.get("signature")),
                },
            );
        }
        Self {
            protocol: number(value.get("protocol")),
            shapes,
        }
    }

    /// The record's own canonical bytes.
    pub(super) fn render(&self) -> String {
        let shapes: Map<String, Value> = self
            .shapes
            .iter()
            .map(|(name, entry)| {
                let body = json!({ "since": entry.since, "signature": entry.signature });
                (name.clone(), body)
            })
            .collect();
        let doc = json!({ "protocol": self.protocol, "shapes": shapes });
        super::canonical(&doc)
    }
}

fn number(value: Option<&Value>) -> u32 {
    let raw = value.and_then(Value::as_u64).unwrap_or_default();
    u32::try_from(raw).unwrap_or_default()
}

fn strings(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

/// Every field path a shape's frames spell, with the JSON type found there.
/// Array elements collapse to one `[]` step, so a two-element list and a
/// one-element list of the same rows are one signature.
pub(super) fn signature(frames: &[Value]) -> Vec<String> {
    let mut out = BTreeSet::new();
    for frame in frames {
        walk("", frame, &mut out);
    }
    out.into_iter().collect()
}

fn walk(path: &str, value: &Value, out: &mut BTreeSet<String>) {
    out.insert(format!("{path}:{}", kind(value)));
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                walk(&format!("{path}/{key}"), child, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                walk(&format!("{path}/[]"), item, out);
            }
        }
        _ => {}
    }
}

fn kind(value: &Value) -> &'static str {
    match value {
        Value::Object(_) => "object",
        Value::Array(_) => "array",
        Value::String(_) => "string",
        Value::Number(_) => "number",
        Value::Bool(_) => "bool",
        Value::Null => "null",
    }
}

/// The record this boundary earns, or the refusal that says why it cannot have
/// one. **A signature may change only above the published floor** (bl-9ced
/// amending bl-00de): `published` is the newest version any peer can be
/// speaking, and any shape that moved — or vanished — is refused unless the
/// version being generated is greater than it. The sentence carries both
/// halves of the remedy, and names the floor so the reader can see which of
/// the two integers is in their way.
pub(super) fn advance(
    shapes: &[Shape],
    previous: &Ledger,
    protocol: u32,
    published: u32,
) -> Result<Ledger, String> {
    let fresh: BTreeMap<String, Vec<String>> = shapes
        .iter()
        .map(|shape| (shape.key(), signature(&shape.frames)))
        .collect();
    let moved: Vec<String> = previous
        .shapes
        .iter()
        .filter(|(name, entry)| fresh.get(*name) != Some(&entry.signature))
        .map(|(name, _)| name.clone())
        .collect();
    if !moved.is_empty() && protocol <= published {
        return Err(format!(
            "these wire shapes changed at a published protocol version: {}. \
             A change to a shape already in use bumps the version: raise PROTOCOL \
             above the published floor of {published} in src/wire/hello/version.rs, \
             then run `make corpus`.",
            moved.join(", ")
        ));
    }
    let shapes = fresh
        .into_iter()
        .map(|(name, signature)| {
            let held = previous
                .shapes
                .get(&name)
                .filter(|e| e.signature == signature);
            let since = held.map_or(protocol, |entry| entry.since);
            (name, Entry { since, signature })
        })
        .collect();
    Ok(Ledger { protocol, shapes })
}
