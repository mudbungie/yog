//! **S11-T1 attempt-plan**: which attempts a workspace holds and what each
//! one's target is — balls' own delivery-target rule, re-derived from facts
//! the snapshot already carries, and `git diff --numstat` read the way git
//! writes it.

use std::collections::HashMap;
use std::path::PathBuf;

use super::{ball, close_gate};
use crate::projects::balls::Ball;
use crate::workdiff::plan::{Plan, parse_numstat, parse_row, plans};
use crate::workdiff::{Churn, FileChurn};

fn by_project(pairs: Vec<(&str, Vec<Ball>)>) -> HashMap<PathBuf, Vec<Ball>> {
    pairs
        .into_iter()
        .map(|(path, balls)| (PathBuf::from(path), balls))
        .collect()
}

/// The binding is the claimant equality: the workspace's own name, and nobody
/// else's. A flat ball targets the integration branch, which only the repo can
/// name — so the plan leaves it open.
#[test]
fn a_workspace_holds_the_balls_that_name_it_and_a_flat_ball_targets_the_branch() {
    let map = by_project(vec![(
        "/p",
        vec![
            ball("bl-1", Some("storeroom"), None),
            ball("bl-2", Some("elsewhere"), None),
            ball("bl-3", None, None),
        ],
    )]);
    assert_eq!(
        plans(&map, "storeroom"),
        vec![Plan {
            project: PathBuf::from("/p"),
            ball_id: "bl-1".to_owned(),
            target_ball: None,
        }]
    );
}

/// balls' rule, both coordinates required: a child ball delivers onto its
/// parent's branch only when the parent close-gates it. Containment alone —
/// a `parent` with no gate — stays flat.
#[test]
fn a_close_gated_child_targets_its_parent_and_bare_containment_does_not() {
    let gated = by_project(vec![(
        "/p",
        vec![
            close_gate(ball("bl-parent", None, None), "bl-kid"),
            ball("bl-kid", Some("storeroom"), Some("bl-parent")),
        ],
    )]);
    assert_eq!(
        plans(&gated, "storeroom")[0].target_ball.as_deref(),
        Some("bl-parent")
    );
    let contained = by_project(vec![(
        "/p",
        vec![
            ball("bl-parent", None, None),
            ball("bl-kid", Some("storeroom"), Some("bl-parent")),
        ],
    )]);
    assert_eq!(plans(&contained, "storeroom")[0].target_ball, None);
    // A parent that is no longer live gates nothing: its file is gone, so the
    // pointer is display-only and the child delivers flat.
    let orphan = by_project(vec![(
        "/p",
        vec![ball("bl-kid", Some("storeroom"), Some("bl-gone"))],
    )]);
    assert_eq!(plans(&orphan, "storeroom")[0].target_ball, None);
}

/// Two projects, two attempts — both said, in project-path order, because
/// nothing picks one and an instance that answered differently from another
/// would break determinism.
#[test]
fn every_attempt_is_listed_in_project_order() {
    let map = by_project(vec![
        ("/z", vec![ball("bl-z", Some("storeroom"), None)]),
        ("/a", vec![ball("bl-a", Some("storeroom"), None)]),
    ]);
    let ids: Vec<String> = plans(&map, "storeroom")
        .into_iter()
        .map(|p| p.ball_id)
        .collect();
    assert_eq!(ids, vec!["bl-a", "bl-z"]);
    assert!(plans(&map, "nobody").is_empty());
}

/// numstat rows read as churn; a binary file's `-`/`-` is said as binary, not
/// as zero lines, and a rename's composite path rides through as git wrote it.
#[test]
fn numstat_rows_read_as_churn_and_binary_is_said_as_itself() {
    let out = b"12\t3\tsrc/a.rs\n-\t-\tlogo.png\n4\t0\t{old => new}/c.rs\n";
    assert_eq!(
        parse_numstat(out),
        vec![
            FileChurn {
                path: "src/a.rs".to_owned(),
                churn: Churn::Text {
                    added: 12,
                    removed: 3
                },
            },
            FileChurn {
                path: "logo.png".to_owned(),
                churn: Churn::Binary,
            },
            FileChurn {
                path: "{old => new}/c.rs".to_owned(),
                churn: Churn::Text {
                    added: 4,
                    removed: 0
                },
            },
        ]
    );
}

/// A row this parser cannot read contributes nothing — it never guesses a
/// count or a path.
#[test]
fn a_row_it_cannot_read_contributes_nothing() {
    assert!(parse_row("no tabs here").is_none());
    assert!(parse_row("1\t2\t").is_none(), "no path");
    assert!(parse_row("many\t2\tsrc/a.rs").is_none(), "not a count");
    assert!(parse_numstat(b"\n").is_empty());
}
