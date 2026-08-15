//! The pin (VISION V1.2): the operator's fold over the answered spine, and the
//! one cut it makes seat-side.
//!
//! **It is a fold, and it stays here** (REMOTE §9.7, §8.5 unamended). Since
//! bl-44e9 every field a pin shows is on the answer — the commit and the cut
//! always were, the budget is a rollup on the notch, and the transcript-as-of
//! is a prefix of the chat the seat was answered — so what is left below is
//! *selection*, not derivation: pick a notch out of the landed rail, and take
//! the prefix of the landed transcript in front of it. Nothing about the
//! operator's selection crosses the wire; the two reads whose subject is a
//! different tree (Files, config-frozen-at) take that tree as a query
//! parameter, which is a question and not a view.
//!
//! Coverage-excluded like the rest of `shell/*`: both decisions here are calls
//! into a tested module ([`rail::pin`], [`rail::transcript_as_of`]).

use std::sync::Arc;

use crate::rail::{self, Pin, Rail};
use crate::transcript::Transcript;

/// The operator's pin, resolved against the rail. A selection the rail no
/// longer carries resolves to `None`, which is today's read — a re-derivation
/// that drops steps can never strand the inspector at a notch that is gone.
pub fn pinned(rail: &Rail, notch_sel: Option<usize>) -> Option<Pin> {
    rail::pin(rail, notch_sel)
}

/// The transcript, cut to the pin. Unpinned it is the landed answer, handed on
/// by pointer; pinned it is that answer's prefix as of the notch's read state,
/// which costs one clone of the entries in front of the pin and no second
/// question at all — the cut is a fold over what was already said.
pub fn transcript(live: &Arc<Transcript>, pin: Option<&Pin>) -> Arc<Transcript> {
    match pin {
        None => Arc::clone(live),
        Some(pin) => Arc::new(rail::transcript_as_of(live, pin.cut)),
    }
}
