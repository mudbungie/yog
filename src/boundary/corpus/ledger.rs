//! **The corpus's standing record** (bl-32cb, reshaped bl-e598): per shape,
//! every field path its fixtures spell and the **edition** at which that path
//! first appeared.
//!
//! It exists to make REMOTE §3.2's two rules mechanical rather than
//! remembered. *An addition is free and is stamped*: a path the boundary
//! gained takes the next edition, and nothing else moves. *A removal or a
//! re-type is a MAJOR bump*: a path that vanished, or a key that gained a
//! second JSON type, is refused unless the repo-root `PROTOCOL` file has been
//! raised above the published one AND the path was deprecated first
//! ([`super::DEPRECATED`]). The fixtures alone cannot enforce either, because
//! regenerating them makes any diff vanish; the record remembers what the
//! shapes *were*.
//!
//! **Three integers, and what each is.** `protocol` is the major the corpus
//! is for — the number the hello compares. The **edition** is the newest stamp
//! in the record, computed and never stored: the old per-bump integer line
//! continued, so the stamps 1..18 a client already vendors keep their meaning.
//! `floor` is the edition at which the current major was cut: every path
//! stamped at or below it is present on every engine speaking this major, and
//! every later path is optional to READ — absent on an engine of an older
//! edition, which the hello's `edition` lets a seat say honestly. It moves in
//! the one regeneration that raises `protocol`, and never after, so a lane
//! that adds a field in the same unreleased wave as a bump lands it
//! post-floor: over-cautious, and safe in the only direction that matters.
//!
//! **A signature is field paths and their JSON types, not bytes.** Adding a
//! sample to a shape leaves it alone, which is right: a new fixture is not a
//! wire change. A new key is a gain; a renamed key is a loss and a gain; a
//! key that spells a second type is a re-type. A new WORD in a vocabulary is
//! invisible here by design — it is additive, and the reader's catch-all is
//! what makes it so (REMOTE §3.2).

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value, json};

use super::Shape;

/// One shape's record: each path, with the edition it appeared at.
#[derive(Debug)]
pub(super) struct Entry {
    pub(super) signature: BTreeMap<String, u32>,
}

/// Every shape's record, the major it is for, the floor of that major, and
/// what is deprecated ([`super::DEPRECATED`], rendered so a consumer reads it
/// where it reads the rest).
#[derive(Debug)]
pub(super) struct Ledger {
    pub(super) protocol: u32,
    pub(super) floor: u32,
    pub(super) deprecated: BTreeSet<String>,
    pub(super) shapes: BTreeMap<String, Entry>,
}

impl Ledger {
    /// Read a committed record. Anything unreadable is an empty record — the
    /// same answer a corpus that does not exist yet gives, and the gate then
    /// asks for a regeneration rather than for a version bump.
    pub(super) fn read(text: &str) -> Self {
        let value = serde_json::from_str::<Value>(text).unwrap_or(Value::Null);
        let mut shapes = BTreeMap::new();
        for (name, entry) in value
            .get("shapes")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
        {
            let signature = entry
                .get("signature")
                .and_then(Value::as_object)
                .into_iter()
                .flatten()
                .map(|(path, at)| (path.clone(), number(Some(at))))
                .collect();
            shapes.insert(name.clone(), Entry { signature });
        }
        let deprecated = value
            .get("deprecated")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect();
        Self {
            protocol: number(value.get("protocol")),
            floor: number(value.get("floor")),
            deprecated,
            shapes,
        }
    }

    /// The newest stamp in the record: the edition this corpus is at.
    pub(super) fn edition(&self) -> u32 {
        self.shapes
            .values()
            .flat_map(|entry| entry.signature.values().copied())
            .max()
            .unwrap_or_default()
    }

    /// The record's own canonical bytes.
    pub(super) fn render(&self) -> String {
        let shapes: Map<String, Value> = self
            .shapes
            .iter()
            .map(|(name, entry)| (name.clone(), json!({ "signature": entry.signature })))
            .collect();
        let doc = json!({
            "deprecated": self.deprecated,
            "floor": self.floor,
            "protocol": self.protocol,
            "shapes": shapes,
        });
        super::canonical(&doc)
    }
}

fn number(value: Option<&Value>) -> u32 {
    let raw = value.and_then(Value::as_u64).unwrap_or_default();
    u32::try_from(raw).unwrap_or_default()
}

/// Every field path a shape's frames spell, with the JSON type found there.
/// Array elements collapse to one `[]` step, so a two-element list and a
/// one-element list of the same rows are one signature.
pub(super) fn signature(frames: &[Value]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for frame in frames {
        walk("", frame, &mut out);
    }
    out
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

/// The path without its type: what a key IS, as against what it spells.
fn key_of(path: &str) -> &str {
    path.rsplit_once(':').map_or(path, |(key, _)| key)
}

/// Whether a vanished path, or its whole shape, was deprecated first.
fn deprecated(list: &[&str], shape: &str, path: &str) -> bool {
    let named = format!("{shape}{}", key_of(path));
    list.contains(&shape) || list.contains(&named.as_str())
}

/// The record this boundary earns, or the refusal that says why it cannot
/// have one. A gained path is stamped the next edition; a path that vanished
/// or re-typed is refused unless `protocol` exceeds `published` — a major bump
/// in flight — and, for a loss, the path is in `deprecated`.
pub(super) fn advance(
    shapes: &[Shape],
    previous: &Ledger,
    protocol: u32,
    published: u32,
    deprecated: &[&str],
) -> Result<Ledger, String> {
    let fresh: BTreeMap<String, BTreeSet<String>> = shapes
        .iter()
        .map(|shape| (shape.key(), signature(&shape.frames)))
        .collect();
    let next = previous.edition() + 1;
    let bumping = protocol > published;
    let mut breaking = Vec::new();
    for (name, entry) in &previous.shapes {
        let now = fresh.get(name);
        for path in entry.signature.keys() {
            if now.is_some_and(|paths| paths.contains(path)) {
                continue;
            }
            match (bumping, self::deprecated(deprecated, name, path)) {
                (true, true) => {}
                (true, false) => breaking.push(format!("{name}{path} vanished undeprecated")),
                (false, _) => breaking.push(format!("{name}{path} vanished")),
            }
        }
    }
    let mut out = BTreeMap::new();
    for (name, paths) in fresh {
        let held = previous.shapes.get(&name);
        let mut signature = BTreeMap::new();
        for path in paths {
            let stamp = held.and_then(|entry| entry.signature.get(&path).copied());
            let retyped = held.is_some_and(|entry| {
                stamp.is_none()
                    && entry
                        .signature
                        .keys()
                        .any(|known| key_of(known) == key_of(&path))
            });
            if retyped && !bumping {
                breaking.push(format!("{name}{path} re-typed a key in use"));
            }
            signature.insert(path, stamp.unwrap_or(next));
        }
        out.insert(name, Entry { signature });
    }
    if !breaking.is_empty() {
        return Err(format!(
            "these wire changes break a reader of the published protocol {published}: {}. \
             A field or a spelling is removed or re-typed only at a MAJOR bump: list it in \
             corpus::DEPRECATED, raise the number in the repo-root PROTOCOL file, then run \
             `make corpus`.",
            breaking.join(", ")
        ));
    }
    let ledger = Ledger {
        protocol,
        floor: previous.floor,
        deprecated: deprecated.iter().map(|&s| s.to_owned()).collect(),
        shapes: out,
    };
    let floor = if protocol > previous.protocol {
        ledger.edition()
    } else {
        previous.floor
    };
    Ok(Ledger { floor, ..ledger })
}
