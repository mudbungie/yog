//! The snapshot's two-direction addressing (REMOTE §8, bl-f5f6).

use crate::app::Snapshot;
use crate::binding::{Workspace, WorkspaceKind};
use std::path::{Path, PathBuf};

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
