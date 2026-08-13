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
        project: PathBuf::from(project),
        id: id.to_owned(),
    }
}

/// A confirmation over `convs`/`claims` for a workspace at `/y/workspaces/<NAME>`.
fn confirm(convs: &[Conversation], claims: Vec<Claim>) -> Confirmation {
    confirmation(NAME, Path::new("/y/workspaces/alba-koi"), convs, claims)
}

#[test]
fn the_confirmation_names_what_dies_what_is_released_and_what_is_live() {
    let c = confirm(
        &[conv("wire the gate", false), conv("c-2", true)],
        vec![claim("/p", "bl-1"), claim("/p", "bl-2")],
    );
    assert_eq!(c.conversations, ["wire the gate", "c-2"]);
    assert_eq!(c.live, ["c-2"], "only the live one is a blocker");
    assert_eq!(c.ball_ids(), ["bl-1", "bl-2"]);
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
        vec![claim("/p", "bl-1"), claim("/q", "bl-2")],
    );
    assert_eq!(
        plan(&c, PathBuf::from("/y/world").as_path()),
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
        PathBuf::from("/y/world").as_path(),
    );
    assert_eq!(steps.len(), 2);
    assert!(matches!(steps[0], Step::Prune { .. }));
    assert!(matches!(steps[1], Step::Remove { .. }));
}
