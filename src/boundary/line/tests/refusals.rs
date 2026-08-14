//! Every way a line fails to be a gesture (§8.5). The rule under all of them:
//! **a missing parameter refuses by name, never by default** — a guessed
//! target would mutate the wrong workspace, ball, or conversation, and the line
//! is an instruction, not an observation.

use super::{ctx, prepared};
use crate::boundary::line::{Context, parse};
use crate::start::BallSpec;
use std::path::PathBuf;

/// Assert `line`, read at `ctx`, refuses with a reason containing `needle`.
fn refuses(line: &str, ctx: &Context, needle: &str) {
    match parse(line, ctx) {
        Ok(gesture) => panic!("{line:?} should refuse, got {gesture:?}"),
        Err(reason) => assert!(
            reason.contains(needle),
            "{line:?} refused with {reason:?}, wanted {needle:?}"
        ),
    }
}

#[test]
fn a_line_that_is_not_a_command_and_a_verb_that_is_not_one() {
    refuses("close bl-1", &ctx(), "starts with '/'");
    refuses("/enhance", &ctx(), "unknown command /enhance");
}

#[test]
fn a_missing_context_fact_refuses_by_name() {
    let bare = Context::default();
    refuses("/scan", &bare, "no workspace in context");
    refuses(
        "/message hi",
        &Context {
            agent: None,
            ..ctx()
        },
        "no conversation is selected",
    );
    refuses("/close bl-1", &bare, "no project in context");
    refuses(
        "/close bl-1",
        &Context {
            name: None,
            ..ctx()
        },
        "no workspace name in context",
    );
    refuses(
        "/prompt go",
        &Context {
            prepared: None,
            ..ctx()
        },
        "nothing is prepared",
    );
}

/// A ball verb with nothing typed and nothing selected has no target — and a
/// *new* spec is not an id: it has none until `bl create` mints one.
#[test]
fn a_ball_verb_with_no_id_anywhere_refuses() {
    refuses(
        "/close",
        &Context {
            ball: None,
            ..ctx()
        },
        "no ball id",
    );
    let minting = Context {
        ball: Some(BallSpec::New {
            title: "t".to_owned(),
            body: String::new(),
        }),
        ..ctx()
    };
    refuses("/release", &minting, "no ball id");
}

#[test]
fn a_verb_that_needs_text_refuses_when_it_gets_none() {
    refuses("/message", &ctx(), "text to send is required");
    refuses("/message    ", &ctx(), "text to send is required");
    refuses("/prompt", &ctx(), "the goal is required");
    refuses("/delete-workspace", &ctx(), "typed out");
    refuses("/create --body b", &ctx(), "a title is required");
    refuses("/prepare dir", &ctx(), "a work directory is required");
    refuses("/fleet", &ctx(), "the cap");
    // The roster is a question about one row (§9.4, bl-dff8): with no row
    // named there is nothing to ask, and yog picks no provider on its own.
    refuses("/models", &ctx(), "usage: /models <provider>");
}

/// A cap yog cannot read is a refusal naming the word, never a guessed number:
/// the cap decides how many drones spend money at once.
#[test]
fn a_fleet_arm_with_an_unreadable_cap_refuses() {
    refuses("/fleet lots", &ctx(), "is not a cap");
}

#[test]
fn a_verb_that_takes_nothing_refuses_arguments() {
    for line in [
        "/scan now",
        "/ack all",
        "/clear-trail please",
        "/workspaces all",
        "/conversations here",
        "/balls mine",
        "/disband now",
        "/lineages default",
    ] {
        refuses(line, &ctx(), "takes no arguments");
    }
}

#[test]
fn the_shaped_verbs_refuse_a_shape_they_do_not_have() {
    refuses("/close bl-1 bl-2", &ctx(), "at most one word");
    refuses("/stop hard", &ctx(), "usage: /stop [children]");
    refuses("/move", &ctx(), "usage: /move [id] <to>");
    refuses("/move a b c", &ctx(), "usage: /move [id] <to>");
    refuses("/ops soon", &ctx(), "not a row count");
}

#[test]
fn a_flag_is_never_silently_dropped() {
    refuses("/create t --boddy x", &ctx(), "unknown flag --boddy");
    refuses("/create t --body", &ctx(), "--body needs a value");
    refuses("/update bl-1 --tilte x", &ctx(), "unknown flag --tilte");
    refuses("/update bl-1", &ctx(), "nothing to change");
}

/// The §3.4 rung is said, never inferred — and an existing ball's roster facts
/// are the seat's, so a seat without one spells the gesture as an envelope.
#[test]
fn the_prepare_rungs_refuse_what_they_cannot_mean() {
    refuses("/prepare sideways", &ctx(), "unknown rung");
    refuses("/prepare ball bl-1", &ctx(), "takes no arguments");
    refuses("/prepare ball --new", &ctx(), "--new needs a value");
    refuses(
        "/prepare ball",
        &Context {
            ball: None,
            ..ctx()
        },
        "no ball is selected",
    );
}

/// The refusal is the reader's; the §3.6 gate is dispatch's. A typed name that
/// does not match still *parses* — fail-closed happens once, at fire time,
/// wherever the gesture came from.
#[test]
fn the_delete_gate_is_not_the_readers_business() {
    assert!(parse("/delete-workspace not-the-name", &ctx()).is_ok());
    assert_eq!(prepared().name, "koi");
}

/// A flag value keeps its inner spacing collapsed — the line's own rule — and
/// a value that follows another word joins it.
#[test]
fn a_flag_value_is_the_words_after_it() {
    let Ok(gesture) = parse("/create a  title --body two  words here", &ctx()) else {
        panic!("should read");
    };
    assert_eq!(
        crate::boundary::line::spell(&gesture),
        "/create a title --body two words here"
    );
    assert_eq!(PathBuf::from("/proj"), ctx().project.unwrap_or_default());
}

/// **S12-T5 three-spellings** (the refusal half): every parameter the attempt
/// cannot invent refuses by name. A fork with no ref is a different gesture, a
/// fork with no role names no model, and an empty goal is not a goal.
#[test]
fn an_attempt_refuses_every_parameter_it_cannot_invent() {
    let full = "/fork --from aaaa1111 --role worker --goal try it";
    refuses("/fork --role worker --goal g", &ctx(), "--from is required");
    refuses("/fork --from r --goal g", &ctx(), "--role is required");
    refuses("/fork --from r --role worker", &ctx(), "--goal is required");
    refuses(
        "/fork --from r --role w --goal   ",
        &ctx(),
        "--goal needs a value",
    );
    refuses(
        "/fork --from --role w --goal g",
        &ctx(),
        "--from needs a value",
    );
    refuses(
        "/fork --from r --role w --model x --goal g",
        &ctx(),
        "unknown flag --model",
    );
    refuses(
        "/fork stray --from r --role w --goal g",
        &ctx(),
        "unexpected words",
    );
    refuses(full, &Context::default(), "no workspace in context");
    refuses(
        full,
        &Context {
            agent: None,
            ..ctx()
        },
        "no conversation is selected",
    );
}

/// A patch read names a ball **and** a path or neither: one word cannot say
/// which attempt's diff it belongs to, and a reader that guessed would open
/// the wrong file.
/// The fan refuses every parameter it cannot invent: the count, a count that
/// is not one, the prepared start, the project and the ball — and a retirement
/// refuses a nameless handle. A line names what a seat has selected and never
/// guesses at the rest.
#[test]
fn the_fan_refuses_every_parameter_it_cannot_invent() {
    refuses("/fan", &ctx(), "how many candidates is required");
    refuses("/fan lots", &ctx(), "is not a count");
    refuses("/retire", &ctx(), "the candidate handle is required");
    let mut seat = ctx();
    seat.prepared = None;
    refuses("/fan 3", &seat, "nothing is prepared");
    let mut seat = ctx();
    seat.project = None;
    refuses("/fan 3", &seat, "no project in context");
    refuses("/retire at-0badcafe", &seat, "no project in context");
    let mut seat = ctx();
    seat.ball = None;
    refuses("/fan 3", &seat, "no ball id");
    refuses("/retire at-0badcafe", &seat, "no ball id");
}

#[test]
fn a_half_named_work_file_is_refused() {
    assert!(parse("/work-diff", &ctx()).is_ok());
    assert!(parse("/work-diff bl-1 src/a.rs", &ctx()).is_ok());
    let Err(reason) = parse("/work-diff src/a.rs", &ctx()) else {
        panic!("one word names no attempt");
    };
    assert!(reason.contains("/work-diff [<ball> <path>]"), "{reason}");
}
