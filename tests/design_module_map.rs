//! The module-map guard (bl-9f72, widened by bl-273c): **every rule DESIGN §12
//! states about itself is checked here** — nothing else holds them.
//!
//! Why: AGENTS.md says *"Over the cap? Split along a real seam and add the row
//! to DESIGN §12"*. A file with no row is a split nobody recorded; a row naming
//! a file that no longer exists maps a tree that is not there. Both were true
//! at scale when this landed (35 unlisted files, one row whose path had become
//! a directory), because nothing checked — and bl-273c found the same failure
//! one layer in: §12 stated three *more* rules about itself and held none
//! (counts wrong in 90 of 147 numeric rows, ≥75 rows out of sort order, 24 test
//! modules carrying rows). The counts were **subtracted** — a line count beside
//! a file is a second representation of a computable fact, and `make line-cap`
//! is the one definition of the cap — so a §12 row is two cells and a third
//! fails here. A stated rule that nothing checks is how each of them drifted.
//!
//! What is NOT a row, per §12: *"a test module is covered by its production
//! module's row and never earns one of its own"* — the `X.rs` ↔ `X/tests.rs`
//! seam. The file sweep skips those, and a Module cell naming one fails.
//!
//! §12 spells families with brace lists (`src/app/{mod,roots}.rs`), so the
//! parse expands them; a spelling it cannot expand reads as a missing row,
//! which is the right pressure — §12's paths are a contract, not typography.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// DESIGN §12's module-map rows: the **first** table under the §12 heading, its
/// header lines skipped. §12.2's drive-harness table is a second table under
/// the same heading, deliberately ordered by tier rather than by path, so it is
/// not this table and the rules below do not reach it.
fn table_rows(design: &str) -> Vec<&str> {
    design
        .lines()
        .skip_while(|l| !l.starts_with("## 12."))
        .take_while(|l| !l.starts_with("## 13."))
        .skip_while(|l| !l.starts_with("| `"))
        .take_while(|l| l.starts_with("| `"))
        .collect()
}

/// A row's Module cell — the first `|`-delimited field. A row's prose may cite
/// modules too, and those are not entries.
fn module_cell(row: &str) -> &str {
    row.split('|').nth(1).unwrap_or_default()
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

/// Every `.rs` path a row's Module cell names, brace lists expanded, in the
/// order written.
fn entries(row: &str) -> Vec<String> {
    module_cell(row)
        .split('`')
        .skip(1)
        .step_by(2)
        .flat_map(|tok| expand(tok.trim()))
        .filter(|p| p.ends_with(".rs"))
        .collect()
}

/// A test corpus, which §12 rules out of the table: `tests.rs`, or anything
/// under a `tests/` directory.
fn is_test_corpus(p: &Path) -> bool {
    p.file_name().is_some_and(|n| n == "tests.rs")
        || p.parent()
            .is_some_and(|d| d.components().any(|c| c.as_os_str() == "tests"))
}

/// A Module cell entry naming a module's test corpus. The repo's own `tests/`
/// crate root is not one — those are integration binaries the cap governs, and
/// they earn rows of their own.
fn names_test_module(p: &str) -> bool {
    !p.starts_with("tests/") && is_test_corpus(Path::new(p))
}

/// §12: *"Rows stay sorted by module path — inserts distribute instead of
/// stacking at subsystem boundaries"*. The key is the row's first path with a
/// trailing `mod.rs` stripped, so a directory's root module leads its subtree
/// (`src/app/mod.rs` → `src/app/`) instead of landing under `m`.
fn sort_key(row: &str) -> String {
    let first = entries(row).first().cloned().unwrap_or_default();
    first.strip_suffix("mod.rs").unwrap_or(&first).to_owned()
}

/// Paths mapped by more than one row.
fn duplicates(design: &str) -> Vec<String> {
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for path in table_rows(design).into_iter().flat_map(entries) {
        *seen.entry(path).or_default() += 1;
    }
    seen.into_iter()
        .filter(|(_, n)| *n > 1)
        .map(|(path, _)| path)
        .collect()
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
    let design = design();
    let mapped: BTreeSet<String> = table_rows(&design)
        .into_iter()
        .flat_map(entries)
        .filter(|p| p.starts_with("src/"))
        .collect();
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
    let ghosts: Vec<String> = table_rows(&design())
        .into_iter()
        .flat_map(entries)
        .filter(|p| (p.starts_with("src/") || p.starts_with("tests/")) && !Path::new(p).exists())
        .collect();
    assert!(
        ghosts.is_empty(),
        "DESIGN §12 rows naming paths that do not exist — a map of a tree that \
         is not there:\n{}",
        ghosts.join("\n")
    );
}

#[test]
fn rows_stay_sorted_by_module_path() {
    let design = design();
    let keys: Vec<String> = table_rows(&design).into_iter().map(sort_key).collect();
    let descents: Vec<String> = keys
        .windows(2)
        .filter(|w| w[1] < w[0])
        .map(|w| format!("{} then {}", w[0], w[1]))
        .collect();
    assert!(
        descents.is_empty(),
        "DESIGN §12 rows out of sort order — an insert stacked at a subsystem \
         boundary instead of distributing:\n{}",
        descents.join("\n")
    );
}

#[test]
fn no_row_names_a_test_module() {
    let design = design();
    let corpora: Vec<String> = table_rows(&design)
        .into_iter()
        .flat_map(entries)
        .filter(|p| names_test_module(p))
        .collect();
    assert!(
        corpora.is_empty(),
        "DESIGN §12 cells naming test modules — §12: \"a test module … never \
         earns one of its own\". Drop the entry; the row may still describe \
         the corpus:\n{}",
        corpora.join("\n")
    );
}

#[test]
fn a_row_is_two_cells() {
    let design = design();
    let wide: Vec<String> = table_rows(&design)
        .into_iter()
        .filter(|row| row.split('|').count() != 4)
        .map(|row| module_cell(row).trim().to_owned())
        .collect();
    assert!(
        wide.is_empty(),
        "DESIGN §12 rows with a third cell — that is where the line counts \
         lived, and bl-273c subtracted them (160 of 389 had drifted). \
         `make line-cap` is the one definition of the cap:\n{}",
        wide.join("\n")
    );
}

#[test]
fn no_path_is_mapped_twice() {
    let dups = duplicates(&design());
    assert!(
        dups.is_empty(),
        "DESIGN §12 paths in more than one row — two answers to one \
         question:\n{}",
        dups.join("\n")
    );
}

#[test]
fn the_sweep_is_not_vacuous() {
    let (rows, files) = (table_rows(&design()).len(), production_files().len());
    assert!(
        rows > 100 && files > 100,
        "the sweep found {rows} rows over {files} files — it is broken, not \
         the tree"
    );
}

#[test]
fn braces_expand() {
    assert_eq!(expand("src/a.rs"), ["src/a.rs"]);
    assert_eq!(expand("src/x/{mod,y}.rs"), ["src/x/mod.rs", "src/x/y.rs"]);
    assert_eq!(expand("src/x{.rs,/y.rs}"), ["src/x.rs", "src/x/y.rs"]);
    assert_eq!(
        expand("src/{a,b}/{c,d}.rs"),
        ["src/a/c.rs", "src/a/d.rs", "src/b/c.rs", "src/b/d.rs"]
    );
}

/// Each rule above is read off a live doc, so each needs its own negative
/// direction: a hand-built table breaking exactly one rule must be seen.
#[test]
fn each_rule_sees_its_own_violation() {
    let table = |rows: &str| {
        format!("## 12. Module map\n\n| Module | Responsibility |\n|---|---|\n{rows}\n\n## 13. x\n")
    };
    let ordered = |doc: &str| {
        table_rows(doc)
            .windows(2)
            .all(|w| sort_key(w[0]) <= sort_key(w[1]))
    };
    assert!(ordered(&table("| `src/a.rs` | a |\n| `src/b.rs` | b |")));
    assert!(!ordered(&table("| `src/b.rs` | b |\n| `src/a.rs` | a |")));
    // a directory's mod.rs leads its subtree rather than sorting under `m`
    assert!(ordered(&table(
        "| `src/x/{mod,y}.rs` | x |\n| `src/x/z.rs` | z |"
    )));

    let corpus = |doc: &str| {
        table_rows(doc)
            .into_iter()
            .flat_map(entries)
            .any(|p| names_test_module(&p))
    };
    assert!(corpus(&table("| `src/a/{mod,tests}.rs` | a |")));
    assert!(!corpus(&table("| `src/a/mod.rs` | a |")));
    // the repo's own tests/ crate root is not a corpus — its rows are lawful
    assert!(!names_test_module("tests/design_module_map.rs"));

    let two_cells = |doc: &str| table_rows(doc).iter().all(|r| r.split('|').count() == 4);
    assert!(!two_cells(&table("| `src/a.rs` | 120 | a |")));
    assert!(two_cells(&table("| `src/a.rs` | a |")));

    assert!(!duplicates(&table("| `src/a.rs` | one |\n| `src/a{.rs,/b.rs}` | two |")).is_empty());
    assert!(duplicates(&table("| `src/a.rs` | one |\n| `src/b.rs` | two |")).is_empty());

    // §12.2's drive-harness table is a second table and is not read here
    let two = table(
        "| `src/a.rs` | a |\n\n### 12.2 harness\n\n| File | Responsibility |\n\
         |---|---|\n| `scripts/z.sh` | z |",
    );
    assert_eq!(table_rows(&two).len(), 1);
}
