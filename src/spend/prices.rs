//! The price table — yog world config (DESIGN §3.5 and §4.1 `prices`).
//!
//! **Keyed by `(provider row, model)`, and a row may price a wildcard**
//! (bl-53d1). The row a call went through is what a token costs: a
//! subscription row bills a model id at nothing marginal while an API-key
//! row bills the same id at list, and one key could not say both. So the
//! document is `{ "<provider row>": { "<model>": <rates>, "*": <rates> } }`,
//! the provider row being the brazen row name litany hands `bz --provider`
//! and the step's `meta.json` names, the model the string the step's own
//! `request.json` carries. A row's [`ANY`] entry is its rate for every model
//! it does not name, so a subscription row is one entry — `{ "*": {} }` — that
//! **prices** everything it serves at `$0.00`, which is a different fact from
//! *unpriced* and is said by the figure's `cost` being present at zero rather
//! than absent. No status vocabulary is added for it.
//!
//! **Severable by construction:** delete the key and every cost figure
//! disappears, deleting a column and not a code path — [`Prices::is_empty`]
//! is the one gate, and an empty table is the default. **No migration:** a
//! document in the pre-bl-53d1 model-keyed shape reads as *empty* under the
//! forgiving read — its rates are not objects where this shape wants a model
//! map — because it was never seeded and existed only where an operator wrote
//! it by hand on the engine's own box, and the boundary's `/price` is how it
//! comes back.
//!
//! **No crate below yog learns a price** (§3.5): brazen counts tokens, litany
//! commits them into step records, balls stays metric-free. The rate lives
//! here and nowhere else. The parse of one rate and its arithmetic are
//! [`parse`]'s; this file is the table and its one lookup.

use std::collections::BTreeMap;

use serde_json::{Map, Value};

mod parse;
pub use parse::{MICRO_PER_CENT, MICRO_PER_USD, Price};
pub(crate) use parse::{decimal, parse_usd, quoted};

/// The wildcard model key (§4.1): a row's rate for any model it does not name.
pub const ANY: &str = "*";

/// Provider row → model id → [`Price`]. Empty is the default and means
/// *unpriced*, not free: [`super::figure`] renders no cost at all rather than
/// `$0.00`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Prices(BTreeMap<String, BTreeMap<String, Price>>);

/// One table entry, flattened for the wire (REMOTE §9.23): the
/// row, the model — [`ANY`] carried as the literal string — and its rates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriceRow {
    pub provider: String,
    pub model: String,
    pub rates: Price,
}

impl Prices {
    /// Read the table from `ui.json`'s `prices` value (§4.1). Forgiving like
    /// every other `ui.json` read: a non-object document, a non-object row, a
    /// non-object rate or a non-numeric figure degrades to "absent" rather
    /// than refusing to load — a typo in a hand-edited table must not cost the
    /// operator the window. A row left with no priced model is no row at all,
    /// which is what makes the old model-keyed shape read as empty.
    pub fn from_json(value: &Value) -> Self {
        let Some(rows) = value.as_object() else {
            return Self::default();
        };
        Self(
            rows.iter()
                .filter_map(|(provider, row)| {
                    let models: BTreeMap<String, Price> = row
                        .as_object()?
                        .iter()
                        .filter_map(|(model, rate)| Some((model.clone(), Price::from_json(rate)?)))
                        .collect();
                    (!models.is_empty()).then(|| (provider.clone(), models))
                })
                .collect(),
        )
    }

    /// The table as the `ui.json` value it is read back from — the one
    /// spelling, so a write-through lands exactly what the next read sees.
    pub fn to_json(&self) -> Value {
        let rows: Map<String, Value> = self
            .0
            .iter()
            .map(|(provider, models)| {
                let row: Map<String, Value> = models
                    .iter()
                    .map(|(model, rate)| (model.clone(), Value::Object(rate.json_fields())))
                    .collect();
                (provider.clone(), Value::Object(row))
            })
            .collect();
        Value::Object(rows)
    }

    /// No rate for anything — the severability gate (§3.5).
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// **The one lookup** (§3.5): the rate a step's `(provider, model)` takes.
    /// With the provider named, the exact entry, else that row's [`ANY`], else
    /// `None`. With no provider — every step record written before litany's
    /// `meta.json` gained one (VISION §6 item 8) — the filter is empty: the
    /// model is matched over every row, `Some` when exactly one row prices it
    /// and `None` when more than one does, ambiguity reported the way an
    /// unknown model is. Not a version branch: the same rule with the filter
    /// empty. A step naming no model is unpriced either way.
    pub fn of(&self, provider: Option<&str>, model: Option<&str>) -> Option<Price> {
        let model = model?;
        let row = |models: &BTreeMap<String, Price>| {
            models.get(model).or_else(|| models.get(ANY)).copied()
        };
        if let Some(provider) = provider {
            return row(self.0.get(provider)?);
        }
        let mut candidates = self.0.values().filter_map(row);
        match (candidates.next(), candidates.next()) {
            (Some(price), None) => Some(price),
            _ => None,
        }
    }

    /// The table flattened, row by row and model by model, in key order.
    pub fn rows(&self) -> Vec<PriceRow> {
        self.0
            .iter()
            .flat_map(|(provider, models)| {
                models.iter().map(|(model, rates)| PriceRow {
                    provider: provider.clone(),
                    model: model.clone(),
                    rates: *rates,
                })
            })
            .collect()
    }

    /// Write one entry (`Some`) or delete it (`None`) — `Action::Price`'s
    /// whole effect. A row emptied by a delete goes with its last entry, so
    /// the document never carries a row that prices nothing.
    pub fn set(&mut self, provider: &str, model: &str, rates: Option<Price>) {
        match rates {
            Some(rates) => {
                self.0
                    .entry(provider.to_owned())
                    .or_default()
                    .insert(model.to_owned(), rates);
            }
            None => {
                if let Some(models) = self.0.get_mut(provider) {
                    models.remove(model);
                    if models.is_empty() {
                        self.0.remove(provider);
                    }
                }
            }
        }
    }
}
