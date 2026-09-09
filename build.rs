//! **The wire's two compiled-in numbers, each read out of a committed file**
//! (bl-3e57, bl-1be7).
//!
//! The repo-root `PROTOCOL` file states the major this build speaks — one
//! line, nothing else — and `corpus/shapes.json` states the corpus this build
//! was generated from: the newest edition stamped on any field path, and the
//! `floor` at which the current major was cut (REMOTE §3.2). This script turns
//! all three into the constants `src/wire/hello/version.rs` includes. **Each
//! file IS its own source**: there is no second copy of any of the numbers in
//! the tree, so there is nothing to hold equal to them and nothing that can go
//! stale against them.
//!
//! **Why a file and not a Rust declaration.** Four repositories gate on the
//! major — yog's release-ordering gate reads the three consumers' mains, and
//! each consumer's own gate reads yog's newest published tag (REMOTE §3) — and
//! every one of those reads is a fetch of one path out of a tree the reader
//! does not build. A Rust path is not a stable address for that: bl-94a5 split
//! `src/wire/hello.rs` into `src/wire/hello/version.rs` and left the old file
//! re-exporting, which is invisible to a build and fatal to a regex, so every
//! consumer's gate silently stopped being able to read the engine's number and
//! would have held its release forever on a bump that had already landed
//! (thrall bl-c618). A repo-root file with no extension is the one address a
//! module split cannot move. The edition needs no such address — a consumer
//! reads it off the hello, or off the ledger it vendors — but it is a fact
//! about the committed corpus, so it is derived FROM that corpus rather than
//! declared beside it and left to drift.
//!
//! **The edition is computed, never stored.** `shapes.json` carries no
//! `edition` key on purpose (REMOTE §3.2: *"the corpus's edition is its newest
//! stamp, computed and never stored"*), so this script takes the maximum over
//! every stamp in every shape's signature — the same arithmetic
//! `Ledger::edition` does, held equal to it by a unit test on the committed
//! record.
//!
//! Errors here fail the build, which is the correct loudness: a `PROTOCOL`
//! file that is missing or is not one integer, or a `shapes.json` that is not
//! the record, means nothing downstream can be trusted to have read it either.

use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

/// The committed record's own path, relative to the manifest directory.
const RECORD: &str = "corpus/shapes.json";

/// Read the two committed files and write the three constants into `OUT_DIR`.
fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rerun-if-changed=PROTOCOL");
    println!("cargo::rerun-if-changed={RECORD}");
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let stated = fs::read_to_string(root.join("PROTOCOL"))?;
    let protocol: u32 = stated.trim().parse().map_err(|_| {
        format!("the repo-root PROTOCOL file must state one integer; it states {stated:?}")
    })?;
    let record: Value = serde_json::from_str(&fs::read_to_string(root.join(RECORD))?)?;
    let floor = stamp(record.get("floor"))
        .ok_or_else(|| format!("{RECORD} states no `floor`; run `make corpus`"))?;
    let edition = editions(&record)
        .into_iter()
        .max()
        .ok_or_else(|| format!("{RECORD} stamps no field path; run `make corpus`"))?;
    write(
        &PathBuf::from(env::var("OUT_DIR")?).join("protocol.rs"),
        protocol,
        edition,
        floor,
    )?;
    Ok(())
}

/// Every edition stamp in the record: one per field path, over every shape.
fn editions(record: &Value) -> Vec<u32> {
    record
        .get("shapes")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .filter_map(|(_, entry)| entry.get("signature")?.as_object())
        .flatten()
        .filter_map(|(_, at)| stamp(Some(at)))
        .collect()
}

/// One stamp, as the record spells it.
fn stamp(value: Option<&Value>) -> Option<u32> {
    u32::try_from(value?.as_u64()?).ok()
}

fn write(out: &Path, protocol: u32, edition: u32, floor: u32) -> Result<(), Box<dyn Error>> {
    fs::write(
        out,
        format!(
            "/// The MAJOR this build speaks: the integer the repo-root\n\
             /// `PROTOCOL` file states, compiled in by `build.rs`. The changelog\n\
             /// of every bump is this module's own documentation.\n\
             pub const PROTOCOL: u32 = {protocol};\n\
             \n\
             /// The edition of the corpus this build was generated from: the\n\
             /// newest stamp in `corpus/shapes.json`, compiled in by `build.rs`\n\
             /// (REMOTE §3.2, bl-1be7). The engine states it in its preface, so\n\
             /// a seat that vendors the ledger can tell a field this engine\n\
             /// cannot spell from a field it chose not to say.\n\
             pub const EDITION: u32 = {edition};\n\
             \n\
             /// The edition at which the current major was cut: `floor` in\n\
             /// `corpus/shapes.json`, compiled in by `build.rs`. Every path\n\
             /// stamped at or below it is written by every engine of this\n\
             /// major, so a peer that states no edition at all is read as\n\
             /// speaking exactly this one.\n\
             pub const FLOOR: u32 = {floor};\n"
        ),
    )?;
    Ok(())
}
