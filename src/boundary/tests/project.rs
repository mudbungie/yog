//! [`Action::project`] (§3.5): which gestures name a project and which name
//! none — the table the ops trail's project column and every balls-side
//! executor read.

use super::*;
use crate::actions::verbs::Verb as BallVerb;
use crate::fan::Verb;
use crate::start::{BallSpec, Payload, Prepared};
use std::path::Path;

#[test]
fn the_bl_family_names_its_project_and_nothing_else_does() {
    let p = "p".to_owned();
    let ws = "ws".to_owned();
    let bl_family = [
        Action::Ball(BallVerb::Close {
            project: p.clone(),
            id: "b-1".into(),
            name: "n".into(),
        }),
        Action::Ball(BallVerb::Assign {
            project: p.clone(),
            id: "b-1".into(),
            name: "n".into(),
        }),
        Action::Ball(BallVerb::Release {
            project: p.clone(),
            id: "b-1".into(),
            name: "n".into(),
        }),
        Action::Ball(BallVerb::Create {
            project: p.clone(),
            name: "n".into(),
            fields: crate::actions::verbs::edit::Create::default(),
        }),
        Action::Ball(BallVerb::Update {
            project: p.clone(),
            id: "b-1".into(),
            name: "n".into(),
            fields: crate::actions::verbs::edit::Update::default(),
        }),
    ];
    for action in bl_family {
        assert_eq!(action.project(), Some(p.clone()), "{action:?}");
    }
    let litany_family = [
        Action::Message {
            workspace: ws.clone(),
            agent: "c".into(),
            content: "hi".into(),
        },
        Action::Stop {
            workspace: ws.clone(),
            agent: "c".into(),
            children: false,
        },
        Action::Interrupt {
            workspace: ws.clone(),
            agent: "c".into(),
            content: "hi".into(),
        },
        Action::Scan {
            workspace: ws.clone(),
        },
        Action::Retarget {
            workspace: ws.clone(),
            agent: "c".into(),
        },
        Action::DeleteWorkspace {
            workspace: ws.clone(),
            typed: "n".into(),
        },
        Action::DeleteAgent {
            workspace: ws.clone(),
            agent: "c".into(),
            typed: "n".into(),
        },
        Action::Fork {
            workspace: ws.clone(),
            parent: "c".into(),
            attempt: crate::fork::Attempt::default(),
            goal: "g".into(),
        },
        Action::Ack,
        Action::MarkSeen {
            workspace: ws.clone(),
            agent: "c".into(),
        },
        Action::ClearTrail,
    ];
    for action in litany_family {
        assert_eq!(action.project(), None, "{action:?}");
    }
}

#[test]
fn a_ball_rung_prepare_carries_its_project_and_the_other_rungs_none() {
    let p = "proj".to_owned();
    let ball = Action::Prepare {
        workspace: "ws".to_owned(),
        payload: Payload::Ball {
            project: p.clone(),
            ball: BallSpec::New {
                title: "t".into(),
                body: String::new(),
            },
        },
    };
    assert_eq!(ball.project(), Some(p.clone()));
    for payload in [
        Payload::Bare,
        Payload::Path {
            dir: Path::new("/d").to_path_buf(),
        },
    ] {
        let a = Action::Prepare {
            workspace: "ws".to_owned(),
            payload,
        };
        assert_eq!(a.project(), None, "{a:?}");
    }
    let prompt = Action::Prompt {
        prepared: Prepared {
            workspace: "ws".into(),
            binding: None,
            goal: "g".into(),
            origin: crate::opslog::Origin::Conversation,
            lineage: None,
        },
        goal: "g".into(),
        seed: None,
    };
    assert_eq!(prompt.project(), None);
}

/// The §4.10 fan's three act in a project's refs rather than on its board, and
/// the §3.5 projection reads that project — so they name it too.
#[test]
fn the_fan_family_names_its_project() {
    let p = "p".to_owned();
    let obligation = crate::fan::Obligation {
        project: p.clone(),
        ball: Some("b-1".into()),
    };
    for action in [
        Action::Fan(Verb::Spread {
            prepared: Prepared {
                workspace: "ws".into(),
                binding: None,
                goal: "g".into(),
                origin: crate::opslog::Origin::Balls,
                lineage: None,
            },
            obligation: obligation.clone(),
            n: 3,
        }),
        Action::Fan(Verb::Retire {
            obligation: obligation.clone(),
            handle: "at-0badcafe".into(),
        }),
        Action::Fan(Verb::Deliver {
            obligation,
            handle: "at-0badcafe".into(),
            summary: "take it".into(),
        }),
    ] {
        assert_eq!(action.project(), Some(p.clone()), "{action:?}");
    }
}
