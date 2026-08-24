//! The §11 steps pane's two reads: the summary list, one step per [`Framing`],
//! [`AuthFailure`] and [`Wound`] arm, and the one step's detail, one record per
//! [`Doc`] arm.

use super::super::spend;
use crate::git_tree::Framing;
use crate::login::auth::AuthFailure;
use crate::steps_view::{Doc, Orphan, StepDetail, StepSummary, StepsView, ToolIo, Wound};

/// One step per [`Framing`], per [`AuthFailure`] and per [`Wound`] arm — the
/// two shapes encoded from more than one key. The login affordance is still a
/// bijective pair; the wound became a class token plus the same optional
/// reason when bl-fb87 gave it a fourth arm.
pub(super) fn steps() -> StepsView {
    let base = StepSummary {
        seq: "001".into(),
        framing: Framing::Complete,
        attempts: 1,
        tokens: spend(),
        commit: Some("abc".into()),
        started_at: Some("t0".into()),
        ended_at: Some("t1".into()),
        auth_failed: AuthFailure::No,
        wound: Wound::None,
    };
    StepsView {
        steps: vec![
            base.clone(),
            StepSummary {
                seq: "002".into(),
                framing: Framing::Failed,
                auth_failed: AuthFailure::Row("anthropic".into()),
                wound: Wound::Spoke("no bytes".into()),
                commit: None,
                started_at: None,
                ended_at: None,
                ..base.clone()
            },
            StepSummary {
                seq: "003".into(),
                framing: Framing::Killed,
                auth_failed: AuthFailure::Unrouted,
                wound: Wound::Mute,
                ..base.clone()
            },
            // The bl-fb87 arm: a wound whose framing is `complete`, so the
            // class cannot be recovered from either the framing or a reason.
            StepSummary {
                seq: "004".into(),
                wound: Wound::OutputLimit,
                ..base
            },
        ],
        // The view-level orphaned-mail pair (bl-ace6): the Spoke arm here,
        // the Mute and None arms as their own replies below.
        orphan: Orphan::Spoke("driver died".into()),
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
