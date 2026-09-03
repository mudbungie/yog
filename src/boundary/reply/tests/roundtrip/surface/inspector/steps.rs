//! The §11 steps pane's two reads: the summary list, one step per [`Framing`]
//! and per [`Wound`] arm — the refusal arm once per [`AuthFailure`] it can
//! carry — and the one step's detail, one record per [`Doc`] arm.

use super::super::spend;
use crate::git_tree::Framing;
use crate::login::auth::AuthFailure;
use crate::steps_view::{Doc, Orphan, StepDetail, StepSummary, StepsView, Tail, ToolIo, Wound};

/// One step per [`Framing`] and per [`Wound`] arm — the shape encoded from
/// more than one key. The wound is a class token plus two optional keys: the
/// reason that separates the no-response arms (bl-fb87), and the `auth_row`
/// that separates the refusal's two (bl-015b). The §8.3 login affordance is
/// the `refused` class itself and has no key of its own, so both
/// [`AuthFailure`] arms it can carry ride here.
pub(super) fn steps() -> StepsView {
    let base = StepSummary {
        seq: "001".into(),
        framing: Framing::Complete,
        attempts: 1,
        tokens: spend(),
        commit: Some("abc".into()),
        started_at: Some("t0".into()),
        ended_at: Some("t1".into()),
        wound: Wound::None,
    };
    StepsView {
        steps: vec![
            base.clone(),
            StepSummary {
                seq: "002".into(),
                framing: Framing::Killed,
                wound: Wound::Spoke("no bytes".into()),
                commit: None,
                started_at: None,
                ended_at: None,
                ..base.clone()
            },
            StepSummary {
                seq: "003".into(),
                framing: Framing::Killed,
                wound: Wound::Mute,
                ..base.clone()
            },
            // The bl-fb87 arm: a wound whose framing is `complete`, so the
            // class cannot be recovered from either the framing or a reason.
            StepSummary {
                seq: "004".into(),
                wound: Wound::OutputLimit,
                ..base.clone()
            },
            // The bl-015b arm, both ways it can route: the provider row it
            // failed on, and the honest middle where no row is derivable.
            StepSummary {
                seq: "005".into(),
                framing: Framing::Failed,
                wound: Wound::Refused(AuthFailure::Row("anthropic".into())),
                ..base.clone()
            },
            StepSummary {
                seq: "006".into(),
                framing: Framing::Failed,
                wound: Wound::Refused(AuthFailure::Unrouted),
                ..base
            },
        ],
        // The view-level orphaned-tail class + reason (bl-ace6, bl-abba):
        // the Spoke arm here, the Mute and None arms as their own replies
        // below — and each of the two `Tail` shapes across them, so no token
        // rides the round trip untested.
        orphan: Orphan::Spoke(Tail::Mail, "driver died".into()),
    }
}

/// One record per [`Doc`] arm: parsed with its bytes, absent, and bytes that
/// are not JSON — plus one capture log present and one absent (bl-83d6), the
/// pair the picker's row set is derived from.
pub(super) fn step_detail() -> StepDetail {
    StepDetail {
        seq: "001".into(),
        meta: Doc::Json {
            value: serde_json::json!({ "commit": "abc" }),
            raw: br#"{"commit":"abc"}"#.to_vec(),
        },
        request: Doc::Absent,
        staging: Doc::Unparsed(b"not json".to_vec()),
        response: vec![Doc::Absent],
        tools: vec![ToolIo {
            tool_id: "toolu_1".into(),
            input: Doc::Absent,
            output: Doc::Unparsed(b"raw".to_vec()),
            is_error: false,
        }],
        stderr: Some(crate::files_view::Preview::Truncated {
            text: "the adapter's last words".into(),
            size: 999_999,
        }),
        driver: None,
    }
}
