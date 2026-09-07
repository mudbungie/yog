//! **The wire version's one file-shaped home, compiled in** (bl-3e57).
//!
//! The repo-root `PROTOCOL` file states the integer this build speaks — one
//! line, nothing else — and this script turns it into the constant
//! `src/wire/hello/version.rs` includes. The file IS the source: there is no
//! second copy of the number anywhere in the tree, so there is nothing to hold
//! equal to it and nothing that can go stale against it.
//!
//! **Why a file and not a Rust declaration.** Four repositories gate on this
//! number — yog's release-ordering gate reads the three consumers' mains, and
//! each consumer's own gate reads yog's newest published tag (REMOTE §3) — and
//! every one of those reads is a fetch of one path out of a tree the reader
//! does not build. A Rust path is not a stable address for that: bl-94a5 split
//! `src/wire/hello.rs` into `src/wire/hello/version.rs` and left the old file
//! re-exporting, which is invisible to a build and fatal to a regex, so every
//! consumer's gate silently stopped being able to read the engine's number and
//! would have held its release forever on a bump that had already landed
//! (thrall bl-c618). A repo-root file with no extension is the one address a
//! module split cannot move.
//!
//! Errors here fail the build, which is the correct loudness: a `PROTOCOL`
//! file that is missing or is not one integer means nothing downstream can be
//! trusted to have read it either.

use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

/// Read the root `PROTOCOL` file and write the constant into `OUT_DIR`.
fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rerun-if-changed=PROTOCOL");
    let stated =
        fs::read_to_string(PathBuf::from(env::var("CARGO_MANIFEST_DIR")?).join("PROTOCOL"))?;
    let protocol: u32 = stated.trim().parse().map_err(|_| {
        format!("the repo-root PROTOCOL file must state one integer; it states {stated:?}")
    })?;
    fs::write(
        PathBuf::from(env::var("OUT_DIR")?).join("protocol.rs"),
        format!(
            "/// The protocol this build speaks: the integer the repo-root\n\
             /// `PROTOCOL` file states, compiled in by `build.rs`. The changelog\n\
             /// of every bump is this module's own documentation.\n\
             pub const PROTOCOL: u32 = {protocol};\n"
        ),
    )?;
    Ok(())
}
