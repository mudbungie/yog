//! STORIES **S6-T3** failure-stirs-then-settles: a conversation whose latest
//! step failed stirs the strip through §6 rule 2; after the acknowledgement the
//! strip is quiet **while the state badge and the Login affordance still
//! render** (STORIES S6.4, DESIGN §6, §8.3).
//!
//! "Acknowledging clears the signal, not the fact." The three surfaces are
//! deliberately read off three different derivations — the strip off
//! `attention`, the badge off `AgentState`, the Login affordance off the step's
//! own error text — so a change that collapsed them into one flag would fail
//! here rather than quietly hide a dead conversation's remedy.

#![allow(clippy::unwrap_used)]

use crate::support::{AgentFixture, FakeBl, build_agents};
use crate::support::{response_tail, write_step};
use std::sync::Arc;
use tempfile::tempdir;
use yog::git_tree::AgentState;
use yog::login::auth;
use yog::steps_view;
use yog::ui_state::SystemClock;
use yog::{AppModel, Roots};

/// An auth-shaped settled failure (§8.3): the framing is Failed **and** the
/// error text matches the auth heuristic, which is what keeps Login one click
/// away rather than a guess.
const AUTH_FAILURE: &str =
    "{\"type\":\"error\",\"message\":\"401 Unauthorized: invalid api key\"}\n{\"type\":\"end\"}\n";

/// STORIES **S6-T3** failure-stirs-then-settles.
#[test]
fn s6_t3_the_ack_quiets_the_signal_and_keeps_every_fact() {
    let root = tempdir().unwrap();
    let roots = Roots {
        yog_data: root.path().join("yog"),
        litany_data: root.path().join("litany"),
        yog_state: root.path().join("state"),
        balls_clones: root.path().join("clones"),
        home: root.path().join("home"),
        world: yog::world::compose(&yog::xdg::Env::from_env()),
    };
    let ws = roots.yog_data.join("workspaces").join("cobalt");
    std::fs::create_dir_all(&ws).unwrap();
    build_agents(&ws, &[AgentFixture::new("d-001", "dead\n").settled(false)]);
    // Overwrite the generic failure with an auth-shaped one.
    write_step(&ws, "d-001", "000", "response.json", AUTH_FAILURE);
    assert_ne!(
        AUTH_FAILURE,
        response_tail(false),
        "the fixture must differ from the generic failure, else the auth half is luck"
    );

    let (mut m, mut worker) = AppModel::boot(
        roots,
        Arc::new(SystemClock),
        Box::new(FakeBl::default()),
        None,
    );
    let name = yog::naming::leaf(&ws);

    // --- Before: it stirs.
    assert_eq!(
        crate::support::strip_total(&m),
        1,
        "a failed latest step stirs (rule 2)"
    );

    // --- The ack: one act at the boundary (§6.3).
    crate::support::act(
        &m,
        &yog::boundary::Action::MarkSeen {
            workspace: name.clone(),
            agent: "d-001".to_owned(),
        },
    )
    .expect("the ack lands");
    for _ in 0..200 {
        worker.step();
        if m.take() && crate::support::strip_total(&m) == 0 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    // --- After: quiet in the strip …
    assert_eq!(
        crate::support::strip_total(&m),
        0,
        "the signal is acknowledged"
    );

    // … and every fact intact. The state badge still says the conversation is
    // dead — an ack is "I have seen this", never "this did not happen".
    let tree = m.tree(&ws).expect("the workspace is derived");
    let agent = tree
        .agents
        .iter()
        .find(|a| a.agent_id == "d-001")
        .expect("the conversation is still on the roster");
    assert_eq!(agent.state, AgentState::Stopped, "the badge keeps the fact");

    // And the auth-shaped death keeps its inline Login one click away: the
    // affordance is derived from the step's own bytes, which no watermark
    // touches.
    let steps = steps_view::build(&ws, "d-001", agent.state);
    let failure = auth::latest_step_auth_failed(&steps);
    assert!(
        failure.offered(),
        "an acknowledged auth failure still offers Login: {failure:?}"
    );
    // This fixture's step names no model, so no provider row is derivable —
    // the honest middle (bl-8e34): the affordance still paints and routes to
    // the Login pane, where a row is chosen by hand. An invented row would be
    // worse than none.
    assert_eq!(failure.row(), None);
    assert!(
        failure.banner().contains("credentials"),
        "the banner states the remedy: {}",
        failure.banner()
    );
    // The heuristic is a property of the text, not of the acknowledgement.
    assert!(auth::looks_auth("401 Unauthorized: invalid api key"));
    assert!(!auth::looks_auth("the model refused"));
}
