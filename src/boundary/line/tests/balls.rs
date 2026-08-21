//! The `bl` family's line grammar, where it refuses (§8.5, bl-dbde). The four
//! scheduling facts are said with eight flags and each one can be mistyped in
//! exactly two ways — a value where none belongs, or none where one does — so
//! the refusals are the surface's other half.

use super::ctx;
use super::refusals::refuses;

/// A flag that carries a value refuses a bare one, and a clearing flag refuses
/// a value: `--no-priority 3` is a typo the operator must see, not a priority
/// silently dropped on the floor.
#[test]
fn a_scheduling_flag_refuses_the_shape_it_does_not_take() {
    for line in [
        "/create t --tag",
        "/create t --priority",
        "/update bl-1 --parent",
        "/update bl-1 --needs",
        "/update bl-1 --no-tag",
        "/update bl-1 --no-needs",
    ] {
        refuses(line, &ctx(), "needs a value");
    }
    for line in ["/create t --no-priority 3", "/update bl-1 --no-parent bl-2"] {
        refuses(line, &ctx(), "takes no value");
    }
}

/// A priority is the one fact yog judges itself — everything else balls rules
/// on, and its refusal rides back verbatim.
#[test]
fn a_priority_that_is_not_a_number_refuses_here() {
    refuses(
        "/create t --priority high",
        &ctx(),
        "--priority takes a number, got \"high\"",
    );
}

/// A near-miss is named rather than dropped, and the refusal lists what the
/// verb does take — the eight included.
#[test]
fn a_misspelled_scheduling_flag_is_named() {
    refuses("/create t --tags x", &ctx(), "unknown flag --tags");
    refuses("/update bl-1 --priorty 1", &ctx(), "unknown flag --priorty");
    refuses("/create t --no-tags x", &ctx(), "--priority, --no-priority");
}
