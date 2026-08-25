//! **The throwaway repository these beats are driven over**, and the two
//! things they read: the scanner's own fixtures, and the rule table's declared
//! text. Split from the beats at §12's budget on the seam the sibling
//! `leak_store_gate` suite already runs on — what a drive *sets up* is one
//! subject, what it must then *see* is another.
//!
//! Nothing here restates leak material: the probe bytes are the scanner's own
//! fixtures, read out of `scripts/leak-fixtures/`, so an example of a leak
//! lives in exactly one directory.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

/// This repository, whose scanner and fixtures are the subject.
pub fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// One of the scanner's declared fixtures, verbatim.
pub fn fixture(name: &str) -> Vec<u8> {
    fs::read(repo().join("scripts/leak-fixtures").join(name)).unwrap()
}

/// The 1-based line numbers of a fixture's cases (its non-comment lines).
pub fn cases(name: &str) -> Vec<usize> {
    let text = String::from_utf8(fixture(name)).unwrap();
    text.lines()
        .enumerate()
        .filter(|(_, l)| !l.is_empty() && !l.starts_with('#'))
        .map(|(i, _)| i + 1)
        .collect()
}

fn git(dir: &Path, args: &[&str]) {
    let status = yog::git_env::git()
        .current_dir(dir)
        .args(args)
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?}");
}

/// A throwaway repository carrying this repo's scanner and the given files,
/// all staged. Nothing is committed: the index is the subject.
pub fn staged(files: &[(&str, Vec<u8>)]) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("scripts")).unwrap();
    // The whole scanner, which is three files since bl-7547 cut the modes off
    // the mechanism: a copy that carried only the entry point would resolve no
    // `scan_tree` and fail for a reason that is not the tree's.
    for name in ["leak-scan.sh", "leak-modes.sh", "leak-rules.sh"] {
        fs::copy(
            repo().join("scripts").join(name),
            root.join("scripts").join(name),
        )
        .unwrap();
    }
    git(root, &["init", "-q", "-b", "main", "."]);
    for (path, body) in files {
        let at = root.join(path);
        fs::create_dir_all(at.parent().unwrap()).unwrap();
        fs::write(at, body).unwrap();
    }
    git(root, &["add", "-A"]);
    dir
}

/// Run the tree scan. `home` is the account the scan believes it runs as —
/// a parameter because what this gate catches must not depend on it.
pub fn scan(dir: &Path, home: &Path) -> (bool, String) {
    let out = yog::git_env::command(Path::new("bash"))
        .current_dir(dir)
        .env("HOME", home)
        .arg("scripts/leak-scan.sh")
        .output()
        .unwrap();
    (out.status.success(), String::from_utf8(out.stderr).unwrap())
}

/// The scan must reject `dir`; returns what it said.
pub fn findings(dir: &TempDir) -> String {
    let (ok, err) = scan(dir.path(), dir.path());
    assert!(!ok, "the scan passed a tree it must reject:\n{err}");
    err
}

/// The text a single-quoted shell assignment binds in `scripts/leak-rules.sh`
/// — read from the table itself so these tests cannot drift from it.
pub fn declared(open: &str, close: &str) -> String {
    let rules = fs::read_to_string(repo().join("scripts/leak-rules.sh")).unwrap();
    let tail = rules.split_once(open).unwrap().1.to_owned();
    tail.split_once(close).unwrap().0.to_owned()
}

/// The content rules, as the table lists them.
pub fn content_rules() -> Vec<String> {
    declared("RULES=(", ")")
        .split_whitespace()
        .map(str::to_owned)
        .collect()
}
