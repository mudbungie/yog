//! The leak gate's own regression suite (bl-167d). `scripts/leak-scan.sh
//! --self-test` proves each RULE still fires; these tests prove the GATE still
//! covers what it claims to — which is the half that was wrong.
//!
//! The scanner advertised "Reading the INDEX", but `git ls-files` yields path
//! NAMES and the grep behind it opened WORKTREE files: a leak that was `git
//! add`ed and then overwritten with a clean copy on disk was committed with the
//! gate never reading the bytes it was gating. Four more blind spots came with
//! it — binary content was skipped rather than rejected, the home rule was
//! built from the scanner's own `$HOME` (so a run under any other account saw
//! nobody's home paths but its own), no rule covered a personal address or
//! pasted dialogue, and a cached gate verdict could skip the scan entirely.
//!
//! Each test drives the REAL `scripts/leak-scan.sh` over a throwaway
//! repository, so nothing here can pass by agreeing with an out-of-date copy.
//! **The probe material is the scanner's own fixtures**, never restated here:
//! `scripts/leak-fixtures/` is the one place an example of a leak may live, it
//! is already the thing `--self-test` holds to the marker rule, and a test file
//! that spelled its own would be one more file the tree scan has to flag.
//!
//! What a commit hook cannot promise — history, other refs, PR and release
//! text, Actions logs, artifacts, published crate versions — is deliberately
//! untested here, because it is deliberately unpromised: `AGENTS.md` carries it
//! as a release checklist instead.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

/// This repository, whose scanner and fixtures are the subject.
fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// One of the scanner's declared fixtures, verbatim.
fn fixture(name: &str) -> Vec<u8> {
    fs::read(repo().join("scripts/leak-fixtures").join(name)).unwrap()
}

/// The 1-based line numbers of a fixture's cases (its non-comment lines).
fn cases(name: &str) -> Vec<usize> {
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
fn staged(files: &[(&str, Vec<u8>)]) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("scripts")).unwrap();
    for name in ["leak-scan.sh", "leak-rules.sh"] {
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
fn scan(dir: &Path, home: &Path) -> (bool, String) {
    let out = yog::git_env::command(Path::new("bash"))
        .current_dir(dir)
        .env("HOME", home)
        .arg("scripts/leak-scan.sh")
        .output()
        .unwrap();
    (out.status.success(), String::from_utf8(out.stderr).unwrap())
}

/// The scan must reject `dir`; returns what it said.
fn findings(dir: &TempDir) -> String {
    let (ok, err) = scan(dir.path(), dir.path());
    assert!(!ok, "the scan passed a tree it must reject:\n{err}");
    err
}

// 1. The headline: index blobs, not worktree bytes.
#[test]
fn a_staged_leak_is_caught_behind_a_clean_worktree_copy() {
    let dir = staged(&[("probe.txt", fixture("ipv4-routable.txt"))]);
    // The commit will carry the staged blob. The file on disk says otherwise —
    // which is exactly what the old scan read, and passed.
    fs::write(dir.path().join("probe.txt"), fixture("clean.txt")).unwrap();
    let err = findings(&dir);
    assert!(err.contains("[ipv4-routable]"), "{err}");
    for line in cases("ipv4-routable.txt") {
        assert!(
            err.contains(&format!("probe.txt:{line}")),
            "case {line} unread:\n{err}"
        );
    }
}

// 2. Home paths, on three platforms, judged the same by any account.
#[test]
fn home_paths_are_caught_whoever_runs_the_scan() {
    let probe = fixture("home-path.txt");
    let text = String::from_utf8(probe.clone()).unwrap();
    for platform in ["/home/", "/Users/", ":\\Users\\"] {
        assert!(
            text.contains(platform),
            "the fixture stopped covering {platform}"
        );
    }
    let dir = staged(&[
        ("probe.txt", probe),
        ("near-miss.txt", fixture("clean.txt")),
    ]);
    // Two runs as two unrelated accounts, neither of them the one named in the
    // fixture. The verdict is a property of the tree, not of the runner.
    let (as_stranger, first) = scan(dir.path(), dir.path());
    let (as_house, second) = scan(dir.path(), Path::new("/home/u"));
    assert!(
        !as_stranger && !as_house,
        "an unrelated account saw nothing:\n{first}{second}"
    );
    assert_eq!(first, second, "the finding depended on who ran the scan");
    for line in cases("home-path.txt") {
        assert!(
            first.contains(&format!("probe.txt:{line}")),
            "case {line} unread:\n{first}"
        );
    }
    assert!(
        !first.contains("near-miss.txt"),
        "a near-miss was flagged:\n{first}"
    );
}

// 3. Personal correspondence and pasted conversation.
#[test]
fn an_address_and_a_pasted_exchange_are_caught() {
    let dir = staged(&[
        ("mail.txt", fixture("personal-email.txt")),
        ("chat.txt", fixture("quoted-dialogue.txt")),
        ("near-miss.txt", fixture("clean.txt")),
    ]);
    let err = findings(&dir);
    assert!(
        err.contains("[personal-email]") && err.contains("[quoted-dialogue]"),
        "{err}"
    );
    for (file, name) in [
        ("mail", "personal-email.txt"),
        ("chat", "quoted-dialogue.txt"),
    ] {
        for line in cases(name) {
            assert!(
                err.contains(&format!("{file}.txt:{line}")),
                "{name}:{line} unread:\n{err}"
            );
        }
    }
    // The near-misses include a documentation address and prose about an
    // assistant: neither is correspondence, and neither may be flagged.
    assert!(
        !err.contains("near-miss.txt"),
        "a near-miss was flagged:\n{err}"
    );
}

// 4. Unreadable content is rejected, not skipped.
#[test]
fn unreadable_content_is_rejected_rather_than_skipped() {
    let blob = fixture("binary-content.bin");
    let err = findings(&staged(&[("evidence.zip", blob.clone())]));
    assert!(
        err.contains("[binary-content]") && err.contains("evidence.zip"),
        "{err}"
    );
    // The one declared exception: an icon PNG, which `make icon` regenerates
    // and src/theme/icon/tests/artifacts.rs asserts byte for byte.
    let allowed = staged(&[("assets/yog-16.png", blob)]);
    let (ok, err) = scan(allowed.path(), allowed.path());
    assert!(ok, "a declared derivation was rejected:\n{err}");
}

// 5. Every fixture value says, in the value, that it is fabricated.
#[test]
fn every_fixture_value_is_unmistakably_fabricated() {
    let marker = declared("FIXTURE_MARKER='", "'");
    for rule in content_rules() {
        let name = format!("{rule}.txt");
        let text = String::from_utf8(fixture(&name)).unwrap();
        for line in cases(&name) {
            let case = text.lines().nth(line - 1).unwrap().to_lowercase();
            assert!(
                case.contains(&marker),
                "{name}:{line} carries no '{marker}' marker: a fixture holds real-SHAPED \
                 values, and no regex can tell those from real ones"
            );
        }
    }
    // The binary fixture can carry neither a marker per line nor a scan: it is
    // capped instead, far below anything worth smuggling.
    let bin = fixture("binary-content.bin");
    assert!(
        bin.len() <= 512,
        "the unreadable fixture is {} bytes",
        bin.len()
    );
    assert!(
        bin.windows(marker.len()).any(|w| w == marker.as_bytes()),
        "no marker in the bytes"
    );
}

// 6. A gate verdict may skip the build. It may not skip the disclosure scan.
#[test]
fn the_leak_scan_runs_before_the_verdict_cache() {
    let gate = fs::read_to_string(repo().join("scripts/pre-commit")).unwrap();
    let scan_at = gate
        .find("\nmake leak-scan")
        .expect("an unconditional leak-scan step");
    let cache_at = gate
        .find("bl-speculate check")
        .expect("the verdict-cache check");
    assert!(
        scan_at < cache_at,
        "the verdict cache can short-circuit the scan: a stored pass — including one \
         imported from a remote builder — would let a leak through unread"
    );
}

// 7. The gate is not only a local hook: CI reaches it from its entry point.
#[test]
fn ci_reaches_the_leak_scan_from_its_own_entry_point() {
    let ci = fs::read_to_string(repo().join(".github/workflows/ci.yml")).unwrap();
    let make = fs::read_to_string(repo().join("Makefile")).unwrap();
    assert!(
        ci.contains("run: make ci"),
        "CI no longer runs the make entry point"
    );
    for link in [
        "\nci: check\n",
        "\ncheck: fmt-check lint\n\t@scripts/check-coverage.sh\n",
        "\t$(MAKE) leak-scan\n",
    ] {
        assert!(
            make.contains(link),
            "the CI -> leak-scan chain is broken at {link:?}"
        );
    }
    // And the speculative builder runs the same gate file the hook does, so a
    // remote verdict is earned under the same uncached scan.
    let spec = fs::read_to_string(repo().join(".github/workflows/speculate.yml")).unwrap();
    assert!(
        spec.contains("scripts/pre-commit"),
        "the remote builder runs some other gate"
    );
}

/// The text a single-quoted shell assignment binds in `scripts/leak-rules.sh`
/// — read from the table itself so these tests cannot drift from it.
fn declared(open: &str, close: &str) -> String {
    let rules = fs::read_to_string(repo().join("scripts/leak-rules.sh")).unwrap();
    let tail = rules.split_once(open).unwrap().1.to_owned();
    tail.split_once(close).unwrap().0.to_owned()
}

/// The content rules, as the table lists them.
fn content_rules() -> Vec<String> {
    declared("RULES=(", ")")
        .split_whitespace()
        .map(str::to_owned)
        .collect()
}
