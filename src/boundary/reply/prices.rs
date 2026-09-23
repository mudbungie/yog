//! **The price table, the ceiling and the world's ledger as one answer**
//! (DESIGN §3.5, §4.1; REMOTE §9.23; bl-53d1) — the type
//! [`Query::Prices`](crate::boundary::Query::Prices) answers and both spend
//! acts receipt with, and both directions of its spelling, cut off the
//! roster on `agent`'s seam: one payload whose rows are its own vocabulary.
//!
//! **Rates cross as the decimals the operator quoted** — `15`, `1.5`,
//! `18.75`, USD per million tokens — read back exactly as written, and the
//! ceiling as its USD number. `spent` is the world's priced figure in the one
//! money spelling every carrier shares ([`super::cost`]). Three keys are
//! absent rather than null, each for the reading its absence is: no
//! `ceiling` is no bound, no `spent` is an unpriced world, and no `released`
//! is an answer nothing was released by — a read, or a row write.

use serde_json::{Map, Value, json};

use crate::boundary::codec::fields::{list_of, opt, str_of, usize_of};
use crate::boundary::codec::spend::{rates_of, usd_of};
use crate::spend::{Ceiling, Cost, PriceRow, decimal};

/// The reply kind, named once for both directions.
pub(super) const KIND: &str = "prices";

/// What the table read answers and the two acts receipt with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PricesView {
    /// The table flattened, in key order — a `*` model as the literal string.
    pub rows: Vec<PriceRow>,
    /// The §3.5 bound; the default is no ceiling, and reads as no key.
    pub ceiling: Ceiling,
    /// The world's priced spend, `None` for an unpriced world.
    pub spent: Option<Cost>,
    /// How many ceiling-parked conversations a `/ceiling` act drove on —
    /// `Some` only on that act's receipt, which is the one answer that could
    /// have released anything.
    pub released: Option<usize>,
}

/// The whole answer as one object.
pub(super) fn reply(view: &PricesView) -> Value {
    let mut map = Map::new();
    map.insert("ok".to_owned(), json!(true));
    map.insert("kind".to_owned(), json!(KIND));
    map.insert(
        "rows".to_owned(),
        Value::Array(view.rows.iter().map(row).collect()),
    );
    if let Some(micro) = view.ceiling.micro_usd() {
        map.insert("ceiling".to_owned(), decimal(micro));
    }
    super::cost::opt_cost("spent", view.spent.as_ref(), &mut map);
    if let Some(released) = view.released {
        map.insert("released".to_owned(), json!(released));
    }
    Value::Object(map)
}

/// One row: the pair, then the four rates flat beside it.
fn row(row: &PriceRow) -> Value {
    let mut map = Map::new();
    map.insert("provider".to_owned(), json!(row.provider));
    map.insert("model".to_owned(), json!(row.model));
    map.append(&mut row.rates.json_fields());
    Value::Object(map)
}

/// The same object read back, strict on every figure: a rate that is not a
/// non-negative number refuses naming it, exactly as the gesture that would
/// have written it refuses.
pub(super) fn view_of(o: &Map<String, Value>) -> Result<PricesView, String> {
    Ok(PricesView {
        rows: list_of(o, "rows", row_of)?,
        // A present ceiling must be a non-negative number: read strictly
        // here, then handed to the same forgiving read `ui.json` takes.
        ceiling: match o.get("ceiling") {
            Some(usd) => {
                usd_of(usd, "ceiling")?;
                Ceiling::from_json(Some(usd))
            }
            None => Ceiling::default(),
        },
        spent: crate::boundary::codec::fields::opt_val(o, "spent", super::cost::cost_of)?,
        released: opt(o, "released", usize_of)?,
    })
}

fn row_of(v: &Value) -> Result<PriceRow, String> {
    let o = v.as_object().ok_or("price row: not an object")?;
    Ok(PriceRow {
        provider: str_of(o, "provider")?,
        model: str_of(o, "model")?,
        rates: rates_of(v)?,
    })
}
