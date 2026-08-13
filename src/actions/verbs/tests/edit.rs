//! The ball-editing verbs' argv tests (`bl create` / `bl update`) — split from
//! `mod` per §12's 300-line budget; the shared World recorder fixture and
//! `args_of` live there. Each verb is stamped `--as <name>`.

use super::{OK_BODY, World, args_of};
use crate::actions::verbs::{Update, create, update};
use crate::opslog;

#[test]
fn create_captures_the_printed_id_and_optional_body() {
    let (w, _g) = World::new("bl", "#!/bin/sh\nprintf 'bl-9zzz\\n'\nexit 0\n");
    // No body.
    let out = create(
        &w.cli,
        w.state.path(),
        "TS",
        &w.cwd,
        "wire it",
        "filtered",
        None,
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
        "wire it",
        "filtered",
        Some("the plan"),
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
    let (w, _g) = World::new("bl", OK_BODY);
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
