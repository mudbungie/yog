//! STORIES **S6-T4** rollups: the workspace rollup is the count of its
//! attention-bearing agents (§6's "max over its agents" is that count's
//! `> 0`), the strip total sums those counts across every workspace, and each
//! acknowledgement takes exactly one off it — while a quiet conversation has
//! none to take (STORIES S6.1, DESIGN §6).
//!
//! **The jump half went with the window** (bl-7942): walking the queue is a
//! seat's cursor. What is left is the fact the cursor walked over, which is the
//! half that was ever the server's.

#![allow(clippy::unwrap_used)]

use crate::support::{AgentFixture, FakeBl, build_agents};
use std::sync::Arc;
use tempfile::tempdir;
use yog::ui_state::SystemClock;
use yog::{AppModel, Roots};

/// Stirs: an unacknowledged rest (§6 rule 2).
fn stirring(id: &str) -> AgentFixture {
    AgentFixture::new(id, "work\n").settled(true)
}

/// Quiet: the rest is abandoned, so rule 2 is suppressed.
fn quiet(id: &str) -> AgentFixture {
    AgentFixture::new(id, "done\n")
        .settled(true)
        .mark("abandoned")
}

/// STORIES **S6-T4** rollups.
#[test]
fn s6_t4_rollups_sum_and_each_acknowledgement_takes_one_off() {
    let root = tempdir().unwrap();
    let roots = Roots {
        yog_data: root.path().join("yog"),
        litany_data: root.path().join("litany"),
        yog_state: root.path().join("state"),
        balls_clones: root.path().join("clones"),
        home: root.path().join("home"),
        world: yog::world::compose(&yog::xdg::Env::from_env()),
    };
    let names_root = roots.yog_data.join("workspaces");
    // alpha: two stir and one does not; bravo: one stirs; charlie: none.
    for (name, agents) in [
        (
            "alpha",
            vec![stirring("a-001"), quiet("a-002"), stirring("a-003")],
        ),
        ("bravo", vec![stirring("b-001"), quiet("b-002")]),
        ("charlie", vec![quiet("c-001")]),
    ] {
        let ws = names_root.join(name);
        std::fs::create_dir_all(&ws).unwrap();
        build_agents(&ws, &agents);
    }

    let (m, _worker) = AppModel::boot(
        roots,
        Arc::new(SystemClock),
        Box::new(FakeBl::default()),
        None,
    );
    let alpha = names_root.join("alpha");
    let bravo = names_root.join("bravo");
    let charlie = names_root.join("charlie");

    // The rollup counts the workspace's OWN attention-bearing agents.
    assert_eq!(m.workspace_stats(&alpha).0, 2);
    assert_eq!(m.workspace_stats(&bravo).0, 1);
    assert_eq!(m.workspace_stats(&charlie).0, 0);
    // §6's boolean "this workspace has attention" is that count's `> 0` — one
    // derivation, not a second predicate that could disagree with the number
    // beside it.
    assert!(m.workspace_stats(&alpha).0 > 0);
    assert!(m.workspace_stats(&charlie).0 == 0);
    // The strip total is their sum across every workspace.
    assert_eq!(crate::support::strip_total(&m), 3, "2 + 1 + 0");

    // --- The queue drains as each signal is answered. The walk itself is a
    // seat's cursor (bl-7942 took it out of this crate with the window); what
    // stays here is the fact the cursor walks over — one acknowledgement, one
    // signal off the strip, and only the flagged agents have one to take.
    let flagged = [("alpha", "a-001"), ("bravo", "b-001"), ("alpha", "a-003")];
    let mut remaining = Vec::new();
    for (workspace, agent) in flagged {
        crate::support::act(
            &m,
            &yog::boundary::Action::MarkSeen {
                workspace: workspace.to_owned(),
                agent: agent.to_owned(),
            },
        )
        .expect("the ack lands");
        remaining.push(crate::support::strip_total(&m));
    }
    assert_eq!(remaining, [2, 1, 0], "each acknowledgement takes one off");

    // A quiet agent has nothing to take: acknowledging one leaves the strip
    // where it was, which is what "attention is evidence, not rows" means.
    crate::support::act(
        &m,
        &yog::boundary::Action::MarkSeen {
            workspace: "alpha".to_owned(),
            agent: "a-002".to_owned(),
        },
    )
    .expect("the ack lands");
    assert_eq!(crate::support::strip_total(&m), 0);
}
