//! STORIES **S6-T2** ack-converges: two `AppModel`s over one `ui.json` —
//! acknowledging at A's boundary clears the signal at B's after the adopt, and
//! clears only the conversation it named (STORIES S6.3, DESIGN §6, §13.1, I0).
//!
//! The viewport half of this rung went with the window (bl-7942): which
//! conversation each instance was *looking at* was per-instance RAM, and a
//! server holds none (REMOTE §7).
//!
//! "The mark is litany's; the acknowledgement is yog's": the acknowledgement is
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
fn s6_t2_the_acknowledgement_converges_through_the_file() {
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
            Arc::new(SystemClock),
            Box::new(FakeBl::default()),
            None,
        )
    };
    let (mut a, mut a_worker) = boot();
    let (mut b, mut b_worker) = boot();
    // One file, two instances (I0) — the convergence is the disk, not a channel.
    assert_eq!(a.ui_json_path(), b.ui_json_path());

    let name = yog::naming::leaf(&ws);
    assert_eq!(
        crate::support::strip_total(&a),
        2,
        "both stir, in both instances"
    );
    assert_eq!(crate::support::strip_total(&b), 2);

    // A acknowledges n-001 — one act at A's boundary, and the only thing it
    // writes is `ui.json` (§6.3, bl-aa1f).
    crate::support::act(
        &a,
        &yog::boundary::Action::MarkSeen {
            workspace: name.clone(),
            agent: "n-001".to_owned(),
        },
    )
    .expect("the ack lands");
    for _ in 0..200 {
        a_worker.step();
        if a.take() && crate::support::strip_total(&a) == 1 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(
        crate::support::strip_total(&a),
        1,
        "A's own acknowledgement quieted n-001"
    );

    // B adopts the same `ui.json` and stops flagging n-001 too — nothing was
    // sent between them.
    for _ in 0..200 {
        b_worker.step();
        b.take();
        if crate::support::strip_total(&b) == 1 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(
        crate::support::strip_total(&b),
        1,
        "the acknowledgement converged through the file"
    );
    // And n-002 is untouched: acknowledging one conversation acknowledges one.
    assert_eq!(
        crate::support::conversation_rows(&b, &name, 1000)
            .iter()
            .filter(|r| r.attention > 0)
            .map(|r| r.root_id.clone())
            .collect::<Vec<_>>(),
        vec!["n-002".to_owned()],
        "the other signal stands"
    );
}
