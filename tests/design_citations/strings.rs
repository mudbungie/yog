//! **The other direction** (bl-cdd2): a `§` belongs in a comment and never in
//! a string. A section number is a coordinate into a document the operator
//! does not have, so every one that reached a rendered string asked them to
//! dereference it. Split from the citation guard at §12's budget on the seam
//! that file's own doc already draws — one half asks *does this citation
//! resolve*, this half asks *should this citation be here at all* — and it
//! carries its own scanner, whose behaviour is pinned beside it because no
//! allowlist stands behind it.

use super::rust_files;
use std::path::{Path, PathBuf};

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
