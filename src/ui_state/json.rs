//! `ui.json` document helpers: forgiving parse, the default root, typed
//! array reads, and the object-descent coercion. All private to the
//! [`ui_state`](super) module (`pub(super)`), so no borrow-returning `descend`
//! reaches the crate's public surface.

use serde_json::{Map, Value};

/// Forgiving parse: a JSON object ⇒ the root map; anything else ⇒ default doc.
pub(super) fn parse_or_default(bytes: &[u8]) -> Map<String, Value> {
    match serde_json::from_slice::<Value>(bytes) {
        Ok(Value::Object(m)) => m,
        _ => default_root(),
    }
}

pub(super) fn default_root() -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("v".to_string(), Value::from(1));
    m
}

pub(super) fn string_array(map: &Map<String, Value>, key: &str) -> Vec<String> {
    let Some(arr) = map.get(key).and_then(Value::as_array) else {
        return Vec::new();
    };
    arr.iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect()
}

/// Ensure `map[key]` is an object and return it (coerce absent/wrong-typed).
pub(super) fn descend(map: &mut Map<String, Value>, key: String) -> &mut Map<String, Value> {
    let slot = map.entry(key).or_insert_with(|| Value::Object(Map::new()));
    if !slot.is_object() {
        *slot = Value::Object(Map::new());
    }
    match slot {
        Value::Object(inner) => inner,
        // Coerced to an object just above, so this arm is dead; `unreachable!`
        // keeps `descend` total without a reachable panic in prod.
        _ => unreachable!(),
    }
}
