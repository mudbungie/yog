//! The §3.8 fan family's own refusals: the attempt's three required flags, the
//! ball id no spread, retirement or delivery can invent, and the work diff's
//! one-word form that names no attempt. Its own file at §12's cap, on the same
//! family seam `codec/fan.rs` and `boundary/fan` are cut along.

use super::super::ctx;
use super::refuses;
use crate::boundary::line::{Context, parse};

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
/// is not one, the prepared start, the project and the ball — a retirement
/// refuses a nameless handle, and a delivery refuses a nameless handle, a
/// missing summary, and a summary of nothing but whitespace. A line names what
/// a seat has selected and never guesses at the rest.
#[test]
fn the_fan_refuses_every_parameter_it_cannot_invent() {
    refuses("/fan", &ctx(), "how many candidates is required");
    refuses("/fan lots", &ctx(), "is not a count");
    refuses("/retire", &ctx(), "the candidate handle is required");
    refuses("/deliver", &ctx(), "the candidate handle is required");
    refuses(
        "/deliver at-0badcafe",
        &ctx(),
        "the delivery summary is required",
    );
    refuses(
        "/deliver at-0badcafe   ",
        &ctx(),
        "the delivery summary is required",
    );
    let mut seat = ctx();
    seat.prepared = None;
    refuses("/fan 3", &seat, "nothing is prepared");
    let mut seat = ctx();
    seat.project = None;
    refuses("/fan 3", &seat, "no project in context");
    refuses("/retire at-0badcafe", &seat, "no project in context");
    refuses(
        "/deliver at-0badcafe take it",
        &seat,
        "no project in context",
    );
    let mut seat = ctx();
    seat.ball = None;
    refuses("/fan 3", &seat, "no ball id");
    refuses("/retire at-0badcafe", &seat, "no ball id");
    refuses("/deliver at-0badcafe take it", &seat, "no ball id");
}

#[test]
fn a_half_named_work_file_is_refused() {
    assert!(parse("/work-diff", &ctx()).is_ok());
    assert!(parse("/work-diff bl-1 src/a.rs", &ctx()).is_ok());
    // Three words name a fan candidate's file (bl-c2bd).
    assert!(parse("/work-diff bl-1 at-0badcafe src/a.rs", &ctx()).is_ok());
    let Err(reason) = parse("/work-diff src/a.rs", &ctx()) else {
        panic!("one word names no attempt");
    };
    assert!(
        reason.contains("/work-diff [<ball> [<handle>] <path>]"),
        "{reason}"
    );
}
