//! **The shell's one spelling of a wire read** (REMOTE §1.2 and its
//! read-path residual; bl-adcb).
//!
//! Since the 2026-08-14 ruling the window is a client of its own engine over
//! loopback mTLS, so a surface that used to derive its content in process now
//! declares a standing question and paints whatever answer has landed
//! ([`AppModel::wire_ask`]). bl-ae05 wrote that shape out once, by hand, for the
//! clients section; a second copy of it in every migrated surface would be four
//! subtle arms restated per pane, and the subtle one is the third.
//!
//! The four states a read can be in, and what each paints:
//!
//! - **an answer of the expected kind** — its payload, which is the surface's
//!   whole content;
//! - **a refusal** — the engine's own sentence, painted rather than swallowed,
//!   because the wire is how this window reads and being told *no* is content;
//! - **nothing yet** — the honest empty state. The frame order is
//!   settle-then-render, so the first frame that declares a question paints
//!   before it has been asked and the answer lands one
//!   [`ASK_PERIOD`](crate::wire::asker::ASK_PERIOD) later;
//! - **an answer of another kind** — a codec that has drifted from the query it
//!   answers, which is a defect rather than a state (the round-trip tests are
//!   its witness). Nothing to paint, and nothing invented.
//!
//! **A surface that stops asking stops being asked.** The question is keyed by
//! its own encoded envelope, so a pane behind a collapsed header simply does not
//! call this and its answer is dropped at the next settle — no unsubscribe, and
//! no bookkeeping here to forget one.
//!
//! Coverage-excluded glue like the rest of `shell/*`: every decision below is a
//! match over [`Reply`], and the decode it matches on is covered where it lives.

use crate::AppModel;
use crate::boundary::reply::Reply;
use crate::boundary::{Gesture, Query, codec};

/// What one standing question has earned this frame: the payload if an answer
/// of the expected kind has landed, and the engine's sentence if it refused.
/// Both empty is the resting state of a question asked a moment ago.
pub(super) struct Landed<T> {
    pub(super) value: Option<T>,
    pub(super) refused: Option<String>,
}

/// **Nothing asked, nothing said** — what a surface that skipped the ask holds,
/// so a collapsed pane and an unanswered one are one code path rather than two.
/// Hand-written rather than derived because `T` need not be [`Default`]: the
/// absence is the `Option`'s, never a zero value of the payload's own type.
impl<T> Default for Landed<T> {
    fn default() -> Self {
        Self {
            value: None,
            refused: None,
        }
    }
}

/// Declare `query` standing and read whatever has landed for it, `take` picking
/// the payload out of the one [`Reply`] variant that query answers.
///
/// Never blocks and never dials: [`AppModel::wire_ask`] is a map read, and the
/// [`Asker`](crate::wire::asker::Asker) does the socket work off-frame at human
/// cadence.
pub(super) fn ask<T>(
    model: &mut AppModel,
    query: Query,
    take: fn(Reply) -> Option<T>,
) -> Landed<T> {
    let envelope = codec::encode(&Gesture::Ask(query));
    match model.wire_ask(&envelope) {
        Some(Ok(reply)) => Landed {
            value: take(reply),
            refused: None,
        },
        Some(Err(said)) => Landed {
            value: None,
            refused: Some(said),
        },
        None => Landed::default(),
    }
}
