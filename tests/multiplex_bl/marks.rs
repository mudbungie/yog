//! The §16.3 **space** half of the parent binary's drive (bl-c21d): one full
//! `prime` → `create` → `claim` under an explicit `YOG_MARKS`, pinning where
//! the code worktree lands.
//!
//! A space is balls' state home and balls' config home together, and balls
//! folds its clone bundle AND its plugin territories off that one state home —
//! so an agent's own space must own its worktrees, not only its store. Before
//! bl-c21d it owned only the store: the arm supplied the space through the
//! `Edge` (which the linked balls reads) while `bl-delivery` — a real
//! subprocess holding no `Edge` — rebuilt its own `balls::layout::Xdg` from
//! `$XDG_STATE_HOME`, still the world's. The negative assertion below is the
//! whole regression: the world's territory must carry no worktree for a ball an
//! own space claimed.
//!
//! Environment mutation is lawful here for the parent binary's reason (its
//! module doc): every `tests/*.rs` is one process and the parent's is this
//! binary's only `#[test]`, so nothing runs concurrently with these writes.

use std::fs;
use std::path::{Path, PathBuf};

use yog::multiplex::dispatch;

use crate::fixtures::sole_child;

/// Drive a claim in an agent's own space and pin both halves: the store AND the
/// worktree land under the space, and the world's plugin territory stays empty
/// of it. `world_balls` is the world's `balls/` state root (the parent's
/// anchor); `proj` is the invocation path, resolved as the parent resolved it.
pub(crate) fn an_own_space_owns_its_worktrees(tmp: &Path, proj: &Path, world_balls: &Path) {
    // `<wall>/marks` is the shape §16.3 names — an own space is one directory
    // serving as both of balls' homes, keyed by the workspace name.
    let space = tmp.join("data/yog/world/walls/spaced/marks");
    set_marks(Some(&space));

    assert_eq!(
        dispatch(&super::argv(&["yog", "bl", "prime", "--as", "spaced"])),
        Some(0)
    );
    let clone = sole_child(&space.join("balls/clones"));
    assert!(
        clone.join("config").is_dir(),
        "landing founded in the space"
    );

    assert_eq!(
        dispatch(&super::argv(&[
            "yog",
            "bl",
            "create",
            "space ball",
            "--as",
            "spaced"
        ])),
        Some(0)
    );
    let id = sole_ball(&clone.join("tasks/tasks"));

    assert_eq!(
        dispatch(&super::argv(&["yog", "bl", "claim", &id, "--as", "spaced"])),
        Some(0)
    );
    let mirrored = proj.strip_prefix("/").unwrap();
    let worktree = space
        .join("balls/plugins/bl-delivery")
        .join(mirrored)
        .join(&id);
    assert!(
        worktree.join("README.md").is_file(),
        "worktree materialized in the space at {}",
        worktree.display()
    );
    let stray = world_balls
        .join("plugins/bl-delivery")
        .join(mirrored)
        .join(&id);
    assert!(
        !stray.exists(),
        "worktree escaped the space into the world at {}",
        stray.display()
    );

    // Leave the world's space standing for whatever the parent drives next: an
    // absent var IS the world's space, and the arm re-folds it on the way in.
    set_marks(None);
}

/// Layer an own space onto this process's env, or take it away.
fn set_marks(space: Option<&Path>) {
    match space {
        // SAFETY: single-threaded — the parent binary runs exactly one #[test]
        // and no thread exists to read the env concurrently (module doc).
        Some(path) => unsafe { std::env::set_var("YOG_MARKS", path) },
        // SAFETY: as above.
        None => unsafe { std::env::remove_var("YOG_MARKS") },
    }
}

/// The id of the one live ball in a store checkout's `tasks/` dir.
fn sole_ball(tasks: &Path) -> String {
    let mut mds: Vec<PathBuf> = fs::read_dir(tasks)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "md"))
        .collect();
    assert_eq!(mds.len(), 1, "one sealed ball: {mds:?}");
    let ball = mds.remove(0);
    ball.file_stem().unwrap().to_str().unwrap().to_owned()
}
