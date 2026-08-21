//! The ball-editing verbs' argv tests (`bl create` / `bl update`) — split from
//! `mod` per §12's 300-line budget; the shared World recorder fixture and
//! `args_of` live there. Each verb is stamped `--as <name>`.

use super::{OK_BODY, World, args_of};
use crate::actions::verbs::edit::{Create, Field, Update};
use crate::actions::verbs::{create, update};
use crate::opslog;

#[test]
fn create_captures_the_printed_id_and_optional_body() {
    let w = World::new("bl", "#!/bin/sh\nprintf 'bl-9zzz\\n'\nexit 0\n");
    // No body.
    let out = create(
        &w.cli,
        w.state.path(),
        "TS",
        &w.cwd,
        "filtered",
        &Create {
            title: "wire it".into(),
            ..Create::default()
        },
    )
    .unwrap();
    assert_eq!(
        out.stdout.trim(),
        "bl-9zzz",
        "the new id is captured on stdout"
    );
    assert_eq!(
        args_of(&w.logged()),
        vec!["create", "wire it", "--as", "filtered"]
    );
    // With body.
    create(
        &w.cli,
        w.state.path(),
        "TS",
        &w.cwd,
        "filtered",
        &Create {
            title: "wire it".into(),
            body: Some("the plan".into()),
            ..Create::default()
        },
    )
    .unwrap();
    let e = opslog::tail(w.state.path(), 8).pop().unwrap();
    assert_eq!(
        args_of(&e),
        vec![
            "create", "wire it", "--as", "filtered", "--body", "the plan"
        ]
    );
}

#[test]
fn update_carries_only_the_changed_fields() {
    let w = World::new("bl", OK_BODY);
    // All fields.
    update(
        &w.cli,
        w.state.path(),
        "TS",
        &w.cwd,
        "bl-3",
        "filtered",
        &Update {
            title: Some("T".into()),
            body: Some("B".into()),
            note: Some("N".into()),
            ..Update::default()
        },
    )
    .unwrap();
    assert_eq!(
        args_of(&w.logged()),
        vec![
            "update", "bl-3", "--as", "filtered", "--title", "T", "--body", "B", "-m", "N"
        ]
    );
    // A bare note-only update (the default's other fields stay None).
    update(
        &w.cli,
        w.state.path(),
        "TS",
        &w.cwd,
        "bl-3",
        "filtered",
        &Update {
            note: Some("progress".into()),
            ..Update::default()
        },
    )
    .unwrap();
    let e = opslog::tail(w.state.path(), 8).pop().unwrap();
    assert_eq!(
        args_of(&e),
        vec!["update", "bl-3", "--as", "filtered", "-m", "progress"]
    );
}

/// The four scheduling facts fold to balls' own flags, **in the order they
/// were said** — and a clearing form is silently empty at create, where a new
/// ball's fields start empty and there is no `--no-…` flag to spend.
#[test]
fn the_scheduling_facts_fold_to_balls_own_flags() {
    let all = || {
        vec![
            Field::Priority(Some(3)),
            Field::Priority(None),
            Field::Tag {
                tag: "boundary".into(),
                on: true,
            },
            Field::Tag {
                tag: "stale".into(),
                on: false,
            },
            Field::Parent(Some("bl-1a2b".into())),
            Field::Parent(None),
            Field::Needs {
                edge: "bl-9:close".into(),
                on: true,
            },
            Field::Needs {
                edge: "bl-8".into(),
                on: false,
            },
        ]
    };
    let w = World::new("bl", "#!/bin/sh\nprintf 'bl-9zzz\\n'\nexit 0\n");
    create(
        &w.cli,
        w.state.path(),
        "TS",
        &w.cwd,
        "filtered",
        &Create {
            title: "wire it".into(),
            fields: all(),
            ..Create::default()
        },
    )
    .unwrap();
    assert_eq!(
        args_of(&w.logged()),
        vec![
            "create",
            "wire it",
            "--as",
            "filtered",
            "-p",
            "3",
            "-t",
            "boundary",
            "--parent",
            "bl-1a2b",
            "--needs",
            "bl-9:close",
        ],
        "the clearing forms are no-ops on a ball that does not exist yet"
    );
    update(
        &w.cli,
        w.state.path(),
        "TS",
        &w.cwd,
        "bl-3",
        "filtered",
        &Update {
            fields: all(),
            ..Update::default()
        },
    )
    .unwrap();
    let e = opslog::tail(w.state.path(), 8).pop().unwrap();
    assert_eq!(
        args_of(&e),
        vec![
            "update",
            "bl-3",
            "--as",
            "filtered",
            "-p",
            "3",
            "--no-priority",
            "-t",
            "boundary",
            "--no-tag",
            "stale",
            "--parent",
            "bl-1a2b",
            "--no-parent",
            "--needs",
            "bl-9:close",
            "--no-needs",
            "bl-8",
        ]
    );
}
