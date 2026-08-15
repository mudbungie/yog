//! The §11 altitude-0 listings (REMOTE §9.7, bl-296f, bl-b4b5): the workspace
//! enumeration — with the §4.1 pin rank, the §2.2 lineage tip and the §7.2
//! currency notes — and one workspace's bound balls with their §3.5 figures.
//!
//! Cut off [`super`] at §12's per-file budget, on the seam the surface draws:
//! these two are what the window's chrome asks, and the rest of that file is
//! what a pane asks. Every arm of every optional field rides, because a listing
//! whose rows are all the easy case proves only that the easy case survives.

use super::super::super::super::super::{Reply, WsRow};
use crate::binding::WorkspaceKind;
use crate::projects::join::JoinState;

fn workspaces() -> Vec<WsRow> {
    let row = |name: &str, kind, attention, agents, running, pinned| WsRow {
        workspace: name.to_owned(),
        kind,
        attention,
        agents,
        running,
        pinned,
        // Both arms of the §2.2 lineage tip ride the round trip too (bl-b4b5):
        // a workspace with no lineage derived yet must read back as one.
        config_tip: (attention > 0).then(|| crate::model_pick::ConfigTip {
            oid: "c".repeat(40),
            short_oid: "cccccccc".into(),
        }),
    };
    vec![
        row(
            "alba",
            WorkspaceKind::Named {
                name: "alba".into(),
            },
            2,
            5,
            true,
            // Both arms of the §4.1 pin rank ride the round trip: an absent one
            // must not read back as rank 0, which is the first hoisted tab.
            Some(1),
        ),
        row("f", WorkspaceKind::Foreign, 0, 0, false, None),
        row("r", WorkspaceKind::Replay, 0, 1, false, Some(0)),
    ]
}

/// Both altitude-0 answers, in the order the round trip reads them.
pub(super) fn chrome() -> Vec<Reply> {
    vec![
        // Both arms of the §7.2 notes: an answer that says how stale it is and
        // one that says nothing, because absent and present are two readings.
        Reply::Workspaces(crate::boundary::reply::Workspaces {
            rows: workspaces(),
            stale: Some("derivation 31 s behind".into()),
            growth: Some("brave-fox +12 branches".into()),
        }),
        Reply::Workspaces(crate::boundary::reply::Workspaces {
            rows: workspaces(),
            stale: None,
            growth: None,
        }),
        Reply::WorkspaceBalls(vec![
            crate::nav::BoundBall {
                id: "bl-1".into(),
                badge: Some("delivered".into()),
                project: "p".into(),
                owner: "alba".into(),
                state: JoinState::Delivered,
                spend: crate::spend::Figure {
                    tokens: crate::budgets::BudgetSpend {
                        input_tokens: 12,
                        ..crate::budgets::BudgetSpend::default()
                    },
                    cost: Some(crate::spend::Cost {
                        micro_usd: 2_500_000,
                        unpriced_tokens: 3,
                    }),
                    attribution: crate::spend::Attribution::Conversations(2),
                },
            },
            // A ball needing no badge, and a figure the price table cannot
            // price: both absences must read back as absences.
            crate::nav::BoundBall {
                id: "bl-2".into(),
                badge: None,
                project: "p".into(),
                owner: "alba".into(),
                state: JoinState::Bound,
                spend: crate::spend::Figure {
                    tokens: crate::budgets::BudgetSpend::default(),
                    cost: None,
                    attribution: crate::spend::Attribution::Workspace,
                },
            },
        ]),
    ]
}
