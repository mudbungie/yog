//! The §3.5 spend family, typed (bl-53d1): `/price`, `/ceiling` and the
//! `/prices` read — its own file at §12's cap on the seam the other
//! per-family grammars are cut on.
//!
//! **Neither act takes anything from the seat.** The table and the ceiling
//! are world facts (DESIGN §4.1), so every word is stated on the line and the
//! context supplies nothing — which is what makes them typable from a seat
//! holding no selection at all, the argv terminal included.
//!
//! **A figure typed is read once, as the USD decimal it is** — `15`, `1.5`,
//! `18.75` — and refused by name when it is not a non-negative number: a rate
//! nobody typed must not be written, and `off` is the one word that is not a
//! figure, spelling the delete on both verbs.

use super::parse::{act, ask};
use super::{Context, args};
use crate::boundary::codec::spend::{CEILING, PRICE, PRICES};
use crate::boundary::{Action, Gesture, Query};
use crate::spend::{Price, decimal, parse_usd};

/// The write's usage line, shared by the reader's refusal and the help page.
pub const PRICE_USAGE: &str = "/price <provider> <model> <input> <output> [<cache_read> [<cache_write>]] | /price <provider> <model> off";
/// The ceiling's usage line, shared the same way.
pub const CEILING_USAGE: &str = "/ceiling <usd> | /ceiling off";

/// The word that spells a delete on either verb.
const OFF: &str = "off";

/// Read one of the family's three lines, or hand a verb that is not one of
/// them on to the populating reads — the grammar's one dead end — so the
/// router's table stays inside its own line budget.
pub(super) fn read(verb: &str, tail: &str, ctx: &Context) -> Result<Gesture, String> {
    match verb {
        PRICES => args::none(tail, verb).map(|()| ask(Query::Prices)),
        PRICE => price(tail),
        CEILING => ceiling(tail),
        _ => super::queries::queries(verb, tail, ctx),
    }
}

/// `/price <provider> <model> <input> <output> [<cache_read> [<cache_write>]]`,
/// or `off` in place of the rates. An omitted rate is zero — the table's own
/// default — so a row priced on two figures leaves cache traffic at nothing
/// rather than guessing it.
fn price(tail: &str) -> Result<Gesture, String> {
    let words: Vec<&str> = tail.split_whitespace().collect();
    let (provider, model, rates) = match words.as_slice() {
        [provider, model, word] if *word == OFF => (*provider, *model, None),
        [provider, model, rates @ ..] if (2..=4).contains(&rates.len()) => {
            let mut micro = [0u64; 4];
            for (slot, word) in micro.iter_mut().zip(rates) {
                *slot = figure(word, PRICE)?;
            }
            let [input, output, cache_read, cache_write] = micro;
            (
                *provider,
                *model,
                Some(Price {
                    input,
                    output,
                    cache_read,
                    cache_write,
                }),
            )
        }
        _ => return Err(format!("/{PRICE}: usage: {PRICE_USAGE}")),
    };
    Ok(act(Action::Price {
        provider: provider.to_owned(),
        model: model.to_owned(),
        rates,
    }))
}

/// `/ceiling <usd>` or `/ceiling off`.
fn ceiling(tail: &str) -> Result<Gesture, String> {
    let micro_usd = match args::optional_word(tail, CEILING)?.as_deref() {
        Some(OFF) => None,
        Some(word) => Some(figure(word, CEILING)?),
        None => return Err(format!("/{CEILING}: usage: {CEILING_USAGE}")),
    };
    Ok(act(Action::Ceiling { micro_usd }))
}

/// One typed USD figure, or the refusal naming it and the verb.
fn figure(word: &str, verb: &str) -> Result<u64, String> {
    parse_usd(word).ok_or_else(|| format!("/{verb}: {word:?} is not a non-negative USD figure"))
}

/// The two acts back to the lines that spell them — the decimals exactly as
/// the reader takes them, and `off` for both absences.
pub(super) fn spell(action: &Action) -> String {
    match action {
        Action::Price {
            provider,
            model,
            rates: Some(rates),
        } => format!(
            "/{PRICE} {provider} {model} {} {} {} {}",
            decimal(rates.input),
            decimal(rates.output),
            decimal(rates.cache_read),
            decimal(rates.cache_write)
        ),
        Action::Price {
            provider, model, ..
        } => format!("/{PRICE} {provider} {model} {OFF}"),
        Action::Ceiling {
            micro_usd: Some(micro),
        } => format!("/{CEILING} {}", decimal(*micro)),
        // The roster's match is exhaustive one level up, so the only other
        // action that reaches here is the ceiling's delete.
        _ => format!("/{CEILING} {OFF}"),
    }
}
