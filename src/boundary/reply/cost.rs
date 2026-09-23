//! **The one spelling of money on the wire** (REMOTE §9.23 —
//! DESIGN §3.5, bl-53d1): the §3.5 [`Cost`] as the three keys the board's
//! ball rows, `reply/workspace-balls` and `reply/agent` already carried —
//! `micro_usd`, `usd` and `unpriced_tokens` — factored out of
//! [`figure_value`](super::board::figure_value) so every carrier of a token
//! count spells its cost identically: a step row, a science attempt, a rail
//! notch and a workspace row each carry it as one `cost`/`spend` object, and
//! the figure carries the same three keys flat beside its tokens.
//!
//! **Absent, never zero, when the price table is empty**: the `Option` is the
//! §3.5 severability gate crossing the wire, and a seat that reads no key
//! reads *unpriced*. A `$0.00` is a priced figure (a subscription row);
//! `unpriced_tokens` above zero makes the figure a floor. Derived text rides
//! beside the fact (`usd` beside `micro_usd`), as REMOTE §9.7 ruled: the box
//! that holds the rates is the only box that may say the number, so a seat
//! paints `usd` verbatim and never multiplies — and `usd` is dropped on the
//! way back in, because [`Cost::usd`] is its one authority.

use serde_json::{Map, Value, json};

use crate::boundary::codec::fields::u64_of;
use crate::spend::Cost;

/// The three keys, written into `map` — the figure's own flat shape.
pub(crate) fn cost_fields(cost: &Cost, map: &mut Map<String, Value>) {
    map.insert("usd".to_owned(), json!(cost.usd()));
    map.insert("micro_usd".to_owned(), json!(cost.micro_usd));
    map.insert("unpriced_tokens".to_owned(), json!(cost.unpriced_tokens));
}

/// The three keys as one object — what a carrier writes under `cost` or
/// `spend` when it has a figure to state.
pub(crate) fn cost_value(cost: &Cost) -> Value {
    let mut map = Map::new();
    cost_fields(cost, &mut map);
    Value::Object(map)
}

/// The same object into `map` under `key`, or nothing at all: absent is the
/// empty table, and a reader must never have to tell that from a `$0.00`.
pub(crate) fn opt_cost(key: &str, cost: Option<&Cost>, map: &mut Map<String, Value>) {
    if let Some(cost) = cost {
        map.insert(key.to_owned(), cost_value(cost));
    }
}

/// The three keys read back off `map`, `None` when the money key is absent —
/// the figure's flat shape, and a carrier's object once it is opened. Strict
/// on the two facts it keeps: a present `micro_usd` without its
/// `unpriced_tokens` is a figure nobody wrote.
pub(crate) fn fields_of(map: &Map<String, Value>) -> Result<Option<Cost>, String> {
    if !map.contains_key("micro_usd") {
        return Ok(None);
    }
    Ok(Some(Cost {
        micro_usd: u64_of(map, "micro_usd")?,
        unpriced_tokens: u64_of(map, "unpriced_tokens")?,
    }))
}

/// One carrier's `cost`/`spend` object read back. A present key must be the
/// object with its money in it: absent is the one spelling of *unpriced*.
pub(crate) fn cost_of(v: &Value) -> Result<Cost, String> {
    let map = v.as_object().ok_or("cost: not an object")?;
    fields_of(map)?.ok_or_else(|| "cost: missing field \"micro_usd\"".to_owned())
}
