//! **What a gesture addresses, one noun down** (bl-49bc): the conversation
//! table over both rosters, and the two-rung resolution that stands ahead of
//! each chokepoint.
//!
//! The claim is the same one the workspace table's suite makes and it is
//! load-bearing for the same reason: a variant the table forgot would execute
//! against the *unresolved* needle — which is the whole defect this closes, a
//! display name reaching `floor`, `flag` and `delete-agent` as though it were
//! an id.
//!
//! Split at §12's cap on the seam the subject draws: the tables here, the
//! ladder in [`resolve`], and in [`chokepoint`] the two beats that matter to an
//! operator — a `Started` handle addressing the root through `dispatch` and
//! through `answer`.

/// The ladder's own beats, and the two refusals that are the defect's fix.
mod chokepoint;
mod resolve;

use super::{Action, Query};
use crate::actions::verbs::Verb as BallVerb;
use crate::boundary::config::Read;

/// The workspace name every fixture gesture here carries.
const WS: &str = "alba";
/// The needle every table assertion carries — its shape is irrelevant to the
/// table (which only reports what a variant holds) and load-bearing to the
/// resolver below.
const NEEDLE: &str = "c";

/// Every conversation-addressed action answers with the needle it holds — the
/// nine that carry an `agent`, the fork that carries a `parent`, and the
/// monitor's flag, which answers *through* its own verb.
#[test]
fn every_conversation_bearing_action_answers_with_its_needle() {
    let aimed = [
        Action::Message {
            workspace: WS.into(),
            agent: NEEDLE.into(),
            content: "hi".into(),
        },
        Action::Stop {
            workspace: WS.into(),
            agent: NEEDLE.into(),
            children: false,
        },
        Action::Interrupt {
            workspace: WS.into(),
            agent: NEEDLE.into(),
            content: "hi".into(),
        },
        Action::Nudge {
            workspace: WS.into(),
            agent: NEEDLE.into(),
        },
        Action::Retarget {
            workspace: WS.into(),
            agent: NEEDLE.into(),
        },
        Action::DeleteAgent {
            workspace: WS.into(),
            agent: NEEDLE.into(),
            typed: "n".into(),
        },
        Action::MarkSeen {
            workspace: WS.into(),
            agent: NEEDLE.into(),
        },
        Action::AnswerHold {
            workspace: WS.into(),
            agent: NEEDLE.into(),
            ruling: crate::control::judge::Ruling::Pass,
        },
        Action::Floor {
            workspace: WS.into(),
            agent: NEEDLE.into(),
            raised: true,
        },
        Action::Fork {
            workspace: WS.into(),
            parent: NEEDLE.into(),
            attempt: crate::fork::Attempt::default(),
            goal: "g".into(),
        },
        Action::Monitor(crate::monitor::Verb::Flag {
            workspace: WS.into(),
            agent: NEEDLE.into(),
            reason: "r".into(),
        }),
    ];
    for action in aimed {
        assert_eq!(action.agent().as_deref(), Some(NEEDLE), "{action:?}");
    }
}

/// The other half: an act aimed at a workspace, a ball, a client or the world
/// names no conversation — including the start, which *makes* one and answers
/// with its name in the receipt instead.
#[test]
fn the_actions_that_name_no_conversation_say_so() {
    let anonymous = [
        Action::Scan {
            workspace: WS.into(),
        },
        Action::Prepare {
            workspace: WS.into(),
            payload: crate::start::Payload::Bare,
        },
        Action::DeleteWorkspace {
            workspace: WS.into(),
            typed: WS.into(),
        },
        Action::SetMarks {
            workspace: WS.into(),
            branch: "balls/tasks".into(),
        },
        Action::PickModel {
            workspace: WS.into(),
            role: "worker".into(),
            provider: "acme".into(),
            model: "m".into(),
        },
        Action::Ball(BallVerb::Close {
            project: "p".into(),
            id: "bl-1".into(),
            name: "n".into(),
        }),
        Action::Monitor(crate::monitor::Verb::Disarm {
            workspace: WS.into(),
        }),
        Action::Monitor(crate::monitor::Verb::Arm {
            workspace: WS.into(),
            model: "haiku".into(),
        }),
        Action::Fleet(crate::fleet::Verb::Disarm {
            workspace: WS.into(),
        }),
        Action::ApplyConfig {
            file: crate::boundary::config::ConfigFile::Cadence,
            text: String::new(),
        },
        Action::Ack,
        Action::ClearTrail,
    ];
    for action in anonymous {
        assert_eq!(action.agent(), None, "{action:?}");
    }
}

/// The read half: the §11 inspector family and the seat's own read are aimed at
/// a conversation; everything else is not.
#[test]
fn the_query_table_answers_both_ways() {
    let aimed = [
        Query::Transcript {
            workspace: WS.into(),
            agent: NEEDLE.into(),
        },
        Query::Steps {
            workspace: WS.into(),
            agent: NEEDLE.into(),
        },
        Query::Step {
            workspace: WS.into(),
            agent: NEEDLE.into(),
            seq: "001".into(),
        },
        Query::Inbox {
            workspace: WS.into(),
            agent: NEEDLE.into(),
        },
        Query::Agent {
            workspace: WS.into(),
            agent: NEEDLE.into(),
        },
    ];
    for query in aimed {
        assert_eq!(query.agent().as_deref(), Some(NEEDLE), "{query:?}");
    }
    for query in [
        Query::Conversations {
            workspace: WS.into(),
        },
        Query::Config(Read::Marks {
            workspace: WS.into(),
        }),
        Query::Workspaces,
        Query::Help { verb: None },
    ] {
        assert_eq!(query.agent(), None, "{query:?}");
    }
}
