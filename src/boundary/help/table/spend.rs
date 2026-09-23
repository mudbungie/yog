//! The §3.5 spend family's three pages (bl-53d1): the table read, the row
//! write and the ceiling — split off at §12's budget on the family's own
//! seam, and joined back after the standing verbs, whose subject (a setting
//! with no conversation under it) these share.

use super::super::{HelpRow, Surface};

/// The reasoning paragraph every gesture help carries, said once for the
/// family: what a rate is, what a row is, and what the engine deliberately
/// does not check.
pub const SPEND: &[HelpRow] = &[
    HelpRow {
        verb: crate::boundary::codec::spend::PRICES,
        usage: "/prices",
        summary: "the price table, the spend ceiling, and what this world has spent against it",
        detail: "Answers every priced (provider, model) row with its four rates in USD per \
                 million tokens, the ceiling in USD when one is set, and the whole world's \
                 priced spend — the ledger the ceiling is compared against. The table and the \
                 ceiling are world facts: one `ui.json` on the engine, shared by every seat, \
                 addressed to no workspace. An empty table means no cost is answered anywhere \
                 and no ceiling can bind; a step whose row is unpriced is counted in \
                 `unpriced_tokens` beside the money, so a partial table reads as *at least*.",
        surface: Surface::Control,
    },
    HelpRow {
        verb: crate::boundary::codec::spend::PRICE,
        usage: crate::boundary::line::PRICE_USAGE,
        summary: "price one (provider, model) row in USD per million tokens, or delete it",
        detail: "Writes one row of the price table, keyed by the provider row a call went \
                 through and the model it named — the same model id bills at list on an API-key \
                 row and at nothing on a subscription row, which is why the key is the pair. \
                 The rates are input, output, cache read and cache write, quoted in USD per \
                 million tokens exactly as a provider's price page prints them; an omitted rate \
                 is zero. `*` as the model prices every model that row serves and does not name, \
                 so a subscription is `/price <row> * 0 0`: priced, and pricing to $0.00, which \
                 is a different fact from unpriced. `off` in place of the rates deletes the row. \
                 A negative or unreadable figure is refused. A provider row this workspace's \
                 wall does not know is NOT refused — a row may be priced before it is \
                 configured, and the name is the one `/providers` lists. The reply is the \
                 re-read table, never an echo.",
        surface: Surface::Control,
    },
    HelpRow {
        verb: crate::boundary::codec::spend::CEILING,
        usage: crate::boundary::line::CEILING_USAGE,
        summary: "set the world's spend ceiling in USD, or remove it",
        detail: "Writes the one number this whole world's priced spend must stay under, or \
                 deletes it with `off`. At or over it, no new conversation is started anywhere \
                 and every running one parks at its next tool call — a hold, never a kill, so \
                 no uncommitted work is lost — until the number moves. It needs a price table \
                 to mean anything: with none, nothing is bounded. Raising it back over the \
                 world's spend, or removing it, releases every conversation the ceiling parked, \
                 and only those — a floor's park or a policy hold is untouched — and the reply \
                 says how many woke as `released`. A literal 0 is the hard stop that starts \
                 nothing new. A negative or unreadable figure is refused.",
        surface: Surface::Control,
    },
];
