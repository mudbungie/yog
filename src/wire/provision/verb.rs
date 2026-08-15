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
    /// The `host:port` the engine binds and a seat dials.
    pub address: String,
    /// Whether to rotate, distrusting every certificate already issued.
    pub force: bool,
}

/// Fold a plan from the four environment readings. Pure, so the verb's whole
/// decision is testable without touching the process environment.
pub fn plan(
    world: &Env,
    dir: Option<String>,
    host: Option<String>,
    port: Option<String>,
    force: Option<String>,
) -> Plan {
    let host = stated(host).unwrap_or_else(|| LOOPBACK.to_owned());
    let port = stated(port).unwrap_or_else(|| PORT.to_owned());
    let dir = stated(dir).map_or_else(|| super::super::material::dir(world), PathBuf::from);
    Plan {
        dir,
        address: format!("{host}:{port}"),
        force: stated(force).is_some(),
    }
}

/// One environment reading, as a statement. An empty value is the same as an
/// unset one: a `make` variable that expanded to nothing must not become an
/// empty host — or, at `FORCE`, a rotation.
fn stated(value: Option<String>) -> Option<String> {
    value.filter(|v| !v.is_empty())
}

/// The four environment readings this verb takes, named once so the process
/// edge that reads them and the plan that folds them cannot disagree.
/// `main.rs` performs the reads, which is where every other environment read in
/// this crate happens (the xdg discipline) and why there is no reader here.
pub const READS: [&str; 4] = ["WIRE_DIR", "WIRE_HOST", "WIRE_PORT", "FORCE"];

/// Perform `plan`, printing what it wrote. Exit `0` minted, `1` refused —
/// including the refusal to overwrite, which is the whole of the rotation
/// guard: material already here is a trust root other machines may hold, and
/// replacing it silently would strand every one of them.
pub fn perform(plan: &Plan) -> i32 {
    if plan.dir.join(ANCHORS).is_file() && !plan.force {
        eprintln!(
            "yog {SUBCMD}: {} already holds material; rotating distrusts every certificate \
             already issued. Re-run with FORCE=1 if that is what you mean.",
            plan.dir.display()
        );
        return 1;
    }
    match super::mint(&plan.dir, &plan.address, plan.force) {
        Ok(()) => {
            report(&plan.dir, &plan.address);
            0
        }
        Err(e) => {
            eprintln!("yog {SUBCMD}: {e}");
            1
        }
    }
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
        "  issue another client with: openssl req … -CA {} -CAkey {}",
        dir.join(ANCHORS).display(),
        dir.join(super::CA_KEY).display()
    );
}

#[cfg(test)]
mod tests;
