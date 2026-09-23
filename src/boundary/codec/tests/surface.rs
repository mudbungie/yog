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
        super::workflow::surface(),
        super::query::surface(),
        login(),
        pins(),
        enroll(),
        spend(),
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

/// The §3.5 spend family (bl-53d1) — **one entry per arm of each act**: a row
/// priced on all four rates, one priced on two (the absent rates are the
/// table's own zeros), the delete, a ceiling set and a ceiling deleted, and
/// the table read beside them. The absences are the second instruction of
/// each act, so a fixture that only ever spelled a write would leave the
/// delete unproven on the wire.
fn spend() -> Vec<Gesture> {
    use crate::boundary::Action;
    let price = |model: &str, rates| {
        Gesture::Act(Action::Price {
            provider: "anthropic".to_owned(),
            model: model.to_owned(),
            rates,
        })
    };
    vec![
        price(
            "claude-opus-4-1",
            Some(crate::spend::Price {
                input: 15_000_000,
                output: 75_000_000,
                cache_read: 1_500_000,
                cache_write: 18_750_000,
            }),
        ),
        price(
            crate::spend::ANY,
            Some(crate::spend::Price {
                input: 3_000_000,
                output: 15_000_000,
                cache_read: 0,
                cache_write: 0,
            }),
        ),
        price("claude-opus-4-1", None),
        Gesture::Act(Action::Ceiling {
            micro_usd: Some(25_000_000),
        }),
        Gesture::Act(Action::Ceiling { micro_usd: None }),
        Gesture::Ask(crate::boundary::Query::Prices),
    ]
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
/// default would prove only that the default crosses. And one more with the
/// address the DEVICE will dial stated (bl-fec6, PROTOCOL 14), because an
/// optional field that no fixture carries is a field the corpus cannot see.
fn enroll() -> Vec<Gesture> {
    let request = |grade, address: Option<&str>| {
        Gesture::Act(crate::boundary::Action::Enroll(
            crate::registry::enroll::Request {
                workspace: "ws".to_owned(),
                name: "phone-1".to_owned(),
                grade,
                address: address.map(str::to_owned),
            },
        ))
    };
    vec![
        request(crate::registry::Grade::Operator, None),
        request(crate::registry::Grade::Foot, None),
        request(
            crate::registry::Grade::Operator,
            Some("engine.invalid:7737"),
        ),
    ]
}
