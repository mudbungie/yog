//! The §3.6 unmaking's derivations ([`super::super::confirm`]): what a delete
//! would destroy, and who it refuses. Split from [`super`] at §12's cap, on the
//! seam the module tree already draws.

use super::*;

#[test]
fn the_confirmation_derives_for_yogs_own_and_refuses_the_rest() {
    let project = PathBuf::from("/proj");
    let mut delivered = bound_row(&project, "bl-2", &ws(), "alba");
    delivered.state = JoinState::Delivered;
    let snap = snapshot(
        &ws(),
        "alba",
        vec![agent("c-1", AgentState::Stopped, 1)],
        vec![bound_row(&project, "bl-1", &ws(), "alba"), delivered],
    );
    let confirm = confirmation_of(&snap, &ws()).expect("named");
    assert_eq!(confirm.name, "alba");
    assert_eq!(
        confirm.ball_ids(),
        ["bl-1"],
        "only the live Bound claim releases — the Delivered row is the obituary"
    );
    assert!(!confirm.refused(), "a stopped conversation is not live");
    assert!(confirmation_of(&snap, Path::new("/other")).is_none());
}

#[test]
fn a_foreign_workspace_earns_no_confirmation() {
    use crate::binding::{Workspace, WorkspaceKind};
    let mut snap = snapshot(&ws(), "alba", vec![], vec![]);
    snap.workspaces = vec![Workspace {
        path: ws(),
        kind: WorkspaceKind::Foreign,
    }];
    assert!(confirmation_of(&snap, &ws()).is_none());
}
