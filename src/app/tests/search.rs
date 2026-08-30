//! The frame's §8.5 search seat: the ask that never runs on the frame, and the
//! routing that reuses the existing selection rather than inventing one.

use super::*;
use crate::boundary::{Query, reply::Reply};
use crate::cli_outbound::Cli;
use crate::search::{Address, Found};

/// The window's seat hands the query over and renders what has landed — so the
/// first ask answers empty (the searcher has not run yet) and the answer
/// arrives once it has, without the frame ever having walked the world.
#[test]
fn the_frame_hands_search_over_and_renders_the_landed_answer() {
    let h = Harness::new();
    let (_c, model) = h.model();
    let deps = model.boundary_deps(&Cli::new("/no/litany"), &Cli::new("/no/bl"));
    let ask = |text: &str| {
        let Ok(Reply::Search(found)) = model.answer(
            &deps,
            &Query::Search {
                text: text.to_owned(),
            },
            200,
        ) else {
            panic!("search answers search");
        };
        found
    };
    assert_eq!(ask("ws"), Found::default(), "nothing has landed yet");
    assert!(model.searching(), "the ask is outstanding");

    // The answer lands through the cell — which is where the searcher puts it
    // and the only thing this seat can see of it. The searcher's own leg (it
    // asks the *engine* now, REMOTE §9.7) is `search::worker`'s to prove; from
    // here what matters is that the frame ran no walk and rendered what landed.
    let cell = model.search_cell();
    let (seq, text) = cell.pending().expect("the ask is outstanding");
    cell.publish(seq, crate::search::run(model.derivation(), &text, &|| true));
    assert!(!model.searching());
    let found = model.found();
    assert!(
        found.hits.iter().any(|hit| matches!(
            &hit.at,
            Address::Workspace { path } if path == &h.ws
        )),
        "the workspace's own §3.1 name matched: {found:?}"
    );
    assert_eq!(ask("ws"), found, "the seat renders exactly what landed");
}

#[test]
fn opening_a_hit_is_the_selection_a_click_on_that_thing_would_have_made() {
    let h = Harness::new();
    let (_c, mut model) = h.model();

    model.open(&Address::Workspace { path: h.ws.clone() });
    assert_eq!(model.focused_workspace(), Some(h.ws.clone()));
    assert!(model.focused_agent().is_none());

    model.open(&Address::Conversation {
        workspace: h.ws.clone(),
        agent: "c-1".to_owned(),
    });
    assert_eq!(
        model.focused_agent().map(|a| a.agent_id.clone()),
        Some("c-1".to_owned())
    );
    assert_eq!(
        model.inspector_tab(),
        crate::keymap::InspectorTab::Transcript
    );
}

/// A ball is selected through the workspace that holds it (§3.5), and a ball no
/// workspace holds moves nothing — the row still names the address every `bl`
/// verb takes.
#[test]
fn a_ball_hit_routes_through_its_workspace_and_an_unheld_one_moves_nothing() {
    let h = Harness::new();
    let (_c, mut model) = h.model();
    let project = std::path::PathBuf::from("/proj");
    let mut snap = (*model.snap).clone();
    snap.projects = vec![project.clone()];
    snap.join_rows = vec![crate::projects::join::JoinRow {
        project: crate::naming::leaf(&project),
        ball_id: "bl-held".to_owned(),
        state: crate::projects::join::JoinState::Bound,
        workspace: Some(crate::naming::leaf(&h.ws)),
        claimant: Some("ws".to_owned()),
        title: None,
    }];
    model.snap = std::sync::Arc::new(snap);

    model.open(&Address::Ball {
        project: project.clone(),
        id: "bl-held".to_owned(),
    });
    assert_eq!(model.focused_workspace(), Some(h.ws.clone()));

    model.open(&Address::Ball {
        project,
        id: "bl-nobody-holds".to_owned(),
    });
    assert_eq!(
        model.focused_workspace(),
        Some(h.ws.clone()),
        "an unheld ball has no selection to route to, so nothing moves"
    );
}
