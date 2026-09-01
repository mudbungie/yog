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

use super::super::material::{ADDRESS, DIR, ENTRIES, Role};
use super::{ANCHORS, LOOPBACK, PORT};
use crate::registry::Grade;
use crate::xdg::Env;
use std::path::{Path, PathBuf};

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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Act {
    /// Mint whatever the directory lacks, aimed at a stated address.
    Mint {
        /// The `host:port` the engine binds and a seat dials.
        address: String,
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
            address: format!(
                "{}:{}",
                stated(host).unwrap_or_else(|| LOOPBACK.to_owned()),
                stated(port).unwrap_or_else(|| PORT.to_owned())
            ),
            force: stated(force).is_some(),
        },
        |cn| Act::Leaf(cn, grade),
    );
    Plan { dir, act }
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

/// Perform `plan`, printing what it wrote. Exit `0` acted, `1` refused, and
/// every refusal is the act's own sentence naming its own remedy.
pub fn perform(plan: &Plan) -> i32 {
    match &plan.act {
        Act::Mint { address, force } => mint(&plan.dir, address, *force),
        Act::Leaf(cn, grade) => leaf(&plan.dir, cn, *grade),
    }
}

/// The mint, and the refusal to overwrite that is the whole of the rotation
/// guard: material already here is a trust root other machines may hold, and
/// replacing it silently would strand every one of them.
fn mint(dir: &Path, address: &str, force: bool) -> i32 {
    if dir.join(ANCHORS).is_file() && !force {
        eprintln!(
            "yog {SUBCMD}: {} already holds material; rotating distrusts every certificate \
             already issued. Re-run with FORCE=1 if that is what you mean.",
            dir.display()
        );
        return 1;
    }
    match super::mint(dir, address, force) {
        Ok(()) => {
            report(dir, address);
            0
        }
        Err(e) => {
            eprintln!("yog {SUBCMD}: {e}");
            1
        }
    }
}

/// One extra client leaf, and the out-of-channel act that follows it. What is
/// printed is where the two files are and what to do with them — REMOTE §1.4 is
/// verbatim and forever, so the carrying is the operator's and yog's last word
/// on it is a sentence.
fn leaf(dir: &Path, cn: &str, grade: Grade) -> i32 {
    if let Err(e) = super::issue(dir, cn, grade) {
        eprintln!("yog {SUBCMD}: {e}");
        return 1;
    }
    println!("yog {SUBCMD}: issued a {} leaf for {cn}", word(grade));
    for name in [format!("{cn}.pem"), format!("{cn}.key")] {
        println!("  {}", dir.join(name).display());
    }
    // Spelled from the reader's own constants, so the instruction and the
    // directory a client files it into cannot drift.
    let client = Role::Client.leaf();
    println!(
        "  carry those and {} to that box by hand, into its {DIR}/{ENTRIES}/<leaf>/ as \
         {client}.pem, {client}.key and {ANCHORS}, beside an {ADDRESS} you state; the common \
         name inside, not the basename, is the identity",
        dir.join(ANCHORS).display()
    );
    0
}

/// Say what is where. **Never say any of it**: the CA key is the whole trust
/// root, and the one thing an operator needs told is where it is.
fn report(dir: &Path, address: &str) {
    println!(
        "yog {SUBCMD}: {} holds {}",
        dir.display(),
        super::artifacts().join(", ")
    );
    println!("  the engine binds and a local seat dials {address}");
    println!(
        "  issue another client with: WIRE_LEAF=<common-name> yog {SUBCMD}, and a tool host's \
         with {}=1 beside that",
        READS[5]
    );
}

/// The word a grade is reported under — the mint's own spelling for a foot
/// (`crate::registry::peer::FOOT`), so what is printed and what is written into
/// the subject cannot drift.
fn word(grade: Grade) -> &'static str {
    match grade {
        Grade::Operator => "client",
        Grade::Foot => crate::registry::peer::FOOT,
    }
}

#[cfg(test)]
mod tests;
