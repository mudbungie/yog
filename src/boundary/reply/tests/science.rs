//! The §3.9 projection at the chokepoint (bl-40ab): the query is answered by
//! the same `science::project` join the §11 seat will read, so there is one
//! derivation and two serializations. The join's own facts are tested where the
//! join lives (`science::tests`); what belongs here is that `answer` routes to
//! it and that its reply spells as this surface's own shape.

use std::path::PathBuf;

use super::super::encode;
use super::workdiff::deps;
use crate::boundary::answer::answer;
use crate::boundary::reply::Reply;
use crate::boundary::{Query, tests::snapshot};
use crate::ui_state::UiState;

/// A workspace that claims nothing projects nothing — and the empty answer is
/// still a `science` reply, never a refusal: an attempt set of zero is a fact.
#[test]
fn the_query_answers_off_the_projections_own_join() {
    let ws = PathBuf::from("/names/alba");
    let snap = snapshot(&ws, "alba", vec![], vec![]);
    let ui = UiState::open(PathBuf::from("/nonexistent/ui.json"));
    let Ok(Reply::Science(rows)) = answer(
        &Query::Science {
            workspace: crate::naming::leaf(&ws),
        },
        &deps(snap),
        &ui,
        200,
    ) else {
        panic!("the science query answers a science projection");
    };
    assert!(rows.is_empty(), "this workspace holds no attempt");
    let body = encode(&Reply::Science(rows));
    assert_eq!(body["kind"], "science");
    assert_eq!(body["ok"], true);
    assert_eq!(body["rows"].as_array().map(Vec::len), Some(0));
}
