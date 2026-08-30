//! **The gesture surface** (bl-32cb): one populated value per boundary
//! spelling, assembled from the family lists the round trips are already cut
//! along.
//!
//! It has two readers and that is the point. The codec's own round trip walks
//! it — encode → decode is the identity over every entry — and
//! [`crate::boundary::corpus`] renders it into the conformance corpus every
//! wire client replays. One list, so a fixture a client is judged against and a
//! fixture yog proves itself against can never be two different things.
//!
//! Where a variant carries an enum, a bounded option or a collection, the list
//! holds **one entry per arm** and the empty case beside the populated one: a
//! table that only ever spells the easy case proves only that the easy case
//! crosses.

mod ball;
mod conversation;

use crate::boundary::Gesture;

/// Every gesture, in family order. Deterministic — the corpus is a committed
/// artifact, so the order this returns is part of what is committed.
pub(crate) fn gestures() -> Vec<Gesture> {
    [
        conversation::surface(),
        ball::surface(),
        super::start::surface(),
        super::fan::surface(),
        super::fork::surface(),
        super::control::surface(),
        super::fleet::surface(),
        super::retarget::surface(),
        super::query::surface(),
        crate::boundary::codec::config::tests::surface(),
    ]
    .concat()
}
