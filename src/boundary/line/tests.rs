//! The line's parity tables (§8.5): **every** gesture spells as a line and
//! reads back as itself, and the roster and the reader name exactly the same
//! verbs. The refusals — every way a line can fail to be a gesture — are the
//! sibling file.

use super::*;
use crate::actions::verbs::edit;
use crate::boundary::{Action, Gesture, Query, help};
use crate::opslog::Origin;
use crate::projects::join::JoinState;
use crate::start::{BallSpec, Prepared};
use std::path::PathBuf;

mod balls;
mod parity;
mod refusals;

/// The seat every round trip is typed at: one workspace focused, one
/// conversation selected, one project, one ball, one prepared start. Exactly
/// the facts the composer holds when its Enter fires.
pub(super) fn ctx() -> Context {
    Context {
        workspace: Some("ws".to_owned()),
        agent: Some("c-1".to_owned()),
        project: Some("proj".to_owned()),
        name: Some("alba".to_owned()),
        ball: Some(existing()),
        prepared: Some(prepared()),
    }
}

pub(super) fn existing() -> BallSpec {
    BallSpec::Existing {
        id: "bl-1".to_owned(),
        title: "a title".to_owned(),
        body: "a body".to_owned(),
        join: JoinState::ReadyStartable,
        tags: Vec::new(),
    }
}

pub(super) fn prepared() -> Prepared {
    Prepared {
        workspace: "ws".to_owned(),
        binding: Some(PathBuf::from("/ws/work")),
        goal: String::new(),
        origin: Origin::Conversation,
        lineage: None,
    }
}

/// The table is the single source: every advertised verb reads, and every verb
/// that reads is advertised. A spelling that drifts either way is a control the
/// operator is told about and cannot use, or can use and is never told about
/// (§11). And **every one of them answers `--help`** — the higher-order rule,
/// asserted over the whole surface rather than trusted per arm.
#[test]
fn the_table_and_the_reader_name_the_same_verbs() {
    for row in &help::table() {
        let line = format!("/{}", row.verb);
        let read = parse(&line, &ctx());
        assert!(
            read != Err(format!("unknown command /{}\n{}", row.verb, help::roster())),
            "{line} is advertised and unreadable"
        );
        assert!(
            row.usage.starts_with(&format!("/{}", row.verb)),
            "{} states another verb's usage",
            row.verb
        );
        for flag in ["--help", "-h"] {
            assert_eq!(
                parse(&format!("/{} {flag}", row.verb), &ctx()),
                Ok(Gesture::Ask(Query::Help {
                    verb: Some(row.verb.to_owned())
                })),
                "/{} {flag} must be help",
                row.verb
            );
        }
    }
}

/// The context, not the line, carries the target: the same words at a seat
/// with a different selection are the same gesture aimed elsewhere.
#[test]
fn the_seat_supplies_what_the_line_elides() {
    let elsewhere = Context {
        workspace: Some("other".to_owned()),
        agent: Some("c-9".to_owned()),
        ..ctx()
    };
    assert_eq!(
        parse("/message ship it", &elsewhere),
        Ok(Gesture::Act(Action::Message {
            workspace: "other".to_owned(),
            agent: "c-9".to_owned(),
            content: "ship it".to_owned(),
        }))
    );
}

/// The two verbatim payloads (§3.3, bl-6920): the tail is the content, its
/// inner spacing and newlines untouched.
#[test]
fn a_message_and_a_goal_reach_the_boundary_verbatim() {
    let Ok(Gesture::Act(Action::Message { content, .. })) =
        parse("/message  two  spaces\nand a line", &ctx())
    else {
        panic!("not a message");
    };
    assert_eq!(content, "two  spaces\nand a line");
    let Ok(Gesture::Act(Action::Prompt { goal, .. })) = parse("/prompt  keep   this", &ctx())
    else {
        panic!("not a prompt");
    };
    assert_eq!(goal, "keep   this");
}

/// The attempt's goal is the third verbatim payload: everything after
/// `--goal` is the operator's text, flags it happens to mention included —
/// which is why the flags lead and the payload is last.
#[test]
fn an_attempts_goal_is_the_whole_tail_after_the_flag() {
    let Ok(Gesture::Act(Action::Fork { attempt, goal, .. })) = parse(
        "/fork --from r --role w --skills bash, ,read_file --goal  keep   --role  this",
        &ctx(),
    ) else {
        panic!("not a fork");
    };
    assert_eq!(goal, "keep   --role  this");
    assert_eq!(
        attempt.skills,
        vec!["bash".to_owned(), "read_file".to_owned()]
    );
}

/// A ball verb with no id typed acts on the focused ball; a typed id wins.
#[test]
fn a_ball_verb_defaults_to_the_focused_ball() {
    assert_eq!(
        parse("/close", &ctx()),
        Ok(Gesture::Act(Action::Close {
            project: "proj".to_owned(),
            id: "bl-1".to_owned(),
            name: "alba".to_owned(),
        }))
    );
    assert_eq!(
        parse("/update --note ping", &ctx()),
        Ok(Gesture::Act(Action::Update {
            project: "proj".to_owned(),
            id: "bl-1".to_owned(),
            name: "alba".to_owned(),
            fields: edit::Update {
                note: Some("ping".to_owned()),
                ..edit::Update::default()
            },
        }))
    );
}

/// `/ops` with no count reads the trail's working tail.
/// The §11 balls section's own read is a line like any other (bl-b4b5): the
/// workspace is the seat's, so the verb is the whole of it, and it spells back
/// as itself.
#[test]
fn the_workspace_balls_read_takes_its_address_from_the_seat() {
    let Ok(gesture) = parse("/workspace-balls", &ctx()) else {
        panic!("should read");
    };
    assert_eq!(
        gesture,
        Gesture::Ask(Query::WorkspaceBalls {
            workspace: "ws".to_owned()
        })
    );
    assert_eq!(crate::boundary::line::spell(&gesture), "/workspace-balls");
}

#[test]
fn ops_without_a_count_reads_the_default_depth() {
    assert_eq!(
        parse("/ops", &ctx()),
        Ok(Gesture::Ask(Query::Ops { max: 50 }))
    );
}

/// The marker and its escape: one slash commands, two say a slash.
#[test]
fn the_escape_lets_a_message_start_with_a_slash() {
    assert!(is_command("/close"));
    assert!(!is_command("//close"));
    assert!(!is_command(" /close"));
    assert!(!is_command("close"));
    assert_eq!(unescape("//close"), "/close");
    assert_eq!(unescape("plain text"), "plain text");
}

/// Four spellings, one gesture (§8.5): a bare `/`, `/help`, `/help <verb>`,
/// and `<verb> --help` are the same question — asked *about* a command, so
/// never that command's own parameter.
#[test]
fn help_is_one_gesture_however_it_is_asked() {
    let all = Ok(Gesture::Ask(Query::Help { verb: None }));
    assert_eq!(parse("/", &ctx()), all);
    assert_eq!(parse("/help", &ctx()), all);
    let about_close = Ok(Gesture::Ask(Query::Help {
        verb: Some("close".to_owned()),
    }));
    assert_eq!(parse("/help close", &ctx()), about_close);
    assert_eq!(parse("/close --help", &ctx()), about_close);
    assert_eq!(parse("/close -h", &ctx()), about_close);
    // Help answers about itself too — it is a gesture like any other.
    assert_eq!(
        parse("/help --help", &ctx()),
        Ok(Gesture::Ask(Query::Help {
            verb: Some("help".to_owned())
        }))
    );
    // An unknown subject is refused with the roster, not answered emptily.
    let unknown = parse("/help enhance", &ctx());
    assert!(
        matches!(&unknown, Err(reason) if reason.contains("unknown command /enhance")),
        "{unknown:?}"
    );
}

/// The flag form reads **only** when the tail is exactly the flag — which is
/// what keeps the two verbatim payloads (§3.3) whole.
#[test]
fn a_verbatim_payload_may_still_mention_the_flag() {
    let Ok(Gesture::Act(Action::Message { content, .. })) =
        parse("/message run --help on it", &ctx())
    else {
        panic!("mentioning the flag must not hijack the message");
    };
    assert_eq!(content, "run --help on it");
}

/// **A `/prompt` with no goal fires the prepared prefill, whole** (bl-06a1).
///
/// The flagship terminal workflow is `/prepare` then `/prompt`, and every
/// conversation it fired went out with the operator's typed sentence ALONE:
/// the path rung lost its `Working directory:` headline and the ball rung lost
/// the ball, including the `Ball <id>:` header §3.2 calls the conversation→ball
/// join. A typed goal is still the whole payload, verbatim and never prefixed —
/// bl-6920's ruling, which a composer seat depends on — so the fix is the empty
/// input, and the three cases are asserted together because they are one rule.
#[test]
fn a_promptless_goal_falls_to_the_prepared_prefill() {
    let ball = Prepared {
        goal: "Ball bl-1: a title\n\na body".to_owned(),
        ..prepared()
    };
    let seat = |p: &Prepared| Context {
        prepared: Some(p.clone()),
        ..ctx()
    };
    let fired = |line: &str, p: &Prepared| match parse(line, &seat(p)) {
        Ok(Gesture::Act(Action::Prompt { goal, .. })) => Ok(goal),
        other => Err(format!("{other:?}")),
    };

    assert_eq!(fired("/prompt", &ball), Ok(ball.goal.clone()));
    // A goal that was typed is the whole payload and nothing is prepended to
    // it: the prefill is a default, never a head.
    assert_eq!(
        fired("/prompt do the thing", &ball),
        Ok("do the thing".to_owned())
    );
    // The bare rung prepares no prefill, so it is the same "the goal is
    // required" refusal it always was — the general path with empty inputs,
    // not an arm of its own.
    assert!(
        fired("/prompt", &prepared())
            .unwrap_err()
            .contains("the goal is required"),
        "a bare rung with nothing typed has nothing to send"
    );
}
