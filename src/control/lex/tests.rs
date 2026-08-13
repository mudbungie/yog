//! The classification lexer: what a `bash` string actually runs.

use super::*;

/// Every segment's program word, for the compact assertions below.
fn programs(command: &str) -> Vec<String> {
    segments(command)
        .into_iter()
        .filter_map(|s| s.words.first().cloned())
        .collect()
}

#[test]
fn one_command_is_one_segment_with_its_words() {
    let segs = segments("ls -la /tmp");
    assert_eq!(segs.len(), 1);
    assert_eq!(segs[0].words, ["ls", "-la", "/tmp"]);
    assert!(segs[0].redirects.is_empty());
}

#[test]
fn every_separator_cuts_a_segment() {
    // `;`, `&&`, `||`, `|`, `&`, a newline and an input redirect all end one
    // program and start another — the whole point: a rule matching only the
    // leading word would call this a read.
    let mut got = programs("ls; a && b || c | d & e\nf < g");
    got.sort();
    assert_eq!(got, ["a", "b", "c", "d", "e", "f", "g", "ls"]);
}

#[test]
fn quotes_and_escapes_keep_a_word_whole() {
    let segs = segments(r#"echo 'a; b' "c && d" e\ f"#);
    assert_eq!(segs.len(), 1);
    assert_eq!(segs[0].words, ["echo", "a; b", "c && d", "e f"]);
    // A trailing backslash consumes nothing and completes the word.
    assert_eq!(segments("echo x\\").len(), 1);
}

#[test]
fn a_write_redirect_is_held_apart_from_the_words() {
    let segs = segments("echo hi > /etc/hosts");
    assert_eq!(segs[0].words, ["echo", "hi"]);
    assert_eq!(segs[0].redirects, ["/etc/hosts"]);
    // `>>` is one redirect, not two, and a `2>` leaves its fd digit in argv.
    let segs = segments("cargo test 2>> log.txt");
    assert_eq!(segs[0].redirects, ["log.txt"]);
}

#[test]
fn a_substitution_is_lifted_out_as_its_own_command() {
    // Backticks, `$(…)` with nesting, and a substitution inside double quotes:
    // each is a command, and a command the classifier cannot see is one that
    // runs unclassified.
    let mut got = programs("cd `git rev-parse --show-toplevel`");
    got.sort();
    assert_eq!(got, ["cd", "git"]);
    let mut got = programs("echo $(id $(whoami))");
    got.sort();
    assert_eq!(got, ["echo", "id", "whoami"]);
    let mut got = programs("echo \"$(curl x)\"");
    got.sort();
    assert_eq!(got, ["curl", "echo"]);
    // Single quotes are literal: nothing is lifted out of them.
    assert_eq!(programs("echo '$(curl x)'"), ["echo"]);
}

#[test]
fn an_empty_or_separator_only_command_yields_nothing() {
    assert!(segments("").is_empty());
    assert!(segments("   ;;  && ").is_empty());
}

#[test]
fn the_substitution_worklist_is_capped_by_an_unclassifiable_segment() {
    // Deeply chained substitutions stop being lexed at the cap, and the tail
    // becomes a segment no rule can match rather than silently vanishing.
    let command = "a ".to_owned() + &"$(b ".repeat(MAX_TEXTS + 4) + &")".repeat(MAX_TEXTS + 4);
    let segs = segments(&command);
    assert!(
        segs.iter().any(|s| s.words == [OVERFLOW_WORD]),
        "the cap must leave a mark: {segs:?}"
    );
}
