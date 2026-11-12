//! **The offer** (REMOTE §15.1; bl-9ea6): how a new device joins without a
//! private key ever travelling. An operator-grade seat mints an offer — a
//! workspace, a client name, a grade — and what leaves the server is one URL
//! carrying a single-use code. The device generates its own key pair, claims
//! the code with the public half, and is handed a leaf signed for it. Both
//! ends then show one confirmation [`word`], the device dials, and the operator
//! approves the row whose word matches.
//!
//! **The values here are the vocabulary, not the acts** — the fold
//! [`mailbox::Verb`](super::mailbox::Verb) and [`enroll`](super::enroll) take.
//! The four operator acts execute in
//! [`boundary::dispatch`](crate::boundary::dispatch); the claim is
//! [`claim`]'s, reached only by the registration door and never by a gesture;
//! the pending offers are [`store`]'s.
//!
//! **The alphabet, the code and the word are a contract.** Three other
//! implementations reproduce the word — the seat, the foot and the phone — so
//! its derivation is pinned by a vector in this module's tests and stated in
//! REMOTE §15.1, and neither moves without the other.

use super::Grade;

/// The certless claim — the registration door's whole body.
pub mod claim;
/// Where pending offers live, and the hold on an unapproved leaf.
pub mod store;
/// The four requests and the four replies, spelled both ways.
pub(crate) mod wire;

/// The 32 symbols a code and a confirmation word are written in: the capital
/// letters and digits with `I`, `O`, `0` and `1` left out, because both strings
/// are read off one screen and typed or compared by a person.
pub const ALPHABET: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
/// A code's length: 8 symbols of 5 bits, 40 bits from the system's random
/// source.
pub const CODE_LEN: usize = 8;
/// A confirmation word's length.
pub const WORD_LEN: usize = 6;
/// How long an unclaimed code lives, in seconds.
pub const LIFE: i64 = 600;
/// The path a registration is POSTed to; the code is its last component.
pub const PATH: &str = "/yog/register/";

/// The four acts an operator-grade seat says about offers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verb {
    /// Mint an offer: the workspace the client will be seated in, the common
    /// name its certificate will carry, and what that certificate may say.
    Mint {
        workspace: String,
        name: String,
        grade: Grade,
    },
    /// A workspace's pending offers.
    List { workspace: String },
    /// Seat the registration a proved offer was minted for.
    Approve { workspace: String, name: String },
    /// Void an offer. One already claimed keeps its name spent.
    Decline { workspace: String, name: String },
}

impl Verb {
    /// The workspace this act names — every member names one.
    pub(crate) fn workspace_slot(&mut self) -> &mut String {
        match self {
            Self::Mint { workspace, .. }
            | Self::List { workspace }
            | Self::Approve { workspace, .. }
            | Self::Decline { workspace, .. } => workspace,
        }
    }
}

/// Where a pending offer stands, as an operator reads it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Minted, its code unspent.
    Open,
    /// A device claimed the code and holds the leaf; it has not dialled.
    Claimed,
    /// The holder of the leaf's private key has spoken on the wire.
    Proved,
}

/// The three state words, one table for both directions.
const STATES: [(State, &str); 3] = [
    (State::Open, "open"),
    (State::Claimed, "claimed"),
    (State::Proved, "proved"),
];

impl State {
    /// This state's wire word.
    pub(crate) fn word(self) -> &'static str {
        STATES
            .iter()
            .find(|(state, _)| *state == self)
            .map_or("", |(_, word)| word)
    }

    /// The state a word names, or `None` — an unknown word is its reader's to
    /// refuse or render, never rounded to a known one.
    pub(crate) fn of(word: &str) -> Option<Self> {
        STATES
            .iter()
            .find(|(_, w)| *w == word)
            .map(|(state, _)| *state)
    }
}

/// One pending offer, as `offers` lists it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// The common name offered — the client identity the leaf carries.
    pub name: String,
    /// The grade offered.
    pub grade: Grade,
    pub state: State,
    /// When the code stops being claimable, unix seconds. It bounds an open
    /// offer only: a claimed one waits for the operator.
    pub expires: i64,
    /// The confirmation word — absent until a device has claimed, because
    /// before that there is no key to derive it from.
    pub word: Option<String>,
}

/// What each act answers with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    /// The registration URL — the whole of what travels to the new device —
    /// and when its code expires, unix seconds.
    Offered { url: String, expires: i64 },
    /// The workspace's pending offers.
    Offers(Vec<Row>),
    /// The registration is seated.
    Approved,
    /// The offer is void.
    Declined,
}

/// A fresh code: [`CODE_LEN`] symbols, each drawn from one byte of the system's
/// random source — 256 is a multiple of 32, so the low five bits are uniform.
/// `None` only when the system has no randomness to give, which the caller
/// refuses on rather than minting a guessable code.
pub(crate) fn code() -> Option<String> {
    use ring::rand::SecureRandom;
    let mut raw = [0u8; CODE_LEN];
    ring::rand::SystemRandom::new().fill(&mut raw).ok()?;
    Some(raw.iter().map(|byte| symbol(u32::from(*byte))).collect())
}

/// **The confirmation word** (REMOTE §15.1 step 3): SHA-256 over the device's
/// public key (its DER `SubjectPublicKeyInfo`) followed by the CA certificate
/// (DER), and the digest's first thirty bits read five at a time, most
/// significant first, as [`WORD_LEN`] symbols of [`ALPHABET`].
///
/// The device computes it from what it sent and what it was handed; the server
/// from what it signed and what it holds. Nothing between them can make the
/// two agree without the operator's CA key.
pub fn word(public_key_der: &[u8], ca_der: &[u8]) -> String {
    let mut hashed = ring::digest::Context::new(&ring::digest::SHA256);
    hashed.update(public_key_der);
    hashed.update(ca_der);
    let digest = hashed.finish();
    let head = digest
        .as_ref()
        .iter()
        .take(4)
        .fold(0u32, |bits, byte| (bits << 8) | u32::from(*byte));
    (0..WORD_LEN)
        .map(|i| symbol(head >> (27 - 5 * i)))
        .collect()
}

/// The symbol the low five bits of `bits` name.
fn symbol(bits: u32) -> char {
    let index = usize::try_from(bits & 31).unwrap_or_default();
    ALPHABET.get(index).copied().map_or('A', char::from)
}

#[cfg(test)]
mod tests;
