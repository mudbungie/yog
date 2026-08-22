//! The `bl` family through the §8.5 chokepoint: five variants, one project, and
//! the exact §8.2 argv each spawns.
//!
//! Split from [`super`] at §12's cap on the seam `codec/balls.rs` is already cut
//! on one layer down — the lernie family and the deposit round trip are one
//! subject, the ball verbs another, and the fixtures they share stay in the
//! parent.

use super::{deps, snapshot_of, ui};
use crate::support::Recorder;
use std::path::Path;
use tempfile::tempdir;
use yog::actions::verbs::edit;
use yog::boundary::Action;
use yog::boundary::dispatch::dispatch;
use yog::boundary::reply::Reply;
use yog::cli_outbound::Cli;
use yog::opslog;

/// The five bl-family variants, against one project.
fn bl_actions(proj: &Path) -> [Action; 5] {
    [
        Action::Close {
            project: yog::naming::leaf(proj),
            id: "bl-1".into(),
            name: "alba".into(),
        },
        Action::Assign {
            project: yog::naming::leaf(proj),
            id: "bl-1".into(),
            name: "alba".into(),
        },
        Action::Release {
            project: yog::naming::leaf(proj),
            id: "bl-1".into(),
            name: "alba".into(),
        },
        Action::Create {
            project: yog::naming::leaf(proj),
            name: "alba".into(),
            fields: edit::Create {
                title: "the title".into(),
                body: Some("body".into()),
                ..edit::Create::default()
            },
        },
        Action::Update {
            project: yog::naming::leaf(proj),
            id: "bl-1".into(),
            name: "alba".into(),
            fields: edit::Update {
                title: Some("t2".into()),
                note: Some("n".into()),
                ..edit::Update::default()
            },
        },
    ]
}

/// The bl-family variants spawn their §8.2 argv through the chokepoint.
#[test]
fn the_bl_actions_spawn_their_exact_argv_and_ops_rows() {
    let bin = tempdir().unwrap();
    let state = tempdir().unwrap();
    let proj = tempdir().unwrap();
    let rec = Recorder::new(bin.path(), "bl").on("create", "bl-77\n", 0);
    let bl = Cli::new(rec.path());
    let d = deps(
        &Cli::new("/no/lernie"),
        &bl,
        state.path(),
        snapshot_of(&[], &[proj.path()]),
    );
    let actions = bl_actions(proj.path());
    for (i, action) in actions.iter().enumerate() {
        match dispatch(&d, &mut ui(), &format!("T{i}"), action).unwrap() {
            Reply::Outcome(outcome) => assert!(outcome.ok(), "{action:?}"),
            other => panic!("a verb answers an outcome, got {other:?}"),
        }
    }
    let argv: Vec<Vec<String>> = rec.invocations().into_iter().map(|i| i.argv).collect();
    assert_eq!(
        argv,
        vec![
            vec![
                "close".to_owned(),
                "bl-1".into(),
                "--as".into(),
                "alba".into()
            ],
            vec![
                "claim".to_owned(),
                "bl-1".into(),
                "--as".into(),
                "alba".into()
            ],
            vec![
                "unclaim".to_owned(),
                "bl-1".into(),
                "--as".into(),
                "alba".into()
            ],
            vec![
                "create".to_owned(),
                "the title".into(),
                "--as".into(),
                "alba".into(),
                "--body".into(),
                "body".into(),
            ],
            vec![
                "update".to_owned(),
                "bl-1".into(),
                "--as".into(),
                "alba".into(),
                "--title".into(),
                "t2".into(),
                "-m".into(),
                "n".into(),
            ],
        ],
        "the §8.2 argv, verbatim"
    );
    let ops = opslog::tail(state.path(), 16);
    assert_eq!(ops.len(), 5, "one ops row per spawn (§4.2)");
    assert!(ops.iter().all(|e| e.exit == 0));
}
