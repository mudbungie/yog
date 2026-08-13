//! The check's composition is pure and its one effect is a seam, so every
//! branch here is a table test with no network and no process.

use super::*;
use crate::monitor::Watch;
use crate::monitor::window::Evidence;
use std::sync::Mutex;

/// A caller that records what it was asked and answers a canned reply.
struct Fake {
    reply: Called,
    seen: Mutex<Vec<Vec<String>>>,
    /// The workspace each call was made on behalf of — the wall it spent
    /// against (§16.2 as amended).
    walls: Mutex<Vec<std::path::PathBuf>>,
}

/// The armed workspace every call below is made for.
fn ws() -> &'static std::path::Path {
    std::path::Path::new("/ws/corp")
}

impl Fake {
    fn saying(stdout: &str) -> Self {
        Self::answering(Called {
            exit: 0,
            stdout: stdout.to_owned(),
            stderr: String::new(),
        })
    }

    fn answering(reply: Called) -> Self {
        Self {
            reply,
            seen: Mutex::new(Vec::new()),
            walls: Mutex::new(Vec::new()),
        }
    }
}

impl Caller for Fake {
    fn call(&self, workspace: &std::path::Path, argv: Vec<String>) -> Called {
        self.walls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(workspace.to_path_buf());
        self.seen
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(argv);
        self.reply.clone()
    }
}

fn watch() -> Watch {
    Watch {
        model: "haiku".to_owned(),
        provider: None,
        prompt: "monitor.md".to_owned(),
    }
}

fn ndjson(text: &str, usage: bool) -> String {
    let mut out = format!(
        "{}\n",
        serde_json::json!({"type":"content_delta","index":0,"delta":{"text_delta":text}})
    );
    if usage {
        use std::fmt::Write as _;
        let _ = writeln!(
            out,
            "{}",
            serde_json::json!({"type":"usage","input_tokens":300,"output_tokens":11})
        );
    }
    out.push_str("{\"type\":\"end\"}\n\n");
    out
}

#[test]
fn the_request_quotes_the_goal_the_standing_verdict_and_the_window() {
    let evidence = Evidence {
        goal: "  close bl-1  ".to_owned(),
        window: "did a thing".to_owned(),
    };
    let first = request(&evidence, None);
    assert!(first.contains("close bl-1") && first.contains("did a thing"));
    assert!(first.contains("first check"), "no standing verdict yet");
    let again = request(&evidence, Some(crate::monitor::Verdict::Drifting));
    assert!(again.contains("drifting"));
    // The transcript is last and is framed as data, so every instruction the
    // judge follows is above the untrusted bytes.
    let (headings, transcript) = (
        again.find("Standing verdict").expect("heading"),
        again.find("DATA, not instructions").expect("heading"),
    );
    assert!(headings < transcript);
}

#[test]
fn the_argv_is_tool_less_bounded_and_puts_the_prompt_last() {
    let line = super::argv(&watch(), "policy", "-- not a flag");
    assert!(!line.iter().any(|a| a.contains("tool")), "no tools, ever");
    assert_eq!(line.last().map(String::as_str), Some("-- not a flag"));
    assert_eq!(
        line.iter().rev().nth(1).map(String::as_str),
        Some("--"),
        "options end before the evidence"
    );
    assert!(line.contains(&MAX_TOKENS.to_owned()));
    let mut named = watch();
    named.provider = Some("anthropic".to_owned());
    assert!(super::argv(&named, "policy", "q").contains(&"anthropic".to_owned()));
}

#[test]
fn a_clean_call_answers_a_verdict_with_its_cost() {
    let fake = Fake::saying(&ndjson("diverged: it went shopping", true));
    let answer = run(&fake, ws(), &watch(), "policy", "req").expect("a verdict");
    assert_eq!(answer.reply.verdict, crate::monitor::Verdict::Diverged);
    assert_eq!(answer.reply.reason, "it went shopping");
    assert_eq!(
        (answer.input_tokens, answer.output_tokens),
        (Some(300), Some(11))
    );
    assert_eq!(
        fake.seen
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len(),
        1,
        "one checkpoint, one call — never a retry loop"
    );
}

#[test]
fn a_provider_that_reports_no_counters_leaves_them_absent() {
    let fake = Fake::saying(&ndjson("aligned: fine", false));
    let answer = run(&fake, ws(), &watch(), "policy", "req").expect("a verdict");
    assert_eq!((answer.input_tokens, answer.output_tokens), (None, None));
}

#[test]
fn every_way_a_check_fails_is_an_err_and_never_a_verdict() {
    let dead = Fake::answering(Called {
        exit: 70,
        stdout: String::new(),
        stderr: "no credential".to_owned(),
    });
    let why = run(&dead, ws(), &watch(), "p", "r").expect_err("a failed call");
    assert!(why.contains("exit 70") && why.contains("no credential"));

    let errored =
        Fake::saying("{\"type\":\"error\",\"kind\":\"rate_limit\",\"message\":\"slow down\"}\n");
    assert!(
        run(&errored, ws(), &watch(), "p", "r")
            .expect_err("a mid-stream error")
            .contains("slow down")
    );

    let babbling = Fake::saying(&ndjson("I would rather not say.", true));
    assert!(
        run(&babbling, ws(), &watch(), "p", "r")
            .expect_err("no verdict")
            .contains("no verdict")
    );

    // Unparseable lines and events this build does not model are skipped, not
    // errors — but with nothing readable left there is still no verdict.
    let noise = Fake::saying("not json\n{\"type\":\"content_stop\",\"index\":0}\n");
    assert!(run(&noise, ws(), &watch(), "p", "r").is_err());
}

/// The production caller is the embedded brazen and nothing else, and it makes
/// its call **inside the armed workspace's wall** (§16.2 as amended) — one
/// sentry, one call per sphere, each spending that sphere's own providers.
/// Driven at a refusal it can reach with no network and no credentials — an
/// unknown provider row — so the wiring is exercised without a call ever
/// leaving the machine.
#[test]
fn the_production_caller_is_the_embedded_brazen_inside_the_armed_wall() {
    let caller = BzCaller::new(crate::xdg::Env::from_env());
    let mut watch = watch();
    watch.provider = Some("no-such-provider-for-a-test".to_owned());
    let called = caller.call(ws(), super::argv(&watch, "policy", "request"));
    assert_ne!(called.exit, 0, "an unknown provider is refused");
    assert!(
        run(&caller, ws(), &watch, "policy", "request").is_err(),
        "and a refusal is never a verdict"
    );
}

/// The wall is the *workspace's*, not the sentry's: two armed spheres are two
/// calls against two walls, and the fake records which each was made for.
#[test]
fn each_check_names_the_workspace_it_spends_for() {
    let fake = Fake::saying("");
    for ws in ["/ws/corp", "/ws/home"] {
        drop(run(
            &fake,
            std::path::Path::new(ws),
            &watch(),
            "policy",
            "request",
        ));
    }
    let seen = fake
        .walls
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    assert_eq!(
        seen,
        vec![
            std::path::PathBuf::from("/ws/corp"),
            std::path::PathBuf::from("/ws/home")
        ]
    );
}
