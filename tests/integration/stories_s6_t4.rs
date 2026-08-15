//! STORIES **S6-T4** rollups-and-jump: the workspace rollup is the count of its
//! attention-bearing agents (§6's "max over its agents" is that count's
//! `> 0`), the strip total sums those counts across every workspace, and
//! jump-to-next walks the derived order — wrapping, and never sticking on the
//! current row (STORIES S6.1, DESIGN §6, §11).

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

/// STORIES **S6-T4** rollups-and-jump.
#[test]
fn s6_t4_rollups_sum_and_jump_to_next_wraps_without_sticking() {
    let root = tempdir().unwrap();
    let roots = Roots {
        yog_data: root.path().join("yog"),
        lernie_data: root.path().join("lernie"),
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

    let (mut m, _worker) = AppModel::boot(
        roots,
        None,
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

    // --- Jump-to-next walks the derived order across workspaces. The control
    // **acknowledges what it lands on** (§6.3: landing is acknowledging), so it
    // is a walk through the queue rather than a cycle over a fixed list — each
    // jump takes one signal off the strip.
    let mut visited = Vec::new();
    let mut remaining = Vec::new();
    for _ in 0..3 {
        m.jump_next_attention();
        visited.push((
            m.focused_workspace().unwrap().clone(),
            m.focused_agent().unwrap().agent_id.clone(),
        ));
        remaining.push(crate::support::strip_total(&m));
    }
    let mut names: Vec<&str> = visited.iter().map(|(_, a)| a.as_str()).collect();
    // Each flagged agent is visited exactly once — no repeats, nothing missed.
    assert!(
        visited.windows(2).all(|w| w[0] != w[1]),
        "no jump lands where it started"
    );
    names.sort_unstable();
    assert_eq!(
        names,
        ["a-001", "a-003", "b-001"],
        "every flagged agent, and only those"
    );
    // It never stops on a quiet agent, and never on charlie, which has none.
    assert!(
        !names.contains(&"a-002") && !names.contains(&"b-002") && !names.contains(&"c-001"),
        "jump visits attention, not rows"
    );
    // The strip drains as it walks — three jumps, three signals answered.
    assert_eq!(remaining, [2, 1, 0], "each landing takes one off the strip");

    // It wraps rather than running off the end: the walk crossed into bravo and
    // came back to alpha for the last one.
    assert_eq!(visited[1].0, bravo);
    assert_eq!(visited[2].0, alpha, "the order wrapped back to the front");

    // With the strip empty the control has nowhere to go, and says so by not
    // moving — which is not "sticking": there is no next signal to stick past.
    let before = m.focused_agent().map(|a| a.agent_id.clone());
    m.jump_next_attention();
    assert_eq!(crate::support::strip_total(&m), 0);
    assert_eq!(m.focused_agent().map(|a| a.agent_id.clone()), before);
}
