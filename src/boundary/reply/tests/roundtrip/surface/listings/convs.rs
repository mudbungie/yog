//! The conversation listing's own rows: the widest row type on the surface,
//! carrying an alignment verdict, a ball, a flight and a tone — its own file at
//! §12's cap, on the same seam `chrome` was cut along (one listing per file).

use crate::git_tree::AgentState;
use crate::monitor::{Check, Verdict};
use crate::nav::convs::Tone;
use crate::nav::convs::{ConvBall, ConvRow, Flight};
use crate::projects::join::JoinState;

/// The §11 conversation rows: the fully-loaded one, the bare one, and the
/// display-only rung whose `name` the wire withholds and the decode recovers
/// off `display` (bl-7067).
pub(super) fn conv_rows() -> Vec<ConvRow> {
    let full = ConvRow {
        root_id: "c-1".into(),
        state: AgentState::InFlight,
        uncertain: true,
        preview: "first line".into(),
        age_secs: 42,
        flight: Some(Flight::Inference),
        attention: 1,
        members: 3,
        depth: 2,
        direct: 2,
        stoppable: false,
        stop_children: false,
        ball: Some(ConvBall {
            id: "bl-7".into(),
            state: Some(JoinState::Bound),
            title: Some("t".into()),
            badge: Some("closed".into()),
        }),
        name: Some("brave-fox".into()),
        name_display_only: false,
        verdict: Some(Check {
            workspace: "/ws".into(),
            agent: "c-1".into(),
            verdict: Verdict::Drifting,
            sha: "deadbeef".into(),
            reason: "wandered".into(),
            model: "m".into(),
            input_tokens: Some(7),
            output_tokens: None,
        }),
        tone: Tone::Weak,
    };
    let bare = ConvRow {
        root_id: "c-2".into(),
        state: AgentState::Quiescent,
        uncertain: false,
        preview: String::new(),
        age_secs: 0,
        flight: None,
        attention: 0,
        members: 1,
        depth: 0,
        direct: 0,
        stoppable: false,
        stop_children: false,
        ball: Some(ConvBall {
            id: "stray".into(),
            state: None,
            title: None,
            badge: None,
        }),
        name: None,
        name_display_only: false,
        verdict: None,
        tone: Tone::Plain,
    };
    let legacy = ConvRow {
        root_id: "c-3".into(),
        name: Some("goal-stamped".into()),
        name_display_only: true,
        ball: None,
        ..bare.clone()
    };
    vec![full, bare, legacy]
}
