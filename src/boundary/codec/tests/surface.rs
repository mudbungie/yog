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
        login(),
        pins(),
        enroll(),
        crate::boundary::codec::config::tests::surface(),
    ]
    .concat()
}

/// The §4.1 pin (bl-b986) — **one entry per direction**, because the direction
/// is the op token and a fixture that only ever spelled `pin` would leave the
/// instruction that is not the default unproven on the wire.
fn pins() -> Vec<Gesture> {
    [true, false]
        .into_iter()
        .map(|pinned| {
            Gesture::Act(crate::boundary::Action::Pin {
                workspace: "ws".to_owned(),
                pinned,
            })
        })
        .collect()
}

/// The §8.3 sign-in (REMOTE §8.3, bl-c285) — one entry, because the act's
/// whole envelope is the pair it names: the flow is the row's own capability
/// and never a field a seat spells (DESIGN §8.3 rule 1), so there is no arm
/// here for a fixture to walk.
fn login() -> Vec<Gesture> {
    vec![Gesture::Act(crate::boundary::Action::Login {
        workspace: "ws".to_owned(),
        provider: "acme".to_owned(),
    })]
}

/// REMOTE §1.4's enrollment (bl-f4e3) — **one entry per grade**, because the
/// grade is a two-armed vocabulary and a fixture that only ever spelled the
/// default would prove only that the default crosses.
fn enroll() -> Vec<Gesture> {
    [
        crate::registry::Grade::Operator,
        crate::registry::Grade::Foot,
    ]
    .into_iter()
    .map(|grade| {
        Gesture::Act(crate::boundary::Action::Enroll(
            crate::registry::enroll::Request {
                workspace: "ws".to_owned(),
                name: "phone-1".to_owned(),
                grade,
            },
        ))
    })
    .collect()
}
