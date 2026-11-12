//! The offer family's four lines (REMOTE §15.1; bl-9ea6): each round-trips
//! modulo the seat's context, the grade is default-operator made typable, and
//! each refusal names what was missing.

use super::rt;
use crate::boundary::line::{Context, parse};
use crate::boundary::{Action, Gesture};
use crate::registry::Grade;
use crate::registry::offer::Verb;

fn act(verb: Verb) -> Gesture {
    Gesture::Act(Action::Offer(verb))
}

#[test]
fn all_four_round_trip_and_refuse_by_name() {
    let (ws, name) = ("ws".to_owned(), "phone-1".to_owned());
    for grade in [Grade::Operator, Grade::Foot] {
        rt(act(Verb::Mint {
            workspace: ws.clone(),
            name: name.clone(),
            grade,
        }));
    }
    rt(act(Verb::List {
        workspace: ws.clone(),
    }));
    rt(act(Verb::Approve {
        workspace: ws.clone(),
        name: name.clone(),
    }));
    rt(act(Verb::Decline {
        workspace: ws,
        name,
    }));

    let ctx = super::ctx();
    // The brands read as their grades, and nothing rounds.
    assert_eq!(
        parse("/offer phone-1 thrall", &ctx),
        parse("/offer phone-1 foot", &ctx)
    );
    assert_eq!(
        parse("/offer phone-1 lernie", &ctx),
        parse("/offer phone-1", &ctx)
    );
    let unknown = parse("/offer phone-1 fott", &ctx).expect_err("refused");
    assert!(unknown.contains("unknown grade \"fott\""), "{unknown}");
    for line in ["/offer", "/approve", "/decline"] {
        let unnamed = parse(line, &ctx).expect_err("the name is required");
        assert!(unnamed.contains("common name"), "{unnamed}");
    }
    for line in [
        "/offers now",
        "/approve phone-1 too",
        "/decline phone-1 too",
    ] {
        assert!(parse(line, &ctx).is_err(), "{line} takes no more words");
    }
    for line in ["/offer phone-1", "/offers", "/approve phone-1"] {
        let unfocused = parse(line, &Context::default()).expect_err("a workspace is named");
        assert!(unfocused.contains("no workspace in context"), "{unfocused}");
    }
}
