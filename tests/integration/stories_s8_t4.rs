//! STORIES **S8-T4** the task-branch knob: each agent tracks on a balls space of
//! its own, that space is where the branch is written and read, a launch decides
//! which space an agent gets, and a subagent inherits its parent's (STORIES S8.4,
//! DESIGN §16.3, the per-agent ruling).
//!
//! **What this row used to assert, and why it does not any more.** Until the
//! ruling the knob was a *project's* publish policy with three modes, and the
//! row asserted the `bl conf set task-remote none` / `task-branch <name>` writes
//! it dispatched. Two facts retired that: the knob's subject is now the AGENT,
//! and `bl conf set task-branch` is scope-keyed to the landing — a per-*clone*
//! file — so it can bind a project but never an agent. The remote writes went
//! with the modes; a store remote is the project's fact, and asserting `origin`
//! as a URL was what put the literal word `origin` into a clone's binding as if
//! it were one (bl-e47b's second finding).
//!
//! The severability point survives whole, and is still made mechanically: the
//! only thing yog writes is balls' own layer-2 config key inside the agent's own
//! space, plus the §4.2 ops trail. Delete the space and the policy is gone; no
//! yog-shaped config file exists to leave behind.
//!
//! Like S8-T1, the fold halves read the **real** ambient snapshot and assert
//! path algebra over its anchor — they write nothing — while the file halves
//! drive the `Space` seam against a temp root, so no test ever writes into the
//! operator's own world.

#![allow(clippy::unwrap_used)]

use std::path::{Path, PathBuf};
use tempfile::tempdir;
use yog::opslog;
use yog::world::marks::{self, Space};
use yog::xdg::Env;

/// Every file under `root`, recursively — yog's own footprint.
fn files(root: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(files(&path));
        } else {
            out.push(path);
        }
    }
    out.sort();
    out
}

/// STORIES **S8-T4** the task-branch knob.
#[test]
fn s8_t4_each_agent_tracks_in_a_space_of_its_own() {
    let world = yog::world::compose(&Env::from_env());
    let root = yog::world::layout(&world).root;
    let alice = Path::new("/anywhere/workspaces/alice");
    let bob = Path::new("/anywhere/workspaces/bob");

    // --- Clause 1, the default: two agents, two spaces. The space is keyed by
    // the §3.1 name that is already the ball claimant, so the claimant and the
    // space it claims into can never disagree — one name, one wall, one branch.
    let a_root = marks::own_root(&world, alice);
    let b_root = marks::own_root(&world, bob);
    assert_eq!(a_root, root.join("walls/alice/marks"));
    assert_eq!(b_root, root.join("walls/bob/marks"));
    assert_ne!(a_root, b_root);

    // --- Clause 2, the launch: a launch pointed at a project layers nothing,
    // so that agent's `bl` IS the board's own — the same clone yog reads,
    // instantly consistent, with no sync in between. Every other rung is given
    // a space of its own, which is the ruling's default.
    assert!(marks::pairs(&world, alice, false).is_empty());
    assert_eq!(
        marks::pairs(&world, alice, true),
        vec![(
            marks::YOG_MARKS.to_owned(),
            a_root.to_string_lossy().into_owned()
        )]
    );

    // --- Clause 3, inheritance: the whole of it is one env pair, so a subagent
    // resolves its parent's space with no mechanism of its own — exactly how
    // `YOG_WALL` already reaches a whole descent (lernie hands its environment
    // to every tool subprocess it spawns).
    // The pair a parent layers names the space root, and a root IS both of
    // balls' homes — so the child process that inherits the var resolves its
    // parent's clone bundle and its parent's branch by the same fold, with
    // nothing passed down but the string.
    let inherited = Space::own(&a_root);
    assert_eq!(inherited.state, a_root);
    assert_eq!(inherited.config, a_root);
    assert_eq!(
        marks::pairs(&world, alice, true)[0].1,
        inherited.state.to_string_lossy()
    );

    // --- The world's own space keeps §16.2's state exactly where it is (every
    // clone yog already founded is still the one it reads) and nests balls'
    // CONFIG home, which is the leak bl-e47b closed: left ambient, balls read
    // the operator's `~/.config/balls`, whose stale seed template pruned
    // `bl-tracker` out of every landing yog founded — so those stores never
    // fetched and never pushed.
    let board = marks::space(&world);
    assert_eq!(board.state, root.join("state"));
    assert_eq!(board.config, root.join("config"));

    // --- Clause 4, the amend, against a temp space: an agent points its own
    // space at its own branch, and the answer is the branch re-read, not an
    // echo of what was asked.
    let dir = tempdir().unwrap();
    let state = dir.path().join("ops");
    let a_space = Space::own(&dir.path().join("alice"));
    let b_space = Space::own(&dir.path().join("bob"));
    assert_eq!(
        a_space.branch(),
        marks::SHARED_BRANCH,
        "untouched = default"
    );

    let landed = marks::apply(&a_space, &state, "T0", "balls/agents/alice").unwrap();
    assert_eq!(landed, "balls/agents/alice");
    assert_eq!(a_space.branch(), "balls/agents/alice");
    // …and it moved nobody else. That is the whole of the ruling's first
    // clause: two agents' task churn never collides.
    assert_eq!(b_space.branch(), marks::SHARED_BRANCH);

    // The write is balls' own §4 layer-2 key, in balls' own file, at balls' own
    // path under the space — never a yog-shaped setting yog would have to
    // translate, which is why `bl conf` (not yog) stays the authority on what a
    // checkout resolves and on which layer answered.
    let file = marks::config_file(&a_space.config);
    assert!(file.ends_with("balls/config.toml"), "{file:?}");
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        marks::body("balls/agents/alice")
    );

    // --- Severability, mechanically: yog's own state root holds the §4.2 trail
    // and nothing else. The policy is one file inside the agent's space, so
    // removing the space removes the policy — config deleted, no code edited.
    let written = files(&state);
    assert_eq!(
        written
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        ["ops.jsonl"],
        "yog owns no config file for this: {written:?}"
    );
    let ops = opslog::tail(&state, 32);
    assert_eq!(ops.len(), 1);
    assert_eq!(ops[0].argv, ["yog-step", "marks", "balls/agents/alice"]);
}
