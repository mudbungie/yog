//! The §9 config family's line parity (bl-3f46): every destination spells and
//! reads back as itself at the seat it was spelled from, and the file's text
//! survives the trip whitespace and all. Every way the grammar can be
//! under-said is [`under_said`], split off at §12's cap.

/// What an under-said config line means: refuse by name, or read.
mod under_said;

use crate::boundary::config::ConfigFile;
use crate::boundary::line::tests::ctx;
use crate::boundary::line::{Context, parse, spell};
use crate::boundary::{Action, Gesture, help};
use crate::config_edit::branch::edit::EditOrigin;

/// The parity claim (§8.5), and the other direction of the single source: the
/// verb the line names has a help page.
fn rt(gesture: Gesture) {
    let line = spell(&gesture);
    assert_eq!(parse(&line, &ctx()), Ok(gesture.clone()), "via {line}");
    let verb = line
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_start_matches('/');
    assert!(help::known(verb), "/{verb} has no help page");
}

fn applying(file: ConfigFile) -> Gesture {
    Gesture::Act(Action::ApplyConfig {
        file,
        text: "roles:\n  worker:\n    provider: codex".to_owned(),
    })
}

/// The brazen destination, naming the seat's own workspace — what `--ws` (or
/// the window's focus) supplies, and what [`ctx`] carries (bl-fcd5).
fn brazen() -> ConfigFile {
    ConfigFile::Brazen {
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
fn every_config_destination_round_trips_as_a_line() {
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
fn a_marks_amendment_and_a_pick_round_trip_as_lines() {
    for branch in ["balls/tasks", "balls/agents/corp"] {
        rt(Gesture::Act(Action::SetMarks {
            workspace: "ws".to_owned(),
            branch: branch.to_owned(),
        }));
    }
    rt(Gesture::Act(Action::PickModel {
        workspace: "ws".to_owned(),
        role: "worker".to_owned(),
        provider: "codex".to_owned(),
        model: "gpt-5.4".to_owned(),
    }));
}

#[test]
fn the_text_is_the_whole_tail_and_no_flag_is_read_out_of_it() {
    // The destination is words; everything after them is the file, including
    // what would be a flag anywhere else on the boundary.
    let read = parse("/config cadence a: 1\n--body: not a flag", &ctx());
    assert_eq!(
        read,
        Ok(Gesture::Act(Action::ApplyConfig {
            file: ConfigFile::Cadence,
            text: "a: 1\n--body: not a flag".to_owned(),
        }))
    );
}

#[test]
fn a_lineage_destination_takes_its_workspace_from_the_seat() {
    let read = parse("/config branch strict workflow.yaml events: {}", &ctx());
    assert_eq!(
        read,
        Ok(Gesture::Act(Action::ApplyConfig {
            file: ConfigFile::Branch {
                workspace: "ws".to_owned(),
                lineage: "strict".to_owned(),
                origin: EditOrigin::Advance,
                path: "workflow.yaml".to_owned(),
            },
            text: "events: {}".to_owned(),
        }))
    );
    // …and a seat with no workspace refuses by naming it rather than guessing.
    let err = parse("/config branch strict workflow.yaml x", &Context::default()).unwrap_err();
    assert!(err.contains("no workspace in context"), "{err}");
}
