//! The module-map guard (bl-9f72): **every tracked production file under
//! `src/` has a row in DESIGN §12, and every `src/` path §12 spells exists** —
//! the same promise `tests/design_citations.rs` makes for section numbers.
//!
//! Why this test exists: AGENTS.md says *"Over the cap? Split along a real seam
//! and add the row to DESIGN §12"*, and §12 is the module map and the line
//! budgets. A file with no row is therefore a split nobody recorded, and a row
//! naming a file that no longer exists is a map of a tree that no longer is.
//! Both were true at scale when this landed (35 unlisted files, one row whose
//! path had become a directory), because nothing checked. Doc-to-code drift is
//! only ever fixed once by hand; this makes the map an invariant instead.
//!
//! What is NOT a row, per §12 itself: *"a test module is covered by its
//! production module's row and never earns one of its own"* — so the sweep
//! skips a file named `tests.rs` or sitting under a `tests/` directory, which
//! is exactly the `X.rs` ↔ `X/tests.rs` seam §12 names.
//!
//! §12 spells families with brace lists (`src/app/{mod,roots}.rs`,
//! `src/multiplex{.rs,/bl.rs}`), so the parse expands them; a spelling this
//! cannot expand shows up as a missing row, which is the right pressure —
//! §12's paths are a machine contract now, not typography.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// DESIGN §12's table rows: the section from its own heading to the next `## `.
fn section_12(design: &str) -> Vec<&str> {
    design
        .lines()
        .skip_while(|l| !l.starts_with("## 12."))
        .take_while(|l| !l.starts_with("## 13."))
        .filter(|l| l.starts_with("| `"))
        .collect()
}

/// One brace list expanded: `a{b,c}d` → `abd`, `acd`. Nested lists recurse;
/// a token with no brace is itself.
fn expand(tok: &str) -> Vec<String> {
    let Some((head, rest)) = tok.split_once('{') else {
        return vec![tok.to_owned()];
    };
    let Some((body, tail)) = rest.split_once('}') else {
        return vec![tok.to_owned()];
    };
    body.split(',')
        .flat_map(|part| expand(&format!("{head}{part}{tail}")))
        .collect()
}

/// Every `src/…rs` path §12's Module cells name, brace lists expanded. The
/// Module cell is the first `|`-delimited field; a row's prose may cite modules
/// too, and those are not rows.
fn mapped(design: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for row in section_12(design) {
        let Some(cell) = row.split('|').nth(1) else {
            continue;
        };
        for tok in cell.split('`').skip(1).step_by(2) {
            out.extend(
                expand(tok.trim())
                    .into_iter()
                    .filter(|p| p.starts_with("src/") && p.ends_with(".rs")),
            );
        }
    }
    out
}

/// A test corpus, which §12 rules out of the table: `tests.rs`, or anything
/// under a `tests/` directory.
fn is_test_corpus(p: &Path) -> bool {
    p.file_name().is_some_and(|n| n == "tests.rs")
        || p.parent()
            .is_some_and(|d| d.components().any(|c| c.as_os_str() == "tests"))
}

/// Every `.rs` file under `dir`, recursively — forgiving, like the citation
/// guard's sweep; [`the_sweep_is_not_vacuous`] is what keeps "nothing" from
/// passing.
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

fn production_files() -> Vec<String> {
    let mut files = Vec::new();
    rust_files(Path::new("src"), &mut files);
    files.retain(|p| !is_test_corpus(p));
    let mut out: Vec<String> = files.iter().map(|p| p.display().to_string()).collect();
    out.sort();
    out
}

/// DESIGN.md, read forgivingly — an unreadable doc yields an empty map, and
/// [`the_sweep_is_not_vacuous`] is what turns that into a failure.
fn design() -> String {
    std::fs::read_to_string("docs/DESIGN.md").unwrap_or_default()
}

#[test]
fn every_source_file_has_a_row() {
    let mapped = mapped(&design());
    let missing: Vec<String> = production_files()
        .into_iter()
        .filter(|p| !mapped.contains(p))
        .collect();
    assert!(
        missing.is_empty(),
        "tracked source files with no DESIGN §12 row — a split nobody \
         recorded. Add the row (AGENTS.md: \"add the row to DESIGN §12\"):\n{}",
        missing.join("\n")
    );
}

#[test]
fn every_row_names_a_file_that_exists() {
    let ghosts: Vec<String> = mapped(&design())
        .into_iter()
        .filter(|p| !Path::new(p).exists())
        .collect();
    assert!(
        ghosts.is_empty(),
        "DESIGN §12 rows naming paths that do not exist — a map of a tree that \
         is not there:\n{}",
        ghosts.join("\n")
    );
}

#[test]
fn the_sweep_is_not_vacuous() {
    let mapped = mapped(&design());
    let files = production_files();
    assert!(
        mapped.len() > 100 && files.len() > 100,
        "the sweep found {} rows over {} files — it is broken, not the tree",
        mapped.len(),
        files.len()
    );
}

#[test]
fn braces_expand() {
    assert_eq!(expand("src/a.rs"), vec!["src/a.rs".to_owned()]);
    assert_eq!(
        expand("src/x/{mod,y}.rs"),
        vec!["src/x/mod.rs".to_owned(), "src/x/y.rs".to_owned()]
    );
    assert_eq!(
        expand("src/x{.rs,/y.rs}"),
        vec!["src/x.rs".to_owned(), "src/x/y.rs".to_owned()]
    );
    assert_eq!(
        expand("src/{a,b}/{c,d}.rs"),
        vec![
            "src/a/c.rs".to_owned(),
            "src/a/d.rs".to_owned(),
            "src/b/c.rs".to_owned(),
            "src/b/d.rs".to_owned()
        ]
    );
}
