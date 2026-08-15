//! The REMOTE §4 narrowing: what a scoped derivation holds, and — the point —
//! what an unregistered name earns when a gesture names it anyway.

use super::*;
use crate::app::snapshot::Growth;
use crate::binding::{Workspace, WorkspaceKind};
use crate::git_tree::GitTree;
use std::path::PathBuf;

const MINE: &str = "/d/yog/workspaces/home";
const THEIRS: &str = "/d/yog/workspaces/corp";

fn allowed(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|n| (*n).to_owned()).collect()
}

/// A derivation holding two workspaces, every workspace-keyed field populated
/// for both.
fn snap() -> Snapshot {
    let mut s = Snapshot::empty(0);
    for path in [MINE, THEIRS] {
        s.workspaces.push(Workspace {
            path: PathBuf::from(path),
            kind: WorkspaceKind::Named {
                name: crate::naming::leaf(std::path::Path::new(path)),
            },
        });
        s.trees.insert(PathBuf::from(path), GitTree::default());
        s.bills.insert(PathBuf::from(path), Vec::new());
        s.growth.push(Growth {
            workspace: PathBuf::from(path),
            conversation: "c-1".to_owned(),
            added: 1,
        });
        s.fleet.insert(
            crate::naming::leaf(std::path::Path::new(path)),
            crate::fleet::Policy {
                project: PathBuf::from("/d/proj"),
                cap: 1,
                lease: None,
            },
        );
    }
    s.projects.push(PathBuf::from("/d/proj"));
    s
}

/// Every workspace-keyed field is narrowed together — one filter, so a field
/// cannot be forgotten and leak the workspace its owner cannot see.
#[test]
fn a_scoped_derivation_holds_only_the_registered_workspaces() {
    let scoped = snap().scoped(&allowed(&["home"]));
    assert_eq!(scoped.workspaces.len(), 1);
    assert_eq!(
        scoped.workspaces.first().map(|w| w.path.clone()),
        Some(PathBuf::from(MINE))
    );
    assert_eq!(
        scoped.trees.keys().collect::<Vec<_>>(),
        [&PathBuf::from(MINE)]
    );
    assert_eq!(
        scoped.bills.keys().collect::<Vec<_>>(),
        [&PathBuf::from(MINE)]
    );
    assert_eq!(scoped.growth.len(), 1);
    assert_eq!(scoped.fleet.keys().collect::<Vec<_>>(), ["home"]);
    // The workspace is the whole trust domain (§1.5): world-wide facts stay.
    assert_eq!(scoped.projects, [PathBuf::from("/d/proj")]);
}

/// **Absence, not a scope error** (§4): an unregistered workspace resolves with
/// the identical bytes a name nobody ever founded earns, so nothing a client
/// can ask confirms that the workspace exists.
#[test]
fn an_unregistered_name_refuses_exactly_as_an_unknown_one_does() {
    let scoped = snap().scoped(&allowed(&["home"]));
    let hidden = scoped.ws_path("corp").expect_err("refused");
    let absent = scoped.ws_path("no-such-thing").expect_err("refused");
    assert_eq!(hidden, "unknown workspace \"corp\"");
    assert_eq!(absent.replace("no-such-thing", "corp"), hidden);
    assert!(scoped.ws_path("home").is_ok());
}

/// A certificate the operator has not seated sees a world with no workspace in
/// it — the same shape `Snapshot::empty` is, so every read surface already
/// answers it without a bootstrap branch.
#[test]
fn an_unseated_client_sees_no_workspace_at_all() {
    let scoped = snap().scoped(&BTreeSet::new());
    assert!(scoped.workspaces.is_empty());
    assert!(scoped.trees.is_empty());
    assert!(scoped.fleet.is_empty());
    assert!(scoped.growth.is_empty());
    assert!(scoped.ws_path("home").is_err());
}
