//! **What one tick does when it takes work**: the §8.1 start flow through the
//! ordinary doors, and what the trail says about a birth that lands, one the
//! substrate refuses, and one whose fire never launches at all (bl-ab13).
//!
//! Split from [`super`] at §12's cap on the seam that module's own doc draws —
//! the loop makes two moves, so its effect tests are cut in two: giving a claim
//! back is there, taking work is here.

use super::*;

/// The spawn: the ordinary §8.1 start flow, through the ordinary doors, leaving
/// the loop's own row naming the ball it took and the conversation it minted.
#[test]
fn a_landed_spawn_starts_a_drone_and_leaves_one_row() {
    let root = tempdir().expect("tempdir");
    let project = root.path().join("proj");
    let ws = root.path().join("ws");
    std::fs::create_dir_all(&project).expect("mkdir");
    let mut ctx = ctx(root.path(), armed_world(&ws, &project, false, None));
    // `bl claim` prints the worktree it minted (§3.2), and the start flow
    // cross-checks it — so the fake prints exactly the one balls' delivery
    // layout puts under this state root.
    let worktree = ctx
        .deps
        .balls_state_root
        .join("plugins/bl-delivery")
        .join(project.strip_prefix("/").expect("absolute"))
        .join("bl-1");
    std::fs::create_dir_all(&worktree).expect("mkdir");
    ctx.deps.bl = fake(
        root.path(),
        "bl",
        &format!(
            "#!/bin/sh\ncase \"$1\" in\n  claim) printf '%s\\n' '{}' ;;\nesac\nexit 0\n",
            worktree.display()
        ),
    );
    ctx.deps.lernie = fake(
        root.path(),
        "lernie",
        &format!(
            "#!/bin/sh\ncase \"$1\" in\n{}esac\nexit 0\n",
            crate::test_support::authoring_new_arm()
        ),
    );
    let acted = ctx.pass();
    let trail0 = std::fs::read_to_string(root.path().join("ops.jsonl")).unwrap_or_default();
    assert!(acted, "a ready ball under the cap is taken: {trail0}");
    let trail = std::fs::read_to_string(root.path().join("ops.jsonl")).expect("trail");
    assert!(trail.contains("yog-fleet"), "{trail}");
    assert!(trail.contains("spawn"), "{trail}");
    assert!(trail.contains("bl-1"), "{trail}");
}

/// **A birth is atomic against its own claim** (bl-ab13). Every piped step of
/// the §8.1 flow lands, the `bl claim` with them — and then the detached fire
/// cannot launch at all. The ball comes straight back rather than being held
/// forever by a workspace with no conversation on it, and there is still no
/// loop row, because the birth did not land.
///
/// The fire fails because the driver's working directory is the workspace and
/// this `lernie new` never made one: a launch into a directory that is not
/// there is exactly the class the loop could not previously survive.
#[test]
fn a_birth_whose_fire_never_launches_gives_its_own_claim_back() {
    let root = tempdir().expect("tempdir");
    let project = root.path().join("proj");
    let ws = root.path().join("ws");
    std::fs::create_dir_all(&project).expect("mkdir");
    let mut ctx = ctx(root.path(), armed_world(&ws, &project, false, None));
    let worktree = ctx
        .deps
        .balls_state_root
        .join("plugins/bl-delivery")
        .join(project.strip_prefix("/").expect("absolute"))
        .join("bl-1");
    std::fs::create_dir_all(&worktree).expect("mkdir");
    let seen = root.path().join("bl.args");
    ctx.deps.bl = fake(
        root.path(),
        "bl",
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >>'{}'\ncase \"$1\" in\n  claim) printf '%s\\n' '{}' ;;\nesac\nexit 0\n",
            seen.display(),
            worktree.display()
        ),
    );
    ctx.deps.lernie = fake(root.path(), "lernie", "#!/bin/sh\nexit 0\n");
    assert!(!ctx.pass(), "the fire never launched, so no birth landed");
    let args = std::fs::read_to_string(&seen).expect("bl was called");
    assert!(
        args.contains("claim bl-1") && args.contains("unclaim bl-1"),
        "the claim landed and was given straight back: {args}"
    );
    let trail = std::fs::read_to_string(root.path().join("ops.jsonl")).unwrap_or_default();
    assert!(
        !trail.contains("yog-fleet"),
        "a birth that did not converge is not a spawn row: {trail}"
    );
}

/// A spawn the start flow refuses leaves no loop row, for the same reason a
/// refused reap does.
#[test]
fn a_refused_spawn_writes_no_loop_row() {
    let root = tempdir().expect("tempdir");
    let project = root.path().join("proj");
    let ws = root.path().join("ws");
    std::fs::create_dir_all(&project).expect("mkdir");
    let mut ctx = ctx(root.path(), armed_world(&ws, &project, false, None));
    ctx.deps.bl = fake(
        root.path(),
        "bl",
        "#!/bin/sh\nprintf 'no\\n' 1>&2\nexit 3\n",
    );
    assert!(!ctx.pass());
    let trail = std::fs::read_to_string(root.path().join("ops.jsonl")).unwrap_or_default();
    assert!(!trail.contains("yog-fleet"), "{trail}");
}
