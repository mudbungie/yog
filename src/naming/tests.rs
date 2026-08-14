//! The wire-name rules, both nouns and both directions (REMOTE §8, bl-f5f6).

use super::{by_leaf, leaf, name_of, resolve};
use std::path::{Path, PathBuf};

fn set(paths: &[&str]) -> Vec<PathBuf> {
    paths.iter().map(PathBuf::from).collect()
}

/// §3.1's rule, said once: a workspace's leaf **is** its name — for one of
/// yog's own and for a foreign one alike, with no kind tag between them.
#[test]
fn a_workspace_is_named_by_its_leaf() {
    assert_eq!(leaf(Path::new("/d/yog/workspaces/home")), "home");
    assert_eq!(
        leaf(Path::new("/d/lernie/workspaces/20260727T093000Z-f0reign")),
        "20260727T093000Z-f0reign"
    );
}

/// A rootless path is no workspace, and names nothing — the general path with
/// no input rather than a branch.
#[test]
fn a_rootless_path_names_nothing() {
    assert_eq!(leaf(Path::new("")), "");
}

/// The reverse read, across all three roots.
#[test]
fn by_leaf_is_the_inverse() {
    let s = set(&[
        "/d/yog/workspaces/home",
        "/d/lernie/workspaces/auto-1",
        "/d/lernie/replays/replay-1",
    ]);
    assert_eq!(by_leaf(&s, "home"), Ok(s[0].clone()));
    assert_eq!(by_leaf(&s, "auto-1"), Ok(s[1].clone()));
    assert_eq!(by_leaf(&s, "replay-1"), Ok(s[2].clone()));
}

/// An unknown leaf refuses naming the token — never a guess at the one
/// workspace that happens to exist.
#[test]
fn an_unknown_leaf_refuses() {
    let s = set(&["/d/yog/workspaces/home"]);
    assert_eq!(
        by_leaf(&s, "nope"),
        Err("unknown workspace \"nope\"".to_owned())
    );
    assert_eq!(
        by_leaf(&[], "home"),
        Err("unknown workspace \"home\"".into())
    );
}

/// A leaf two roots both hold addresses no one workspace — that world's §3.2
/// join is already ambiguous, so the refusal says so instead of picking.
#[test]
fn an_ambiguous_leaf_refuses() {
    let s = set(&["/d/yog/workspaces/home", "/d/lernie/replays/home"]);
    assert_eq!(
        by_leaf(&s, "home"),
        Err("ambiguous workspace \"home\"".to_owned())
    );
}

/// A project has no name of its own, so one is derived: the basename wherever
/// it is already unique.
#[test]
fn a_project_names_by_its_basename() {
    let s = set(&["/home/u/dev/yog", "/home/u/dev/lernie"]);
    assert_eq!(name_of(&s, &s[0]), "yog");
    assert_eq!(name_of(&s, &s[1]), "lernie");
}

/// …and grows exactly enough to separate two checkouts that share one.
#[test]
fn a_shared_basename_grows_one_component() {
    let s = set(&["/a/x/proj", "/b/x/proj", "/a/y/proj"]);
    let names: Vec<String> = s.iter().map(|p| name_of(&s, p)).collect();
    assert_eq!(names, vec!["a/x/proj", "b/x/proj", "y/proj"]);
}

/// The fallback: a relative member whose whole path is another's tail can never
/// be told apart by a suffix, so it names itself.
#[test]
fn an_unseparable_relative_names_itself() {
    let s = set(&["yog", "/x/yog"]);
    assert_eq!(name_of(&s, &s[0]), "yog");
    assert_eq!(name_of(&s, &s[1]), "x/yog");
}

/// Naming is total over any path, and a path the set does not hold names
/// **itself** — no suffix of it is unique, because no suffix of it occurs. That
/// is the honest answer: [`resolve`] then refuses it, at the edge where a token
/// arrives, rather than the name quietly aliasing an enumerated project.
#[test]
fn a_path_outside_the_set_names_itself() {
    let s = set(&["/d/clones/yog"]);
    assert_eq!(name_of(&s, Path::new("/elsewhere/ops")), "/elsewhere/ops");
    assert!(resolve(&s, "/elsewhere/ops").is_err());
}

/// The project mapping, read backwards.
#[test]
fn resolve_is_the_inverse() {
    let s = set(&["/a/x/proj", "/b/x/proj"]);
    assert_eq!(resolve(&s, "a/x/proj"), Ok(s[0].clone()));
    assert_eq!(
        resolve(&s, "proj"),
        Err("unknown project \"proj\"".to_owned())
    );
}
