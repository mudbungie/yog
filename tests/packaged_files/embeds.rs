//! **The compile-time embed sweep** — the other question the packaged-file
//! guard asks, and a different one: not *"what would ship that must not"* but
//! *"what does the build read that an allowlist could silently drop"*.
//!
//! Split from the beats at §12's budget on that seam. The guard beside it
//! restates the manifest's `include` policy and judges the packaged list
//! against it; nothing here knows what the policy is. This walks `src`, reads
//! every `include_bytes!`/`include_str!` target that resolves OUTSIDE `src`,
//! and hands back the set — a fact about the TREE, gathered rather than
//! listed, so it covers embeds that do not exist yet.
//!
//! `src/**` ships whole, so an embed of a sibling module's data cannot be
//! dropped by the allowlist and is not interesting here. Forgiving throughout
//! (`unwrap_or_default`, `continue` on every miss), like the citation and
//! module-map sweeps; the vacuity beat is what stops "nothing" from passing.

#![allow(clippy::unwrap_used)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The package root — this crate's own manifest directory, so the guard reads
/// the tree it was compiled from rather than a working directory.
pub fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every `.rs` file under `dir`, recursively.
fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let p = entry.path();
        if p.is_dir() {
            rust_files(&p, out);
        } else if p.extension().is_some_and(|e| e == "rs") {
            out.push(p);
        }
    }
}

/// One source file's compile-time embeds, resolved against its own directory
/// and expressed relative to the package root.
fn embeds_of(file: &Path, root: &Path, out: &mut BTreeSet<String>) {
    let text = std::fs::read_to_string(file).unwrap_or_default();
    let dir = file.parent().unwrap_or(root);
    for macro_name in ["include_bytes!(\"", "include_str!(\""] {
        for tail in text.split(macro_name).skip(1) {
            let Some((target, _)) = tail.split_once('"') else {
                continue;
            };
            let Ok(abs) = dir.join(target).canonicalize() else {
                continue;
            };
            let Ok(rel) = abs.strip_prefix(root) else {
                continue;
            };
            let rel = rel.display().to_string();
            if !rel.starts_with("src/") {
                out.insert(rel);
            }
        }
    }
}

/// Every non-`src` path the crate embeds at compile time — the build inputs an
/// allowlist can silently drop, gathered from the tree rather than listed here.
pub fn embedded_paths() -> BTreeSet<String> {
    let root = root();
    let mut files = Vec::new();
    rust_files(&root.join("src"), &mut files);
    let mut out = BTreeSet::new();
    for file in &files {
        embeds_of(file, &root, &mut out);
    }
    out
}
