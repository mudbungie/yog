//! Goal composition + the pre-mint preview (§3.3): the per-rung prefills and
//! typed target bindings, the ball header and its inverse, [`compose_prepared`]
//! (whose workspace name is a query over the target path, §3.1), and
//! [`preview`]. The name prediction's own tables are [`super::identity`].

use crate::binding::{work_worktree_path, workspace_path};
use crate::projects::join::JoinState;
use crate::start::goal::{compose_prepared, prefill, target_binding};
use crate::start::identity::mint_conversation;
use crate::start::{BallSpec, Payload, StartInputs, parse_ball_stamp, preview};
use std::path::{Path, PathBuf};

const YOG: &str = "/yog";
const BALLS: &str = "/balls";
const HOME: &str = "/home/op";
const PROJ: &str = "/proj";

fn existing_ball() -> Payload {
    Payload::Ball {
        project: "proj".to_owned(),
        ball: BallSpec::Existing {
            id: "bl-1".to_owned(),
            title: "T".to_owned(),
            body: "B".to_owned(),
            join: JoinState::ReadyStartable,
            tags: Vec::new(),
        },
    }
}
fn new_ball() -> Payload {
    Payload::Ball {
        project: "proj".to_owned(),
        ball: BallSpec::New {
            title: "Fresh".to_owned(),
            body: "do".to_owned(),
        },
    }
}

#[test]
fn prefill_is_empty_for_bare() {
    assert_eq!(prefill(&Payload::Bare), "");
}

/// The path rung leads with its headline (§3.3): the directory is on line one,
/// verbatim, because the derived conversation display name is the goal's first
/// payload line. The prose that once buried it on line two follows.
#[test]
fn prefill_names_the_path_verbatim_on_line_one() {
    let g = prefill(&Payload::Path {
        dir: PathBuf::from("/work/here"),
    });
    assert_eq!(
        g,
        "Working directory: /work/here\nDo all work there, by absolute path. Do not rely on the current directory.",
    );
    assert_eq!(g.lines().next(), Some("Working directory: /work/here"));
    assert!(!g.contains("workspace"));
}

/// The ball rung's prefill is **payload and nothing else** (§3.3, bl-6654): the
/// `Ball <id>: <title>` header — the §3.2 conversation→ball join — and the body
/// verbatim. The worktree paragraph that used to trail it is retired; location
/// is the typed `--cwd` binding now, so no absolute path and no
/// do-the-work-there instruction may reappear in the goal text.
#[test]
fn prefill_for_a_ball_is_the_header_and_body_with_no_location_prose() {
    let wt = work_worktree_path(Path::new(BALLS), Path::new(PROJ), "bl-1", None);
    let g = prefill(&existing_ball());
    assert_eq!(g, "Ball bl-1: T\n\nB");
    assert!(!g.contains(&wt.display().to_string()));
    assert!(!g.contains(PROJ));
    assert!(!g.contains("worktree"));
    assert!(!g.contains("absolute path"));
}

#[test]
fn prefill_for_a_new_ball_is_title_and_body_only() {
    // Pre-create the id is unknown, so the header cannot carry it; the real
    // `Ball <id>:` header joins on the re-plan once `bl create` mints the id
    // (§8.1), and the binding joins with the claim that follows it.
    assert_eq!(prefill(&new_ball()), "Ball (new): Fresh\n\ndo");
}

#[test]
fn parse_ball_stamp_is_the_inverse_of_the_composed_header() {
    // A modern goal.md is the prefill verbatim (bl-6920: nothing prepended); a
    // pre-0.0.4 root carries the legacy identity stamp above the header. The
    // parse reads the id back off the `Ball <id>:` line regardless.
    let modern = prefill(&existing_ball());
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

/// The typed work target, per rung (§3.3, bl-6654 / VISION §4.10 item 2) — the
/// value the fire passes as lernie's `--cwd`.
#[test]
fn target_binding_is_the_per_rung_work_target() {
    let wt = work_worktree_path(Path::new(BALLS), Path::new(PROJ), "bl-1", None);
    // The bare rung binds nothing: lernie's own default (the agent worktree)
    // stands, which is what "no work target" means.
    assert_eq!(target_binding(&Payload::Bare, None), None);
    // The path rung binds the directory box's value.
    assert_eq!(
        target_binding(
            &Payload::Path {
                dir: PathBuf::from("/d")
            },
            None,
        ),
        Some(PathBuf::from("/d")),
    );
    // The ball rung binds the claim's cross-checked work worktree, never the
    // project root it lives beside.
    assert_eq!(target_binding(&existing_ball(), Some(&wt)), Some(wt));
    // A not-yet-created ball has no claim yet, so it binds nothing; the re-plan
    // after `bl create` binds the worktree the claim mints (§8.1).
    assert_eq!(target_binding(&new_ball(), None), None);
}

#[test]
fn compose_prepared_derives_the_fire_params() {
    let inputs = StartInputs {
        workspace: workspace_path(Path::new(YOG), "cobalt-gecko"),
        repo: Some(PathBuf::from(PROJ)),
        payload: Payload::Bare,
        home: PathBuf::from(HOME),
        yog_data_root: PathBuf::from(YOG),
        balls_state_root: PathBuf::from(BALLS),
        conversation_names: Vec::new(),
    };
    let p = compose_prepared(&inputs, None);
    // §3.1: the name IS the address — one string, one fact (bl-f5f6).
    assert_eq!(p.workspace, "cobalt-gecko");
    assert_eq!(p.binding, None, "the bare rung binds no work target");
    assert_eq!(p.goal, "");
    // **The work target is the binding and nothing else** (bl-6654): there is no
    // per-rung driver cwd field left to disagree with it.
    let wt = work_worktree_path(Path::new(BALLS), Path::new(PROJ), "bl-1", None);
    let ball = StartInputs {
        payload: existing_ball(),
        ..inputs.clone()
    };
    let bound = compose_prepared(&ball, Some(&wt));
    assert_eq!(bound.binding, Some(wt));
    // **The workspace name is a query, not a field** (§3.1, bl-d942): it is the
    // target path's leaf, for a foreign workspace outside the names root exactly
    // as for one of yog's own — nothing carries a second copy of it.
    let foreign = StartInputs {
        workspace: PathBuf::from("/lernie/workspaces/20260101T-aa"),
        ..inputs
    };
    assert_eq!(compose_prepared(&foreign, None).workspace, "20260101T-aa");
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
    let taken = mint_conversation(&[], &super::rng()).unwrap();
    inputs.conversation_names = vec![taken.clone()];
    let c = preview(&inputs, &super::rng());
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
    let c = preview(&inputs, &super::rng());
    assert_eq!(
        c.preview,
        format!(
            "will be named {}",
            mint_conversation(&[], &super::rng()).unwrap()
        )
    );
    assert_eq!(c.prefill, "", "bare has no prefill");
}
