//! STORIES **S6-T2** ack-converges: two `AppModel`s over one `ui.json` —
//! acknowledging in A clears the signal in B after the adopt, while B's own
//! focus stays B's (STORIES S6.3, DESIGN §6, §13.1, I0).
//!
//! "The mark is lernie's; the acknowledgement is yog's": the acknowledgement is
//! shared state on disk and converges, and viewport ephemera (focus, scroll,
//! unsent drafts) never leaves the instance that owns it. Both halves are one
//! test because the interesting claim is that they hold *at the same time*.

#![allow(clippy::unwrap_used)]

use crate::support::{AgentFixture, FakeBl, build_agents};
use std::sync::Arc;
use tempfile::tempdir;
use yog::ui_state::SystemClock;
use yog::{AppModel, Roots};

/// STORIES **S6-T2** ack-converges.
#[test]
fn s6_t2_the_acknowledgement_converges_and_the_viewport_does_not() {
    let root = tempdir().unwrap();
    let roots = Roots {
        yog_data: root.path().join("yog"),
        lernie_data: root.path().join("lernie"),
        yog_state: root.path().join("state"),
        balls_clones: root.path().join("clones"),
        home: root.path().join("home"),
        world: yog::world::compose(&yog::xdg::Env::from_env()),
    };
    let ws = roots.yog_data.join("workspaces").join("cobalt");
    std::fs::create_dir_all(&ws).unwrap();
    // Two agents that both stir: one each instance will look at.
    build_agents(
        &ws,
        &[
            AgentFixture::new("n-001", "one\n")
                .settled(true)
                .mark("notify"),
            AgentFixture::new("n-002", "two\n")
                .settled(true)
                .mark("notify"),
        ],
    );

    let boot = || {
        AppModel::boot(
            roots.clone(),
            None,
            Arc::new(SystemClock),
            Box::new(FakeBl::default()),
            None,
        )
    };
    let (mut a, mut a_worker) = boot();
    let (mut b, mut b_worker) = boot();
    // One file, two instances (I0) — the convergence is the disk, not a channel.
    assert_eq!(a.ui_json_path(), b.ui_json_path());

    a.focus_workspace(&ws);
    b.focus_workspace(&ws);
    assert_eq!(a.strip_total(), 2, "both stir, in both instances");
    assert_eq!(b.strip_total(), 2);

    // B is looking at n-002 — its viewport, and nobody else's.
    b.focus_agent(&ws, "n-002");
    b.refresh();

    // A lands on n-001. Landing IS acknowledging (§6.3, bl-aa1f: the ack is a
    // state re-stamped every frame, not a gesture).
    a.focus_agent(&ws, "n-001");
    a.refresh();
    for _ in 0..200 {
        a_worker.step();
        if a.refresh() && a.strip_total() == 1 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(a.strip_total(), 1, "A's own landing quieted n-001");

    // B adopts the same `ui.json` and stops flagging n-001 too — nothing was
    // sent between them.
    for _ in 0..200 {
        b_worker.step();
        b.refresh();
        if b.strip_total() == 1 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(
        b.strip_total(),
        1,
        "the acknowledgement converged through the file"
    );

    // …and B is still where B was. The adopt takes the seen marks wholesale; it
    // never takes the other instance's viewport with them (§13.1).
    assert_eq!(
        b.focused_agent().map(|a| a.agent_id.clone()),
        Some("n-002".to_owned()),
        "B's focus is B's"
    );
    assert_eq!(b.focused_workspace(), Some(ws.as_path()));
}
