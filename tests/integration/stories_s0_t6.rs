//! STORIES **S0-T6** login-detection: an auth-failed step on disk → the step's
//! view-model carries the **Login affordance** (§8.3 detection, §15 M6 Z8). The
//! detached driver has no captured stderr, so a prompt-time credential failure
//! surfaces as *derived agent state* (§13.3): the settled `response.json`'s error
//! text is auth-shaped, and [`steps_view::build`] flags the step so the shell
//! paints Login one click away beside it.
//!
//! Pure derivation over an on-disk fixture — no subprocess, so no fake substrate
//! (the reply-streams precedent): this drives the same public view-model the
//! shell's Steps inspector paints.

#![allow(clippy::unwrap_used)]

use std::path::Path;
use tempfile::tempdir;
use yog::steps_view;

const AGENT: &str = "20260719T120000Z-login";

/// Write one step's settled `response.json` under `steps/<agent>/NNN/`.
fn write_response(ws: &Path, seq: u32, body: &[u8]) {
    let dir = ws.join("steps").join(AGENT).join(format!("{seq:03}"));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("response.json"), body).unwrap();
}

#[test]
fn s0_t6_an_auth_failed_step_carries_the_login_affordance() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    // Step 001 completed cleanly; step 002's model call failed with a 401 — an
    // auth-shaped failure (credential/auth class, §8.3).
    write_response(
        ws,
        1,
        b"{\"type\":\"finish\",\"reason\":\"stop\"}\n{\"type\":\"end\"}\n",
    );
    write_response(
        ws,
        2,
        b"{\"type\":\"error\",\"status\":401,\"message\":\"Unauthorized: check credentials\"}\n{\"type\":\"end\"}\n",
    );

    // The §7.2 in-flight window (bl-776a) waits out only the *unanswered*
    // classes; a refusal is settled on disk the instant it is written, so a
    // zero grace here is the general path and not a test's special case.
    let view = steps_view::build(
        ws,
        AGENT,
        yog::git_tree::AgentState::Stopped,
        0,
        std::time::Duration::ZERO,
    );
    assert_eq!(view.steps.len(), 2);

    // The clean step offers no Login; only the auth-shaped failure carries it.
    assert!(
        !view.steps[0].auth_failed().offered(),
        "a complete step needs no Login"
    );
    assert!(
        view.steps[1].auth_failed().offered(),
        "the auth-failed step carries the Login affordance one click away"
    );
    // It is a genuine failure, not merely flagged (framing is Failed).
    assert_eq!(view.steps[1].framing, yog::git_tree::Framing::Failed);
}
