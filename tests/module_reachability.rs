//! **Every `.rs` under `src/` is declared by its parent module** (bl-862e).
//!
//! `rustc` never looks at a file nobody wrote a `mod` for, so an orphan is not
//! an error — it is *dead text*, and every gate above it agrees the tree is
//! clean: it compiles (it isn't compiled), it is covered (tarpaulin measures
//! what ran), and `tests/design_module_map.rs` skips it by rule, because §12
//! says *"a test module is covered by its production module's row and never
//! earns one of its own"*. That exemption is only sound while the corpus is
//! actually reached from its production module, and nothing checked. It had
//! stopped being true: `src/app/balls/tests/glue/ceiling.rs` carried two
//! spend-ceiling beats that no `mod` reached, so they had not compiled — let
//! alone run — since the day they were written.
//!
//! The guard is stated over the **whole** of `src/` rather than over test
//! corpora, because "a file with no `mod`" is one rule and `tests/` is not a
//! special case of it. Both directions, like `rules-audit`: the real tree must
//! be clean, and a fabricated tree with one orphan must still be caught.

use std::path::{Path, PathBuf};

/// The crate roots, which no `mod` declares.
const ROOTS: [&str; 2] = ["lib.rs", "main.rs"];

/// Every `.rs` file under `dir`, recursively. Forgiving, like the §12 sweep —
/// [`the_sweep_is_not_vacuous`] is what keeps "nothing" from passing.
fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Does `text` declare `mod <name>`? Any visibility, inline or by file; the
/// `#[cfg(test)]` that usually precedes one sits on its own line.
fn declares(text: &str, name: &str) -> bool {
    text.lines().any(|line| {
        let line = line.trim_start();
        let line = line.strip_prefix("pub").map_or(line, |rest| {
            rest.trim_start_matches(|c| c != ' ').trim_start()
        });
        line.strip_prefix("mod ")
            .map(str::trim_start)
            .and_then(|rest| rest.strip_prefix(name))
            .is_some_and(|tail| tail.starts_with([';', '{', ' ']))
    })
}

/// The files under `root` that no parent module declares.
///
/// `src/a/b/c.rs` is the module `c` of `src/a/b/{mod.rs,.rs}`; `src/a/b/mod.rs`
/// is the module `b` of `src/a/{mod.rs,.rs}`; and a file directly under `root`
/// answers to a crate root.
fn orphans(root: &Path) -> Vec<String> {
    let mut files = Vec::new();
    rust_files(root, &mut files);
    files.sort();
    files
        .into_iter()
        .filter(|path| {
            let is_root = path.parent() == Some(root)
                && ROOTS
                    .iter()
                    .any(|r| path.file_name().is_some_and(|n| n == *r));
            !is_root
        })
        .filter(|path| {
            let by_mod_rs = path.file_name().is_some_and(|n| n == "mod.rs");
            let owner = if by_mod_rs {
                path.parent()
            } else {
                Some(path.as_path())
            };
            let Some(owner) = owner else { return false };
            let (Some(name), Some(dir)) = (owner.file_stem(), owner.parent()) else {
                return false;
            };
            let name = name.to_string_lossy();
            let parents: Vec<PathBuf> = if dir == root {
                ROOTS.iter().map(|r| root.join(r)).collect()
            } else {
                vec![dir.join("mod.rs"), dir.with_extension("rs")]
            };
            !parents
                .iter()
                .any(|p| std::fs::read_to_string(p).is_ok_and(|text| declares(&text, &name)))
        })
        .map(|path| path.display().to_string())
        .collect()
}

#[test]
fn every_source_file_is_reached_by_a_mod_declaration() {
    let found = orphans(Path::new("src"));
    assert!(
        found.is_empty(),
        "files under src/ that no `mod` declaration reaches — dead text every \
         other gate reads as clean:\n{}",
        found.join("\n")
    );
}

#[test]
fn the_sweep_is_not_vacuous() {
    let mut files = Vec::new();
    rust_files(Path::new("src"), &mut files);
    assert!(files.len() > 100, "the sweep found {} files", files.len());
}

#[test]
fn an_orphan_in_a_fabricated_tree_is_caught() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::create_dir_all(root.join("app/tests")).unwrap();
    std::fs::write(root.join("lib.rs"), "pub(crate) mod app;\n").unwrap();
    std::fs::write(root.join("app/mod.rs"), "#[cfg(test)]\nmod tests;\n").unwrap();
    std::fs::write(root.join("app/tests/mod.rs"), "mod seen;\n").unwrap();
    std::fs::write(root.join("app/tests/seen.rs"), "").unwrap();
    assert_eq!(orphans(root), Vec::<String>::new(), "the declared tree");

    std::fs::write(root.join("app/tests/orphan.rs"), "").unwrap();
    assert_eq!(
        orphans(root),
        vec![root.join("app/tests/orphan.rs").display().to_string()]
    );
}
