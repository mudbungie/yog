//! The **auth heuristic** (DESIGN §8.3): which failures are credential
//! failures, which provider row a failed model names, and what each derived
//! state says. Split from [`super`] at §12's cap along the seam that file's own
//! doc already drew — this is the pure classification table; the streamed
//! [`LoginRun`](crate::login::LoginRun) view-model and its outcome row are the
//! other half and stay next door.

use crate::git_tree::AgentState;
use crate::login::auth::{AuthFailure, classify, looks_auth, row_of_model};
use crate::model_pick::grammar::RoleModel;
use tempfile::tempdir;

#[test]
fn looks_auth_fires_on_the_credential_auth_class() {
    for line in [
        r#"{"type":"error","status":401}"#,
        r#"{"type":"error","status":403}"#,
        r#"{"message":"401 Unauthorized"}"#,
        r#"{"message":"403 Forbidden"}"#,
        r#"{"message":"permission denied"}"#,
        r#"{"message":"permission-denied"}"#,
        r#"{"message":"missing credential"}"#,
        r#"{"message":"authentication failed"}"#,
        r#"{"message":"not authorized"}"#,
        r#"{"message":"authorisation error"}"#,
        r#"{"message":"no API key configured"}"#,
        r#"{"message":"invalid api_key"}"#,
        r#"{"message":"bad apikey"}"#,
        r#"{"error":{"code":"unauthenticated"}}"#,
    ] {
        assert!(looks_auth(line), "should classify auth-shaped: {line}");
    }
}

#[test]
fn looks_auth_ignores_non_auth_failures() {
    for line in [
        r#"{"type":"error","kind":"transport","message":"connection reset"}"#,
        r#"{"message":"500 internal server error"}"#,
        r#"{"message":"rate limit exceeded"}"#,
        r#"{"author":"someone"}"#, // 'author' must not trip a bare 'auth'
        r#"{"message":"request timeout"}"#,
    ] {
        assert!(!looks_auth(line), "should not classify auth: {line}");
    }
}

#[test]
fn classify_needs_failed_framing_and_auth_text() {
    let auth =
        b"{\"type\":\"error\",\"status\":401,\"message\":\"Unauthorized\"}\n{\"type\":\"end\"}\n";
    assert!(classify(auth).offered());
    // Failed, but a transport reset is not auth-shaped.
    let other =
        b"{\"type\":\"error\",\"kind\":\"transport\",\"message\":\"reset\"}\n{\"type\":\"end\"}\n";
    assert!(!classify(other).offered());
    // Complete framing is not a failure at all; nor is an empty response.
    let ok = b"{\"type\":\"finish\",\"reason\":\"stop\"}\n{\"type\":\"end\"}\n";
    assert!(!classify(ok).offered());
    assert!(!classify(b"").offered());
}

/// The join that routes the affordance: a model id against the roles the
/// governing config declares. One row serves it → that row; nothing binds it,
/// or two rows do → no row, because a guessed row sends the operator through a
/// browser sign-in for a credential that was never the problem.
#[test]
fn row_of_model_names_a_row_only_when_the_config_binds_exactly_one() {
    let role = |role: &str, provider: &str, model: &str| RoleModel {
        role: role.to_string(),
        provider: provider.to_string(),
        model: model.to_string(),
        effort: None,
        priority: false,
    };
    let roles = [
        role("worker", "claude-session-direct", "claude-opus-4-6"),
        role("compactor", "openai-chatgpt", "gpt-5.4"),
    ];
    assert_eq!(
        row_of_model("claude-opus-4-6", &roles).as_deref(),
        Some("claude-session-direct")
    );
    // A model no role binds — a custom id, or one the config has since moved off.
    assert_eq!(row_of_model("gpt-5.6-sol", &roles), None);
    assert_eq!(row_of_model("claude-opus-4-6", &[]), None);
    // Two roles on one model under one row is still one answer.
    let shared = [
        role("worker", "openai-chatgpt", "gpt-5.4"),
        role("compactor", "openai-chatgpt", "gpt-5.4"),
    ];
    assert_eq!(
        row_of_model("gpt-5.4", &shared).as_deref(),
        Some("openai-chatgpt")
    );
    // Two roles on one model under *different* rows is not an answer.
    let split = [
        role("worker", "openai-chatgpt", "gpt-5.4"),
        role("compactor", "codex", "gpt-5.4"),
    ];
    assert_eq!(row_of_model("gpt-5.4", &split), None);
}

/// The three states say three different things, and each says as much as is
/// known — never more. The wordings live beside the classification (one home),
/// so the banner and the Steps mark cannot drift from what was derived.
#[test]
fn each_state_states_exactly_what_was_derived() {
    let no = AuthFailure::No;
    assert!(!no.offered());
    assert_eq!(no.row(), None);
    assert_eq!(no.banner(), "");
    assert_eq!(no.step_mark(), "");

    let unrouted = AuthFailure::Unrouted;
    assert!(unrouted.offered());
    assert_eq!(unrouted.row(), None);
    assert_eq!(
        unrouted.banner(),
        "⚠ the last step failed on credentials — log in to carry it on"
    );
    assert_eq!(unrouted.step_mark(), "⚠ auth — Login ↙");

    let routed = AuthFailure::Row("claude-session-direct".to_string());
    assert!(routed.offered());
    assert_eq!(routed.row(), Some("claude-session-direct"));
    assert_eq!(
        routed.banner(),
        "⚠ the last step failed on claude-session-direct's credentials — log in to carry it on"
    );
    assert_eq!(
        routed.step_mark(),
        "⚠ auth: claude-session-direct — Login ↙"
    );
}

#[test]
fn latest_step_auth_failed_reads_the_last_step_only() {
    let ws = tempdir().unwrap();
    let steps = ws.path().join("steps").join("root-1");
    // Step 001: the live auth-failure fixture shape (stench-pug, kind:auth).
    std::fs::create_dir_all(steps.join("001")).unwrap();
    std::fs::write(
        steps.join("001").join("response.json"),
        b"{\"type\":\"error\",\"kind\":\"auth\",\"message\":\"no credential for this provider: run `bz --login --provider <id>`\",\"provider_detail\":null}\n{\"type\":\"end\"}\n",
    )
    .unwrap();
    let steps_of = |agent: &str| crate::steps_view::build(ws.path(), agent, AgentState::Stopped);
    assert!(
        crate::login::auth::latest_step_auth_failed(&steps_of("root-1")).offered(),
        "an auth-failed latest step banners Login (§11)"
    );
    // Step 002 succeeds: the latest step decides, the old failure is history.
    std::fs::create_dir_all(steps.join("002")).unwrap();
    std::fs::write(
        steps.join("002").join("response.json"),
        b"{\"type\":\"finish\",\"reason\":\"stop\"}\n{\"type\":\"end\"}\n",
    )
    .unwrap();
    assert!(!crate::login::auth::latest_step_auth_failed(&steps_of("root-1")).offered());
    // No steps at all: nothing to banner.
    assert!(!crate::login::auth::latest_step_auth_failed(&steps_of("ghost")).offered());
}
