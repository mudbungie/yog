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

use super::super::entries::ENTRIES;
use super::super::material::{ADDRESS, DIR, Role};
use super::{ANCHORS, LOOPBACK, PORT};
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
    /// host half of provisioning an entry on a visiting box.
    Leaf(String),
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
) -> Plan {
    let dir = stated(dir).map_or_else(|| super::super::material::dir(world), PathBuf::from);
    let act = stated(leaf).map_or_else(
        || Act::Mint {
            address: format!(
                "{}:{}",
                stated(host).unwrap_or_else(|| LOOPBACK.to_owned()),
                stated(port).unwrap_or_else(|| PORT.to_owned())
            ),
            force: stated(force).is_some(),
        },
        Act::Leaf,
    );
    Plan { dir, act }
}

/// One environment reading, as a statement. An empty value is the same as an
/// unset one: a `make` variable that expanded to nothing must not become an
/// empty host — or, at `FORCE`, a rotation.
fn stated(value: Option<String>) -> Option<String> {
    value.filter(|v| !v.is_empty())
}

/// The five environment readings this verb takes, named once so the process
/// edge that reads them and the plan that folds them cannot disagree.
/// `main.rs` performs the reads, which is where every other environment read in
/// this crate happens (the xdg discipline) and why there is no reader here.
pub const READS: [&str; 5] = ["WIRE_DIR", "WIRE_HOST", "WIRE_PORT", "FORCE", "WIRE_LEAF"];

/// Perform `plan`, printing what it wrote. Exit `0` acted, `1` refused, and
/// every refusal is the act's own sentence naming its own remedy.
pub fn perform(plan: &Plan) -> i32 {
    match &plan.act {
        Act::Mint { address, force } => mint(&plan.dir, address, *force),
        Act::Leaf(cn) => leaf(&plan.dir, cn),
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
fn leaf(dir: &Path, cn: &str) -> i32 {
    if let Err(e) = super::issue(dir, cn) {
        eprintln!("yog {SUBCMD}: {e}");
        return 1;
    }
    println!("yog {SUBCMD}: issued a client leaf for {cn}");
    for name in [format!("{cn}.pem"), format!("{cn}.key")] {
        println!("  {}", dir.join(name).display());
    }
    // Spelled from the reader's own constants, so the instruction and the
    // directory `entries` walks cannot drift.
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
    println!("  issue another client with: {SUBCMD} WIRE_LEAF=<common-name>");
}

#[cfg(test)]
mod tests;
