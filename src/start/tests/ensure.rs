//! The workspace-and-name executors (§4.2, Z5): the idempotent `lernie new`
//! ensure, the mint mapping the fire applies ([`on_mint`]), and the worktree
//! resolution ladder. The `bl`-facing executors are [`super::exec`]'s concern.

use super::{World, ball, fake_fail, fake_lernie};
use crate::binding::{work_worktree_path, workspace_path};
use crate::cli_outbound::Cli;
use crate::opslog::{Origin, SYNTHETIC_EXIT, YOG_STEP};
use crate::projects::join::JoinState;
use crate::start::{
    Deps, Payload, StartError, execute_ensure_workspace, on_mint, resolve_worktree,
};
use crate::test_support::spawn_guard;
use crate::world::{Layout, layout_under};
use lernie::mint::MintError;
use std::path::PathBuf;

/// The world layout anchored on this world's yog data root — where the §8.6
/// capability-control shim is resolved from.
fn layout(w: &World) -> Layout {
    layout_under(w.yog.path())
}

/// Start deps whose `lernie` is the only binary these rungs reach.
fn deps(w: &World, lernie: &Cli) -> Deps {
    Deps {
        bl: Cli::new("/no/bl"),
        lernie: lernie.clone(),
        state_root: w.state.path().to_path_buf(),
        yog_binary: PathBuf::from("/no/yog"),
    }
}

#[test]
fn ensure_skips_when_the_workspace_already_exists() {
    let w = World::new();
    let ws = workspace_path(w.yog.path(), "n");
    std::fs::create_dir_all(ws.join("repo.git")).unwrap();
    let lernie = Cli::new("/definitely/not/a/real/lernie");
    let created =
        execute_ensure_workspace(&deps(&w, &lernie), "TS", &ws, &layout(&w), Origin::Balls)
            .unwrap();
    assert!(!created, "existing workspace skipped");
    assert!(w.ops().is_empty(), "skip runs and logs nothing");
}

#[test]
fn ensure_creates_the_workspace_and_logs() {
    let _g = spawn_guard();
    let w = World::new();
    let ws = workspace_path(w.yog.path(), "cobalt-gecko");
    let lernie = Cli::new(fake_lernie(w.bin.path()));
    assert!(
        execute_ensure_workspace(&deps(&w, &lernie), "TS", &ws, &layout(&w), Origin::Balls)
            .unwrap()
    );
    assert!(ws.parent().unwrap().is_dir(), "parent chain mkdir -p'd");
    assert_eq!(
        &w.ops()[0].argv[1..],
        &["new", ws.to_string_lossy().as_ref()]
    );
}

#[test]
fn ensure_errors_and_logs_on_a_nonzero_new() {
    let _g = spawn_guard();
    let w = World::new();
    let ws = workspace_path(w.yog.path(), "n");
    let lernie = Cli::new(fake_fail(w.bin.path(), "lernie", "disk full"));
    let err = execute_ensure_workspace(&deps(&w, &lernie), "TS", &ws, &layout(&w), Origin::Balls)
        .unwrap_err();
    assert!(matches!(err, StartError::VerbFailed { verb: "new", .. }));
}

#[test]
fn ensure_logs_a_mkdir_step_failure() {
    // The parent chain cannot be created (a file sits where a dir must go) → a
    // `["yog-step","mkdir"]` row before the Io error (§4.2, Z5), no `lernie` spawn.
    let w = World::new();
    let blocker = w.yog.path().join("blocked");
    std::fs::write(&blocker, b"x").unwrap();
    let ws = blocker.join("workspaces").join("n");
    let lernie = Cli::new("/definitely/not/a/real/lernie");
    let err = execute_ensure_workspace(&deps(&w, &lernie), "TS", &ws, &layout(&w), Origin::Balls)
        .unwrap_err();
    assert!(matches!(err, StartError::Io(_)));
    assert_eq!(w.ops()[0].argv, [YOG_STEP, "mkdir"]);
}

#[test]
fn ensure_creates_whatever_the_birth_template_names() {
    // bl-00ee: bl-c3a9 refused this exact fixture — a template naming a row
    // brazen's table lacks — and §16.2's wall made that refusal permanent, since
    // a newborn workspace's provider table is brazen's shipped rows and the
    // operator's sign-in only reaches it AFTER birth. Birth now judges nothing
    // about providers: the workspace is created, and a dead row is faulted in
    // the §9.5 pane and surfaced at the first dispatch (§8.3) instead.
    let _g = spawn_guard();
    let w = World::new();
    let tmpl = layout(&w).lernie.join("template");
    std::fs::create_dir_all(&tmpl).unwrap();
    std::fs::write(
        tmpl.join("providers.yaml"),
        "roles:\n  worker:\n    provider: codex\n    model: gpt-5.4\n",
    )
    .unwrap();
    let ws = workspace_path(w.yog.path(), "n");
    let lernie = Cli::new(fake_lernie(w.bin.path()));
    assert!(
        execute_ensure_workspace(&deps(&w, &lernie), "TS", &ws, &layout(&w), Origin::Balls)
            .unwrap()
    );
    assert!(
        !w.ops().iter().any(|e| e.argv == [YOG_STEP, "template"]),
        "no birth-time provider step remains"
    );
}

#[test]
fn on_mint_passes_a_name_through() {
    let w = World::new();
    let name = on_mint(
        Ok("cobalt-gecko".to_owned()),
        w.state.path(),
        "TS",
        w.home.path(),
        Origin::Conversation,
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
