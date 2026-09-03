//! **S12-T1 attempt-argv**: what one attempt actually fires, and the world
//! pool its skill pins are drawn from.

use super::attempt;
use crate::fork::{Fire, argv, skills_root};
use std::path::{Path, PathBuf};

/// One addressed attempt, spelled out.
fn fire(ws: &str, parent: &str, goal: &str, attempt: crate::fork::Attempt, pool: &str) -> Fire {
    Fire {
        workspace: PathBuf::from(ws),
        parent: parent.to_owned(),
        goal: goal.to_owned(),
        attempt,
        skills_root: PathBuf::from(pool),
    }
}

/// The attempt is the ordinary fork: the role, the workspace, the dispatching
/// parent, the goal **verbatim**, and the fork point on `--from`. Nothing else
/// — a fork is a dispatch with a ref, never a second kind of dispatch.
#[test]
fn one_attempt_is_the_ordinary_dispatch_with_a_ref() {
    let out = argv(&fire(
        "/w/home",
        "20260803T090000Z-aaaa",
        "try it   the other  way\nagain",
        attempt("aaaa1111", "worker", &[]),
        "/pool",
    ));
    assert_eq!(
        out,
        vec![
            "dispatch",
            "worker",
            "/w/home",
            "20260803T090000Z-aaaa",
            "--goal",
            "try it   the other  way\nagain",
            "--from",
            "aaaa1111",
        ],
        "the goal reaches the model unmutated, spacing and newlines included"
    );
}

/// A config branch is a fork point like any other — the same argv, a different
/// `--from`. That is VISION V1.3's "one spawn gesture with one parameter".
#[test]
fn a_clean_start_differs_only_in_the_ref() {
    let here = argv(&fire(
        "/w",
        "root",
        "g",
        attempt("aaaa1111", "worker", &[]),
        "/pool",
    ));
    let clean = argv(&fire(
        "/w",
        "root",
        "g",
        attempt("config/strict", "worker", &[]),
        "/pool",
    ));
    let differ: Vec<usize> = here
        .iter()
        .zip(&clean)
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(differ, vec![7], "only the ref moves: {here:?} vs {clean:?}");
}

/// Each skill is one pin, and the destination mirrors the pool's own layout —
/// so a pinned skill lands exactly where `load_skill` would have put it and the
/// config's `skills/**` glob composes it.
#[test]
fn every_skill_is_one_pin_at_the_pools_own_layout() {
    let out = argv(&fire(
        "/w",
        "root",
        "g",
        attempt("aaaa1111", "worker", &["bash", "read_file"]),
        "/pool/skills",
    ));
    let pins: Vec<&String> = out
        .iter()
        .skip_while(|a| a.as_str() != "--pin")
        .filter(|a| a.as_str() != "--pin")
        .collect();
    assert_eq!(
        pins,
        vec![
            "skills/bash/SKILL.md=/pool/skills/bash/SKILL.md",
            "skills/read_file/SKILL.md=/pool/skills/read_file/SKILL.md",
        ]
    );
}

/// The pool hangs off the world's `$LITANY_HOME`, derived from the same anchor
/// every other world path is (§16.2) — never an ambient litany's.
#[test]
fn the_pool_lives_in_yogs_nested_world() {
    assert_eq!(
        skills_root(Path::new("/d/yog")),
        Path::new("/d/yog/world/litany/skills")
    );
}
