//! The §3.6 unmaking, pure half: the confirmation's gate and arming, and the
//! plan's load-bearing order. The executor's per-step records and aborts need
//! the effect world and live in [`super::exec::tests`].

use super::{Claim, Confirmation, DeleteError, Step, confirmation, plan};
use crate::nav::convs::Conversation;
use std::path::{Path, PathBuf};

pub(super) const NAME: &str = "alba-koi";

fn conv(name: &str, live: bool) -> Conversation {
    Conversation {
        name: name.to_owned(),
        live,
    }
}

pub(super) fn claim(project: &str, id: &str) -> Claim {
    Claim {
        project: project.to_owned(),
        id: id.to_owned(),
    }
}

/// A confirmation over `convs`/`claims`.
fn confirm(convs: &[Conversation], claims: Vec<Claim>) -> Confirmation {
    confirmation(NAME, convs, claims)
}

/// The workspace the plans below unmake, and the §5.1 #1 naming set their
/// claims resolve against — both the caller's, since bl-b4b5: a confirmation
/// says names and the plan is what turns them back into the directories `bl`
/// runs in.
fn at() -> PathBuf {
    PathBuf::from("/y/workspaces/alba-koi")
}

fn projects() -> Vec<PathBuf> {
    vec![PathBuf::from("/p"), PathBuf::from("/q")]
}

#[test]
fn the_confirmation_names_what_dies_what_is_released_and_what_is_live() {
    let c = confirm(
        &[conv("wire the gate", false), conv("c-2", true)],
        vec![claim("p", "bl-1"), claim("p", "bl-2")],
    );
    assert_eq!(c.conversations, ["wire the gate", "c-2"]);
    assert_eq!(c.live, ["c-2"], "only the live one is a blocker");
    assert_eq!(c.ball_ids(), ["bl-1", "bl-2"]);
}

/// **The seat builds the same object off answers** (REMOTE §9.7, bl-b4b5): the
/// §3.6 dialog folds the landed conversation forest and the landed ball listing
/// rather than the window's own snapshot, and it must produce the very
/// `Confirmation` the chokepoint gates on — one `armed`, one `refused`, one
/// `ball_ids`, whichever side asks. Only **Bound** balls are released: a
/// delivered or claimed-elsewhere row names a claim this workspace does not
/// hold.
#[test]
fn the_seat_builds_the_same_confirmation_off_the_answers() {
    let agents = [
        crate::boundary::tests::agent("c-1", crate::git_tree::AgentState::Live, 1),
        crate::boundary::tests::agent("c-2", crate::git_tree::AgentState::Quiescent, 2),
    ];
    let rows = crate::nav::convs::forest_rows(
        &agents,
        "/ws",
        &|_, _: &str, _: &str, _: &str| false,
        100,
        &|id: &str| crate::nav::convs::ConvBall {
            id: id.to_owned(),
            state: None,
            title: None,
            badge: None,
        },
        &[],
    );
    let ball = |id: &str, state| crate::nav::BoundBall {
        id: id.to_owned(),
        badge: None,
        project: "p".to_owned(),
        owner: NAME.to_owned(),
        state,
        spend: crate::spend::Figure {
            tokens: crate::budgets::BudgetSpend::default(),
            cost: None,
            attribution: crate::spend::Attribution::Workspace,
        },
    };
    let seat = super::confirmation_of_rows(
        NAME,
        &rows,
        &[
            ball("bl-1", crate::projects::join::JoinState::Bound),
            ball("bl-done", crate::projects::join::JoinState::Delivered),
        ],
    );
    assert_eq!(seat.ball_ids(), ["bl-1"], "a delivered claim is not held");
    assert_eq!(seat.live, ["c-1"], "the live conversation is the blocker");
    assert!(seat.refused() && !seat.armed(NAME));
    // One confirmation, two projections — over the *set* rather than the
    // sequence, and lawfully so: the answer is sorted by recency (§11) while
    // the agent set is in §2.3 descent order, and neither the gate nor the
    // arming reads an order.
    let engine = confirm(
        &crate::nav::convs::liveness(&agents),
        vec![claim("p", "bl-1")],
    );
    let sorted = |mut c: Confirmation| {
        c.conversations.sort();
        c
    };
    assert_eq!(sorted(seat), sorted(engine));
}

#[test]
fn a_live_conversation_refuses_and_no_typed_name_can_arm_it() {
    // §3.6 fail-closed: an `rm` under a flock-holding driver is a race with a
    // running process, and no confirmation may buy past it.
    let c = confirm(&[conv("c-1", true)], Vec::new());
    assert!(c.refused());
    assert!(!c.armed(NAME));
    assert_eq!(
        DeleteError::Live(c.live).to_string(),
        "refused — live conversations: c-1"
    );
}

#[test]
fn arming_is_the_typed_workspace_name_and_nothing_else() {
    let c = confirm(&[conv("c-1", false)], Vec::new());
    assert!(!c.refused());
    assert!(c.armed(NAME));
    assert!(
        c.armed("  alba-koi  "),
        "surrounding whitespace is forgiven"
    );
    assert!(!c.armed(""));
    assert!(!c.armed("alba"));
    assert!(!c.armed("ALBA-KOI"));
}

#[test]
fn the_plan_releases_every_claim_then_prunes_then_removes() {
    let c = confirm(
        &[conv("c-1", false)],
        vec![claim("p", "bl-1"), claim("q", "bl-2")],
    );
    assert_eq!(
        plan(&c, &at(), Path::new("/y/world"), &projects()),
        [
            Step::Release {
                project: PathBuf::from("/p"),
                id: "bl-1".to_owned(),
                name: NAME.to_owned(),
            },
            Step::Release {
                project: PathBuf::from("/q"),
                id: "bl-2".to_owned(),
                name: NAME.to_owned(),
            },
            Step::Prune {
                key: "/y/workspaces/alba-koi".to_owned(),
            },
            Step::Remove {
                workspace: PathBuf::from("/y/workspaces/alba-koi"),
                wall: PathBuf::from("/y/world/walls/alba-koi"),
            },
        ]
    );
}

#[test]
fn a_workspace_with_no_claims_plans_only_the_prune_and_the_removal() {
    // Convergence (§3.6): a re-run after the releases landed is the shorter
    // remainder — the join no longer binds them, so no special case exists.
    let steps = plan(
        &confirm(&[], Vec::new()),
        &at(),
        Path::new("/y/world"),
        &projects(),
    );
    assert_eq!(steps.len(), 2);
    assert!(matches!(steps[0], Step::Prune { .. }));
    assert!(matches!(steps[1], Step::Remove { .. }));
}
