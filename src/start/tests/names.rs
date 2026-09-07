//! The **name-shaped** halves of the start's executors, split off
//! [`ensure`](super::ensure) at §12's cap on the seam that file's own header
//! already drew: the mint mapping the fire applies ([`on_mint`]) and the §5.1
//! #5 worktree resolution ladder. Both answer *which name* rather than *does
//! the workspace exist*, and neither spawns `litany`.

use super::{World, ball};
use crate::binding::work_worktree_path;
use crate::opslog::{Origin, SYNTHETIC_EXIT, YOG_STEP};
use crate::projects::join::JoinState;
use crate::start::{Payload, StartError, on_mint, resolve_worktree};
use litany::mint::MintError;
use std::path::PathBuf;

#[test]
fn on_mint_passes_a_name_through() {
    let w = World::new();
    let name = on_mint(
        Ok("cobalt-gecko".to_owned()),
        w.state.path(),
        "TS",
        w.home.path(),
        Origin::Conversation,
        crate::registry::Client::default(),
    )
    .unwrap();
    assert_eq!(name, "cobalt-gecko");
    assert!(w.ops().is_empty(), "a clean mint logs nothing");
}

#[test]
fn on_mint_logs_an_exhausted_pool() {
    // Pool exhaustion is a non-spawn abort: a `["yog-step","mint"]` row (Z5) then
    // the error — the conversation mint's one non-spawn failure, made visible
    // (§3.3, §8.1 step 2; the workspace mint it once also served is gone).
    let w = World::new();
    let err = on_mint(
        Err(MintError::Exhausted(6)),
        w.state.path(),
        "TS",
        w.home.path(),
        Origin::Conversation,
        crate::registry::Client::default(),
    )
    .unwrap_err();
    assert!(matches!(err, StartError::Mint(MintError::Exhausted(6))));
    let e = &w.ops()[0];
    assert_eq!(e.argv, [YOG_STEP, "mint"]);
    assert_eq!(e.exit, SYNTHETIC_EXIT);
    assert!(e.stderr.contains("pool exhausted"));
}

#[test]
fn resolve_worktree_prefers_the_claim_then_disk_then_canonical() {
    // Addendum: the composer's ball worktree must be the path bl actually minted,
    // never a hardcoded canonical guess. Four cases, one function.
    let w = World::new();
    let (balls, project, name) = (w.balls.path(), w.project.path(), "cobalt-gecko");
    let existing = |id: &str| ball(project, id, JoinState::Bound);
    // A non-ball rung names no worktree.
    assert_eq!(
        resolve_worktree(&Payload::Bare, Some(project), balls, name, None),
        None
    );
    // The claim's cross-checked worktree wins verbatim (the `<id>-<claimant>`
    // variant when bl minted it — threaded from `ClaimResolved`).
    let claimed = PathBuf::from("/claimed/wt-suffixed");
    assert_eq!(
        resolve_worktree(
            &existing("bl-1"),
            Some(project),
            balls,
            name,
            Some(claimed.clone())
        ),
        Some(claimed),
    );
    // Resume (no claim), neither variant on disk → the canonical `<id>` formula.
    assert_eq!(
        resolve_worktree(&existing("bl-2"), Some(project), balls, name, None),
        Some(work_worktree_path(balls, project, "bl-2", None)),
    );
    // Resume where only the `<id>-<claimant>` worktree exists → that suffixed path.
    let suffixed = work_worktree_path(balls, project, "bl-3", Some(name));
    std::fs::create_dir_all(&suffixed).unwrap();
    assert_eq!(
        resolve_worktree(&existing("bl-3"), Some(project), balls, name, None),
        Some(suffixed),
    );
}
