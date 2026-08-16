//! The snapshot's two-direction addressing (REMOTE §8, bl-f5f6), and the set it
//! reads at the boundary (bl-6c9e).

use super::addressable;
use crate::app::Snapshot;
use crate::binding::{Workspace, WorkspaceKind};
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn snap(workspaces: &[(&str, WorkspaceKind)], projects: &[&str]) -> Snapshot {
    let mut s = Snapshot::empty(0);
    s.workspaces = workspaces
        .iter()
        .map(|(p, kind)| Workspace {
            path: PathBuf::from(p),
            kind: kind.clone(),
        })
        .collect();
    s.projects = projects.iter().map(PathBuf::from).collect();
    s
}

fn named(name: &str) -> WorkspaceKind {
    WorkspaceKind::Named {
        name: name.to_owned(),
    }
}

/// A named workspace's wire name is its §3.1 name, and the round trip is the
/// identity — the property every gesture's addressing rests on.
#[test]
fn workspace_round_trips_through_its_name() {
    let s = snap(
        &[
            ("/d/yog/workspaces/home", named("home")),
            ("/d/lernie/workspaces/auto-1", WorkspaceKind::Foreign),
        ],
        &[],
    );
    assert_eq!(s.ws_name(Path::new("/d/yog/workspaces/home")), "home");
    assert_eq!(
        s.ws_path("home"),
        Ok(PathBuf::from("/d/yog/workspaces/home"))
    );
    assert_eq!(
        s.ws_name(Path::new("/d/lernie/workspaces/auto-1")),
        "auto-1"
    );
    assert_eq!(
        s.ws_path("auto-1"),
        Ok(PathBuf::from("/d/lernie/workspaces/auto-1"))
    );
}

/// A project round-trips the same way, over the enumerated clone set.
#[test]
fn project_round_trips_through_its_name() {
    let s = snap(&[], &["/home/u/dev/yog", "/home/u/dev/lernie"]);
    assert_eq!(s.project_name(Path::new("/home/u/dev/yog")), "yog");
    assert_eq!(s.project_path("yog"), Ok(PathBuf::from("/home/u/dev/yog")));
}

/// An unknown name refuses naming the noun and the token, on both sets — the
/// engine never guesses at an address.
#[test]
fn unknown_names_refuse() {
    let s = snap(&[("/d/yog/workspaces/home", named("home"))], &["/p/yog"]);
    assert_eq!(
        s.ws_path("nope"),
        Err("unknown workspace \"nope\"".to_owned())
    );
    assert_eq!(
        s.project_path("nope"),
        Err("unknown project \"nope\"".to_owned())
    );
}

/// **A wall born since the last derivation resolves** (bl-6c9e): the enumeration
/// handed to [`addressable`] is the authority, so the name the reply that
/// founded it made addressable is addressable on the very next call.
#[test]
fn the_live_enumeration_resolves_a_wall_the_derivation_has_not_read() {
    let cached = snap(&[("/d/yog/workspaces/home", named("home"))], &[]);
    let published = Arc::new(cached);
    assert_eq!(
        published.ws_path("fresh"),
        Err("unknown workspace \"fresh\"".to_owned()),
        "the cached set is what the defect resolved over"
    );
    let live = vec![
        Workspace {
            path: PathBuf::from("/d/yog/workspaces/home"),
            kind: named("home"),
        },
        Workspace {
            path: PathBuf::from("/d/yog/workspaces/fresh"),
            kind: named("fresh"),
        },
    ];
    let current = addressable(Arc::clone(&published), live);
    assert_eq!(
        current.ws_path("fresh"),
        Ok(PathBuf::from("/d/yog/workspaces/fresh"))
    );
    // …and the derived facts are the ones that were published: the enumeration
    // is asked per gesture, the walks behind it are not re-run.
    assert_eq!(current.derived_at_unix, published.derived_at_unix);
    assert_eq!(current.projects, published.projects);
}

/// The other direction, for free: a workspace disk no longer holds leaves the
/// resolution at once rather than at the next sweep (§3.6's unmaking).
#[test]
fn a_workspace_disk_no_longer_holds_stops_resolving() {
    let published = Arc::new(snap(&[("/d/yog/workspaces/gone", named("gone"))], &[]));
    let current = addressable(published, Vec::new());
    assert_eq!(
        current.ws_path("gone"),
        Err("unknown workspace \"gone\"".to_owned())
    );
}

/// **The steady state pays nothing.** Two sets that already agree hand the
/// published derivation straight back — the same allocation, not a clone of
/// every tree and bill on it.
#[test]
fn an_unchanged_enumeration_is_handed_straight_back() {
    let published = Arc::new(snap(&[("/d/yog/workspaces/home", named("home"))], &[]));
    let live = vec![Workspace {
        path: PathBuf::from("/d/yog/workspaces/home"),
        kind: named("home"),
    }];
    let current = addressable(Arc::clone(&published), live);
    assert!(Arc::ptr_eq(&published, &current));
}
