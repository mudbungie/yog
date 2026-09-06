//! **What the verb does**, once [`plan`](super::plan) has said what was asked
//! for: the mint and its rotation guard, the server leaf re-issued over the CA
//! already here, one extra client leaf, and the sentences each prints.
//!
//! Split from [`verb`](super) at §12's per-file budget, on the same seam the
//! test module already names: that file folds the environment into a value and
//! this one performs it. Nothing here reads the environment; nothing there
//! touches the directory.

use super::super::{ANCHORS, LOOPBACK, PORT};
use super::{Act, Plan, READS, SUBCMD};
use crate::registry::Grade;
use crate::wire::material::{ADDRESS, DIR, ENTRIES, ENTRY, Role};
use std::path::Path;

/// Perform `plan`, printing what it wrote. Exit `0` acted, `1` refused, and
/// every refusal is the act's own sentence naming its own remedy.
pub fn perform(plan: &Plan) -> i32 {
    match &plan.act {
        Act::Mint { hosts, port, force } => mint(&plan.dir, hosts, port.as_deref(), *force),
        Act::Leaf(cn, grade) => leaf(&plan.dir, cn, *grade),
    }
}

/// The mint, the refusal to overwrite that is the whole of the rotation guard —
/// material already here is a trust root other machines may hold, and replacing
/// it silently would strand every one of them — and, between them, the act a
/// stated host asks for on a directory that already holds a CA.
///
/// **A stated host on standing material is not a rotation** (bl-52f4). It is a
/// statement about ONE artifact, the server's own leaf, and re-issuing that leaf
/// over the CA already here strands nobody: a client verifies the CA. So the
/// signal is the one already spelled — `WIRE_HOST` on a directory that holds
/// material re-issues the server leaf, no new reading and no new verb — and the
/// standing refusal is what an operator who stated NOTHING still gets, because
/// a bare re-run asks for nothing this act could perform.
fn mint(dir: &Path, hosts: &[String], port: Option<&str>, force: bool) -> i32 {
    if dir.join(ANCHORS).is_file() && !force {
        if hosts.is_empty() {
            eprintln!(
                "yog {SUBCMD}: {} already holds material; rotating distrusts every certificate \
                 already issued. Re-run with FORCE=1 if that is what you mean, or state \
                 {}=<host>[,<host>…] {}=<port> to say where this engine listens — which \
                 re-issues the server leaf and writes the address, over the CA already here, \
                 distrusting nothing.",
                dir.display(),
                READS[1],
                READS[2]
            );
            return 1;
        }
        return restate(dir, hosts, port);
    }
    let address = format!(
        "{}:{}",
        hosts.first().map_or(LOOPBACK, String::as_str),
        port.unwrap_or(PORT)
    );
    match super::super::mint(dir, &address, hosts.get(1..).unwrap_or_default(), force) {
        Ok(()) => {
            report(dir, &address);
            0
        }
        Err(e) => {
            eprintln!("yog {SUBCMD}: {e}");
            1
        }
    }
}

/// **State where this engine listens** (bl-98ef): re-issue the server leaf over
/// the CA already here, and write the `address` file to match it.
///
/// One act, because a stated host is one statement about two facts — what the
/// engine binds, and what a client may verify what it dialled against — and
/// splitting them left the second sayable only by a rotation. `FORCE=1` was
/// then the only spelling that could replace a `127.0.0.1:0`, which is the
/// address a self-provisioned boot writes and the one address nothing
/// downstream can use: it distrusts every leaf already carried away in order to
/// change one line of text. Nothing here distrusts anything — the CA stands,
/// every leaf already issued still verifies, and the engine binds the stated
/// endpoint when it is next started.
///
/// The port is the operator's when they state one and **the standing one
/// otherwise**: an operator widening the SAN of a box that binds 7752 states no
/// port, and moving it to the default underneath them would be this act
/// breaking the box it was asked to describe.
fn restate(dir: &Path, hosts: &[String], port: Option<&str>) -> i32 {
    let standing = super::super::port_at(dir);
    let address = format!(
        "{}:{}",
        hosts.first().map_or(LOOPBACK, String::as_str),
        port.unwrap_or(&standing)
    );
    let acted = super::super::reissue(dir, hosts).and_then(|()| super::super::state(dir, &address));
    if let Err(e) = acted {
        eprintln!("yog {SUBCMD}: {e}");
        return 1;
    }
    let leaf = Role::Server.leaf();
    println!(
        "yog {SUBCMD}: re-issued the {leaf} leaf over the CA already in {}",
        dir.display()
    );
    for name in [format!("{leaf}.pem"), format!("{leaf}.key")] {
        println!("  {}", dir.join(name).display());
    }
    println!("  it answers to {}", hosts.join(", "));
    println!(
        "  {} names {address} — restart the engine, which binds it as it starts",
        dir.join(ADDRESS).display()
    );
    println!("  the CA is untouched, so every leaf already issued still verifies");
    0
}

/// One extra client leaf, and the out-of-channel act that follows it. What is
/// printed is where the two files are and what to do with them — REMOTE §1.4 is
/// verbatim and forever, so the carrying is the operator's and yog's last word
/// on it is a sentence.
fn leaf(dir: &Path, cn: &str, grade: Grade) -> i32 {
    if let Err(e) = super::super::issue(dir, cn, grade) {
        eprintln!("yog {SUBCMD}: {e}");
        return 1;
    }
    println!("yog {SUBCMD}: issued a {} leaf for {cn}", word(grade));
    for name in [format!("{cn}.pem"), format!("{cn}.key")] {
        println!("  {}", dir.join(name).display());
    }
    // Spelled from the reader's own constants — the destination's own name
    // ([`ENTRY`]) among them, because that was the one token still written by
    // hand here and it is the one that drifted (bl-686c): it said `<leaf>`, and
    // a directory named for the leaf just issued is a channel no gesture routes
    // to.
    let client = Role::Client.leaf();
    println!(
        "  carry those and {} to that box by hand, into its {DIR}/{ENTRIES}/{ENTRY}/ — named for \
         the WORKSPACE it will address, not for {cn} — as {client}.pem, {client}.key and \
         {ANCHORS}, beside an {ADDRESS} you state; the common name inside, not the basename, is \
         the identity",
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
        super::super::artifacts().join(", ")
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
pub(super) fn word(grade: Grade) -> &'static str {
    match grade {
        Grade::Operator => "client",
        Grade::Foot => crate::registry::peer::FOOT,
    }
}
