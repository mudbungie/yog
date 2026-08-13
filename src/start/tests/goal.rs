//! Goal composition + the pre-mint preview (§3.3): the per-rung prefills and
//! driver cwds, the ball header and its inverse, [`compose_prepared`] (whose
//! workspace name is a query over the target path, §3.1), and [`preview`]. The
//! name prediction's own tables are [`super::identity`].

use crate::binding::{work_worktree_path, workspace_path};
use crate::projects::join::JoinState;
use crate::start::goal::{compose_prepared, driver_cwd, prefill};
use crate::start::identity::mint_conversation;
use crate::start::{BallSpec, Payload, StartInputs, parse_ball_stamp, preview};
use std::path::{Path, PathBuf};

const YOG: &str = "/yog";
const BALLS: &str = "/balls";
const HOME: &str = "/home/op";
const PROJ: &str = "/proj";

fn existing_ball() -> Payload {
    Payload::Ball {
        project: PathBuf::from(PROJ),
        ball: BallSpec::Existing {
            id: "bl-1".to_owned(),
            title: "T".to_owned(),
            body: "B".to_owned(),
            join: JoinState::ReadyStartable,
        },
    }
}
fn new_ball() -> Payload {
    Payload::Ball {
        project: PathBuf::from(PROJ),
        ball: BallSpec::New {
            title: "Fresh".to_owned(),
            body: "do".to_owned(),
        },
    }
}

#[test]
fn prefill_is_empty_for_bare() {
    assert_eq!(prefill(&Payload::Bare, None), "");
}

/// The path rung leads with its headline (§3.3): the directory is on line one,
/// verbatim, because the derived conversation display name is the goal's first
/// payload line. The prose that once buried it on line two follows.
#[test]
fn prefill_names_the_path_verbatim_on_line_one() {
    let g = prefill(
        &Payload::Path {
            dir: PathBuf::from("/work/here"),
        },
        None,
    );
    assert_eq!(
        g,
        "Working directory: /work/here\nDo all work there, by absolute path. Do not rely on the current directory.",
    );
    assert_eq!(g.lines().next(), Some("Working directory: /work/here"));
    assert!(!g.contains("workspace"));
}

#[test]
fn prefill_is_the_ball_worktree_preamble() {
    let wt = work_worktree_path(Path::new(BALLS), Path::new(PROJ), "bl-1", None);
    let g = prefill(&existing_ball(), Some(&wt));
    assert_eq!(
        g,
        format!(
            "Ball bl-1: T\n\nB\n\nThe project repository checkout for this work is the git worktree at:\n{}   (branch work/bl-1 of {PROJ})\nDo all repository work there, by absolute path. Do not rely on the current directory.",
            wt.display(),
        )
    );
}

#[test]
fn prefill_for_a_new_ball_is_title_and_body_only() {
    // Pre-create the id/worktree are unknown; the worktree preamble joins on the
    // re-plan once `bl create` mints the id (§8.1).
    assert_eq!(prefill(&new_ball(), None), "Ball (new): Fresh\n\ndo");
}

#[test]
fn parse_ball_stamp_is_the_inverse_of_the_composed_header() {
    // A modern goal.md is the prefill verbatim (bl-6920: nothing prepended); a
    // pre-0.0.4 root carries the legacy identity stamp above the header. The
    // parse reads the id back off the `Ball <id>:` line regardless.
    let wt = work_worktree_path(Path::new(BALLS), Path::new(PROJ), "bl-1", None);
    let modern = prefill(&existing_ball(), Some(&wt));
    assert_eq!(parse_ball_stamp(&modern).as_deref(), Some("bl-1"));
    let legacy = format!("You are cobalt-gecko.\n\n{modern}");
    assert_eq!(parse_ball_stamp(&legacy).as_deref(), Some("bl-1"));
}

#[test]
fn parse_ball_stamp_rejects_non_ball_goals_and_malformed_headers() {
    // A bare/path conversation has no header.
    assert_eq!(parse_ball_stamp("You are x.\n\nfree prose"), None);
    // The word "Ball" opening a prose line is not a header: an empty id and a
    // multi-word id are both rejected (a real id is one token).
    assert_eq!(parse_ball_stamp("Ball : no id"), None);
    assert_eq!(parse_ball_stamp("Ball two words: nope"), None);
    // A header buried after the preamble still parses (line-wise scan) — and the
    // scan is preamble-agnostic, so a pre-bl-df65 goal (whose `<y>` was a
    // *workspace*, still on disk until lernie's 30-day retention ages it out,
    // §3.3) reads its ball back unchanged.
    assert_eq!(
        parse_ball_stamp("You are y.\n\nBall bz-9: T\n\nbody").as_deref(),
        Some("bz-9"),
    );
}

#[test]
fn driver_cwd_is_the_per_rung_directory() {
    let home = Path::new(HOME);
    let wt = work_worktree_path(Path::new(BALLS), Path::new(PROJ), "bl-1", None);
    assert_eq!(driver_cwd(&Payload::Bare, home, None), PathBuf::from(HOME));
    assert_eq!(
        driver_cwd(
            &Payload::Path {
                dir: PathBuf::from("/d")
            },
            home,
            None,
        ),
        PathBuf::from("/d"),
    );
    assert_eq!(driver_cwd(&existing_ball(), home, Some(&wt)), wt);
    // A not-yet-created ball has no worktree, so it falls back to `~`.
    assert_eq!(driver_cwd(&new_ball(), home, None), PathBuf::from(HOME));
}

#[test]
fn compose_prepared_derives_the_fire_params() {
    let inputs = StartInputs {
        workspace: workspace_path(Path::new(YOG), "cobalt-gecko"),
        payload: Payload::Bare,
        home: PathBuf::from(HOME),
        yog_data_root: PathBuf::from(YOG),
        balls_state_root: PathBuf::from(BALLS),
        conversation_names: Vec::new(),
    };
    let p = compose_prepared(&inputs, None);
    assert_eq!(p.name, "cobalt-gecko");
    assert_eq!(p.workspace, workspace_path(Path::new(YOG), "cobalt-gecko"));
    assert_eq!(p.cwd, PathBuf::from(HOME));
    assert_eq!(p.goal, "");
    // **The workspace name is a query, not a field** (§3.1, bl-d942): it is the
    // target path's leaf, for a foreign workspace outside the names root exactly
    // as for one of yog's own — nothing carries a second copy of it.
    let foreign = StartInputs {
        workspace: PathBuf::from("/lernie/workspaces/20260101T-aa"),
        ..inputs
    };
    assert_eq!(compose_prepared(&foreign, None).name, "20260101T-aa");
}

#[test]
fn preview_pairs_the_predicted_identity_with_the_prefill() {
    let w = super::World::new();
    let mut inputs = w.inputs(
        "cobalt-gecko",
        Payload::Path {
            dir: PathBuf::from("/d"),
        },
    );
    // The workspace's one live root already wears this name; the prediction
    // scans past it, exactly as fire will.
    let taken = mint_conversation(&[], &mut super::rng()).unwrap();
    inputs.conversation_names = vec![taken.clone()];
    let c = preview(&inputs, &mut super::rng());
    assert_ne!(c.preview, format!("will be named {taken}"));
    assert!(c.preview.starts_with("will be named "));
    assert!(
        !c.preview.contains("cobalt-gecko"),
        "the workspace name is never the previewed identity (§3.3, bl-df65)"
    );
    assert!(c.prefill.contains("/d"));
}

#[test]
fn preview_predicts_a_minted_name() {
    let w = super::World::new();
    let inputs = w.inputs(crate::names::DEFAULT_NAME, Payload::Bare);
    let c = preview(&inputs, &mut super::rng());
    assert_eq!(
        c.preview,
        format!(
            "will be named {}",
            mint_conversation(&[], &mut super::rng()).unwrap()
        )
    );
    assert_eq!(c.prefill, "", "bare has no prefill");
}
