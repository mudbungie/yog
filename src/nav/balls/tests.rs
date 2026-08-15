//! Tables for the ball-row selections ([`super`], bl-b4b5): the roster's
//! partition and the row a pointer-targeted menu acts on, both out of the
//! listing `Query::WorkspaceBalls` answered.

use super::*;
use crate::projects::join::JoinState;

fn ball(id: &str, state: JoinState) -> BoundBall {
    BoundBall {
        id: id.to_owned(),
        badge: crate::projects::join::badge(state, Some("cobalt")),
        project: "yog".to_owned(),
        owner: "cobalt".to_owned(),
        state,
        spend: crate::spend::Figure {
            tokens: crate::budgets::BudgetSpend::default(),
            cost: None,
            attribution: crate::spend::Attribution::Workspace,
        },
    }
}

/// The section's rows partition the §3.5 states: a Bound ball is rendered in
/// full by ▶ Continue, so the roster's own list must not emit it a second time
/// as a bare id. Everything ▶ Continue does not reach stays.
#[test]
fn the_roster_drops_exactly_what_continue_already_renders() {
    let rows = [
        ball("bl-bound", JoinState::Bound),
        ball("bl-done", JoinState::Delivered),
        ball("bl-elsewhere", JoinState::ClaimedElsewhere),
    ];
    let ids: Vec<String> = roster(&rows).into_iter().map(|b| b.id).collect();
    assert_eq!(ids, ["bl-done".to_owned(), "bl-elsewhere".to_owned()]);
    assert!(roster(&[]).is_empty(), "an unanswered listing has no rows");
}

/// The ▶ Continue row's §11 menu acts on the ball it names, never on whatever
/// the focus happens to be — so the selection is by id and a miss is `None`
/// rather than an arbitrary row.
#[test]
fn one_ball_is_picked_by_its_own_id_and_a_miss_is_absent() {
    let rows = [
        ball("bl-1", JoinState::Bound),
        ball("bl-2", JoinState::Bound),
    ];
    let picked = bound(&rows, "bl-2").expect("the listing carries bl-2");
    assert_eq!(picked.id, "bl-2");
    assert_eq!(picked.owner, "cobalt", "the claimant its verbs stamp --as");
    assert!(bound(&rows, "bl-9").is_none(), "a ball nothing bound");
}
