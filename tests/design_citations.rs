//! The citation guard (bl-43cd): **every `§N` / `§N.M` cited anywhere in
//! `src/`, `docs/STORIES.md`, or `docs/DESIGN.md` itself must resolve to a
//! numbered heading in `docs/DESIGN.md`** — or be a known foreign key
//! (another repo's doc, cited bare).
//!
//! Why this test exists: 2400+ section citations in code are what forbid ever
//! renumbering DESIGN. The sanctioned way to retire a section is a tombstone
//! heading that keeps its number resolvable (the doc's header records the
//! doctrine). This guard turns that promise into an invariant: deleting a
//! heading that anything still cites fails the build, so retirement is safe
//! exactly when this test says it is.
//!
//! Foreign keys: prose cites lernie ARCH (`§2.2`, `§4.4`, …) and brazen arch
//! (`§5.5`) bare, without a doc prefix. Those keys are listed in [`FOREIGN`]
//! — lawful only while DESIGN itself has no such heading, which
//! [`foreign_keys_are_not_design_headings`] pins, so the allowlist can never
//! mask a genuine DESIGN section.
//!
//! And the other direction (bl-cdd2): **a `§` belongs in a comment and never in
//! a string.** That half is [`strings`], split off at §12's budget on this
//! seam — one half asks *does this citation resolve*, the other *should this
//! citation be here at all* — and it carries the scanner only it uses.

// The other-direction half. `#[path]` because this file IS the test target's
// crate root, so a bare `mod` would resolve to `tests/strings.rs` — and a
// second top-level `tests/*.rs` is a second test binary, not a module.
#[path = "design_citations/strings.rs"]
mod strings;

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

/// Section keys lawfully cited bare that belong to *other* repos' docs:
/// lernie ARCH §2.2–§2.11 / §4.3 / §4.4, brazen arch §5.5. DESIGN has no such
/// headings (see `foreign_keys_are_not_design_headings`).
const FOREIGN: &[&str] = &[
    // lernie ARCH coordinates, cited bare. §2.5 is caller-supplied pinned
    // documents — the mechanism DESIGN §3.7's instruction freeze rides.
    "2.2", "2.3", "2.4", "2.5", "2.6", "2.9", "2.10", "2.11", "4.3", "4.4", "5.5",
    // VISION.md's §4.5 (spend attribution's join discipline, bl-afc4), §4.8
    // (the control-boundary ruling), §4.9 (the alignment
    // monitor, bl-af1a), §4.10 (the project-delivery contract, bl-2b8c) and
    // §4.11 (the capability boundary, bl-0cea), always cited with their doc
    // prefix ("VISION §4.8", "VISION §4.9", …) — foreign here because the
    // scanner is deliberately prefix-blind.
    // VISION §4.6 (model selection — the policy-table ruling §8.7's birth
    // policy answers) joins them, cited "VISION §4.6" for the same reason.
    "4.5", "4.6", "4.8", "4.9", "4.10", "4.11",
    // REMOTE.md's own rulings — §1.2 (one method, one channel), §1.3 (the
    // channel is mTLS), §1.4 (bootstrapping is out-of-channel) and §1.5 (the
    // workspace is the trust domain, bl-8bbc) — always cited with their doc
    // prefix ("REMOTE §1.4"), foreign here for the same reason VISION's are:
    // the scanner is deliberately prefix-blind, and DESIGN §1 has no
    // subsections at all (bl-b6fa).
    "1.2", "1.3", "1.4", "1.5",
    // REMOTE.md's build-sequence residuals — §9.7 (the read path, bl-ae05) and
    // §9.8 (the act path, bl-4841) — cited the same prefixed way. DESIGN's own
    // §9 stops short of both today, which
    // [`foreign_keys_are_not_design_headings`] is what keeps honest: the day it
    // grows one, this entry fails rather than masking it.
    "9.7", "9.8",
];

/// Parse a section key (`digits`, optionally `.digits`) starting at `i`;
/// returns the key and the index just past it, or `None` if no digit follows.
fn key_at(s: &[char], i: usize) -> Option<(String, usize)> {
    let mut j = i;
    let mut out = String::new();
    while let Some(c) = s.get(j).copied().filter(char::is_ascii_digit) {
        out.push(c);
        j += 1;
    }
    if out.is_empty() {
        return None;
    }
    if s.get(j).copied() == Some('.') && s.get(j + 1).copied().is_some_and(|c| c.is_ascii_digit()) {
        out.push('.');
        j += 1;
        while let Some(c) = s.get(j).copied().filter(char::is_ascii_digit) {
            out.push(c);
            j += 1;
        }
    }
    Some((out, j))
}

/// Every `§`-citation key in `text`, mapped to the `path:line` sites using it.
fn cite(text: &str, path: &str, out: &mut BTreeMap<String, Vec<String>>) {
    for (n, line) in text.lines().enumerate() {
        let s: Vec<char> = line.chars().collect();
        let mut i = 0;
        while i < s.len() {
            if s.get(i).copied() == Some('§')
                && let Some((k, j)) = key_at(&s, i + 1)
            {
                out.entry(k).or_default().push(format!("{path}:{}", n + 1));
                i = j;
                continue;
            }
            i += 1;
        }
    }
}

/// The numbered-heading key set of DESIGN.md (`## N.` and `### N.M` lines).
fn headings(design: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for line in design.lines() {
        let rest = line
            .strip_prefix("## ")
            .or_else(|| line.strip_prefix("### "));
        if let Some(r) = rest {
            let s: Vec<char> = r.chars().collect();
            if let Some((k, _)) = key_at(&s, 0) {
                out.insert(k);
            }
        }
    }
    out
}

/// Every `.rs` file under `dir`, recursively. Forgiving reads, like the
/// glyph guard's sweep: a vanished path yields nothing, and
/// [`the_sweep_is_not_vacuous`] is what keeps "nothing" from passing.
pub(crate) fn rust_files(dir: &PathBuf, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let p = entry.path();
        if p.is_dir() {
            rust_files(&p, out);
        } else if p.extension().is_some_and(|e| e == "rs") {
            out.push(p);
        }
    }
}

fn cited_everywhere() -> BTreeMap<String, Vec<String>> {
    let mut cites = BTreeMap::new();
    let mut files = Vec::new();
    rust_files(&PathBuf::from("src"), &mut files);
    files.push(PathBuf::from("docs/STORIES.md"));
    files.push(PathBuf::from("docs/DESIGN.md"));
    files.sort();
    for f in files {
        let text = std::fs::read_to_string(&f).unwrap_or_default();
        cite(&text, &f.display().to_string(), &mut cites);
    }
    cites
}

#[test]
fn every_citation_resolves() {
    let design = std::fs::read_to_string("docs/DESIGN.md").expect("DESIGN.md");
    let known = headings(&design);
    let cites = cited_everywhere();
    let dangling: Vec<String> = cites
        .iter()
        .filter(|(k, _)| !known.contains(*k) && !FOREIGN.contains(&k.as_str()))
        .map(|(k, sites)| {
            let head: Vec<&str> = sites.iter().take(3).map(String::as_str).collect();
            format!("§{k} ({} sites, e.g. {})", sites.len(), head.join(", "))
        })
        .collect();
    assert!(
        dangling.is_empty(),
        "citations with no DESIGN.md heading — retire sections behind a \
         tombstone heading, never by deletion:\n{}",
        dangling.join("\n")
    );
}

#[test]
fn foreign_keys_are_not_design_headings() {
    let design = std::fs::read_to_string("docs/DESIGN.md").expect("DESIGN.md");
    let known = headings(&design);
    let shadowed: Vec<&&str> = FOREIGN.iter().filter(|k| known.contains(**k)).collect();
    assert!(
        shadowed.is_empty(),
        "FOREIGN keys now exist as DESIGN headings; remove them from the \
         allowlist: {shadowed:?}"
    );
}

#[test]
fn the_sweep_is_not_vacuous() {
    let design = std::fs::read_to_string("docs/DESIGN.md").expect("DESIGN.md");
    let known = headings(&design);
    let cites = cited_everywhere();
    let sites: usize = cites.values().map(Vec::len).sum();
    assert!(known.len() >= 40, "only {} headings parsed", known.len());
    assert!(sites >= 1000, "only {sites} citation sites found");
    assert!(
        cites.keys().any(|k| k.contains('.')),
        "no dotted subsection citations parsed"
    );
}
