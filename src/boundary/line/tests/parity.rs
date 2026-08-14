//! The parity table (§8.5): **every** gesture spells as a line and reads back
//! as itself at the seat it was spelled from. This is the claim the third
//! serialization makes, mechanized — the compile gate says a variant *has* a
//! spelling, and this says the spelling is the gesture.

use super::{ctx, existing, prepared};
use crate::boundary::config::ConfigFile;
use crate::boundary::help;
use crate::boundary::line::{parse, spell};
use crate::boundary::{Action, Gesture, Query};
use crate::start::{BallSpec, Payload};
use std::path::PathBuf;

mod policy;

/// The parity claim, mechanized: spell it, read it back at the seat it was
/// spelled from, get the same gesture.
pub(super) fn rt(gesture: Gesture) {
    let line = spell(&gesture);
    assert_eq!(parse(&line, &ctx()), Ok(gesture.clone()), "via {line}");
    // The other direction of the single source (§8.5): the line names a verb,
    // and **every verb has a page**. Asserted here, on the one table that
    // enumerates every variant by hand, so a gesture added tomorrow cannot ship
    // helpless — the compile gate makes it spellable, this makes it explained.
    let verb = line
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_start_matches('/');
    assert!(help::known(verb), "/{verb} has no help page");
}

/// **S12-T5 three-spellings** (the line half): the attempt is typable, its
/// goal survives the trip verbatim, and a cohort needs no spelling of its own
/// — it is this line, typed again.
#[test]
fn the_attempt_round_trips_with_and_without_skills() {
    for skills in [Vec::new(), vec!["bash".to_owned(), "read_file".to_owned()]] {
        rt(Gesture::Act(Action::Fork {
            workspace: PathBuf::from("/ws"),
            parent: "c-1".to_owned(),
            attempt: crate::fork::Attempt {
                from: "config/strict".to_owned(),
                role: "worker".to_owned(),
                skills,
            },
            goal: "try it  the other way".to_owned(),
        }));
    }
}

#[test]
fn every_lernie_action_round_trips() {
    rt(Gesture::Act(Action::Message {
        workspace: PathBuf::from("/ws"),
        agent: "c-1".to_owned(),
        content: "ship it".to_owned(),
    }));
    for children in [false, true] {
        rt(Gesture::Act(Action::Stop {
            workspace: PathBuf::from("/ws"),
            agent: "c-1".to_owned(),
            children,
        }));
    }
    rt(Gesture::Act(Action::Scan {
        workspace: PathBuf::from("/ws"),
    }));
}

#[test]
fn every_ball_action_round_trips() {
    let (project, id, name) = (PathBuf::from("/proj"), "bl-1".to_owned(), "alba".to_owned());
    rt(Gesture::Act(Action::Close {
        project: project.clone(),
        id: id.clone(),
        name: name.clone(),
    }));
    rt(Gesture::Act(Action::Assign {
        project: project.clone(),
        id: id.clone(),
        name: name.clone(),
    }));
    rt(Gesture::Act(Action::Release {
        project: project.clone(),
        id: id.clone(),
        name: name.clone(),
    }));
    rt(Gesture::Act(Action::Move {
        project: project.clone(),
        id: id.clone(),
        from: name.clone(),
        to: "koi".to_owned(),
    }));
    for body in [None, Some("the body".to_owned())] {
        rt(Gesture::Act(Action::Create {
            project: project.clone(),
            title: "a new ball".to_owned(),
            name: name.clone(),
            body,
        }));
    }
    for fields in [
        (Some("t".to_owned()), None, None),
        (None, Some("b".to_owned()), None),
        (None, None, Some("a note".to_owned())),
        (
            Some("t".to_owned()),
            Some("b".to_owned()),
            Some("n".to_owned()),
        ),
    ] {
        rt(Gesture::Act(Action::Update {
            project: project.clone(),
            id: id.clone(),
            name: name.clone(),
            title: fields.0,
            body: fields.1,
            note: fields.2,
        }));
    }
}

#[test]
fn every_start_rung_and_the_prompt_round_trip() {
    for payload in [
        Payload::Bare,
        Payload::Path {
            dir: PathBuf::from("/tmp/work"),
        },
        Payload::Ball {
            project: PathBuf::from("/proj"),
            ball: existing(),
        },
        Payload::Ball {
            project: PathBuf::from("/proj"),
            ball: BallSpec::New {
                title: "a fresh ball".to_owned(),
                body: String::new(),
            },
        },
        Payload::Ball {
            project: PathBuf::from("/proj"),
            ball: BallSpec::New {
                title: "a fresh ball".to_owned(),
                body: "with a body".to_owned(),
            },
        },
    ] {
        rt(Gesture::Act(Action::Prepare {
            workspace: PathBuf::from("/ws"),
            payload,
        }));
    }
    rt(Gesture::Act(Action::Prompt {
        prepared: prepared(),
        goal: "do the thing".to_owned(),
    }));
}

#[test]
fn every_trail_verb_and_query_round_trips() {
    rt(Gesture::Act(Action::DeleteWorkspace {
        workspace: PathBuf::from("/ws"),
        typed: "alba".to_owned(),
    }));
    // Both arming moods of the one-conversation delete (bl-f17a): bare (the
    // leaf) and typed (the subtree).
    for typed in ["", "the goal name"] {
        rt(Gesture::Act(Action::DeleteAgent {
            workspace: PathBuf::from("/ws"),
            agent: "c-1".to_owned(),
            typed: typed.to_owned(),
        }));
    }
    rt(Gesture::Act(Action::Ack));
    // The §6 queue's answer: aimed by the seat, exactly as `/message` is.
    rt(Gesture::Act(Action::MarkSeen {
        workspace: PathBuf::from("/ws"),
        agent: "c-1".to_owned(),
    }));
    rt(Gesture::Act(Action::ClearTrail));
}

/// The config family's reads (§8.5, bl-0164): the same verb as the write
/// beside them, spelled with nothing after the destination.
#[test]
fn the_config_familys_reads_round_trip() {
    for file in [
        ConfigFile::Brazen {
            workspace: PathBuf::from("/ws"),
        },
        ConfigFile::LernieModels,
        ConfigFile::Cadence,
        ConfigFile::LernieWorkflow {
            name: "review".to_owned(),
        },
    ] {
        rt(Gesture::Ask(Query::ReadConfig { file }));
    }
    // A lineage destination reads too (bl-dff8), in all three of its origins:
    // the same words the write takes, with nothing after them.
    for origin in [
        crate::config_edit::branch::edit::EditOrigin::Advance,
        crate::config_edit::branch::edit::EditOrigin::Orphan,
        crate::config_edit::branch::edit::EditOrigin::Fork {
            source: "base".to_owned(),
        },
    ] {
        rt(Gesture::Ask(Query::ReadConfig {
            file: ConfigFile::Branch {
                workspace: PathBuf::from("/ws"),
                lineage: "strict".to_owned(),
                origin,
                path: "workflow.yaml".to_owned(),
            },
        }));
    }
    rt(Gesture::Ask(Query::Marks {
        workspace: PathBuf::from("/ws"),
    }));
    rt(Gesture::Ask(Query::Providers {
        workspace: PathBuf::from("/ws"),
    }));
    // The browse and the roster beside them (bl-dff8).
    rt(Gesture::Ask(Query::Lineages {
        workspace: PathBuf::from("/ws"),
    }));
    rt(Gesture::Ask(Query::Models {
        workspace: PathBuf::from("/ws"),
        provider: "acme".to_owned(),
    }));
}
