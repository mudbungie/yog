//! STORIES **S4-T5** group-partition: `by ball` is a pure stable partition of
//! the *already-sorted* rows — group order is first appearance, within-group
//! order is preserved, the unassociated group is emitted last and only when
//! non-empty, and flattening the groups yields exactly the rows that went in
//! (STORIES S4.6, DESIGN §3.5, §11).
//!
//! Driven over a real workspace rather than hand-built rows (which is what
//! `src/nav/convs/group.rs`'s own tables do): the property that matters is that
//! the toggle **re-orders rows already on screen** — no row appears,
//! disappears, or changes meaning — so both orderings are taken off one
//! `AppModel` over one disk.

#![allow(clippy::unwrap_used)]

use crate::support::{AgentFixture, FakeBl, build_agents, clone_dir};
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::tempdir;
use yog::nav::convs::group::group_by_ball;
use yog::ui_state::SystemClock;
use yog::{AppModel, Roots};

const LIVE: &str = r#"[
    {"id":"bl-1","title":"One","claimant":"cobalt"},
    {"id":"bl-2","title":"Two","claimant":"cobalt"}
]"#;

/// STORIES **S4-T5** group-partition.
#[test]
fn s4_t5_grouping_is_a_stable_partition_of_the_sorted_rows() {
    let root = tempdir().unwrap();
    let project = tempdir().unwrap();
    let roots = Roots {
        yog_data: root.path().join("yog"),
        lernie_data: root.path().join("lernie"),
        yog_state: root.path().join("state"),
        balls_clones: root.path().join("clones"),
        home: root.path().join("home"),
        world: yog::world::compose(&yog::xdg::Env::from_env()),
    };
    clone_dir(&roots.balls_clones, project.path());
    let ws = roots.yog_data.join("workspaces").join("cobalt");
    std::fs::create_dir_all(&ws).unwrap();
    // Dated so the flat order is known: newest first (§11 recency, bl-cad5).
    // bl-1 leads bl-2 in the sorted list, and each ball has a conversation far
    // down the list too — so "group order = first appearance" is distinguishable
    // from "group order = ball id".
    build_agents(
        &ws,
        &[
            AgentFixture::stamped("a-001", "bl-1", "One").at(5000),
            AgentFixture::stamped("b-001", "bl-2", "Two").at(4000),
            AgentFixture::stamped("c-001", "bl-1", "One").at(3000),
            AgentFixture::new("d-001", "bare\n").at(2000),
            AgentFixture::new("e-001", "also bare\n").at(1000),
        ],
    );

    let (mut m, _worker) = AppModel::boot(
        roots,
        None,
        Arc::new(SystemClock),
        Box::new(FakeBl {
            live: HashMap::from([(project.path().to_path_buf(), LIVE.to_owned())]),
            ..FakeBl::default()
        }),
        None,
    );
    m.focus_workspace(&ws);

    let flat = crate::support::conversation_rows(&m, 9000);
    let flat_ids: Vec<&str> = flat.iter().map(|r| r.root_id.as_str()).collect();
    assert_eq!(
        flat_ids,
        ["a-001", "b-001", "c-001", "d-001", "e-001"],
        "the default `recent` ordering is the input to the partition"
    );

    let groups = group_by_ball(crate::support::conversation_rows(&m, 9000));
    let heads: Vec<Option<&str>> = groups
        .iter()
        .map(|g| g.ball.as_ref().map(|b| b.id.as_str()))
        .collect();
    // Group order is FIRST APPEARANCE in the sorted rows, not ball-id order and
    // not group size — bl-1 leads because a-001 is the newest row.
    assert_eq!(
        heads,
        [Some("bl-1"), Some("bl-2"), None],
        "first-appearance order, unassociated last"
    );
    // Within a group the sorted order survives untouched.
    let bl1: Vec<&str> = groups[0].convs.iter().map(|r| r.root_id.as_str()).collect();
    assert_eq!(bl1, ["a-001", "c-001"], "within-group order preserved");
    let bare: Vec<&str> = groups[2].convs.iter().map(|r| r.root_id.as_str()).collect();
    assert_eq!(bare, ["d-001", "e-001"], "the tail keeps its order too");

    // Flattening returns exactly the rows that went in: the toggle re-orders,
    // it never adds, drops or rewrites a row (§11). Compared as a multiset,
    // because moving the unassociated rows to the tail IS the re-ordering.
    let mut flattened: Vec<_> = groups.iter().flat_map(|g| g.convs.clone()).collect();
    let mut original = flat.clone();
    flattened.sort_by(|a, b| a.root_id.cmp(&b.root_id));
    original.sort_by(|a, b| a.root_id.cmp(&b.root_id));
    assert_eq!(
        flattened, original,
        "grouping is a partition, not a rewrite"
    );

    // "Only when non-empty": drop the bare conversations and no `None` group is
    // emitted at all — the trailing group is the general tail, not a slot.
    let stamped: Vec<_> = flat.into_iter().filter(|r| r.ball.is_some()).collect();
    let groups = group_by_ball(stamped);
    assert!(
        groups.iter().all(|g| g.ball.is_some()),
        "no unassociated rows ⇒ no unassociated group"
    );
    assert_eq!(groups.len(), 2);
}
