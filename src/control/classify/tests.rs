//! The effect vocabulary and the built-in intrinsic map.

use super::*;
use serde_json::json;

fn root() -> Root {
    Root {
        writable: vec![PathBuf::from("/w/agent"), PathBuf::from("/state/bl-1a2b")],
        cwd: PathBuf::from("/w/agent"),
        home: PathBuf::from("/home/op"),
    }
}

use std::path::PathBuf;

/// A request for `name` with `input`, over fixed identity fields.
fn req(name: &str, input: serde_json::Value) -> Request {
    Request {
        id: "toolu_1".to_owned(),
        name: name.to_owned(),
        input,
        role: "worker".to_owned(),
        agent_id: "amber".to_owned(),
    }
}

fn effect(name: &str, input: serde_json::Value) -> Effect {
    classify(
        &req(name, input),
        &root(),
        &crate::control::policy::Policy::default(),
    )
    .effect
}

#[test]
fn the_vocabulary_orders_by_reach_and_names_itself() {
    assert!(Effect::Read < Effect::TargetWrite);
    assert!(Effect::TargetWrite < Effect::Process);
    assert!(Effect::Process < Effect::OpenWorld);
    assert!(Effect::OpenWorld < Effect::Destructive);
    assert!(Effect::Destructive < Effect::Secret);
    assert_eq!(Effect::Read.worst(Effect::Secret), Effect::Secret);
    assert_eq!(Effect::Secret.worst(Effect::Read), Effect::Secret);
    for (effect, word) in [
        (Effect::Read, "read"),
        (Effect::TargetWrite, "target write"),
        (Effect::Process, "process"),
        (Effect::OpenWorld, "open-world"),
        (Effect::Destructive, "destructive"),
        (Effect::Secret, "secret"),
    ] {
        assert_eq!(effect.word(), word);
    }
}

#[test]
fn every_built_in_carries_its_intrinsic_class() {
    assert_eq!(effect(READ_FILE, json!({"path": "x"})), Effect::Read);
    assert_eq!(
        effect(LOAD_SKILL, json!({"name": "s"})),
        Effect::TargetWrite
    );
    // The world's own substrates, through their gated verbs, are target writes
    // — not an exemption, the second half of the definition.
    assert_eq!(
        effect(MESSAGE, json!({"agent": "a", "content": "c"})),
        Effect::TargetWrite
    );
    assert_eq!(effect(DISPATCH, json!({"role": "worker"})), Effect::Process);
}

#[test]
fn an_unknown_tool_is_open_world_so_error_never_falls_toward_a_read() {
    let c = classify(
        &req("litany-tool-deploy", json!({})),
        &root(),
        &crate::control::policy::Policy::default(),
    );
    assert_eq!(c.effect, Effect::OpenWorld);
    assert!(c.why.contains("litany-tool-deploy"), "{}", c.why);
}

#[test]
fn a_cd_is_a_read_inside_the_root_and_open_world_out_of_it() {
    assert_eq!(effect(CD, json!({"path": "src"})), Effect::Read);
    assert_eq!(effect(CD, json!({"path": "/state/bl-1a2b"})), Effect::Read);
    assert_eq!(effect(CD, json!({"path": "/tmp"})), Effect::OpenWorld);
    // An off-schema input names no destination, which resolves to the cwd —
    // inside the root, and the general path rather than a branch.
    assert_eq!(effect(CD, json!({})), Effect::Read);
}

#[test]
fn a_patch_is_judged_by_every_path_its_envelope_names() {
    let inside = "*** Begin Patch\n*** Update File: src/a.rs\n*** Add File: /w/agent/b\n";
    assert_eq!(
        effect(APPLY_PATCH, json!({ "input": inside })),
        Effect::TargetWrite
    );
    let out = "*** Begin Patch\n*** Update File: src/a.rs\n*** Delete File: /etc/hosts\n";
    let c = classify(
        &req(APPLY_PATCH, json!({ "input": out })),
        &root(),
        &crate::control::policy::Policy::default(),
    );
    assert_eq!(c.effect, Effect::OpenWorld);
    assert!(c.why.contains("/etc/hosts"), "{}", c.why);
    // A `Move to:` destination counts, and an envelope naming nothing patches
    // nothing.
    let moved = "*** Update File: a\n*** Move to: /var/b\n";
    assert_eq!(
        effect(APPLY_PATCH, json!({ "input": moved })),
        Effect::OpenWorld
    );
    assert_eq!(
        effect(APPLY_PATCH, json!({ "input": "" })),
        Effect::TargetWrite
    );
    // A marker with an empty operand names no path.
    assert_eq!(
        effect(APPLY_PATCH, json!({ "input": "*** Add File:   \n" })),
        Effect::TargetWrite
    );
}

#[test]
fn bash_routes_to_the_ruleset() {
    assert_eq!(effect(BASH, json!({"command": "ls -la"})), Effect::Read);
    assert_eq!(
        effect(BASH, json!({"command": "curl x"})),
        Effect::OpenWorld
    );
}

/// `outside` answers even when nothing is outside — the reason line is built
/// from it and must always read as a sentence.
#[test]
fn the_offending_operand_is_named_or_empty() {
    assert_eq!(outside(&[], &root()), "");
}
