//! What a hit *says*: the windowed matched line, the row label every seat
//! prints, and the two enums' headless tokens.

use super::*;

#[test]
fn a_long_line_is_windowed_around_the_match_at_char_boundaries() {
    let head = "ä".repeat(200);
    let text = format!("first line\n{head}NEEDLE{}", "z".repeat(400));
    let at = text.find("NEEDLE").unwrap();
    let out = excerpt(&text, at);
    assert!(out.starts_with('…') && out.ends_with('…'), "{out}");
    assert!(out.contains("NEEDLE"), "{out}");
    assert!(
        !out.contains("first line"),
        "the match's own line only: {out}"
    );
    assert!(out.len() <= 200, "bounded: {}", out.len());
}

#[test]
fn an_excerpt_of_a_short_line_is_that_line_whole_and_unmarked() {
    assert_eq!(excerpt("alpha\nbeta gamma\ndelta", 6), "beta gamma");
    assert_eq!(
        excerpt("only", 99),
        "only",
        "an offset past the end is the tail"
    );
}

#[test]
fn a_hit_labels_as_what_it_is_and_where() {
    let hit = |at| Hit {
        at,
        field: Field::Name,
        offset: 0,
        excerpt: "x".to_owned(),
    };
    assert_eq!(
        label(&hit(Address::Ball {
            project: "p".to_owned(),
            id: "bl-1".to_owned()
        })),
        "ball bl-1 — x"
    );
    assert_eq!(
        label(&hit(Address::Workspace {
            name: "storm".to_owned()
        })),
        "workspace storm — x"
    );
    assert_eq!(
        label(&hit(Address::Conversation {
            workspace: "storm".to_owned(),
            agent: "aa".to_owned()
        })),
        "storm/aa — x"
    );
}

#[test]
fn the_tokens_are_the_headless_spelling_of_the_two_enums() {
    assert_eq!(
        [Field::Name, Field::Summary, Field::Text].map(Field::token),
        ["name", "summary", "text"]
    );
    assert_eq!(
        Address::Ball {
            project: String::new(),
            id: String::new()
        }
        .token(),
        "ball"
    );
    assert_eq!(
        Address::Workspace {
            name: String::new()
        }
        .token(),
        "workspace"
    );
    assert_eq!(
        Address::Conversation {
            workspace: String::new(),
            agent: String::new()
        }
        .token(),
        "conversation"
    );
}
