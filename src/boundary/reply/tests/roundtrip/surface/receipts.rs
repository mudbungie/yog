//! The receipts (§8.5): what one act earned. Small values, but each one is a
//! variant, and a variant with no fixture leaves its encode arm unexecuted.

use std::path::PathBuf;

use super::super::super::super::Reply;
use crate::actions::verbs::Outcome;
use crate::opslog::Origin;
use crate::start::Prepared;

pub(super) fn receipts() -> Vec<Reply> {
    vec![
        // A non-zero exit on purpose: `ok: false` on an answer is the one
        // envelope the refusal shape could be confused with.
        Reply::Outcome(Outcome {
            exit: 3,
            stdout: "out".into(),
            stderr: "err".into(),
        }),
        Reply::Prepared(Prepared {
            workspace: crate::naming::leaf(&(PathBuf::from("/ws"))),
            binding: Some(PathBuf::from("/target")),
            goal: "g".into(),
            origin: Origin::Balls,
        }),
        Reply::Started {
            conversation: "brave-fox".into(),
        },
        Reply::Deleted,
        Reply::Armed { armed: true },
        Reply::Flagged,
        Reply::Answered {
            tool_use: "toolu_1".into(),
            tool: "Bash".into(),
            ruling: crate::control::judge::Ruling::Hold,
            advanced: false,
        },
        Reply::Floored { standing: true },
        Reply::Nudged,
        Reply::Acked,
        Reply::TrailCleared,
        Reply::Applied,
        Reply::Marks {
            branch: "marks/alba".into(),
        },
        Reply::Config {
            text: "roles: []".into(),
        },
        Reply::Advertised,
    ]
}
