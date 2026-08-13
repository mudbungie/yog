//! The board's spend corpus (STORIES S13) — cut from the sibling table at its
//! real seam: those hold a row's column and its gate, these hold the figures —
//! one row's, and the epic rollup that crosses workspaces.

use super::*;

/// A ball bound to a workspace no pass has derived yet: no drones to name and
/// a zero figure. The general path with no inputs — a claimed row does not
/// wait on its workspace's first derivation to appear on the board.
#[test]
fn a_workspace_with_no_derivation_yet_contributes_no_drones_and_no_spend() {
    let w = world(
        vec![ball("bl-new", Some("alfa"), vec![])],
        vec![join("bl-new", JoinState::Bound, Some(WS_A), Some("alfa"))],
        vec![],
    );
    let row = w.board().rows.into_iter().next().unwrap();
    assert_eq!(row.column, Column::Claimed);
    assert!(row.drones.is_empty());
    let figure = row.spend.unwrap();
    assert_eq!(figure.tokens.total_tokens(), 0);
    assert_eq!(
        figure.attribution,
        crate::spend::Attribution::Workspace,
        "nothing stamps it here, so the figure widens and says so"
    );
}

/// The epic rollup: the subtree crosses workspaces, each workspace is billed
/// once, and a leaf gets no rollup at all.
#[test]
fn an_epic_rolls_up_its_live_subtree_across_workspaces_billing_each_once() {
    let mut epic = ball("bl-epic", None, vec![]);
    epic.priority = 1;
    let mut kid_a = ball("bl-kida", Some("alfa"), vec![]);
    kid_a.parent = Some("bl-epic".to_owned());
    let mut kid_b = ball("bl-kidb", Some("bravo"), vec![]);
    kid_b.parent = Some("bl-epic".to_owned());
    let mut grandkid = ball("bl-gkid", Some("bravo"), vec![]);
    grandkid.parent = Some("bl-kidb".to_owned());

    let w = world(
        vec![epic, kid_a, kid_b, grandkid],
        vec![
            join("bl-epic", JoinState::ReadyStartable, None, None),
            join("bl-kida", JoinState::Bound, Some(WS_A), Some("alfa")),
            join("bl-kidb", JoinState::Bound, Some(WS_B), Some("bravo")),
            join("bl-gkid", JoinState::Bound, Some(WS_B), Some("bravo")),
        ],
        vec![
            (WS_A, vec![agent("conv1", Some("bl-kida"), None)]),
            (
                WS_B,
                vec![
                    agent("conv2", Some("bl-kidb"), None),
                    agent("conv3", Some("bl-gkid"), None),
                ],
            ),
        ],
    );
    let board = w.board();
    let row = |id: &str| board.rows.iter().find(|r| r.id == id).unwrap();

    assert_eq!(
        descendants(
            "bl-epic",
            &HashMap::from([
                ("bl-kida", &ball("bl-kida", None, vec![])),
                ("bl-nope", &ball("bl-nope", None, vec![])),
            ])
        ),
        Vec::<String>::new(),
        "descent follows the stored parent pointer and nothing else"
    );

    let rollup = row("bl-epic").rollup.as_ref().unwrap();
    // conv1 (1 Mtok) + conv2 + conv3 (1 Mtok each) at $1/Mtok.
    assert_eq!(rollup.cost.unwrap().usd(), "$3.00");
    assert_eq!(
        rollup.attribution,
        crate::spend::Attribution::Conversations(3)
    );
    assert!(
        row("bl-kida").rollup.is_none(),
        "a leaf's rollup is its own figure; a second copy is not a fact"
    );
    let sub = row("bl-kidb").rollup.as_ref().unwrap();
    assert_eq!(sub.cost.unwrap().usd(), "$2.00", "kidb + its grandkid");
}

/// The §3.5 workspace-granularity arm, folded: one unstamped ball makes its
/// whole workspace the slice, and a stamped sibling in that same workspace is
/// absorbed rather than added — a workspace is billed once or not at all.
#[test]
fn a_whole_workspace_slice_absorbs_the_tree_slices_inside_it() {
    let mut epic = ball("bl-epic", None, vec![]);
    epic.priority = 1;
    let mut stamped = ball("bl-stmp", Some("alfa"), vec![]);
    stamped.parent = Some("bl-epic".to_owned());
    let mut picked = ball("bl-pick", Some("alfa"), vec![]);
    picked.parent = Some("bl-epic".to_owned());

    let w = world(
        vec![epic, stamped, picked],
        vec![
            join("bl-epic", JoinState::ReadyStartable, None, None),
            join("bl-stmp", JoinState::Bound, Some(WS_A), Some("alfa")),
            join("bl-pick", JoinState::Bound, Some(WS_A), Some("alfa")),
        ],
        // Two conversations, only one of which stamps a ball.
        vec![(
            WS_A,
            vec![
                agent("conv1", Some("bl-stmp"), None),
                agent("conv2", None, None),
            ],
        )],
    );
    let board = w.board();
    let rollup = board
        .rows
        .iter()
        .find(|r| r.id == "bl-epic")
        .unwrap()
        .rollup
        .as_ref()
        .unwrap();
    assert_eq!(
        rollup.attribution,
        crate::spend::Attribution::Workspace,
        "an unstamped member widens the whole rollup, and it says so"
    );
    assert_eq!(
        rollup.cost.unwrap().usd(),
        "$2.00",
        "the workspace's two conversations, counted once — not three"
    );
}

/// An epic whose subtree is bound nowhere has no spend to roll up, which is
/// different from a rollup of zero.
#[test]
fn an_unbound_subtree_has_no_rollup_rather_than_a_zero_one() {
    let mut epic = ball("bl-epic", None, vec![]);
    epic.priority = 1;
    let mut kid = ball("bl-kid", None, vec![]);
    kid.parent = Some("bl-epic".to_owned());
    let w = world(
        vec![epic, kid],
        vec![
            join("bl-epic", JoinState::ReadyStartable, None, None),
            join("bl-kid", JoinState::ReadyStartable, None, None),
        ],
        vec![],
    );
    let board = w.board();
    assert!(
        board
            .rows
            .iter()
            .find(|r| r.id == "bl-epic")
            .unwrap()
            .rollup
            .is_none()
    );
}
