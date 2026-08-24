//! **How a §12 row is read** — the table's bounds, a row's Module cell, the
//! brace expansion that makes a family one row, the corpus rule that keeps a
//! test module out of the table, the sort key, and the tree sweep the guards
//! measure the table against. Split from the guards at §12's budget on the
//! seam between *reading* the doc and *what the doc promises about itself*:
//! every rule the guards state is a claim over these functions, and a rule
//! read wrongly is indistinguishable from a rule broken.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// DESIGN §12's module-map rows: the **first** table under the §12 heading, its
/// header lines skipped. §12.2's drive-harness table is a second table under
/// the same heading, deliberately ordered by tier rather than by path, so it is
/// not this table and the rules below do not reach it.
pub(crate) fn table_rows(design: &str) -> Vec<&str> {
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
pub(crate) fn module_cell(row: &str) -> &str {
    row.split('|').nth(1).unwrap_or_default()
}

/// One brace list expanded: `a{b,c}d` → `abd`, `acd`. Nested lists recurse;
/// a token with no brace is itself.
pub(crate) fn expand(tok: &str) -> Vec<String> {
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
pub(crate) fn entries(row: &str) -> Vec<String> {
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
pub(crate) fn is_test_corpus(p: &Path) -> bool {
    p.file_name().is_some_and(|n| n == "tests.rs")
        || p.parent()
            .is_some_and(|d| d.components().any(|c| c.as_os_str() == "tests"))
}

/// A Module cell entry naming a module's test corpus. The repo's own `tests/`
/// crate root is not one — those are integration binaries the cap governs, and
/// they earn rows of their own.
pub(crate) fn names_test_module(p: &str) -> bool {
    !p.starts_with("tests/") && is_test_corpus(Path::new(p))
}

/// §12: *"Rows stay sorted by module path — inserts distribute instead of
/// stacking at subsystem boundaries"*. The key is the row's first path with a
/// trailing `mod.rs` stripped, so a directory's root module leads its subtree
/// (`src/app/mod.rs` → `src/app/`) instead of landing under `m`.
pub(crate) fn sort_key(row: &str) -> String {
    let first = entries(row).first().cloned().unwrap_or_default();
    first.strip_suffix("mod.rs").unwrap_or(&first).to_owned()
}

/// Paths mapped by more than one row.
pub(crate) fn duplicates(design: &str) -> Vec<String> {
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
pub(crate) fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let p = entry.path();
        if p.is_dir() {
            rust_files(&p, out);
        } else if p.extension().is_some_and(|e| e == "rs") {
            out.push(p);
        }
    }
}

pub(crate) fn production_files() -> Vec<String> {
    let mut files = Vec::new();
    rust_files(Path::new("src"), &mut files);
    files.retain(|p| !is_test_corpus(p));
    let mut out: Vec<String> = files.iter().map(|p| p.display().to_string()).collect();
    out.sort();
    out
}

/// DESIGN.md, read forgivingly — an unreadable doc yields an empty map, and
/// [`the_sweep_is_not_vacuous`] is what turns that into a failure.
pub(crate) fn design() -> String {
    std::fs::read_to_string("docs/DESIGN.md").unwrap_or_default()
}
