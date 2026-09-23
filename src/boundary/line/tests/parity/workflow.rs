//! The §9.4 workflow mark's two lines (bl-b680): both directions round-trip
//! modulo the seat's context, the set refuses without its lineage, and the
//! clear refuses a tail.

use super::rt;
use crate::boundary::line::{Context, parse};
use crate::boundary::{Action, Gesture};

fn gesture(config: Option<&str>) -> Gesture {
    Gesture::Act(Action::Workflow {
        workspace: "ws".to_owned(),
        agent: "c-1".to_owned(),
        config: config.map(str::to_owned),
    })
}

#[test]
fn both_directions_round_trip_and_the_set_names_its_lineage() {
    rt(gesture(Some("strict")));
    rt(gesture(None));
    let bare = parse("/workflow", &super::ctx()).expect_err("a set names its lineage");
    assert!(bare.contains("the lineage whose workflow to run"), "{bare}");
    let tail = parse("/clear-workflow strict", &super::ctx()).expect_err("a clear takes no words");
    assert!(tail.contains("takes no arguments"), "{tail}");
    let unselected =
        parse("/workflow strict", &Context::default()).expect_err("aimed at a conversation");
    assert!(
        unselected.contains("no workspace in context"),
        "{unselected}"
    );
}
