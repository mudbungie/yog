//! **What a certificate authorizes** (REMOTE §4.2, bl-1dd3): the grade its
//! subject carries, and the peer an intake answers as.
//!
//! [`Client`] is *who* is asking — one certificate, one identity, one directory
//! under the registry root. This is *what that identity may say*, and the two
//! are deliberately separate values: the identity keys the presence map, the
//! mailbox and every registration on disk, so folding a second fact into it
//! would make a peer that connected under one grade a different key from the
//! same client read off the `clients/` listing.
//!
//! **There are exactly two grades and neither is configured.** An operator-grade
//! caller has the whole boundary, within the registrations §4 already scopes.
//! A foot may say three gestures — advertise its tool set, take the invocations
//! addressed to its machine, and complete one — and nothing else: it cannot ask
//! about the world and it cannot act on it. Note which of the routing leg's
//! four verbs is absent: `invoke`, the asking side's. A foot is invoked; it
//! never invokes.
//!
//! **Default-operator, and that is load-bearing.** A certificate minted before
//! the grade existed, or by a recipe that has not learned the flag, is operator
//! grade — a silently demoted seat would be an outage with no sentence attached,
//! while a silently promoted foot cannot happen, because promotion requires the
//! operator's own CA to have written the word.
//!
//! **This is not a per-verb policy layer** (REMOTE §11): the set is enumerated
//! in the match below, so a new [`Action`] is operator-only by construction and
//! adds no row anywhere. There is no table and nothing an operator writes.

use crate::boundary::{Action, Gesture, Query};
use crate::registry::Client;

/// The organizational unit that spells the foot grade — the one word, shared by
/// the mint that writes it into a subject
/// ([`provision`](crate::wire::provision)) and the walk that reads it back
/// ([`leaf::grade`](super::leaf::grade)). Two spellings of it would be two
/// authorities for one fact.
pub const FOOT: &str = "foot";

/// The other grade's word — spelled only where a grade is *said*: the
/// enrollment envelope, its line, and its reply (REMOTE §1.4 as amended,
/// bl-f4e3). It is deliberately not what the mint writes into a subject:
/// operator grade is the **absence** of `OU=foot`, which is what
/// default-operator means, so a certificate never carries this word.
pub const OPERATOR: &str = "operator";

/// The one sentence a refused foot earns — **in band and naming the grade**,
/// never absent-shaped. §4's absence rule exists so a scoped caller cannot map
/// what it is not registered in; a foot asking for the board learns nothing
/// about the world from being told it is a foot, and it made a category error
/// the sentence is worth more than the silence for.
pub const REFUSAL: &str = "this certificate is a foot: it may advertise its tools, take the \
                           invocations addressed to it and complete them, and nothing else. \
                           An operator-grade certificate is what the rest of the boundary needs.";

/// The two grades a leaf can carry (REMOTE §4.2).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Grade {
    /// The whole boundary, within the registrations §4 scopes. Every leaf
    /// minted before the grade existed, and every seat after it.
    #[default]
    Operator,
    /// The tool host's three gestures and nothing else.
    Foot,
}

impl Grade {
    /// Whether a caller of this grade may say `gesture`. The foot arm is the
    /// enumeration itself — the set is closed here, in code, and subtracting
    /// from the operator's roster is never how it is written.
    pub fn admits(self, gesture: &Gesture) -> bool {
        match self {
            Self::Operator => true,
            Self::Foot => matches!(
                gesture,
                Gesture::Act(
                    Action::Advertise { .. } | Action::Route(super::mailbox::Verb::Complete(_))
                ) | Gesture::Ask(Query::Invocations)
            ),
        }
    }

    /// This grade's one word (bl-f4e3) — the token the enrollment envelope, its
    /// line and its reply all carry. [`of`](Self::of) is its exact inverse, the
    /// `Ruling::word`/`of` pair's shape: a match is the compile gate and a
    /// table is the parser, and one vocabulary serves every serialization.
    ///
    /// `pub(crate)` for `Ruling::word`'s reason exactly (AGENTS.md rule 2): a
    /// `pub fn` may not hand back a borrow, and the honest demotion is cheaper
    /// than cloning a `&'static str` to own it.
    pub(crate) fn word(self) -> &'static str {
        match self {
            Self::Operator => OPERATOR,
            Self::Foot => FOOT,
        }
    }

    /// The grade a word names, or `None` — an unknown token is refused by its
    /// reader, never rounded to a default. Rounding down would demote a seat
    /// silently and rounding up would promote a foot, and §4.2 forbids the
    /// second outright.
    pub(crate) fn of(word: &str) -> Option<Self> {
        match word {
            OPERATOR => Some(Self::Operator),
            FOOT => Some(Self::Foot),
            _ => None,
        }
    }
}

/// One connection's authorization: who it is, and what that certificate lets it
/// say. Built where the identity is — off the presented leaf, per request — and
/// spent at the one chokepoint that already spends the identity for scoping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Peer {
    /// The leaf's subject common name, as a registry identity.
    pub client: Client,
    /// The grade its subject carries.
    pub grade: Grade,
}

#[cfg(test)]
mod tests;
