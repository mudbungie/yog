//! STORIES **S7-T5** descent-only-with-children: a single-agent conversation
//! grows **no** descent to unfold; adding a child gives its list row one row per
//! member at its own depth, and selecting a member retargets the inspector
//! (STORIES S7.1, DESIGN §2.3, §11).
//!
//! "Only a conversation **with children** has a descent" — membership is a query
//! over the id set, so the single-agent case is the general path with one member
//! rather than a branch that hides a widget.
//!
//! **Re-pointed by bl-8905**, which retired the altitude-1 compact descent tree:
//! after bl-fa82 the conversation list renders that same membership itself, so
//! the tree was a second rendering of one fact on the same screen. The story is
//! unchanged — it is about the descent being derived rather than registered —
//! and it now reads the surface that survived — the boundary's forest answer
//! folded by an expanded set (`support::conversations`, REMOTE §9.7) — instead
//! of the one that was deleted.

#![allow(clippy::unwrap_used)]

use crate::support::{AgentFixture, FakeBl, add_agents, build_agents};
use std::sync::Arc;
use tempfile::tempdir;
use yog::nav::convs;
use yog::ui_state::SystemClock;
use yog::{AppModel, Roots};

/// The rows the conversation list paints for `root`'s subtree with `open`
/// unfolded — the surface that replaced the retired altitude-1 descent tree
/// (bl-8905). Read off the derivation the frame reads, so the story names no
/// title of its own.
fn subtree_rows(m: &AppModel, root: &str, open: &[&str]) -> Vec<convs::ConvRow> {
    let open: std::collections::HashSet<String> = open.iter().map(|s| (*s).to_owned()).collect();
    crate::support::conversations(m, 9000, &open)
        .into_iter()
        .filter(|r| r.root_id.starts_with(root))
        .collect()
}

/// STORIES **S7-T5** descent-only-with-children.
#[test]
fn s7_t5_a_lone_root_grows_no_descent_and_selecting_a_member_retargets() {
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
    // One lone root, and one root that will grow children (§2.3: a child's id
    // is its parent's plus two tokens).
    build_agents(
        &ws,
        &[
            AgentFixture::new("a-001", "alone\n").at(5000).settled(true),
            AgentFixture::new("b-001", "parent\n")
                .at(4000)
                .settled(true),
        ],
    );

    let boot = |roots: Roots| {
        AppModel::boot(
            roots,
            None,
            Arc::new(SystemClock),
            Box::new(FakeBl::default()),
            None,
        )
        .0
    };
    let mut m = boot(roots.clone());
    m.focus_workspace(&yog::naming::leaf(&ws));

    // --- Before: two lone roots. Neither grows a tree.
    let rows = crate::support::conversation_rows(&m, 9000);
    assert_eq!(rows.len(), 2);
    for row in &rows {
        assert_eq!(row.members, 1, "{} is alone", row.root_id);
        assert!(!row.has_children(), "a lone root renders no descent tree");
    }
    m.focus_agent(&ws, "a-001");
    // Unfolding a lone root reveals nothing: there is no descent to open, which
    // is the negative half of the story and the reason the field is absent.
    let all_open: std::collections::HashSet<String> = ["a-001".to_owned(), "b-001".to_owned()]
        .into_iter()
        .collect();
    assert_eq!(
        crate::support::conversations(&m, 9000, &all_open).len(),
        2,
        "two lone roots stay two rows however wide the fold is opened"
    );

    // --- Add a child and a grandchild to b-001. The tree is a query over the
    // id set: nothing was registered, the ids simply now name a descent.
    add_agents(
        &ws,
        &[
            AgentFixture::new("b-001-c-002", "child\n")
                .at(3000)
                .settled(true),
            AgentFixture::new("b-001-c-002-d-003", "grandchild\n")
                .at(2000)
                .settled(false),
        ],
    );

    let mut m = boot(roots);
    m.focus_workspace(&yog::naming::leaf(&ws));
    let rows = crate::support::conversation_rows(&m, 9000);
    // Still TWO conversations — the descendants are members of b-001's, never
    // rows of their own.
    assert_eq!(rows.len(), 2, "descendants are members, not conversations");
    let parent = rows.iter().find(|r| r.root_id == "b-001").unwrap();
    let lone = rows.iter().find(|r| r.root_id == "a-001").unwrap();
    assert_eq!(parent.members, 3, "root + child + grandchild");
    assert!(parent.has_children(), "and so it grows a tree");
    assert_eq!(lone.members, 1);
    assert!(!lone.has_children(), "its neighbour still does not");

    // One row per member, in §2.3 descent order, each at its nesting depth —
    // read off the list's own derivation with the descent chain unfolded.
    m.focus_agent(&ws, "b-001");
    let opened = ["b-001", "b-001-c-002"];
    let members = subtree_rows(&m, "b-001", &opened);
    assert_eq!(members.len(), 3, "one row per member");
    assert_eq!(
        members.iter().map(|r| r.depth).collect::<Vec<_>>(),
        [0, 1, 2],
        "children sit directly under their parent"
    );

    // --- Selecting a member retargets the inspector to that member, while the
    // conversation stays whole: the tree is the subtree of the selection's
    // ROOT, so descending does not shrink what is on screen.
    m.focus_agent(&ws, "b-001-c-002");
    assert_eq!(
        m.focused_agent().map(|a| a.agent_id.clone()),
        Some("b-001-c-002".to_owned()),
        "the inspector follows the selection"
    );
    assert_eq!(
        subtree_rows(&m, "b-001", &opened).len(),
        3,
        "selecting a child keeps the whole conversation on screen"
    );
    // The root of any member is the conversation's root — the fact the tree and
    // the retarget both read.
    let tree = m.tree(&ws).unwrap();
    assert_eq!(
        convs::root_of(&tree.agents, "b-001-c-002-d-003").as_deref(),
        Some("b-001"),
        "every member resolves to the one root"
    );
    assert_eq!(convs::members(&tree.agents, "b-001").len(), 3);
    assert_eq!(
        convs::members(&tree.agents, "a-001").len(),
        1,
        "and a lone root's subtree is itself"
    );
}
