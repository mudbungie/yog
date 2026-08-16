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
        // The fan family's three receipts (§3.8; V3's delivery). The fanned
        // rows are `prepared` bodies; the delivery is taken at both of its
        // shapes — the landed squash, and upstream's "the target already
        // contained everything the source had", whose two fields are absent.
        Reply::Fanned(vec![Prepared {
            workspace: crate::naming::leaf(&(PathBuf::from("/ws"))),
            binding: Some(PathBuf::from("/candidate")),
            goal: "g".into(),
            origin: Origin::Balls,
        }]),
        Reply::Retired { discarded: true },
        Reply::Delivered(crate::fan::Delivery {
            target: "work/bl-1f2a".into(),
            base: "aaa1".into(),
            source: Some("bbb2".into()),
            commit: Some("ccc3".into()),
        }),
        Reply::Delivered(crate::fan::Delivery {
            target: "main".into(),
            base: "aaa1".into(),
            source: None,
            commit: None,
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
        // The routing leg's one answer at both of its moments (bl-024b): the
        // handle alone while the far machine runs it, and the capture once it
        // has answered — including a non-zero verdict and text on stderr, which
        // is the arm a tool that failed takes.
        Reply::Routed {
            invocation: "inv-1".into(),
            capture: None,
        },
        Reply::Routed {
            invocation: "inv-2".into(),
            capture: Some(crate::registry::mailbox::Capture {
                stdout: "hello\n".into(),
                stderr: "warned\n".into(),
                exit_code: 3,
            }),
        },
    ]
}
