//! The fixture the round trip is taken over (bl-7067): **one populated value
//! per [`Reply`] variant**, and where a variant's rows have arms of their own,
//! one row per arm — a listing whose rows are all the easy case proves only
//! that the easy case survives.
//!
//! It is deliberately a flat list rather than a table of names: the test walks
//! it, and a variant added tomorrow with no entry here leaves its own encode
//! arm unexecuted, which the coverage floor refuses. That is the gate a decoder
//! keyed on strings cannot get from the compiler.
//!
//! Split three ways at §12's budget, on the **same seam the decoder is cut
//! along** (`decode`'s `receipt`/`listing`/`inspector` chain), so a variant's
//! fixture and its arm are found by the same reading of §8.5's taxonomy.

mod agent;
mod board;
mod inspector;
mod listings;
mod receipts;

use super::super::super::Reply;
use crate::budgets::BudgetSpend;
use crate::files_view::Preview;

/// The four ARCH §6 counters, all distinct, so a decoder that transposed two
/// of them would not pass by luck.
fn spend() -> BudgetSpend {
    BudgetSpend {
        input_tokens: 11,
        output_tokens: 22,
        cache_read_tokens: 33,
        cache_write_tokens: 44,
    }
}

/// The bounded-preview arm the work diff and the Files tab both spend.
fn preview() -> Preview {
    Preview::Truncated {
        text: "head".into(),
        size: 9_000,
    }
}

/// Every variant, populated, in [`Reply`]'s own order.
pub(super) fn surface() -> Vec<Reply> {
    [
        receipts::receipts(),
        listings::listings(),
        inspector::inspector(),
        agent::agent(),
    ]
    .concat()
}
