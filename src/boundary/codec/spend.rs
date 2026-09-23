//! The spend family's half of the [`codec`](super) (DESIGN §3.5; REMOTE
//! §9.23; bl-53d1): the price-table write, the ceiling write and the
//! table read, split from the top-level codec beside the capability family's
//! own half, per §12's line budget.
//!
//! **Money crosses as the USD decimal the operator quoted** — `15`, `1.5`,
//! `18.75` — and is held as micro-USD on this side (`spend::prices`): the one
//! `f64` is the read, and a negative or non-numeric figure refuses **by
//! name** here rather than reading as zero, because a gesture is an
//! instruction and a rate nobody typed must not be written.
//!
//! **Absence is the delete.** `price` with no `rates` key deletes the entry
//! and `ceiling` with no `usd` key deletes the number: an act with two
//! directions spells the second as the absence of the first's payload, the
//! `retire`/`fan` shape rather than the `pin`/`unpin` one, because there is
//! no second instruction here — there is one instruction with nothing in it.

use serde_json::{Map, Value, json};

use crate::spend::{Price, decimal, quoted};

use super::{Action, str_of};

/// The three op tokens, named once for the envelope, the line and the help
/// page (the `pin`/`unpin` precedent).
pub(crate) const PRICES: &str = "prices";
pub(crate) const PRICE: &str = "price";
pub(crate) const CEILING: &str = "ceiling";

/// The four rate keys, in the order a row is written — the §4.1 spelling.
const RATES: [&str; 4] = ["input", "output", "cache_read", "cache_write"];

/// The two acts as their envelopes — `op` is already known to be one.
pub(super) fn encode(action: &Action) -> Value {
    let mut map = Map::new();
    if let Action::Price {
        provider,
        model,
        rates,
    } = action
    {
        map.insert("op".to_owned(), json!(PRICE));
        map.insert("provider".to_owned(), json!(provider));
        map.insert("model".to_owned(), json!(model));
        if let Some(rates) = rates {
            map.insert("rates".to_owned(), Value::Object(rates.json_fields()));
        }
    } else {
        // A ceiling is the other act; the roster's match is exhaustive, so
        // every other variant is unreachable here and is not spelled.
        map.insert("op".to_owned(), json!(CEILING));
        if let Action::Ceiling {
            micro_usd: Some(micro),
        } = action
        {
            map.insert("usd".to_owned(), decimal(*micro));
        }
    }
    Value::Object(map)
}

/// The inverse, strict on every figure: `op` is one of the two.
pub(super) fn decode(op: &str, o: &Map<String, Value>) -> Result<Action, String> {
    if op == PRICE {
        return Ok(Action::Price {
            provider: str_of(o, "provider")?,
            model: str_of(o, "model")?,
            rates: match o.get("rates") {
                None | Some(Value::Null) => None,
                Some(rates) => Some(rates_of(rates)?),
            },
        });
    }
    Ok(Action::Ceiling {
        micro_usd: match o.get("usd") {
            None | Some(Value::Null) => None,
            Some(usd) => Some(usd_of(usd, "usd")?),
        },
    })
}

/// One `rates` object: every key optional and zero when absent (the table's
/// own default), every present one a non-negative number.
pub(crate) fn rates_of(v: &Value) -> Result<Price, String> {
    let o = v.as_object().ok_or("rates: not an object")?;
    let mut micro = [0u64; 4];
    for (slot, key) in micro.iter_mut().zip(RATES) {
        if let Some(rate) = o.get(key) {
            *slot = usd_of(rate, key)?;
        }
    }
    let [input, output, cache_read, cache_write] = micro;
    Ok(Price {
        input,
        output,
        cache_read,
        cache_write,
    })
}

/// One USD figure read strictly: present, numeric and non-negative, or the
/// refusal naming the key.
pub(crate) fn usd_of(v: &Value, key: &str) -> Result<u64, String> {
    quoted(Some(v)).ok_or_else(|| format!("{key}: {v} is not a non-negative USD figure"))
}
