//! The corpus's **disk half** (bl-32cb): render every shape, then either verify
//! what is committed or write it.
//!
//! Both directions render the same bytes from the same surface, so the gate and
//! the regeneration can never disagree about what the corpus should be — the
//! only difference is whether a mismatch is reported or repaired.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use super::ledger::{Ledger, advance};
use super::{REPLY, REQUEST, shapes};

/// The standing record's own file, beside the fixtures it is about.
const RECORD: &str = "shapes.json";

/// Everything the corpus is at this protocol: relative path → canonical bytes.
fn rendered(protocol: u32, dir: &Path) -> Result<BTreeMap<String, String>, String> {
    let shapes = shapes();
    let previous = Ledger::read(&read(dir, RECORD));
    let next = advance(&shapes, &previous, protocol)?;
    let mut out: BTreeMap<String, String> = shapes
        .iter()
        .map(|shape| {
            let since = next.shapes.get(&shape.key()).map_or(protocol, |e| e.since);
            (shape.path(), shape.render(since))
        })
        .collect();
    out.insert(RECORD.to_owned(), next.render());
    Ok(out)
}

/// A committed file's bytes, or none — an absent corpus and an unreadable one
/// are one case, and both mean *stale*.
fn read(dir: &Path, path: &str) -> String {
    fs::read_to_string(dir.join(path)).unwrap_or_default()
}

/// **The gate.** Every fixture the boundary spells is committed verbatim, and
/// nothing else is.
pub(super) fn check(dir: &Path) -> Result<(), String> {
    let want = rendered(super::protocol(), dir)?;
    let mut stale: Vec<String> = want
        .iter()
        .filter(|(path, text)| &&read(dir, path) != text)
        .map(|(path, _)| path.clone())
        .collect();
    stale.extend(
        present(dir)
            .into_iter()
            .filter(|path| !want.contains_key(path)),
    );
    if stale.is_empty() {
        return Ok(());
    }
    stale.sort();
    Err(format!(
        "the wire conformance corpus is stale at {}. Run `make corpus` to \
         regenerate it; if a shape already in use changed, raise PROTOCOL in \
         src/wire/hello.rs first.",
        stale.join(", ")
    ))
}

/// **The regeneration.** Write what the boundary spells and drop what it no
/// longer does — refusing, before either, a shape that moved under a standing
/// protocol version.
pub(super) fn bless(dir: &Path) -> Result<(), String> {
    let want = rendered(super::protocol(), dir)?;
    for path in present(dir) {
        if !want.contains_key(&path) {
            fs::remove_file(dir.join(&path)).map_err(|err| err.to_string())?;
        }
    }
    for (path, text) in &want {
        let file = dir.join(path);
        let parent = file.parent().unwrap_or(dir).to_owned();
        fs::create_dir_all(&parent).map_err(|err| err.to_string())?;
        fs::write(&file, text).map_err(|err| err.to_string())?;
    }
    Ok(())
}

/// The `.json` files the corpus directory actually holds — what a check
/// compares against, so a fixture for a spelling that no longer exists is
/// found rather than merely unread.
fn present(dir: &Path) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for sub in [REQUEST, REPLY] {
        for entry in fs::read_dir(dir.join(sub)).into_iter().flatten().flatten() {
            out.insert(format!("{sub}/{}", entry.file_name().to_string_lossy()));
        }
    }
    if dir.join(RECORD).is_file() {
        out.insert(RECORD.to_owned());
    }
    out
}
