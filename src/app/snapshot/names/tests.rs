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
            ("/d/litany/workspaces/auto-1", WorkspaceKind::Foreign),
        ],
        &[],
    );
    assert_eq!(s.ws_name(Path::new("/d/yog/workspaces/home")), "home");
    assert_eq!(
        s.ws_path("home"),
        Ok(PathBuf::from("/d/yog/workspaces/home"))
    );
    assert_eq!(
        s.ws_name(Path::new("/d/litany/workspaces/auto-1")),
        "auto-1"
    );
    assert_eq!(
        s.ws_path("auto-1"),
        Ok(PathBuf::from("/d/litany/workspaces/auto-1"))
    );
}

/// A project round-trips the same way, over the enumerated clone set.
#[test]
fn project_round_trips_through_its_name() {
    let s = snap(&[], &["/home/u/dev/yog", "/home/u/dev/litany"]);
    assert_eq!(s.project_name(Path::new("/home/u/dev/yog")), "yog");
    assert_eq!(s.project_path("yog"), Ok(PathBuf::from("/home/u/dev/yog")));
}

/// An unknown name refuses naming the noun, the token **and what could have
/// been typed instead** (bl-3377) — the engine never guesses at an address, and
/// never leaves the caller without one either.
#[test]
fn unknown_names_refuse_and_name_the_set() {
    let s = snap(&[("/d/yog/workspaces/home", named("home"))], &["/p/yog"]);
    assert_eq!(
        s.ws_path("nope"),
        Err("unknown workspace \"nope\" — known: home".to_owned())
    );
    assert_eq!(
        s.project_path("nope"),
        Err("unknown project \"nope\" — known: yog".to_owned())
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
        Err("unknown workspace \"fresh\" — known: home".to_owned()),
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
    let current = addressable(Arc::clone(&published), live, published.projects.clone());
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
    let current = addressable(published, Vec::new(), Vec::new());
    assert_eq!(
        current.ws_path("gone"),
        Err("unknown workspace \"gone\" — none is enumerated here".to_owned())
    );
}

/// **The noun bl-6c9e left behind** (bl-3377). `yog bl prime` founds a project
/// the way a raise founds a wall, and every ball gesture refused it — with a
/// sentence indistinguishable from a typo — until the next full sweep. The
/// same barrier now covers it, and runs backwards the same way.
#[test]
fn the_live_enumeration_resolves_a_project_the_derivation_has_not_read() {
    let published = Arc::new(snap(&[], &["/d/clones/old"]));
    assert_eq!(
        published.project_path("proj"),
        Err("unknown project \"proj\" — known: old".to_owned()),
        "the cached set is what the defect resolved over"
    );
    let live = vec![PathBuf::from("/d/clones/old"), PathBuf::from("/d/proj")];
    let current = addressable(Arc::clone(&published), Vec::new(), live);
    assert_eq!(current.project_path("proj"), Ok(PathBuf::from("/d/proj")));
    // …and a project disk no longer holds stops resolving at once.
    let gone = addressable(Arc::clone(&published), Vec::new(), Vec::new());
    assert_eq!(
        gone.project_path("old"),
        Err("unknown project \"old\" — none is enumerated here".to_owned())
    );
    // The derived facts are still the published ones: the enumeration is asked
    // per gesture, the ball walks behind it are not re-run.
    assert_eq!(current.derived_at_unix, published.derived_at_unix);
    assert_eq!(current.balls_by_project, published.balls_by_project);
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
    let current = addressable(Arc::clone(&published), live, published.projects.clone());
    assert!(Arc::ptr_eq(&published, &current));
}
