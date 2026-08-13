//! View-model tests: enumeration, the reused framing/attempts/token folds,
//! forgiving meta parsing, and the on-demand drill-in.

use tempfile::tempdir;

use super::{AGENT, step_dir, write_file, write_tool};
use crate::git_tree::{AgentState, Framing};
use crate::steps_view::{Doc, build, detail};

const COMPLETE: &[u8] = br#"{"type":"message_start"}
{"type":"usage","input_tokens":10,"output_tokens":5}
{"type":"finish","reason":"stop"}
{"type":"end"}
"#;
const FAILED: &[u8] = br#"{"type":"error","kind":"x"}
{"type":"end"}
"#;
const AUTH_FAILED: &[u8] = br#"{"type":"error","status":401,"message":"Unauthorized"}
{"type":"end"}
"#;

#[test]
fn build_summarizes_steps_in_order_reusing_framing_attempts_tokens() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_file(ws, "002", "response.json", FAILED);
    write_file(ws, "001", "response.json", COMPLETE);
    write_file(
        ws,
        "001",
        "meta.json",
        br#"{"commit":"abcdef1234567890","started_at":"t-start","ended_at":"t-end"}"#,
    );
    // A step with no response.json at all → killed, zero attempts/tokens.
    write_file(ws, "003", "meta.json", b"{}");
    // Non-step entries: wrong width, non-digits, and a file named like a seq
    // (a 3-digit name that is not a directory).
    write_file(ws, "0004", "meta.json", b"{}");
    std::fs::create_dir_all(step_dir(ws, "notaseq")).unwrap();
    std::fs::write(ws.join("steps").join(AGENT).join("006"), b"x").unwrap();

    let view = build(ws, AGENT, AgentState::Stopped);
    let seqs: Vec<&str> = view.steps.iter().map(|s| s.seq.as_str()).collect();
    assert_eq!(seqs, vec!["001", "002", "003"]);

    let s1 = &view.steps[0];
    assert_eq!(s1.framing, Framing::Complete);
    assert_eq!(s1.attempts, 1);
    assert_eq!(s1.tokens.total_tokens(), 15);
    assert_eq!(s1.commit.as_deref(), Some("abcdef1234567890"));
    assert_eq!(s1.started_at.as_deref(), Some("t-start"));
    assert_eq!(s1.ended_at.as_deref(), Some("t-end"));

    assert_eq!(view.steps[1].framing, Framing::Failed);
    assert_eq!(view.steps[2].framing, Framing::Killed);
    assert_eq!(view.steps[2].attempts, 0);
    // 003's meta is `{}` — every timestamp/commit field absent.
    assert_eq!(view.steps[2].commit, None);
}

#[test]
fn build_flags_auth_shaped_step_failures_for_the_login_affordance() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_file(ws, "001", "response.json", COMPLETE);
    write_file(ws, "002", "response.json", FAILED); // a generic failure
    write_file(ws, "003", "response.json", AUTH_FAILED); // a 401 auth failure
    let view = build(ws, AGENT, AgentState::Stopped);
    // Only the auth-shaped failure carries the Login affordance (§8.3 detection):
    // a complete step and a non-auth failure do not.
    assert!(!view.steps[0].auth_failed.offered());
    assert!(!view.steps[1].auth_failed.offered());
    assert!(view.steps[2].auth_failed.offered());
}

#[test]
fn build_missing_tree_is_empty() {
    let dir = tempdir().unwrap();
    assert_eq!(
        build(dir.path(), AGENT, AgentState::Stopped).steps,
        Vec::new()
    );
}

#[test]
fn build_forgiving_when_meta_is_malformed_or_absent() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_file(ws, "001", "response.json", COMPLETE);
    write_file(ws, "001", "meta.json", b"not json at all");
    let view = build(ws, AGENT, AgentState::Stopped);
    // Malformed meta → all meta-derived fields absent, framing still read.
    assert_eq!(view.steps[0].commit, None);
    assert_eq!(view.steps[0].started_at, None);
    assert_eq!(view.steps[0].framing, Framing::Complete);
}

#[test]
fn detail_parses_records_splits_response_and_derives_tool_is_error() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_file(ws, "001", "meta.json", br#"{"commit":"c"}"#);
    write_file(ws, "001", "request.json", br#"{"model":"opus"}"#);
    write_file(
        ws,
        "001",
        "staging.json",
        br#"[{"type":"text","text":"hi"}]"#,
    );
    write_file(
        ws,
        "001",
        "response.json",
        b"{\"type\":\"message_start\"}\nnot json\n{\"type\":\"end\"}\n",
    );
    // exit 0 → ok; exit 2 → error; malformed output → raw, not an error;
    // output with no exit_code → not an error.
    write_tool(ws, "001", "toolu_1", b"{}", Some(br#"{"exit_code":0}"#));
    write_tool(ws, "001", "toolu_2", b"{}", Some(br#"{"exit_code":2}"#));
    write_tool(ws, "001", "toolu_0", b"{}", Some(b"oops"));
    write_tool(ws, "001", "toolu_3", b"{}", Some(br#"{"stdout":"x"}"#));
    // A stray file at the tools/ level is not a tool call.
    std::fs::write(step_dir(ws, "001").join("tools").join(".keep"), b"").unwrap();

    let d = detail(ws, AGENT, "001");
    assert!(matches!(d.meta, Doc::Json { .. }));
    assert!(matches!(d.request, Doc::Json { .. }));
    assert!(matches!(d.staging, Doc::Json { .. }));

    // response: three events, the middle one kept raw; the empty trailing
    // line dropped.
    assert_eq!(d.response.len(), 3);
    assert!(matches!(d.response[0], Doc::Json { .. }));
    assert_eq!(d.response[1], Doc::Unparsed(b"not json".to_vec()));
    assert!(matches!(d.response[2], Doc::Json { .. }));

    // tools sorted by id; is_error derived from exit_code.
    let ids: Vec<&str> = d.tools.iter().map(|t| t.tool_id.as_str()).collect();
    assert_eq!(ids, vec!["toolu_0", "toolu_1", "toolu_2", "toolu_3"]);
    let errs: Vec<bool> = d.tools.iter().map(|t| t.is_error).collect();
    assert_eq!(errs, vec![false, false, true, false]);
    assert_eq!(d.tools[0].output, Doc::Unparsed(b"oops".to_vec()));
}

/// STORIES **S7-T2** / DESIGN §11, view-model half (bl-307f): the drive's own
/// repro — a real step whose `request.json` is overwritten with `{ this is not
/// json`. The doc is [`Doc::Unparsed`], *not* [`Doc::Absent`] and not silently
/// raw: the two are distinct facts here so the renderer can frame one and not
/// the other. Every byte is kept, and the sibling records still build.
#[test]
fn a_malformed_record_is_unparsed_not_absent_and_siblings_still_build() {
    let dir = tempdir().unwrap();
    let ws = dir.path();
    write_file(ws, "001", "request.json", b"{ this is not json");
    write_file(ws, "001", "meta.json", br#"{"commit":"c0ffee"}"#);
    // A file of pure whitespace parses as nothing either — malformed, not absent.
    write_file(ws, "001", "staging.json", b"   \n");

    let d = detail(ws, AGENT, "001");
    assert_eq!(
        d.request,
        Doc::Unparsed(b"{ this is not json".to_vec()),
        "malformed keeps its bytes AND says it is malformed"
    );
    assert_eq!(d.staging, Doc::Unparsed(b"   \n".to_vec()));
    assert!(
        matches!(d.meta, Doc::Json { .. }),
        "the sibling record parses"
    );
    assert!(d.response.is_empty(), "no response.json ⇒ no events");
}

#[test]
fn detail_is_forgiving_when_every_file_is_absent() {
    let dir = tempdir().unwrap();
    let d = detail(dir.path(), AGENT, "001");
    assert_eq!(d.meta, Doc::Absent);
    assert_eq!(d.request, Doc::Absent);
    assert_eq!(d.staging, Doc::Absent);
    assert!(d.response.is_empty());
    assert!(d.tools.is_empty());
    assert_eq!(d.seq, "001");
}

/// The affordance is **routed**: an auth-shaped step is upgraded from
/// `Unrouted` to the provider row its model is bound to by the agent's own
/// governing config — the join that turns "log in" into "log in to this".
/// Driven over a real workspace repo, because the row comes out of git.
#[test]
fn an_auth_failed_step_names_the_row_its_governing_config_binds() {
    use crate::git_tree::tests::fixture::Fixture;
    use crate::login::auth::AuthFailure;
    use crate::test_support::TEMPLATE_PROVIDERS;

    const CONV: &str = "20260807T165806Z-1dc97a92";
    let fx = Fixture::new();
    fx.commit_other("providers.yaml", TEMPLATE_PROVIDERS);
    fx.build_agent(CONV, "what balls are open for yog?");

    let step = |seq: &str, response: &[u8], request: Option<&str>| {
        let dir = fx.path.join("steps").join(CONV).join(seq);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("response.json"), response).unwrap();
        if let Some(model) = request {
            let body = format!(r#"{{"model":"{model}","max_tokens":4096}}"#);
            std::fs::write(dir.join("request.json"), body).unwrap();
        }
    };
    // 001 completes on the worker's model; 002 fails on it, auth-shaped.
    step("001", COMPLETE, Some("claude-sonnet-5"));
    step("002", AUTH_FAILED, Some("claude-sonnet-5"));

    let view = build(&fx.path, CONV, AgentState::Stopped);
    // A healthy step is never routed — there is nothing to log in to.
    assert_eq!(view.steps[0].auth_failed, AuthFailure::No);
    // The failing one names the row TEMPLATE_PROVIDERS binds claude-sonnet-5 to.
    assert_eq!(
        view.steps[1].auth_failed,
        AuthFailure::Row("anthropic".to_string())
    );
}

/// Three ways the row is not derivable, and all three are the same value: the
/// affordance still paints, unrouted, routing the operator to the pane where a
/// row is picked by hand. Absent data is a value, never a branch — and never a
/// guess.
#[test]
fn an_unroutable_auth_failure_still_offers_the_affordance() {
    use crate::git_tree::tests::fixture::Fixture;
    use crate::login::auth::AuthFailure;
    use crate::test_support::TEMPLATE_PROVIDERS;

    const CONV: &str = "20260807T165806Z-1dc97a92";
    let fx = Fixture::new();
    fx.commit_other("providers.yaml", TEMPLATE_PROVIDERS);
    fx.build_agent(CONV, "what balls are open for yog?");

    let step = |seq: &str, request: Option<&[u8]>| {
        let dir = fx.path.join("steps").join(CONV).join(seq);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("response.json"), AUTH_FAILED).unwrap();
        if let Some(bytes) = request {
            std::fs::write(dir.join("request.json"), bytes).unwrap();
        }
    };
    // No request.json at all; unparseable bytes; a model no role binds.
    step("001", None);
    step("002", Some(b"not json"));
    step("003", Some(br#"{"max_tokens":4096}"#));
    step("004", Some(br#"{"model":"gpt-5.6-sol"}"#));

    let view = build(&fx.path, CONV, AgentState::Stopped);
    for step in &view.steps {
        assert_eq!(step.auth_failed, AuthFailure::Unrouted, "step {}", step.seq);
    }
}
