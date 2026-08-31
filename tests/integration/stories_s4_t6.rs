//! STORIES **S4-T6** board-order: each conversation row carries its **subtree's**
//! aggregated state (§10 uncertainty included) and the list is ordered by the
//! §11 sort — over one fixture holding an attention-flagged idle root, a
//! wounded root, and settled roots of different ages.
//!
//! **The sort this row was written for is gone.** STORIES S4.4 ranked
//! attention over running over recency; bl-cad5 amended §11 to **recency
//! alone**, and `src/nav/convs/row.rs`'s `build` says why:
//!
//! > Attention and liveness are badges here, not ranks: pinning a
//! > flagged-but-stale row above one that moved a second ago is exactly what
//! > read as broken.
//!
//! So the assertion below is the amended rule *and its regression guard*: the
//! attention-flagged row must NOT be hoisted. The old ranking survives, but on
//! the other roster — the §6 jump order (`attention::roster`), which S6-T4
//! covers. The real-wire beat that still asserts the old head is stale, not a
//! regression (bl-2d45).

#![allow(clippy::unwrap_used)]

use crate::support::{AgentFixture, FakeBl, build_agents};
use std::sync::Arc;
use tempfile::tempdir;
use yog::git_tree::AgentState;
use yog::ui_state::SystemClock;
use yog::{AppModel, Roots};

/// STORIES **S4-T6** board-order.
#[test]
fn s4_t6_rows_aggregate_their_subtree_and_order_by_recency_alone() {
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
    build_agents(
        &ws,
        &[
            // Idle but FLAGGED (an unacknowledged notify, §6 rule 1) and stale.
            AgentFixture::new("a-001", "flagged\n")
                .at(5000)
                .settled(true)
                .mark("notify"),
            // The freshest row, flagged by nothing: `abandoned` is the §6 rule-2
            // suppressor (the will-not-retry assertion), and without it a clean
            // rest stirs too (bl-2194), so the comparison below would be
            // flagged-vs-flagged and prove nothing.
            AgentFixture::new("b-001", "newest\n")
                .at(9000)
                .settled(true)
                .mark("abandoned"),
            // Came to rest wounded — a failed latest step (§4.4 failed framing).
            AgentFixture::new("c-001", "wounded\n")
                .at(3000)
                .settled(false),
            AgentFixture::new("d-001", "oldest\n")
                .at(1000)
                .settled(true),
            // a-001's child (§2.3: an id is its parent's plus two tokens), and
            // it is Stopped while its root is Quiescent.
            AgentFixture::new("a-001-a-002", "child\n")
                .at(2000)
                .settled(false),
        ],
    );

    let (m, _worker) = AppModel::boot(
        roots,
        Arc::new(SystemClock),
        Box::new(FakeBl::default()),
        None,
    );
    let name = yog::naming::leaf(&ws);
    let rows = crate::support::conversation_rows(&m, &name, 10_000);

    // Four conversations — the child is a member of a-001's, never a row of its
    // own (§2.3 descent).
    let ids: Vec<&str> = rows.iter().map(|r| r.root_id.as_str()).collect();
    assert_eq!(
        ids,
        ["b-001", "a-001", "c-001", "d-001"],
        "ordered by last action, newest first"
    );

    // The regression guard for bl-cad5: a-001 fires attention and still sits
    // BELOW b-001, which merely moved more recently. Attention is a badge.
    let flagged = &rows[1];
    assert_eq!(flagged.root_id, "a-001");
    assert!(
        flagged.attention >= 1,
        "the fixture's notify must actually fire, else the guard is vacuous"
    );
    assert!(
        rows[0].attention == 0,
        "and the row above it must NOT be flagged, else the order proves nothing"
    );

    // Subtree aggregation: a-001 has a Stopped child but is itself at rest
    // cleanly, and the settled row reads the ROOT's state — an aggregate of
    // "worst wins" would paint the conversation wounded when it is not.
    assert_eq!(flagged.state, AgentState::Quiescent);
    assert_eq!(flagged.members, 2, "root + descendant");
    assert!(
        flagged.has_children(),
        "and so its row grows a subagent field"
    );
    // A root that came to rest wounded reads Stopped; the ages differ, and only
    // the state badge tells the two kinds of rest apart.
    let wounded = rows.iter().find(|r| r.root_id == "c-001").unwrap();
    assert_eq!(wounded.state, AgentState::Stopped);
    assert_eq!(wounded.members, 1);
    assert!(!wounded.has_children(), "a lone root grows no tree");

    // §10: every probe here is procfs-authoritative, so no row is uncertain and
    // no "?" suffix is rendered.
    assert!(
        rows.iter().all(|r| !r.uncertain),
        "a definite reading carries no §10 uncertainty"
    );
    // The ages are the injected clock's, derived once at snapshot time.
    assert_eq!(rows[0].age_secs, 1000, "10000 - 9000");
    assert_eq!(rows[3].age_secs, 9000, "10000 - 1000");
}
