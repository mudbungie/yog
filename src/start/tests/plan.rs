//! The pure planner (§8.1): the amended-order step sequence per rung — bare,
//! path, and ball (ready/bound/new) — and start eligibility. Every plan is a
//! function of the target workspace + payload; the name is the workspace's leaf.

use crate::binding::{work_worktree_path, workspace_path};
use crate::projects::join::JoinState;
use crate::start::{
    BallSpec, Payload, StartInputs, Step, is_resume_eligible, is_start_eligible, plan,
};
use std::path::{Path, PathBuf};

const NAME: &str = "cobalt-gecko";
const YOG: &str = "/yog";
const BALLS: &str = "/balls";
const HOME: &str = "/home/op";
const PROJ: &str = "/proj";

fn inputs(payload: Payload) -> StartInputs {
    StartInputs {
        workspace: ws(),
        payload,
        home: PathBuf::from(HOME),
        yog_data_root: PathBuf::from(YOG),
        balls_state_root: PathBuf::from(BALLS),
        conversation_names: Vec::new(),
    }
}
fn ws() -> PathBuf {
    workspace_path(Path::new(YOG), NAME)
}
fn ball_payload(join: JoinState) -> Payload {
    Payload::Ball {
        project: PathBuf::from(PROJ),
        ball: BallSpec::Existing {
            id: "bl-1".to_owned(),
            title: "T".to_owned(),
            body: "B".to_owned(),
            join,
        },
    }
}

#[test]
fn bare_plans_seed_new_prompt() {
    // The bootstrap-shaped plan: substrate then the deferred prompt, cwd `~`, an
    // empty prefill (the operator types), no `bl` mutation.
    assert_eq!(
        plan(&inputs(Payload::Bare)),
        vec![
            Step::EnsureSeeded,
            Step::EnsureWorkspace { workspace: ws() },
            Step::Prompt {
                name: NAME.to_owned(),
                workspace: ws(),
                cwd: PathBuf::from(HOME),
                goal: String::new(),
            },
        ]
    );
}

#[test]
fn path_rung_targets_the_dir_verbatim() {
    let dir = PathBuf::from("/work/here");
    let steps = plan(&inputs(Payload::Path { dir: dir.clone() }));
    assert!(matches!(steps[0], Step::EnsureSeeded));
    assert!(matches!(steps[1], Step::EnsureWorkspace { .. }));
    let Some(Step::Prompt {
        cwd, goal, name, ..
    }) = steps.last()
    else {
        panic!("path plan ends in a prompt");
    };
    assert_eq!(*cwd, dir, "driver cwd is the directory");
    assert_eq!(name, NAME);
    assert!(
        goal.starts_with("Working directory: /work/here"),
        "the target preamble leads with the dir (§3.3 headline-first)"
    );
    assert!(
        !goal.contains("You are"),
        "the identity line is stamped at fire, not here"
    );
}

#[test]
fn ball_ready_claims_after_new_and_binds_the_worktree() {
    let steps = plan(&inputs(ball_payload(JoinState::ReadyStartable)));
    // Amended §8.1 order: seed → new → claim → prompt.
    assert!(matches!(steps[0], Step::EnsureSeeded));
    assert!(matches!(steps[1], Step::EnsureWorkspace { .. }));
    assert_eq!(
        steps[2],
        Step::Claim {
            project: PathBuf::from(PROJ),
            id: "bl-1".to_owned(),
            name: NAME.to_owned(),
        },
        "claim stamped with the workspace name, after the substrate"
    );
    let wt = work_worktree_path(Path::new(BALLS), Path::new(PROJ), "bl-1", None);
    let Step::Prompt { cwd, goal, .. } = &steps[3] else {
        panic!("ends in a prompt");
    };
    assert_eq!(*cwd, wt, "driver cwd is the work worktree");
    assert!(goal.contains("Ball bl-1: T"));
    assert!(
        goal.contains(&wt.display().to_string()),
        "worktree preamble"
    );
}

#[test]
fn ball_bound_drops_the_claim() {
    // Resume: a ball already claimed by its workspace re-plans as a prompt — no
    // second claim, no mint (§8.1).
    let steps = plan(&inputs(ball_payload(JoinState::Bound)));
    assert!(!steps.iter().any(|s| matches!(s, Step::Claim { .. })));
    assert!(steps.iter().any(|s| matches!(s, Step::Prompt { .. })));
    assert_eq!(steps.len(), 3, "seed, new, prompt");
}

#[test]
fn ball_new_defers_to_a_single_create_after_the_substrate() {
    let payload = Payload::Ball {
        project: PathBuf::from(PROJ),
        ball: BallSpec::New {
            title: "Fresh".to_owned(),
            body: "do".to_owned(),
        },
    };
    assert_eq!(
        plan(&inputs(payload)),
        vec![
            Step::EnsureSeeded,
            Step::EnsureWorkspace { workspace: ws() },
            Step::Create {
                project: PathBuf::from(PROJ),
                title: "Fresh".to_owned(),
                body: "do".to_owned(),
            },
        ],
        "create is deferred to after seed+new; the id re-plans the rest"
    );
}

#[test]
fn eligibility_is_the_ready_ball() {
    assert!(is_start_eligible(JoinState::ReadyStartable));
    assert!(!is_start_eligible(JoinState::Bound));
    assert!(!is_start_eligible(JoinState::Blocked));
    assert!(!is_start_eligible(JoinState::ClaimedElsewhere));
    assert!(!is_start_eligible(JoinState::Delivered));
    assert!(!is_start_eligible(JoinState::UnassignedWorkspace));
    assert!(!is_start_eligible(JoinState::OrphanedProject));
}

#[test]
fn resume_eligibility_is_the_bound_ball() {
    // ▶ Continue reaches a bound ball stranded between claim and prompt (addendum);
    // it is the ONLY guard, since `plan` is total over every join state.
    assert!(is_resume_eligible(JoinState::Bound));
    assert!(!is_resume_eligible(JoinState::ReadyStartable));
    assert!(!is_resume_eligible(JoinState::Blocked));
    assert!(!is_resume_eligible(JoinState::ClaimedElsewhere));
    assert!(!is_resume_eligible(JoinState::Delivered));
    assert!(!is_resume_eligible(JoinState::UnassignedWorkspace));
    assert!(!is_resume_eligible(JoinState::OrphanedProject));
}

/// bl-48f8: the rung IS the §7.3 origin, and it is total over the three of them.
/// A ball rung was offered on the roster's balls section, so its whole flow —
/// the `lernie prime`/`lernie new`/`["yog-step","mkdir"]` substrate steps
/// included, which name no ball and could never be classified from their argv —
/// banners there (§11, bl-6ad8). The bare and path rungs are the composer's own
/// Enter, the empty world's bootstrap box being that same box with no workspace.
#[test]
fn the_rung_decides_which_surface_the_starts_failures_banner_on() {
    use crate::opslog::Origin;
    assert_eq!(Payload::Bare.origin(), Origin::Conversation);
    assert_eq!(
        Payload::Path {
            dir: PathBuf::from("/work")
        }
        .origin(),
        Origin::Conversation
    );
    let existing = Payload::Ball {
        project: PathBuf::from(PROJ),
        ball: BallSpec::Existing {
            id: "bl-7".to_owned(),
            title: "T".to_owned(),
            body: "B".to_owned(),
            join: JoinState::ReadyStartable,
        },
    };
    assert_eq!(existing.origin(), Origin::Balls);
    let new = Payload::Ball {
        project: PathBuf::from(PROJ),
        ball: BallSpec::New {
            title: "T".to_owned(),
            body: "B".to_owned(),
        },
    };
    assert_eq!(new.origin(), Origin::Balls);
}
