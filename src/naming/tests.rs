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
        leaf(Path::new("/d/litany/workspaces/20260727T093000Z-f0reign")),
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
        "/d/litany/workspaces/auto-1",
        "/d/litany/replays/replay-1",
    ]);
    assert_eq!(by_leaf(&s, "home"), Ok(s[0].clone()));
    assert_eq!(by_leaf(&s, "auto-1"), Ok(s[1].clone()));
    assert_eq!(by_leaf(&s, "replay-1"), Ok(s[2].clone()));
}

/// An unknown leaf refuses naming the token — never a guess at the one
/// workspace that happens to exist — **and names what could have been typed**
/// (bl-3377), so the refusal is a way out rather than a dead end.
#[test]
fn an_unknown_leaf_refuses_and_names_the_set() {
    let s = set(&["/d/yog/workspaces/home"]);
    assert_eq!(
        by_leaf(&s, "nope"),
        Err("unknown workspace \"nope\" — known: home".to_owned())
    );
    // An empty world says so outright: "nothing answers to a name here" is a
    // different fact from "you typed the wrong one".
    assert_eq!(
        by_leaf(&[], "home"),
        Err("unknown workspace \"home\" — none is enumerated here".into())
    );
}

/// A leaf two roots both hold addresses no one workspace — that world's §3.2
/// join is already ambiguous, so the refusal says so instead of picking.
#[test]
fn an_ambiguous_leaf_refuses() {
    let s = set(&["/d/yog/workspaces/home", "/d/litany/replays/home"]);
    assert_eq!(
        by_leaf(&s, "home"),
        Err("ambiguous workspace \"home\"".to_owned())
    );
}

/// A project has no name of its own, so one is derived: the basename wherever
/// it is already unique.
#[test]
fn a_project_names_by_its_basename() {
    let s = set(&["/home/u/dev/yog", "/home/u/dev/litany"]);
    assert_eq!(name_of(&s, &s[0]), "yog");
    assert_eq!(name_of(&s, &s[1]), "litany");
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
        Err("unknown project \"proj\" — known: a/x/proj, b/x/proj".to_owned()),
        "the refusal names the derived words `--project` takes, not the paths"
    );
}

/// A refusal is a sentence, not a listing (bl-3377): past the cap it says how
/// many more rather than scrolling the token away.
#[test]
fn a_crowded_world_names_a_bounded_prefix_and_a_count() {
    let paths: Vec<String> = (0..20).map(|n| format!("/d/p{n:02}")).collect();
    let s = set(&paths.iter().map(String::as_str).collect::<Vec<_>>());
    let why = resolve(&s, "nope").expect_err("refused");
    assert!(
        why.starts_with("unknown project \"nope\" — known: p00, p01,"),
        "{why}"
    );
    assert!(why.ends_with(", and 8 more"), "{why}");
}
