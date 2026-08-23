//! The §11 inspector family's half of the parity table (bl-6233, bl-13f9): the
//! conversation reads spell as lines and read back as themselves at the seat
//! they were spelled from.
//!
//! Split from [`super`] at §12's cap, on the seam the family already draws:
//! these are the only reads aimed at a *conversation*, so they are the only
//! ones that elide two halves of an address rather than one.

use super::super::ctx;
use super::rt;
use crate::boundary::line::{Context, parse};
use crate::boundary::{Gesture, Query};

fn at() -> (String, String) {
    ("ws".to_owned(), "c-1".to_owned())
}

/// Aimed by the seat, exactly as `/message` is — so most are the verb alone,
/// and the round trip holds modulo context.
#[test]
fn every_conversation_read_round_trips() {
    let (workspace, agent) = at();
    for query in [
        Query::Transcript {
            workspace: workspace.clone(),
            agent: agent.clone(),
        },
        Query::Follow {
            workspace: workspace.clone(),
            agent: agent.clone(),
        },
        Query::Steps {
            workspace: workspace.clone(),
            agent: agent.clone(),
        },
        Query::Rail {
            workspace: workspace.clone(),
            agent: agent.clone(),
        },
        Query::Inbox {
            workspace: workspace.clone(),
            agent: agent.clone(),
        },
        Query::Step {
            workspace: workspace.clone(),
            agent: agent.clone(),
            seq: "003".to_owned(),
        },
        Query::Agent {
            workspace: workspace.clone(),
            agent: agent.clone(),
        },
    ] {
        rt(Gesture::Ask(query));
    }
    for path in [None, Some("src/a.rs".to_owned())] {
        for at in [None, Some("abcdef1234".to_owned())] {
            rt(Gesture::Ask(Query::Files {
                workspace: workspace.clone(),
                agent: agent.clone(),
                path: path.clone(),
                at,
            }));
        }
    }
    // Config-frozen-at (bl-13f9): the same optional commit at the family's
    // other tree-subject read, and bare it is the conversation's own tip.
    for at in [None, Some("abcdef1234".to_owned())] {
        rt(Gesture::Ask(Query::Governing {
            workspace: workspace.clone(),
            agent: agent.clone(),
            at,
        }));
    }
}

/// The address is refused by name, never guessed: a conversation read aimed at
/// nothing would answer about a different chat.
#[test]
fn a_conversation_read_at_an_unaimed_seat_refuses_by_name() {
    for verb in ["transcript", "steps", "rail", "inbox", "files", "governing"] {
        let unfocused =
            parse(&format!("/{verb}"), &Context::default()).expect_err("no workspace is focused");
        assert!(unfocused.contains("no workspace in context"), "{unfocused}");
        let unselected = parse(
            &format!("/{verb}"),
            &Context {
                workspace: Some("ws".to_owned()),
                ..Context::default()
            },
        )
        .expect_err("no conversation is selected");
        assert!(
            unselected.contains("no conversation is selected"),
            "{unselected}"
        );
    }
}

/// What each line *does* state is the one thing no seat can supply — and the
/// grammar refuses the rest rather than reading it as a parameter.
#[test]
fn the_bare_reads_take_no_words_and_a_step_names_its_step() {
    for verb in ["transcript", "steps", "rail", "inbox", "governing"] {
        let extra = parse(&format!("/{verb} 003"), &ctx()).expect_err("it takes no arguments");
        assert!(extra.contains("takes no arguments"), "{extra}");
    }
    let bare = parse("/step", &ctx()).expect_err("some step is not a question");
    assert!(bare.contains("usage: /step <seq>"), "{bare}");
    let two = parse("/files a b", &ctx()).expect_err("a path is one word");
    assert!(two.contains("at most one word"), "{two}");
    // `/governing` takes a commit only behind its flag: a bare word would read
    // as a positional the grammar has no place for.
    let word = parse("/governing abcdef1", &ctx()).expect_err("the commit rides --at");
    assert!(word.contains("takes no arguments"), "{word}");
}
