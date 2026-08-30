//! Round-trip and refusal tables for the §9 config family's envelope (bl-3f46):
//! every destination, every lineage mode and every marks mode re-enters as
//! itself, and every malformed target refuses by name.

use crate::boundary::codec::{decode, encode};
use crate::boundary::config::ConfigFile;
use crate::boundary::{Action, Gesture, Query};
use crate::config_edit::branch::edit::EditOrigin;
use serde_json::json;

fn rt(gesture: Gesture) {
    let encoded = encode(&gesture);
    assert_eq!(decode(&encoded), Ok(gesture.clone()), "via {encoded}");
}

fn applying(file: ConfigFile) -> Gesture {
    Gesture::Act(Action::ApplyConfig {
        file,
        // Newlines and indentation ride through untouched — a config file's
        // whitespace is the file.
        text: "models:\n  gpt-5.4:\n    provider: codex\n".to_owned(),
    })
}

/// The brazen destination and the provider table, each naming their sphere
/// (bl-fcd5) — a wall-scoped gesture has no spelling without one.
fn brazen() -> ConfigFile {
    ConfigFile::Brazen {
        workspace: "ws".to_owned(),
    }
}

fn providers() -> Query {
    Query::Providers {
        workspace: "ws".to_owned(),
    }
}

fn branch(origin: EditOrigin) -> ConfigFile {
    ConfigFile::Branch {
        workspace: "ws".to_owned(),
        lineage: "default".to_owned(),
        origin,
        path: "providers.yaml".to_owned(),
    }
}

#[test]
fn every_config_destination_round_trips() {
    for file in [
        brazen(),
        ConfigFile::LitanyModels,
        ConfigFile::Cadence,
        ConfigFile::LitanyWorkflow {
            name: "review".to_owned(),
        },
        branch(EditOrigin::Advance),
        branch(EditOrigin::Fork {
            source: "base".to_owned(),
        }),
        branch(EditOrigin::Orphan),
    ] {
        rt(applying(file));
    }
}

#[test]
fn a_marks_amendment_round_trips() {
    for branch in ["balls/tasks", "balls/agents/corp"] {
        rt(Gesture::Act(Action::SetMarks {
            workspace: "ws".to_owned(),
            branch: branch.to_owned(),
        }));
    }
}

#[test]
fn a_pick_round_trips_with_its_whole_triple() {
    rt(Gesture::Act(Action::PickModel {
        workspace: "ws".to_owned(),
        role: "worker".to_owned(),
        provider: "codex".to_owned(),
        model: "gpt-5.4".to_owned(),
    }));
}

#[test]
fn the_config_envelope_names_its_target_and_carries_the_text_whole() {
    let text = "[providers.codex]\nauth = \"none\"\n";
    let encoded = encode(&Gesture::Act(Action::ApplyConfig {
        file: brazen(),
        text: text.to_owned(),
    }));
    assert_eq!(encoded["op"], "config");
    assert_eq!(encoded["target"]["file"], "brazen");
    assert_eq!(encoded["text"], text);
}

/// bl-fcd5 — the envelope carries the sphere, and strictly: a wall-scoped op
/// with no `workspace` is refused by name rather than decoded into a gesture
/// the executor would have to guess a wall for.
#[test]
fn a_wall_scoped_envelope_without_its_workspace_is_refused() {
    for value in [
        json!({ "op": "providers" }),
        json!({ "op": "config", "target": { "file": "brazen" } }),
        json!({ "op": "config", "target": { "file": "brazen" }, "text": "x" }),
    ] {
        let err = decode(&value).unwrap_err();
        assert!(err.contains("workspace"), "{value}: {err}");
    }
}

/// The config family's reads (§8.5, bl-0164): the same op as their write,
/// minus the field that makes it one — `text` for a config destination,
/// `mode` for the knob.
#[test]
fn a_field_left_out_reads_instead_of_writing() {
    for file in [
        brazen(),
        ConfigFile::LitanyModels,
        ConfigFile::Cadence,
        ConfigFile::LitanyWorkflow {
            name: "review".to_owned(),
        },
    ] {
        rt(Gesture::Ask(Query::ReadConfig { file }));
    }
    rt(Gesture::Ask(Query::Marks {
        workspace: "ws".to_owned(),
    }));
    rt(Gesture::Ask(providers()));

    let read = encode(&Gesture::Ask(Query::ReadConfig { file: brazen() }));
    assert_eq!(read["op"], "config");
    assert!(read.get("text").is_none(), "{read}");

    let marks = encode(&Gesture::Ask(Query::Marks {
        workspace: "ws".to_owned(),
    }));
    assert_eq!(marks["op"], "marks");
    assert!(marks.get("branch").is_none(), "{marks}");
}

#[test]
fn a_marks_envelope_names_the_workspace_and_the_branch() {
    let encoded = encode(&Gesture::Act(Action::SetMarks {
        workspace: "ws".to_owned(),
        branch: "balls/agents/corp".to_owned(),
    }));
    assert_eq!(encoded["op"], "marks");
    assert_eq!(encoded["workspace"], "ws");
    assert_eq!(encoded["branch"], "balls/agents/corp");
}

#[test]
fn malformed_config_envelopes_refuse_with_a_reason() {
    let cases = [
        (json!({"op": "config", "text": "x"}), "missing target"),
        (
            json!({"op": "config", "target": "brazen", "text": "x"}),
            "not an object",
        ),
        (
            json!({"op": "config", "target": {"file": "enhance"}, "text": "x"}),
            "unknown target file",
        ),
        (
            json!({"op": "config", "target": {"file": "litany-workflow"}, "text": "x"}),
            "\"name\"",
        ),
        (
            json!({"op": "config", "target": {"file": "branch", "workspace": "/ws",
                   "lineage": "d", "path": "p", "origin": "rebase"}, "text": "x"}),
            "unknown origin",
        ),
        (
            json!({"op": "config", "target": {"file": "branch", "workspace": "/ws",
                   "lineage": "d", "path": "p", "origin": "fork"}, "text": "x"}),
            "\"source\"",
        ),
        // The space's own lawfulness rule refuses these, not a second check
        // here: a branch with whitespace, and balls' own landing branch.
        (
            json!({"op": "marks", "workspace": "/ws", "branch": "two words"}),
            "one word",
        ),
        (
            json!({"op": "marks", "workspace": "/ws", "branch": "balls/config"}),
            "landing branch",
        ),
        (
            json!({"op": "model", "workspace": "/ws", "role": "worker", "provider": "codex"}),
            "\"model\"",
        ),
    ];
    for (envelope, needle) in cases {
        let err = decode(&envelope).unwrap_err();
        assert!(err.contains(needle), "{envelope} refused with {err:?}");
    }
}
