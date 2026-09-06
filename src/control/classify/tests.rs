//! The effect vocabulary, the intrinsic map, and the fail-closed lane every
//! name outside it takes.

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

fn judged(name: &str, input: serde_json::Value) -> Classified {
    classify(
        &req(name, input),
        &root(),
        &crate::control::policy::Policy::default(),
    )
}

fn effect(name: &str, input: serde_json::Value) -> Effect {
    judged(name, input).effect
}

#[test]
fn the_vocabulary_orders_by_reach_and_names_itself() {
    assert!(Effect::Read < Effect::TargetWrite);
    assert!(Effect::TargetWrite < Effect::Process);
    assert!(Effect::Process < Effect::OpenWorld);
    assert!(Effect::OpenWorld < Effect::Destructive);
    assert!(Effect::Destructive < Effect::Secret);
    // The unknown is widest of all, so a fold can never bury it (bl-72bd).
    assert!(Effect::Secret < Effect::Opaque);
    assert_eq!(Effect::Read.worst(Effect::Secret), Effect::Secret);
    assert_eq!(Effect::Secret.worst(Effect::Read), Effect::Secret);
    assert_eq!(Effect::Destructive.worst(Effect::Opaque), Effect::Opaque);
    for (effect, word) in [
        (Effect::Read, "read"),
        (Effect::TargetWrite, "target write"),
        (Effect::Process, "process"),
        (Effect::OpenWorld, "open-world"),
        (Effect::Destructive, "destructive"),
        (Effect::Secret, "secret"),
        (Effect::Opaque, "opaque"),
    ] {
        assert_eq!(effect.word(), word);
        assert_eq!(Effect::of(&word.replace(' ', "-")), Some(effect));
    }
    assert_eq!(Effect::of("unheard-of"), None);
}

#[test]
fn every_built_in_carries_its_intrinsic_class() {
    assert_eq!(effect("read_file", json!({"path": "x"})), Effect::Read);
    assert_eq!(
        effect("search_history", json!({"query": "x"})),
        Effect::Read
    );
    assert_eq!(
        effect("load_skill", json!({"name": "s"})),
        Effect::TargetWrite
    );
    // The world's own substrates, through their gated verbs, are target writes
    // — not an exemption, the second half of the definition.
    assert_eq!(
        effect("message", json!({"agent": "a", "content": "c"})),
        Effect::TargetWrite
    );
    assert_eq!(
        effect("dispatch", json!({"role": "worker"})),
        Effect::Process
    );
}

/// The compactor's procedure pair and yog's own roster tool carry rows of
/// their own since bl-72bd. Before it they fell off the match into open-world
/// — which passed, so they worked; the point of the rows is that nothing now
/// reaches a class by falling.
#[test]
fn the_injected_names_carry_rows_rather_than_falling_off_the_match() {
    assert_eq!(
        effect("write_summary", json!({"body": "b"})),
        Effect::TargetWrite
    );
    assert_eq!(
        effect("mark_for_deletion", json!({"paths": ["a"]})),
        Effect::TargetWrite
    );
    assert_eq!(effect("clients", json!({"op": "list"})), Effect::Read);
    assert_eq!(effect("clients", json!({})), Effect::Read);
    assert_eq!(
        effect("clients", json!({"op": "load", "client": "c"})),
        Effect::TargetWrite
    );
    // `python` runs a program the model authored; open-world by its own row,
    // which is what the shipped table passes exactly as it did before.
    assert_eq!(
        effect("python", json!({"program": "print(1)"})),
        Effect::OpenWorld
    );
}

/// Ruling 2 of the round-1 triage, first half: a routed tool whose input
/// carries a command line is classified by that line exactly as the engine's
/// own `bash` is. The drive that filed bl-72bd deleted 180 MB through the
/// first of these and was not asked a question.
#[test]
fn a_routed_shell_is_classified_by_its_command_line() {
    let destructive = judged(
        "box2_shell",
        json!({"command": "find /srv/data/blobs -mindepth 1 -delete"}),
    );
    assert_eq!(destructive.effect, Effect::Destructive);
    assert!(
        destructive.why.contains("/srv/data/blobs"),
        "{}",
        destructive.why
    );
    assert_eq!(
        effect("box2_shell", json!({"command": "env"})),
        Effect::Secret
    );
    // …and the same table, so a read on the foot is still a read.
    assert_eq!(
        effect("box2_shell", json!({"command": "ls -la"})),
        Effect::Read
    );
    // The engine's own bash answers identically — that is what "exactly as"
    // means, and the two are one call into one ruleset.
    assert_eq!(
        judged(
            "bash",
            json!({"command": "find /srv/data/blobs -mindepth 1 -delete"})
        ),
        destructive
    );
}

/// Ruling 2, second half: a tool this control cannot read is HELD, never
/// passed. The class it lands in is the one the shipped table parks.
#[test]
fn a_tool_this_control_cannot_read_is_opaque_and_never_a_passing_class() {
    for (name, input) in [
        ("litany-tool-deploy", json!({})),
        ("box2_install_package", json!({"name": "curl"})),
        ("box2_rotate_log", json!({"path": "/var/log/x"})),
        // A `command` key that is not a string, and an empty one: neither is a
        // command line, so neither may borrow the shell's classification.
        ("box2_shell", json!({"command": 7})),
        ("box2_shell", json!({"command": "   "})),
    ] {
        let c = judged(name, input);
        assert_eq!(c.effect, Effect::Opaque, "{name}");
        assert!(c.why.contains(name), "{}", c.why);
        assert_eq!(
            crate::control::judge::Table::ruling(c.effect),
            crate::control::judge::Ruling::Hold,
            "{name}"
        );
    }
}

#[test]
fn a_cd_is_a_read_inside_the_root_and_open_world_out_of_it() {
    assert_eq!(effect("cd", json!({"path": "src"})), Effect::Read);
    assert_eq!(
        effect("cd", json!({"path": "/state/bl-1a2b"})),
        Effect::Read
    );
    assert_eq!(effect("cd", json!({"path": "/tmp"})), Effect::OpenWorld);
    // An off-schema input names no destination, which resolves to the cwd —
    // inside the root, and the general path rather than a branch.
    assert_eq!(effect("cd", json!({})), Effect::Read);
}

#[test]
fn a_patch_is_judged_by_every_path_its_envelope_names() {
    let inside = "*** Begin Patch\n*** Update File: src/a.rs\n*** Add File: /w/agent/b\n";
    assert_eq!(
        effect("apply_patch", json!({ "input": inside })),
        Effect::TargetWrite
    );
    let out = "*** Begin Patch\n*** Update File: src/a.rs\n*** Delete File: /etc/hosts\n";
    let c = judged("apply_patch", json!({ "input": out }));
    assert_eq!(c.effect, Effect::OpenWorld);
    assert!(c.why.contains("/etc/hosts"), "{}", c.why);
    // A `Move to:` destination counts, and an envelope naming nothing patches
    // nothing.
    let moved = "*** Update File: a\n*** Move to: /var/b\n";
    assert_eq!(
        effect("apply_patch", json!({ "input": moved })),
        Effect::OpenWorld
    );
    assert_eq!(
        effect("apply_patch", json!({ "input": "" })),
        Effect::TargetWrite
    );
    // A marker with an empty operand names no path.
    assert_eq!(
        effect("apply_patch", json!({ "input": "*** Add File:   \n" })),
        Effect::TargetWrite
    );
}

#[test]
fn bash_routes_to_the_ruleset() {
    assert_eq!(effect("bash", json!({"command": "ls -la"})), Effect::Read);
    assert_eq!(
        effect("bash", json!({"command": "curl x"})),
        Effect::OpenWorld
    );
}

/// `outside` answers even when nothing is outside — the reason line is built
/// from it and must always read as a sentence.
#[test]
fn the_offending_operand_is_named_or_empty() {
    assert_eq!(operand::outside(&[], &root()), "");
}

/// The name fold is the closed set, and nothing else is in it.
#[test]
fn the_known_set_is_the_names_it_names() {
    assert_eq!(intrinsic::Known::of("bash"), Some(intrinsic::Known::Bash));
    assert_eq!(intrinsic::Known::of("box2_bash"), None);
    assert_eq!(intrinsic::Known::of(""), None);
}
