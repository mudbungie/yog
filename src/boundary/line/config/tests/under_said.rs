//! Every way a §9 config line can be under-said, and what each one means. The
//! rule is not "refuse" — it is that a line stating no *destination* refuses by
//! name, while a destination stating no *text* is a READ. Its own file at
//! §12's cap, on the seam the module above states: parity there, grammar here.

use super::brazen;
use crate::boundary::config::ConfigFile;
use crate::boundary::line::tests::ctx;
use crate::boundary::line::{Context, parse};
use crate::boundary::{Gesture, Query, help};
use crate::config_edit::branch::edit::EditOrigin;

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
        ("/config models", ConfigFile::LitanyModels),
        ("/config cadence", ConfigFile::Cadence),
        (
            "/config workflow review",
            ConfigFile::LitanyWorkflow {
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
    assert!(err.contains(super::super::USAGE), "{err}");
    // …and it is the page `/help config` renders, not a second wording.
    assert_eq!(
        help::rows(Some("config")).first().map(|r| r.usage),
        Some(super::super::USAGE)
    );
}
