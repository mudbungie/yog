//! **What a gesture addresses** (REMOTE §8, bl-f5f6): the two `Action` tables
//! and the `Query` one, asserted where the chokepoints read them.
//!
//! The claim is narrow and load-bearing: every variant that names a workspace
//! answers with it, the nested payloads answer *through* their own carriers
//! (`monitor::Verb`, `fleet::Verb`, `start::Prepared`), and the variants that
//! name none say `None` — because `dispatch` resolves that one answer *once*,
//! ahead of its table, and a variant the table forgot would silently execute
//! against the empty path instead of refusing.

use super::super::{Action, Query};
use crate::start::{Payload, Prepared};

const WS: &str = "alba";

fn prepared() -> Prepared {
    Prepared {
        workspace: WS.to_owned(),
        binding: None,
        goal: "g".to_owned(),
        origin: crate::opslog::Origin::Conversation,
    }
}

/// The nested carriers and the deferred start family, which answer *through*
/// their own payloads rather than a field of their own — the half a table
/// forgets first.
fn nested() -> Vec<Action> {
    vec![
        Action::Monitor(crate::monitor::Verb::Arm {
            workspace: WS.into(),
            model: "haiku".into(),
        }),
        Action::Monitor(crate::monitor::Verb::Disarm {
            workspace: WS.into(),
        }),
        Action::Monitor(crate::monitor::Verb::Flag {
            workspace: WS.into(),
            agent: "c".into(),
            reason: "r".into(),
        }),
        Action::Fleet(crate::fleet::Verb::Arm {
            workspace: WS.into(),
            project: "p".into(),
            cap: 1,
        }),
        Action::Fleet(crate::fleet::Verb::Disarm {
            workspace: WS.into(),
        }),
        Action::Prompt {
            prepared: prepared(),
            goal: "g".into(),
            seed: None,
        },
        Action::Fan {
            prepared: prepared(),
            obligation: crate::fan::Obligation {
                project: "p".into(),
                ball: None,
            },
            n: 2,
        },
    ]
}

/// Every conversation- and workspace-shaped action names its sphere, the two
/// nested verb families answer through their own carriers, and the deferred
/// start family answers through the `Prepared` it fires.
#[test]
fn every_workspace_bearing_action_answers_with_its_name() {
    let mut named: Vec<Action> = vec![
        Action::Message {
            workspace: WS.into(),
            agent: "c".into(),
            content: "hi".into(),
        },
        Action::Stop {
            workspace: WS.into(),
            agent: "c".into(),
            children: false,
        },
        Action::Scan {
            workspace: WS.into(),
        },
        Action::Nudge {
            workspace: WS.into(),
            agent: "c".into(),
        },
        Action::Retarget {
            workspace: WS.into(),
            agent: "c".into(),
        },
        Action::Fork {
            workspace: WS.into(),
            parent: "c".into(),
            attempt: crate::fork::Attempt::default(),
            goal: "g".into(),
        },
        Action::Prepare {
            workspace: WS.into(),
            payload: Payload::Bare,
        },
        Action::DeleteWorkspace {
            workspace: WS.into(),
            typed: "alba".into(),
        },
        Action::DeleteAgent {
            workspace: WS.into(),
            agent: "c".into(),
            typed: "n".into(),
        },
        Action::MarkSeen {
            workspace: WS.into(),
            agent: "c".into(),
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
        Action::AnswerHold {
            workspace: WS.into(),
            agent: "c".into(),
            ruling: crate::control::judge::Ruling::Pass,
        },
        Action::Floor {
            workspace: WS.into(),
            agent: "c".into(),
            raised: true,
        },
    ];
    named.extend(nested());
    for action in named {
        assert_eq!(action.workspace().as_deref(), Some(WS), "{action:?}");
    }
}

/// The other half: the acts whose subject is a ball, a file or the trail name
/// no sphere at all, so the chokepoint resolves nothing for them.
#[test]
fn the_actions_that_name_no_workspace_say_so() {
    let anonymous = [
        Action::Close {
            project: "p".into(),
            id: "bl-1".into(),
            name: "n".into(),
        },
        Action::Retire {
            obligation: crate::fan::Obligation {
                project: "p".into(),
                ball: None,
            },
            handle: "at-0badcafe".into(),
        },
        Action::ApplyConfig {
            file: super::super::config::ConfigFile::Cadence,
            text: String::new(),
        },
        Action::Ack,
        Action::ClearTrail,
    ];
    for action in anonymous {
        assert_eq!(action.workspace(), None, "{action:?}");
    }
}

/// The read half of the same table: a conversation- or workspace-shaped query
/// names its sphere, and a world-wide one names none.
#[test]
fn the_query_table_answers_both_ways() {
    let aimed = [
        Query::Conversations {
            workspace: WS.into(),
        },
        Query::Marks {
            workspace: WS.into(),
        },
        Query::Transcript {
            workspace: WS.into(),
            agent: "c".into(),
        },
        Query::Providers {
            workspace: WS.into(),
        },
    ];
    for query in aimed {
        assert_eq!(query.workspace().as_deref(), Some(WS), "{query:?}");
    }
    for query in [
        Query::Workspaces,
        Query::Balls,
        Query::Board,
        Query::Attention,
        Query::Ops { max: 1 },
        Query::Search { text: "x".into() },
        Query::Help { verb: None },
        Query::ReadConfig {
            file: super::super::config::ConfigFile::Cadence,
        },
    ] {
        assert_eq!(query.workspace(), None, "{query:?}");
    }
}
