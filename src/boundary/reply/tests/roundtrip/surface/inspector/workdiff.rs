//! The work-diff fixture (§5.1 #32, bl-c2bd) — split from the inspector
//! family's file at §12's budget on the seam its own doc drew: the work diff
//! *shares* the family's shape, and its rows are the one fixture with arms of
//! their own (every [`Change`] arm, both churn classes, and the fan
//! candidate's two optional fields, present and absent).

use crate::workdiff::{Attempt, Change, Churn, FileChurn};

/// One attempt per [`Change`] arm, and both churn classes inside the diff.
pub(super) fn attempts() -> Vec<Attempt> {
    let attempt = |id: &str, change| Attempt {
        project: "p".to_owned(),
        ball_id: id.into(),
        handle: None,
        delivered: None,
        change,
    };
    vec![
        attempt("bl-1", Change::Unreadable),
        attempt(
            "bl-2",
            Change::Absent {
                target: "main".into(),
                source: "work/bl-2".into(),
                missing: vec!["work/bl-2".into()],
            },
        ),
        attempt(
            "bl-3",
            Change::Diff {
                target: "main".into(),
                source: "work/bl-3".into(),
                target_oid: "aaa".into(),
                source_oid: "bbb".into(),
                files: vec![
                    FileChurn {
                        path: "src/a.rs".into(),
                        churn: Churn::Text {
                            added: 3,
                            removed: 1,
                        },
                    },
                    FileChurn {
                        path: "assets/x.png".into(),
                        churn: Churn::Binary,
                    },
                ],
                truncated: true,
            },
        ),
        // A fan candidate wearing the derived acceptance mark (bl-c2bd): the
        // two optional fields populated, so a decoder that dropped either
        // would not pass on the claim rows' absences alone.
        Attempt {
            project: "p".to_owned(),
            ball_id: "bl-3".into(),
            handle: Some("at-0badcafe".into()),
            delivered: Some("ccc".into()),
            change: Change::Diff {
                target: "work/bl-3".into(),
                source: "attempt/at-0badcafe".into(),
                target_oid: "ccc".into(),
                source_oid: "ddd".into(),
                files: vec![],
                truncated: false,
            },
        },
    ]
}
