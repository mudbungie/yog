//! **The unattended reconciler's own regression suite** (bl-4e3c). Each beat
//! drives the REAL `scripts/deploy/reconcile.sh` — and, through it, the real
//! `scripts/deploy/verify.sh` — over a throwaway box whose `curl`, `docker` and
//! `systemctl` are shims. No live box, no registry, no engine.
//!
//! The three things a reconciler can get wrong are the three things a human
//! upgrade cannot, so they are what is proved here:
//!
//!   * **it acts when it should not** — on a dev tag, on a downgrade, on a tag
//!     already refused, or while a turn is in flight;
//!   * **it acts blind** — on a boundary that did not answer, or on a reading
//!     the engine itself has labelled stale;
//!   * **it leaves the box worse** — a release that does not serve must end
//!     with the box back on the tag that did, and never re-attempted.
//!
//! A deferral is asserted by what the box did NOT do (`calls`), because a
//! deferral has no other product; a rollback is asserted by what the box ends
//! up **serving**, because "the rollback script ran" is exactly the claim
//! bl-0719 caught a status print making.

#![allow(clippy::unwrap_used)]
#[path = "reconcile/harness.rs"]
mod harness;
use harness::{
    IDLE, PACKAGE, boundary_says, box_at, calls, crash_loops, deploy_env, key, reconcile, registry,
    registry_answers, serving,
};

/// Did the pass touch the unit at all? The one question a deferral answers.
fn restarted(dir: &tempfile::TempDir) -> bool {
    calls(dir).iter().any(|c| c.contains("restart"))
}

// ---------------------------------------------------------------------------
// It acts when there is something to do.

#[test]
fn a_newer_release_is_pulled_seated_and_proved() {
    let dir = box_at("yog:0.0.8-abc1234");
    registry(&dir, &["0.0.8", "0.0.9"]);
    boundary_says(&dir, IDLE);
    let (code, said) = reconcile(&dir);
    assert_eq!(code, 0, "{said}");
    assert_eq!(serving(&dir), format!("{PACKAGE}:0.0.9"), "{said}");
    assert_eq!(key(&dir, "YOG_IMAGE"), format!("{PACKAGE}:0.0.9"));
    let calls = calls(&dir);
    assert!(calls.iter().any(|c| c.contains("pull")), "{said}");
    // And the gate really ran: `s_client` is `verify.sh`'s fifth beat, the one
    // that separates "a process is up" from "the wire answers" (bl-0719). A
    // pass that seated a tag without reaching it has proved nothing.
    assert!(calls.iter().any(|c| c.contains("s_client")), "{calls:?}");
}

#[test]
fn the_identity_the_container_commits_under_survives_the_rewrite() {
    let dir = box_at("yog:0.0.8-abc1234");
    registry(&dir, &["0.0.9"]);
    boundary_says(&dir, IDLE);
    assert_eq!(reconcile(&dir).0, 0);
    // `deploy.env` is the one home for both facts; rewriting the tag must not
    // cost the box the identity git refuses to commit without.
    let env = deploy_env(&dir);
    for line in ["GIT_AUTHOR_NAME=", "GIT_COMMITTER_EMAIL="] {
        assert!(env.contains(line), "lost {line}:\n{env}");
    }
}

#[test]
fn a_tripped_start_limit_is_cleared_before_every_restart() {
    // The unit's start limit is twenty starts in a hundred seconds; a unit that
    // tripped it refuses to start, so a restart without `reset-failed` is a
    // no-op that reads as success — bl-0719's defect one layer up.
    let dir = box_at("yog:0.0.8-abc1234");
    registry(&dir, &["0.0.9"]);
    boundary_says(&dir, IDLE);
    assert_eq!(reconcile(&dir).0, 0);
    let calls = calls(&dir);
    let cleared = calls.iter().position(|c| c.contains("reset-failed"));
    let restarted = calls.iter().position(|c| c.contains("restart"));
    assert!(cleared < restarted, "{calls:?}");
}

// ---------------------------------------------------------------------------
// It does not act when there is nothing to do.

#[test]
fn a_box_on_the_newest_release_is_left_alone() {
    let dir = box_at(&format!("{PACKAGE}:0.0.9"));
    registry(&dir, &["0.0.8", "0.0.9"]);
    boundary_says(&dir, IDLE);
    let (code, said) = reconcile(&dir);
    assert_eq!(code, 0, "{said}");
    assert!(!restarted(&dir), "{said}");
}

#[test]
fn a_dev_tag_is_never_adopted_and_neither_is_latest() {
    // A dev build travels by `save | load` under a human who is watching. The
    // registry listing carrying one must not move an unattended box.
    let dir = box_at(&format!("{PACKAGE}:0.0.9"));
    registry(&dir, &["0.0.9", "0.1.0-abc1234", "latest"]);
    boundary_says(&dir, IDLE);
    let (code, said) = reconcile(&dir);
    assert_eq!(code, 0, "{said}");
    assert!(said.contains("0.0.9"), "{said}");
    assert!(!restarted(&dir), "{said}");
}

#[test]
fn a_box_ahead_of_the_registry_is_not_downgraded() {
    // `seat.sh` carried an unreleased build here on purpose. Undoing that
    // behind the operator's back is the reconciler acting against its author.
    let dir = box_at("yog:0.1.0-abc1234");
    registry(&dir, &["0.0.9"]);
    boundary_says(&dir, IDLE);
    let (code, said) = reconcile(&dir);
    assert_eq!(code, 0, "{said}");
    assert!(!restarted(&dir), "{said}");
    assert_eq!(serving(&dir), "yog:0.1.0-abc1234");
}

// ---------------------------------------------------------------------------
// It never upgrades over a turn, and never upgrades blind.

#[test]
fn an_in_flight_turn_defers_the_upgrade() {
    let dir = box_at("yog:0.0.8-abc1234");
    registry(&dir, &["0.0.9"]);
    boundary_says(
        &dir,
        "{\"ok\":true,\"kind\":\"workspaces\",\"rows\":[\
           {\"workspace\":\"a\",\"running\":false},{\"workspace\":\"b\",\"running\":true}]}",
    );
    let (code, said) = reconcile(&dir);
    // Deferring is success: the box is fine, the upgrade simply waits.
    assert_eq!(code, 0, "{said}");
    assert!(!restarted(&dir), "a turn was killed to make room:\n{said}");
    assert_eq!(serving(&dir), "yog:0.0.8-abc1234");
    assert_eq!(key(&dir, "YOG_IMAGE"), "yog:0.0.8-abc1234");
}

#[test]
fn a_stale_derivation_defers_the_upgrade() {
    // The engine is saying its own `running` bits may be behind the world. A
    // reading labelled stale is not a reading of "idle".
    let dir = box_at("yog:0.0.8-abc1234");
    registry(&dir, &["0.0.9"]);
    boundary_says(
        &dir,
        "{\"ok\":true,\"kind\":\"workspaces\",\"rows\":[{\"workspace\":\"a\",\"running\":false}],\
          \"stale\":\"derived 40s ago\"}",
    );
    let (code, said) = reconcile(&dir);
    assert_eq!(code, 0, "{said}");
    assert!(!restarted(&dir), "{said}");
}

#[test]
fn a_boundary_that_does_not_answer_defers_the_upgrade() {
    // No `workspaces.json`: the gesture exits non-zero, as it does when no
    // consumer answers. A failure to READ is not a reading.
    let dir = box_at("yog:0.0.8-abc1234");
    registry(&dir, &["0.0.9"]);
    let (code, said) = reconcile(&dir);
    assert_eq!(code, 0, "{said}");
    assert!(!restarted(&dir), "{said}");
}

// ---------------------------------------------------------------------------
// It leaves the box no worse.

#[test]
fn a_release_that_does_not_serve_is_rolled_back_and_refused() {
    let dir = box_at("yog:0.0.8-abc1234");
    registry(&dir, &["0.0.9"]);
    boundary_says(&dir, IDLE);
    // 0.0.9 crash-loops here, so `verify.sh`'s beats fail on it.
    crash_loops(&dir, &[&format!("{PACKAGE}:0.0.9")]);
    let (code, said) = reconcile(&dir);
    assert_ne!(code, 0, "a failed verify passed:\n{said}");
    // The proof is what the box SERVES, not that a rollback ran.
    assert_eq!(serving(&dir), "yog:0.0.8-abc1234", "{said}");
    assert_eq!(key(&dir, "YOG_IMAGE"), "yog:0.0.8-abc1234");
    assert_eq!(key(&dir, "YOG_REFUSED"), format!("{PACKAGE}:0.0.9"));
}

#[test]
fn a_refused_tag_is_never_attempted_a_second_time() {
    // The bound, and it is an invariant rather than a counter: without it a bad
    // release restarts the engine every fifteen minutes forever.
    let dir = box_at("yog:0.0.8-abc1234");
    registry(&dir, &["0.0.9"]);
    boundary_says(&dir, IDLE);
    crash_loops(&dir, &[&format!("{PACKAGE}:0.0.9")]);
    assert_ne!(reconcile(&dir).0, 0);
    std::fs::remove_file(dir.path().join("calls")).unwrap();
    let (code, said) = reconcile(&dir);
    assert_eq!(code, 0, "{said}");
    assert!(
        !restarted(&dir),
        "the refused tag was re-attempted:\n{said}"
    );
    assert!(said.contains("YOG_REFUSED"), "no remedy named:\n{said}");
}

#[test]
fn an_unpublished_package_is_a_clean_no_op() {
    // **The standing state until bl-6b96 lands.** DESIGN §10.1 names the
    // registry, the tag convention and the publishing authority, and no job
    // performs the push yet — so every pass finds nothing. A timer that goes
    // red every fifteen minutes on the one box state this was written to sit in
    // quietly is a timer an operator switches off.
    let dir = box_at("yog:0.0.8-abc1234");
    registry_answers(&dir, "404");
    boundary_says(&dir, IDLE);
    let (code, said) = reconcile(&dir);
    assert_eq!(code, 0, "{said}");
    assert!(!restarted(&dir), "{said}");
}

#[test]
fn a_package_with_no_released_tag_yet_is_a_clean_no_op() {
    // The same standing state one HTTP status over: the package exists but
    // carries nothing this reconciler may adopt.
    let dir = box_at("yog:0.0.8-abc1234");
    registry(&dir, &["latest"]);
    boundary_says(&dir, IDLE);
    let (code, said) = reconcile(&dir);
    assert_eq!(code, 0, "{said}");
    assert!(!restarted(&dir), "{said}");
}

#[test]
fn a_package_that_went_private_refuses_with_a_named_remedy() {
    // Distinguished from the two no-ops above by status alone, which is why the
    // status is read rather than collapsed into `curl -f`'s single exit code:
    // an empty registry is fine and a locked one is a box that has silently
    // stopped upgrading.
    let dir = box_at("yog:0.0.8-abc1234");
    registry_answers(&dir, "403");
    let (code, said) = reconcile(&dir);
    assert_ne!(code, 0, "{said}");
    assert!(said.contains("make deploy"), "no remedy named:\n{said}");
    assert!(!restarted(&dir), "{said}");
}

#[test]
fn a_registry_that_wants_a_credential_fails_with_a_named_remedy() {
    // The package is public by ruling, so this is a ruling having changed under
    // the box — and a box with no credential can only say so.
    let dir = box_at("yog:0.0.8-abc1234");
    boundary_says(&dir, IDLE);
    let (code, said) = reconcile(&dir);
    assert_ne!(code, 0, "{said}");
    assert!(said.contains("make deploy"), "no remedy named:\n{said}");
    assert!(!restarted(&dir), "{said}");
}

#[test]
fn a_box_that_was_never_seated_refuses_rather_than_guessing() {
    let dir = box_at("yog:0.0.8-abc1234");
    std::fs::remove_file(dir.path().join("home/.config/yog/deploy.env")).unwrap();
    let (code, said) = reconcile(&dir);
    assert_ne!(code, 0, "{said}");
    assert!(said.contains("seat.sh"), "{said}");
}
