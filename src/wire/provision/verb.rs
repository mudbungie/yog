//! `yog wire-certs` — **the operator's explicit mint** (REMOTE §8; bl-ae05).
//!
//! The boot's [`ensure`](super::ensure) covers the box yog runs on, aimed at
//! loopback. This is the act for everything else: a server another machine
//! dials by name, and a rotation. It is the same recipe — there is one, in
//! [`provision`](super) — reached by a verb rather than by a boot, so what a
//! `make wire-certs` runs and what a desktop launch performs cannot drift.
//!
//! `scripts/wire-certs.sh` was that recipe until bl-ae05 and is gone: an
//! installed binary has no repository to find a script in, and boot needed the
//! act. Its interface survives verbatim — `WIRE_DIR`, `WIRE_HOST`, `WIRE_PORT`
//! and `FORCE` are read from the environment, exactly as the Makefile already
//! passed them — so the operator's spelling is unchanged.
//!
//! **`WIRE_HOST` is a list, and it selects between two answers** (REMOTE §8,
//! bl-52f4). Comma-separated, each entry read as an address or a name exactly
//! as the single one was, so every existing spelling is a list of one. And on a
//! directory that already holds material a stated host is not the mint being
//! asked again: it re-issues THE SERVER LEAF over the CA already there, which
//! is [`issuing`](super::issuing)'s kind of act and takes its guard. A box that
//! gained a way in then costs one signature instead of the `FORCE=1` that
//! distrusts every leaf already carried away. Nothing stated is still the
//! standing refusal — a bare re-run asks for nothing this act could perform.
//!
//! **`WIRE_LEAF` is the fifth reading** (REMOTE §8.2, bl-64a7), and it selects
//! the other act rather than modifying this one: issue ONE extra client leaf
//! under the common name it states, over the CA already here. That is the host
//! half of provisioning an entry — a leaf for a visiting box — and it remains
//! out of channel in every respect §1.4 means, because what it produces is two
//! files an operator picks up and carries.
//!
//! **`WIRE_FOOT` is the sixth** (REMOTE §4.2, bl-7ff3), and it is presence-shaped
//! like `FORCE` rather than a word to spell: the stated leaf it modifies is
//! minted as a **foot** — a tool host that may advertise, take invocations and
//! complete them, and say nothing else to the boundary. Unset is operator grade,
//! which is the whole of "default-operator": there is no value to mistype into a
//! demotion, and a promotion still costs a certificate. It is read only where a
//! stated leaf is, exactly as `WIRE_HOST` and `FORCE` are read only where a mint
//! is — the readings that do not apply to the selected act are inert here and
//! always have been.

use super::PORT;
use crate::registry::Grade;
use crate::xdg::Env;
use std::path::PathBuf;

/// What the verb does once the environment has been folded — the acts and the
/// sentences they print.
mod acts;

pub use acts::perform;

/// This verb's own word: `yog wire-certs`.
pub const SUBCMD: &str = "wire-certs";

/// What one invocation was asked to do, folded from the environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    /// The material directory — `WIRE_DIR`, or the composed world's own.
    pub dir: PathBuf,
    /// Which of the one recipe's two acts this invocation asked for.
    pub act: Act,
}

/// The two acts the recipe can be asked for, as an enum rather than a mint
/// carrying an optional extra: `WIRE_LEAF` selects an act **over a trust root
/// that already exists**, so the rotation guard standing in front of a mint
/// would be exactly backwards in front of it — a leaf is issued *because* the
/// directory already holds material, and it founds no CA, writes no address and
/// touches no other leaf. Nothing here is a state a `Mint` could also be in.
///
/// **A mint states hosts and a port, never an address** (bl-52f4). Both facts
/// the mint writes derive from those two: the `address` file is the first host
/// on the port, and the server leaf's SAN is every host stated. Holding the
/// composed address here as well would be one fact in two places, and the empty
/// host list is load-bearing besides — it is exactly "the operator stated no
/// host", which is what [`perform`] reads to tell a bare re-run apart from a
/// statement about the server leaf.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Act {
    /// Mint whatever the directory lacks, aimed at the hosts stated.
    Mint {
        /// Every host the box answers to, in `WIRE_HOST`'s own order. Empty is
        /// unstated, and the mint's address is then loopback.
        hosts: Vec<String>,
        /// The port the engine binds and a seat dials.
        port: String,
        /// Whether to rotate, distrusting every certificate already issued.
        force: bool,
    },
    /// Issue one extra client leaf under this common name (REMOTE §8.2) — the
    /// host half of provisioning an entry on a visiting box — at the grade the
    /// operator asked for (REMOTE §4.2).
    Leaf(String, Grade),
}

/// Fold a plan from the five environment readings. Pure, so the verb's whole
/// decision is testable without touching the process environment.
pub fn plan(
    world: &Env,
    dir: Option<String>,
    host: Option<String>,
    port: Option<String>,
    force: Option<String>,
    leaf: Option<String>,
    foot: Option<String>,
) -> Plan {
    let dir = stated(dir).map_or_else(|| super::super::material::dir(world), PathBuf::from);
    let grade = if stated(foot).is_some() {
        Grade::Foot
    } else {
        Grade::Operator
    };
    let act = stated(leaf).map_or_else(
        || Act::Mint {
            hosts: hosts(stated(host).as_deref()),
            port: stated(port).unwrap_or_else(|| PORT.to_owned()),
            force: stated(force).is_some(),
        },
        |cn| Act::Leaf(cn, grade),
    );
    Plan { dir, act }
}

/// `WIRE_HOST` as the **list** it is (bl-52f4): a box on an overlay network has
/// a resolvable name, an overlay address and a LAN address, and a client whose
/// resolver cannot reach the name has no lawful spelling left unless the
/// certificate says so. Comma-separated, because a host may not contain one and
/// the reading it replaces was a single host that still parses as a list of
/// one. Empty entries and surrounding space are dropped, on the same argument
/// [`stated`] makes for the whole value: a `make` variable that expanded to
/// nothing must not become an empty host, and a trailing comma is that.
fn hosts(stated: Option<&str>) -> Vec<String> {
    stated
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(str::to_owned)
        .collect()
}

/// One environment reading, as a statement. An empty value is the same as an
/// unset one: a `make` variable that expanded to nothing must not become an
/// empty host — or, at `FORCE`, a rotation.
fn stated(value: Option<String>) -> Option<String> {
    value.filter(|v| !v.is_empty())
}

/// The six environment readings this verb takes, named once so the process
/// edge that reads them and the plan that folds them cannot disagree.
/// `main.rs` performs the reads, which is where every other environment read in
/// this crate happens (the xdg discipline) and why there is no reader here.
pub const READS: [&str; 6] = [
    "WIRE_DIR",
    "WIRE_HOST",
    "WIRE_PORT",
    "FORCE",
    "WIRE_LEAF",
    "WIRE_FOOT",
];

/// The refusal a word on the command line earns, or `None` for the empty tail
/// this verb is the whole of (bl-a0dd).
///
/// **Every setting is an environment reading** ([`READS`]), so a word here is a
/// setting the shell put in the wrong place — and the shape of the mistake is
/// exactly the one the Makefile teaches: `make wire-certs WIRE_HOST=…` is a
/// make variable the recipe passes on, while `yog wire-certs WIRE_HOST=…` is
/// argv. Accepting it silently was the real defect behind two wrong remedy
/// sentences: the words vanished, the mint aimed at the *default* loopback
/// endpoint, exit was `0`, and the operator was told it had worked — with a
/// trust root that then needs `FORCE=1` to correct, distrusting everything
/// already issued. A refusal costs one re-run; a wrong CA costs the fleet.
///
/// It names the first offender and the prefix spelling, and it is `pub` because
/// the process edge is where argv lives and `main.rs` holds only the call.
pub fn stray(tail: &[String]) -> Option<String> {
    let word = tail.first()?;
    Some(format!(
        "yog {SUBCMD}: {word:?} is not a setting this verb reads — {} are environment readings, \
         so they go BEFORE the verb: `WIRE_HOST=<host> WIRE_PORT=<port> yog {SUBCMD}`",
        READS.join(", ")
    ))
}

#[cfg(test)]
mod tests;
