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

// The parser half. `#[path]` because this file IS the test target's crate
// root, so a bare `mod` would resolve to `tests/parse.rs` — and a second
// top-level `tests/*.rs` is a second test binary, not a module.
#[path = "design_module_map/parse.rs"]
mod parse;
use std::collections::BTreeSet;
use std::path::Path;

use parse::{
    design, duplicates, entries, expand, module_cell, names_test_module, production_files,
    sort_key, table_rows,
};

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
