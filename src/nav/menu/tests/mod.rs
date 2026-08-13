//! The per-seat tables: what verbs each seat fires, in roster order. The
//! universally-quantified half — the sole-carrier sweep over every seat — is
//! [`doctrine`]. Split from [`super`] per §12's 300-line budget.

use super::*;

/// The §11 doctrine swept over the whole seat space, one budget below this one.
mod doctrine;

/// Every join state the ball-row seat can be opened on — the §3.5 table's seven
/// rows, listed so a new one has to be added here to compile.
pub(super) const JOIN_STATES: [JoinState; 7] = [
    JoinState::ReadyStartable,
    JoinState::Blocked,
    JoinState::Bound,
    JoinState::ClaimedElsewhere,
    JoinState::Delivered,
    JoinState::UnassignedWorkspace,
    JoinState::OrphanedProject,
];

/// The verbs a seat fires, submenu destinations flattened in render order — the
/// shape every per-seat table below asserts on.
fn verbs(seat: Seat) -> Vec<Verb> {
    let mut out = Vec::new();
    for entry in entries(seat) {
        match entry.action {
            Action::Fire(verb) => out.push(verb),
            Action::Submenu(children) => {
                for child in children {
                    if let Action::Fire(verb) = child.action {
                        out.push(verb);
                    }
                }
            }
        }
    }
    out
}

#[test]
fn a_named_pinned_tab_carries_delete_then_unpin() {
    let seat = Seat::WorkspaceTab {
        named: true,
        pinned: true,
    };
    assert_eq!(verbs(seat.clone()), [Verb::DeleteWorkspace, Verb::Unpin]);
    assert_eq!(entries(seat)[0].label, "delete this workspace…");
}

#[test]
fn a_foreign_tab_offers_no_delete_and_an_unpinned_one_no_unpin() {
    // §3.6 scope: foreign/replay workspaces are lernie's territory.
    assert_eq!(
        verbs(Seat::WorkspaceTab {
            named: false,
            pinned: true,
        }),
        [Verb::Unpin]
    );
    assert_eq!(
        verbs(Seat::WorkspaceTab {
            named: true,
            pinned: false,
        }),
        [Verb::DeleteWorkspace]
    );
}

#[test]
fn a_foreign_unpinned_tab_has_no_menu_at_all() {
    assert!(
        entries(Seat::WorkspaceTab {
            named: false,
            pinned: false,
        })
        .is_empty()
    );
}

#[test]
fn a_live_conversation_with_children_carries_stop_both_ways_then_flush() {
    let seat = Seat::ConversationRow {
        stoppable: true,
        has_children: true,
        named: false,
    };
    assert_eq!(
        verbs(seat.clone()),
        [
            Verb::Stop { children: false },
            Verb::Stop { children: true },
            Verb::Flush,
        ]
    );
    assert_eq!(entries(seat)[1].label, "stop + children");
}

#[test]
fn a_lone_live_conversation_offers_no_children_cascade() {
    assert_eq!(
        verbs(Seat::ConversationRow {
            stoppable: true,
            has_children: false,
            named: false,
        }),
        [Verb::Stop { children: false }, Verb::Flush]
    );
}

#[test]
fn a_settled_conversation_still_carries_flush() {
    // Flush is workspace-scoped and unconditional (§8.2 Scan has no predicate),
    // so a conversation row always has a menu — even with nothing to stop.
    for has_children in [false, true] {
        assert_eq!(
            verbs(Seat::ConversationRow {
                stoppable: false,
                has_children,
                named: false,
            }),
            [Verb::Flush]
        );
    }
}

#[test]
fn a_named_workspaces_conversation_carries_delete_last_and_a_foreign_ones_none() {
    // §3.6 scope, the workspace tab's own rule: yog offers no delete inside a
    // workspace it did not place. On a named one the entry is the danger-zone
    // tail — after every recoverable verb, opening the dialog and never past it.
    let seat = Seat::ConversationRow {
        stoppable: true,
        has_children: true,
        named: true,
    };
    assert_eq!(
        verbs(seat.clone()),
        [
            Verb::Stop { children: false },
            Verb::Stop { children: true },
            Verb::Flush,
            Verb::DeleteAgent,
        ]
    );
    assert_eq!(entries(seat)[3].label, "delete this conversation…");
    assert_eq!(
        verbs(Seat::ConversationRow {
            stoppable: false,
            has_children: false,
            named: true,
        }),
        [Verb::Flush, Verb::DeleteAgent]
    );
}

#[test]
fn a_ready_ball_offers_only_assign_and_only_with_a_focused_workspace() {
    assert_eq!(
        verbs(Seat::BallRow {
            state: JoinState::ReadyStartable,
            assign_to: Some("alba-koi".to_owned()),
            move_to: vec!["zeta-pug".to_owned()],
        }),
        [Verb::Assign("alba-koi".to_owned())],
        "unclaimed ⇒ nothing to move, release or close"
    );
    assert!(
        entries(Seat::BallRow {
            state: JoinState::ReadyStartable,
            assign_to: None,
            move_to: vec!["zeta-pug".to_owned()],
        })
        .is_empty(),
        "no focused workspace ⇒ no destination ⇒ no menu at all"
    );
}

#[test]
fn a_bound_ball_carries_move_release_close_in_roster_order() {
    let seat = Seat::BallRow {
        state: JoinState::Bound,
        assign_to: Some("alba-koi".to_owned()),
        move_to: vec!["zeta-pug".to_owned(), "moss-hare".to_owned()],
    };
    assert_eq!(
        verbs(seat.clone()),
        [
            Verb::MoveTo("zeta-pug".to_owned()),
            Verb::MoveTo("moss-hare".to_owned()),
            Verb::Release,
            Verb::CloseBall,
        ],
        "bound ⇒ no assign; destinations in the caller's order"
    );
    let rows = entries(seat);
    assert_eq!(rows[0].label, "move to");
    let Action::Submenu(destinations) = &rows[0].action else {
        panic!("Move's destination is a submenu (§11)");
    };
    let labels: Vec<&str> = destinations.iter().map(|d| d.label.as_str()).collect();
    assert_eq!(
        labels,
        ["zeta-pug", "moss-hare"],
        "each row names its target"
    );
    assert_eq!(destinations[0].carrier, rows[0].carrier);
}

#[test]
fn the_only_workspace_holding_a_ball_offers_no_move() {
    assert_eq!(
        verbs(Seat::BallRow {
            state: JoinState::Bound,
            assign_to: None,
            move_to: Vec::new(),
        }),
        [Verb::Release, Verb::CloseBall],
        "nowhere to move to ⇒ no empty submenu"
    );
}

#[test]
fn an_unactionable_ball_row_has_no_menu_at_all() {
    // Blocked / claimed-elsewhere / delivered / unassigned / orphaned: every §8.2
    // predicate refuses, so the roster paints nothing rather than a dead popup.
    for state in JOIN_STATES {
        if matches!(state, JoinState::ReadyStartable | JoinState::Bound) {
            continue;
        }
        assert!(
            entries(Seat::BallRow {
                state,
                assign_to: Some("alba-koi".to_owned()),
                move_to: vec!["zeta-pug".to_owned()],
            })
            .is_empty(),
            "{state:?} carries no ball verb"
        );
    }
}
