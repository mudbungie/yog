//! The consult end to end: the wire contract, the writable root a real
//! workspace resolves to, and the verdict a `world/tools/` shim would print.

use super::*;
use crate::opslog::YOG_CONTROL;
use serde_json::json;

mod world;
use world::{Closed, World, request};

#[test]
fn the_request_parse_requires_every_field_and_ignores_the_rest() {
    let r = Request::parse(&request("bash", json!({"command": "ls"}))).unwrap();
    assert_eq!(r.id, "toolu_01");
    assert_eq!(r.name, "bash");
    assert_eq!(r.role, "worker");
    assert_eq!(r.agent_id, "amber");
    assert_eq!(r.field("command"), "ls");
    // Absent or non-string fields of `input` read as empty rather than failing:
    // an off-schema input is an invocation with no operands, not a panic.
    assert_eq!(r.field("path"), "");
    // A field litany adds later must not brick every tool call.
    let extra = json!({"id":"i","name":"n","input":{},"role":"r","agent_id":"a","sandbox":true});
    assert!(Request::parse(&extra.to_string()).is_some());
    // Each required field, missing in turn.
    for key in ["id", "name", "input", "role", "agent_id"] {
        let mut value = json!({"id":"i","name":"n","input":{},"role":"r","agent_id":"a"});
        value.as_object_mut().unwrap().remove(key);
        assert!(Request::parse(&value.to_string()).is_none(), "{key}");
    }
    assert!(Request::parse("not json").is_none());
}

#[test]
fn a_pass_carries_no_reason_and_the_other_two_require_one() {
    // litany's own parser rejects `{"verdict":"pass","reason":…}`.
    assert_eq!(Verdict::Pass.json(), r#"{"verdict":"pass"}"#);
    assert_eq!(
        Verdict::Hold("why".to_owned()).json(),
        r#"{"reason":"why","verdict":"hold"}"#
    );
    assert_eq!(
        Verdict::Refuse("why".to_owned()).json(),
        r#"{"reason":"why","verdict":"refuse"}"#
    );
}

#[test]
fn the_writable_root_is_the_agent_worktree_plus_the_claim_this_workspace_made() {
    let w = World::new();
    w.claim("/dev/proj", "bl-1a2b", "cobalt-gecko");
    let entries = opslog::tail(&w.state(), usize::MAX);
    let root = w.consult().root("amber", &entries);
    assert_eq!(root.cwd, w.workspace().join("agents").join("amber"));
    assert!(root.holds(&w.workspace().join("agents/amber/src/x")));
    assert!(
        root.holds(
            &w.balls()
                .join("plugins/bl-delivery/dev/proj/bl-1a2b/README.md")
        )
    );
    assert!(!root.holds(Path::new("/etc/hosts")));
    // Another workspace's claim is not this workspace's root.
    w.claim("/dev/proj", "bl-9999", "other-name");
    let entries = opslog::tail(&w.state(), usize::MAX);
    let root = w.consult().root("amber", &entries);
    assert!(!root.holds(&w.balls().join("plugins/bl-delivery/dev/proj/bl-9999/x")));
}

#[test]
fn the_default_table_decides_an_unanswered_invocation() {
    let w = World::new();
    let c = w.consult();
    let verdict = |name: &str, input: serde_json::Value| {
        adjudicate(&c, &Request::parse(&request(name, input)).unwrap())
    };
    assert_eq!(
        verdict("bash", json!({"command": "cargo test"})),
        Verdict::Pass
    );
    assert_eq!(verdict("read_file", json!({"path": "x"})), Verdict::Pass);
    assert_eq!(
        verdict("dispatch", json!({"role": "worker", "goal": "g"})),
        Verdict::Pass
    );
    // Leaving the world is the job too: the shipped table parks nothing.
    assert_eq!(
        verdict("bash", json!({"command": "curl https://x"})),
        Verdict::Pass
    );
    let Verdict::Refuse(reason) = verdict("bash", json!({"command": "rm -rf /etc"})) else {
        panic!("loss is declined in band");
    };
    assert!(reason.contains("destructive"), "{reason}");
    assert!(reason.contains("bash"), "{reason}");
}

#[test]
fn the_operator_s_answers_fold_over_the_table() {
    let w = World::new();
    // A once-answer to this exact tool_use id releases what the table declined …
    w.answer(&[YOG_CONTROL, "answer", "toolu_01", "pass"]);
    assert_eq!(
        adjudicate(
            &w.consult(),
            &Request::parse(&request("bash", json!({"command": "rm -rf /etc"}))).unwrap()
        ),
        Verdict::Pass
    );
    // … and a floor over the conversation holds everything above read.
    w.answer(&[YOG_CONTROL, "floor", "amber", "raise"]);
    let held = adjudicate(
        &w.consult(),
        &Request::parse(
            &json!({"id":"toolu_02","name":"bash","input":{"command":"cargo test"},
                    "role":"worker","agent_id":"amber"})
            .to_string(),
        )
        .unwrap(),
    );
    assert!(matches!(held, Verdict::Hold(_)), "{held:?}");
}

#[test]
fn a_reason_never_hands_the_reader_a_section_number() {
    let w = World::new();
    let verdict = adjudicate(
        &w.consult(),
        &Request::parse(&request("bash", json!({"command": "rm -rf /etc"}))).unwrap(),
    );
    let Verdict::Refuse(reason) = verdict else {
        panic!("loss is declined in band");
    };
    assert!(!reason.contains('§'), "{reason}");
}

/// The process body — the `world/tools/` shim's own stdin/stdout contract —
/// and the workspace's own standing policy, split from the pure consult at
/// §12's cap along the seam the ruling already draws: this file is *what the
/// control decides*, those are *how it is run* and *what one workspace tells
/// it to be*.
mod policy;
mod shim;

#[test]
fn the_workspace_is_litany_s_own_env_var_else_the_cwd_it_runs_in() {
    let env = crate::xdg::Env::from_pairs([("LITANY_CONV_REPO", "/w/ws")]);
    assert_eq!(workspace_of(&env), PathBuf::from("/w/ws"));
    let env = crate::xdg::Env::from_pairs([("LITANY_CONV_REPO", "")]);
    assert_eq!(workspace_of(&env), std::env::current_dir().unwrap());
}
