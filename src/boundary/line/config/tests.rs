//! The §9 config family's line parity and refusals (bl-3f46): every
//! destination spells and reads back as itself at the seat it was spelled
//! from, the file's text survives the trip whitespace and all, and every way
//! the grammar can be under-said names what is missing.

use crate::boundary::config::ConfigFile;
use crate::boundary::line::tests::ctx;
use crate::boundary::line::{Context, parse, spell};
use crate::boundary::{Action, Gesture, Query, help};
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
        ConfigFile::LernieModels,
        ConfigFile::Cadence,
        ConfigFile::LernieWorkflow {
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

/// bl-fcd5 — the wall-scoped gestures state their sphere or they are refused,
/// at the edge and by name. Read and write alike, and `/providers` with them:
/// each reaches inside one workspace's wall, and a seat with no focus has
/// nothing for the executor to fall back on, so guessing is the one thing it
/// must not do.
#[test]
fn a_wall_scoped_gesture_with_no_workspace_refuses_by_name() {
    for line in [
        "/config brazen",
        "/config brazen [[provider]]",
        "/providers",
    ] {
        let err = parse(line, &Context::default()).unwrap_err();
        assert!(err.contains("no workspace in context"), "{line:?}: {err}");
    }
    // Named, they resolve — the same lines the window's focus supplies.
    assert_eq!(
        parse("/providers", &ctx()),
        Ok(Gesture::Ask(Query::Providers {
            workspace: "ws".to_owned(),
        }))
    );
}

#[test]
fn an_under_said_config_line_names_what_is_missing() {
    let cases = [
        ("/config", "unknown destination \"\""),
        ("/config enhance x", "unknown destination \"enhance\""),
        ("/config workflow", "a workflow name is required"),
        ("/config branch", "the lineage name is required"),
        ("/config branch strict", "the file's path"),
        ("/config orphan strict", "the file's path"),
        (
            "/config fork strict",
            "the lineage to fork from is required",
        ),
        ("/config fork strict base", "the file's path"),
    ];
    for (line, needle) in cases {
        let err = parse(line, &ctx()).unwrap_err();
        assert!(err.contains(needle), "{line:?} refused with {err:?}");
    }
}

/// A lineage destination reads like every other one (bl-dff8): the destination's
/// words with nothing after them are `git show config/<lineage>:<path>`, the
/// pane's Load. All three origins spell a read, because the origin says where a
/// *write* would land and a read lands nowhere.
#[test]
fn a_lineage_with_no_text_reads_the_file_at_its_tip() {
    let cases = [
        ("/config branch strict workflow.yaml", EditOrigin::Advance),
        ("/config orphan strict workflow.yaml", EditOrigin::Orphan),
        (
            "/config fork strict base workflow.yaml",
            EditOrigin::Fork {
                source: "base".to_owned(),
            },
        ),
    ];
    for (line, origin) in cases {
        assert_eq!(
            parse(line, &ctx()),
            Ok(Gesture::Ask(Query::ReadConfig {
                file: ConfigFile::Branch {
                    workspace: "ws".to_owned(),
                    lineage: "strict".to_owned(),
                    origin,
                    path: "workflow.yaml".to_owned(),
                }
            })),
            "{line:?}"
        );
    }
}

/// The destinations that support the §8.5 (bl-0164) read shortcut: a line
/// with nothing after the destination's words reads it instead of refusing —
/// the same rule as `/config`'s own doc: "nothing after them is the read".
#[test]
fn a_destination_with_no_text_reads_instead_of_refusing() {
    let cases = [
        ("/config brazen", brazen()),
        ("/config models", ConfigFile::LernieModels),
        ("/config cadence", ConfigFile::Cadence),
        (
            "/config workflow review",
            ConfigFile::LernieWorkflow {
                name: "review".to_owned(),
            },
        ),
    ];
    for (line, file) in cases {
        assert_eq!(
            parse(line, &ctx()),
            Ok(Gesture::Ask(Query::ReadConfig { file })),
            "{line:?}"
        );
    }
}

#[test]
fn an_under_said_marks_or_model_line_names_what_is_missing() {
    let cases = [
        ("/marks balls/x extra", "takes no arguments"),
        ("/marks balls/config", "landing branch"),
        ("/model worker codex", "usage: /model"),
        ("/model a b c d", "usage: /model"),
    ];
    for (line, needle) in cases {
        let err = parse(line, &ctx()).unwrap_err();
        assert!(err.contains(needle), "{line:?} refused with {err:?}");
    }
    // The knob's subject is the agent, so a seat naming no workspace cannot
    // aim it — read or write alike.
    let err = parse("/marks balls/mine", &Context::default()).unwrap_err();
    assert!(err.contains("no workspace in context"), "{err}");
    let err = parse("/marks", &Context::default()).unwrap_err();
    assert!(err.contains("no workspace in context"), "{err}");
    let err = parse("/model worker codex gpt-5.4", &Context::default()).unwrap_err();
    assert!(err.contains("no workspace in context"), "{err}");
}

/// `/marks` bare reads the branch (§8.5, bl-0164) — a branch is always
/// required to write, so the empty tail cannot mean anything else.
#[test]
fn a_bare_marks_line_reads_instead_of_refusing() {
    assert_eq!(
        parse("/marks", &ctx()),
        Ok(Gesture::Ask(Query::Marks {
            workspace: "ws".to_owned()
        }))
    );
}

#[test]
fn an_unlawful_branch_refuses_at_the_line_in_the_spaces_own_words() {
    let err = parse("/marks balls/config", &ctx()).unwrap_err();
    assert!(err.contains(crate::world::marks::REFUSAL), "{err}");
    // A second word is not part of a branch name — a branch is one word, so
    // the tail is refused rather than folded into the value.
    assert!(parse("/marks balls/x and more", &ctx()).is_err());
}

#[test]
fn the_config_usage_line_is_what_a_bad_destination_prints() {
    let err = parse("/config enhance x", &ctx()).unwrap_err();
    assert!(err.contains(super::USAGE), "{err}");
    // …and it is the page `/help config` renders, not a second wording.
    assert_eq!(
        help::rows(Some("config")).first().map(|r| r.usage),
        Some(super::USAGE)
    );
}
