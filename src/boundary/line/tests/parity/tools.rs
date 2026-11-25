//! REMOTE §5's pair on the line (bl-4e08): a presentation whose whole tail is a
//! JSON document, and the roster that reads it back.

use super::rt;
use crate::boundary::{Action, Gesture, Query};
use crate::registry::mailbox::{Capture, Completion, Offsets, Piece, Post, Verb};

/// One advertised tool, with a schema deep enough that a spelling which rebuilt
/// it rather than carrying it verbatim would show.
fn advertised() -> crate::registry::tools::Tool {
    crate::registry::tools::Tool {
        name: "Bash".to_owned(),
        description: "run a command".to_owned(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {"command": {"type": "string", "minLength": 1}},
            "required": ["command"],
        }),
        subject_cwd: false,
    }
}

/// The set is the whole tail, so the round trip is what says a **document**
/// survives a line — and the empty set has to be typable, or a tool host could
/// never withdraw one.
#[test]
fn the_presentation_round_trips_with_its_schema_and_when_it_is_empty() {
    let consenting = crate::registry::tools::Tool {
        subject_cwd: true,
        ..advertised()
    };
    for tools in [Vec::new(), vec![advertised()], vec![consenting]] {
        rt(Gesture::Act(Action::Advertise { tools }));
    }
}

/// The routing leg's four (bl-024b). Both acts end in a document taken
/// verbatim, so the round trip is what says a JSON payload survives a line;
/// both reads state only what no seat can supply, which for one of them is
/// nothing at all.
#[test]
fn the_routing_legs_gestures_round_trip() {
    for cwd in [None, Some("/w/home/agents/c-1".to_owned())] {
        rt(Gesture::Act(Action::Route(
            crate::registry::mailbox::Verb::Invoke(crate::registry::mailbox::Call {
                client: "laptop".to_owned(),
                tool: "Bash".to_owned(),
                input: serde_json::json!({"command": "ls -l", "timeout": 30}),
                cwd,
            }),
        )));
    }
    for post in [
        Post::Capture(Capture {
            stdout: "hello\n".to_owned(),
            stderr: "warned\n".to_owned(),
            exit_code: 3,
        }),
        Post::Partial(Piece {
            stdout: "hel".to_owned(),
            stderr: String::new(),
        }),
    ] {
        rt(Gesture::Act(Action::Route(Verb::Complete(Completion {
            invocation: "inv-1".to_owned(),
            post,
        }))));
    }
    for holding in [
        None,
        Some(Vec::new()),
        Some(vec!["inv-1".to_owned(), "inv-2".to_owned()]),
    ] {
        rt(Gesture::Ask(Query::Invocations { holding }));
    }
    for since in [
        None,
        Some(Offsets {
            stdout: 512,
            stderr: 0,
        }),
    ] {
        rt(Gesture::Ask(Query::Capture {
            invocation: "inv-1".to_owned(),
            since,
        }));
    }
}

/// **The streamed form's two line refusals** (REMOTE §5.7): a poll states
/// both counts or neither, each a byte count; and a piece is read as strictly
/// as a capture is.
#[test]
fn a_malformed_offset_or_piece_is_refused_on_the_line() {
    let ctx = crate::boundary::line::Context::default();
    for line in [
        "/capture",
        "/capture inv-1 4",
        "/capture inv-1 4 x",
        "/capture inv-1 x 4",
    ] {
        let e = crate::boundary::line::parse(line, &ctx).expect_err(line);
        assert!(e.contains("<stdout bytes> <stderr bytes>"), "{e}");
    }
    let e = crate::boundary::line::parse("/invocations inv-1", &ctx).expect_err("not a flag");
    assert!(e.contains("--holding"), "{e}");
    let e = crate::boundary::line::parse("/complete inv-1 --partial {\"stdout\":\"a\"}", &ctx)
        .expect_err("a piece states both streams");
    assert!(e.contains("stderr"), "{e}");
}

/// The lane's flag must name a directory: a bare `--cwd` is the line's own
/// refusal, before any gesture exists (bl-77be).
#[test]
fn a_cwd_flag_naming_nothing_is_refused() {
    let e = crate::boundary::line::parse(
        "/invoke laptop Bash --cwd",
        &crate::boundary::line::Context::default(),
    )
    .expect_err("refused");
    assert!(e.contains("--cwd names no directory"), "{e}");
}

/// The roster spells as the verb alone: its workspace is the seat's, exactly as
/// `/providers`' is.
#[test]
fn the_roster_spells_as_the_verb_alone() {
    rt(Gesture::Ask(Query::Clients {
        workspace: "ws".to_owned(),
    }));
}

/// **The doctor spells as the verb alone too, and takes the seat's workspace
/// when it has one** (bl-28f4). A seat with none selected still asks — which no
/// other workspace-addressed read allows, and is the whole point: the box this
/// gesture is for may hold no workspace at all.
#[test]
fn the_doctor_spells_as_the_verb_alone_with_or_without_a_workspace() {
    rt(Gesture::Ask(Query::Doctor {
        workspace: Some("ws".to_owned()),
    }));
    // Unselected is its own gesture and spells the same line; parsed back with a
    // seat that HAS a workspace it becomes that one, which is the context doing
    // its job rather than a round trip failing.
    let bare = Gesture::Ask(Query::Doctor { workspace: None });
    assert_eq!(crate::boundary::line::spell(&bare), "/doctor");
    assert_eq!(
        crate::boundary::line::parse("/doctor", &crate::boundary::line::Context::default()),
        Ok(bare)
    );
}
