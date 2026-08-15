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
//! a string.** A section number is a coordinate into a document the operator
//! does not have; every one that reached a rendered string —
//! `"§9.5 — every setting is a control over the file that holds it"`, `"raw
//! config.toml — bz is the only lawful parser (§9.1)"`, a dozen more hovers and
//! refusals — asked them to dereference it. [`the_operator_never_reads_a_section_number`]
//! makes the sweep an invariant rather than a one-time cleanup, with no
//! allowlist to grow: the only place a `§` may still stand in a string is test
//! code, whose assertion messages address the author.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Section keys lawfully cited bare that belong to *other* repos' docs:
/// lernie ARCH §2.2–§2.11 / §4.3 / §4.4, brazen arch §5.5. DESIGN has no such
/// headings (see `foreign_keys_are_not_design_headings`).
const FOREIGN: &[&str] = &[
    // lernie ARCH coordinates, cited bare. §2.5 is caller-supplied pinned
    // documents — the mechanism DESIGN §3.7's instruction freeze rides.
    "2.2", "2.3", "2.4", "2.5", "2.6", "2.9", "2.10", "2.11", "4.3", "4.4", "5.5",
    // VISION.md's §4.8 (the control-boundary ruling), §4.9 (the alignment
    // monitor, bl-af1a), §4.10 (the project-delivery contract, bl-2b8c) and
    // §4.11 (the capability boundary, bl-0cea), always cited with their doc
    // prefix ("VISION §4.8", "VISION §4.9", …) — foreign here because the
    // scanner is deliberately prefix-blind.
    "4.8", "4.9", "4.10", "4.11",
    // REMOTE.md's own rulings — §1.2 (one method, one channel), §1.3 (the
    // channel is mTLS), §1.4 (bootstrapping is out-of-channel) and §1.5 (the
    // workspace is the trust domain, bl-8bbc) — always cited with their doc
    // prefix ("REMOTE §1.4"), foreign here for the same reason VISION's are:
    // the scanner is deliberately prefix-blind, and DESIGN §1 has no
    // subsections at all (bl-b6fa).
    "1.2", "1.3", "1.4", "1.5",
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
fn rust_files(dir: &PathBuf, out: &mut Vec<PathBuf>) {
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

/// Test code, where a `§` in a string is an assertion message the author reads.
/// Everything else under `src/` is what the operator reads, or one hand-off from
/// it: a `help`, a hover, a refusal, a label.
fn is_test_file(p: &Path) -> bool {
    p.components().any(|c| c.as_os_str() == "tests")
        || p.file_name().is_some_and(|n| n == "tests.rs")
        || p.starts_with("src/shell/acceptance")
}

/// `text` up to its inline `#[cfg(test)] mod tests { … }`, which is test code
/// living beside the module it tests. Truncation keeps every line number.
fn without_inline_tests(text: &str) -> String {
    let cut = text.find("#[cfg(test)]\nmod tests {").unwrap_or(text.len());
    text.get(..cut).unwrap_or(text).to_owned()
}

/// The 1-based lines on which a `§` stands **inside a string literal** —
/// comments, which are the lawful home of a citation, are skipped. Strings may
/// span lines (every `help` in the config schema does), so the state is carried
/// across newlines rather than decided per line.
fn quoted_section_marks(text: &str) -> Vec<usize> {
    let (mut hits, mut line) = (Vec::new(), 1);
    let (mut in_string, mut escaped, mut block) = (false, false, 0u32);
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\n' {
            line += 1;
        }
        if escaped {
            escaped = false;
        } else if block > 0 {
            match (c, chars.peek()) {
                ('*', Some('/')) => block -= 1,
                ('/', Some('*')) => block += 1,
                _ => continue,
            }
            chars.next();
        } else if in_string {
            match c {
                '\\' => escaped = true,
                '"' => in_string = false,
                '§' => hits.push(line),
                _ => {}
            }
        } else {
            match (c, chars.peek()) {
                ('"', _) => in_string = true,
                ('/', Some('/')) => {
                    while chars.next().is_some_and(|d| d != '\n') {}
                    line += 1;
                }
                ('/', Some('*')) => {
                    chars.next();
                    block = 1;
                }
                _ => {}
            }
        }
    }
    hits
}

#[test]
fn the_operator_never_reads_a_section_number() {
    let mut files = Vec::new();
    rust_files(&PathBuf::from("src"), &mut files);
    files.sort();
    let mut scanned = 0;
    let mut leaks = Vec::new();
    for f in files.iter().filter(|f| !is_test_file(f)) {
        scanned += 1;
        let text = without_inline_tests(&std::fs::read_to_string(f).unwrap_or_default());
        leaks.extend(
            quoted_section_marks(&text)
                .into_iter()
                .map(|n| format!("{}:{n}", f.display())),
        );
    }
    assert!(scanned >= 100, "only {scanned} non-test files scanned");
    assert!(
        leaks.is_empty(),
        "a `§` reached a string the operator can read — say the thing, do not \
         cite the section:\n{}",
        leaks.join("\n")
    );
}

#[test]
fn the_scanner_reads_strings_and_never_comments() {
    assert_eq!(quoted_section_marks("// §1\nlet s = \"a §2\";\n"), vec![2]);
    assert!(quoted_section_marks("/* §1 /* §2 */ §3 */\n/// §4\n").is_empty());
    assert_eq!(
        quoted_section_marks("let s = \"one \\\"§2\\\"\";\n"),
        vec![1]
    );
    // A multi-line literal keeps its state across the newline (line 2), and an
    // inline test module is cut away before the scan ever sees it.
    assert_eq!(quoted_section_marks("let s = \"one\n§2\";\n"), vec![2]);
    let with_tests = "let s = \"clean\";\n#[cfg(test)]\nmod tests {\n    \"§9\"\n}\n";
    assert!(quoted_section_marks(&without_inline_tests(with_tests)).is_empty());
    assert!(without_inline_tests(with_tests).contains("clean"));
    assert!(is_test_file(Path::new("src/app/tests/focus.rs")));
    assert!(is_test_file(Path::new("src/jsonview/tests.rs")));
    assert!(!is_test_file(Path::new("src/shell/config_edit/mod.rs")));
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
