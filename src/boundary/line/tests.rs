//! The line's parity tables (§8.5): **every** gesture spells as a line and
//! reads back as itself, and the roster and the reader name exactly the same
//! verbs. The refusals — every way a line can fail to be a gesture — are the
//! sibling file.

use super::*;
use crate::boundary::{Action, Gesture, Query, help};
use crate::opslog::Origin;
use crate::projects::join::JoinState;
use crate::start::{BallSpec, Prepared};
use std::path::PathBuf;

mod parity;
mod refusals;

/// The seat every round trip is typed at: one workspace focused, one
/// conversation selected, one project, one ball, one prepared start. Exactly
/// the facts the composer holds when its Enter fires.
pub(super) fn ctx() -> Context {
    Context {
        workspace: Some(PathBuf::from("/ws")),
        agent: Some("c-1".to_owned()),
        project: Some(PathBuf::from("/proj")),
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
    }
}

pub(super) fn prepared() -> Prepared {
    Prepared {
        name: "koi".to_owned(),
        workspace: PathBuf::from("/ws"),
        binding: Some(PathBuf::from("/ws/work")),
        goal: String::new(),
        origin: Origin::Conversation,
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
        workspace: Some(PathBuf::from("/other")),
        agent: Some("c-9".to_owned()),
        ..ctx()
    };
    assert_eq!(
        parse("/message ship it", &elsewhere),
        Ok(Gesture::Act(Action::Message {
            workspace: PathBuf::from("/other"),
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
            project: PathBuf::from("/proj"),
            id: "bl-1".to_owned(),
            name: "alba".to_owned(),
        }))
    );
    assert_eq!(
        parse("/move koi", &ctx()),
        Ok(Gesture::Act(Action::Move {
            project: PathBuf::from("/proj"),
            id: "bl-1".to_owned(),
            from: "alba".to_owned(),
            to: "koi".to_owned(),
        }))
    );
    assert_eq!(
        parse("/update --note ping", &ctx()),
        Ok(Gesture::Act(Action::Update {
            project: PathBuf::from("/proj"),
            id: "bl-1".to_owned(),
            name: "alba".to_owned(),
            title: None,
            body: None,
            note: Some("ping".to_owned()),
        }))
    );
}

/// `/ops` with no count reads the trail's working tail.
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
