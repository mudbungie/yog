//! The parity table (§8.5): **every** gesture spells as a line and reads back
//! as itself at the seat it was spelled from. This is the claim the third
//! serialization makes, mechanized — the compile gate says a variant *has* a
//! spelling, and this says the spelling is the gesture.

use super::{ctx, existing, prepared};
use crate::boundary::help;
use crate::boundary::line::{parse, spell};
use crate::boundary::{Action, Gesture};
use crate::fan::Verb;
use crate::start::{BallSpec, Payload};
use std::path::PathBuf;

mod balls;
mod config;
mod inspector;
mod policy;
mod tools;

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
            workspace: "ws".to_owned(),
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
fn every_litany_action_round_trips() {
    rt(Gesture::Act(Action::Message {
        workspace: "ws".to_owned(),
        agent: "c-1".to_owned(),
        content: "ship it".to_owned(),
    }));
    // Send-and-interrupt (bl-a33d): the deposit's own grammar, so the tail
    // survives the trip with its inner spacing exactly as `/message`'s does.
    rt(Gesture::Act(Action::Interrupt {
        workspace: "ws".to_owned(),
        agent: "c-1".to_owned(),
        content: "stop  and do this".to_owned(),
    }));
    for children in [false, true] {
        rt(Gesture::Act(Action::Stop {
            workspace: "ws".to_owned(),
            agent: "c-1".to_owned(),
            children,
        }));
    }
    rt(Gesture::Act(Action::Scan {
        workspace: "ws".to_owned(),
    }));
    // The nudge (bl-9bef): aimed by the seat, carrying nothing — the verb is
    // the whole line, because what it says is what is already there.
    rt(Gesture::Act(Action::Nudge {
        workspace: "ws".to_owned(),
        agent: "c-1".to_owned(),
    }));
    let extra = parse("/nudge again", &ctx()).expect_err("it takes no arguments");
    assert!(extra.contains("takes no arguments"), "{extra}");
    let unselected = parse("/nudge", &crate::boundary::line::Context::default())
        .expect_err("a nudge is fired at a conversation, and none is selected");
    assert!(
        unselected.contains("no workspace in context"),
        "{unselected}"
    );
    // The §9.4 exit (bl-2d19): both its targets are the seat's, so the verb is
    // the whole line and the round trip holds modulo that context.
    rt(Gesture::Act(Action::Retarget {
        workspace: "ws".to_owned(),
        agent: "c-1".to_owned(),
    }));
}

#[test]
fn every_start_rung_and_the_prompt_round_trip() {
    for payload in [
        Payload::Bare,
        Payload::Path {
            dir: PathBuf::from("/tmp/work"),
        },
        Payload::Ball {
            project: "proj".to_owned(),
            ball: existing(),
        },
        Payload::Ball {
            project: "proj".to_owned(),
            ball: BallSpec::New {
                title: "a fresh ball".to_owned(),
                body: String::new(),
            },
        },
        Payload::Ball {
            project: "proj".to_owned(),
            ball: BallSpec::New {
                title: "a fresh ball".to_owned(),
                body: "with a body".to_owned(),
            },
        },
    ] {
        rt(Gesture::Act(Action::Prepare {
            workspace: "ws".to_owned(),
            payload,
        }));
    }
    rt(Gesture::Act(Action::Prompt {
        prepared: prepared(),
        goal: "do the thing".to_owned(),
        // A typed line predicts no name (bl-1747), so the round trip is the
        // seedless one — the seat that has a prediction fills it after the read.
        seed: None,
    }));
}

/// The seat's own delivery obligation, as every fan-family line reads it.
fn obligation() -> crate::fan::Obligation {
    crate::fan::Obligation {
        project: "proj".to_owned(),
        ball: Some("bl-1".to_owned()),
    }
}

/// The §4.10 fan's three: N is the whole line, the handle is the whole line,
/// the delivery's summary is the verbatim tail after its handle, and the
/// obligation is the seat's — so all round-trip **modulo context**, the parity
/// claim the line makes everywhere.
#[test]
fn the_fan_family_round_trips() {
    for n in [0, 1, 5] {
        rt(Gesture::Act(Action::Fan(Verb::Spread {
            prepared: prepared(),
            obligation: obligation(),
            n,
        })));
    }
    rt(Gesture::Act(Action::Fan(Verb::Retire {
        obligation: obligation(),
        handle: "at-0badcafe".to_owned(),
    })));
    rt(Gesture::Act(Action::Fan(Verb::Deliver {
        obligation: obligation(),
        handle: "at-0badcafe".to_owned(),
        summary: "take the winner — internal  spaces stay".to_owned(),
    })));
}

#[test]
fn every_trail_verb_and_query_round_trips() {
    rt(Gesture::Act(Action::DeleteWorkspace {
        workspace: "ws".to_owned(),
        typed: "alba".to_owned(),
    }));
    // Both arming moods of the one-conversation delete (bl-f17a): bare (the
    // leaf) and typed (the subtree).
    for typed in ["", "the goal name"] {
        rt(Gesture::Act(Action::DeleteAgent {
            workspace: "ws".to_owned(),
            agent: "c-1".to_owned(),
            typed: typed.to_owned(),
        }));
    }
    rt(Gesture::Act(Action::Ack));
    // The §6 queue's answer: aimed by the seat, exactly as `/message` is.
    rt(Gesture::Act(Action::MarkSeen {
        workspace: "ws".to_owned(),
        agent: "c-1".to_owned(),
    }));
    rt(Gesture::Act(Action::ClearTrail));
}

/// REMOTE §1.4's enrollment, typed (bl-f4e3): the common name is the one word
/// no seat's context can hold, and the grade is default-operator made typable —
/// bare is a seat, one word demotes, and nothing promotes by accident.
#[test]
fn the_enrollment_round_trips_at_both_grades() {
    for grade in [
        crate::registry::Grade::Operator,
        crate::registry::Grade::Foot,
    ] {
        rt(Gesture::Act(Action::Enroll(
            crate::registry::enroll::Request {
                workspace: "ws".to_owned(),
                name: "phone-1".to_owned(),
                grade,
            },
        )));
    }
    // `operator` said outright is the same gesture as saying nothing — one
    // vocabulary, read by the registry's own table in both serializations.
    assert_eq!(
        parse("/enroll phone-1 operator", &ctx()),
        parse("/enroll phone-1", &ctx())
    );
    // The brands read as their grades (bl-427b): thrall is the foot
    // spelling and lernie the operator one, at this layer only — spell
    // still emits the registry's words, so the round trip stays canonical.
    assert_eq!(
        parse("/enroll phone-1 thrall", &ctx()),
        parse("/enroll phone-1 foot", &ctx())
    );
    assert_eq!(
        parse("/enroll phone-1 lernie", &ctx()),
        parse("/enroll phone-1", &ctx())
    );
    // Nothing rounds: an unknown word refuses naming itself, rather than
    // becoming either grade — and the refusal teaches both vocabularies.
    let refusal = parse("/enroll phone-1 fott", &ctx()).expect_err("refused");
    assert!(refusal.contains("unknown grade \"fott\""), "{refusal}");
    assert!(refusal.contains("thrall"), "{refusal}");
    let unnamed = parse("/enroll", &ctx()).expect_err("usage names both vocabularies");
    assert!(unnamed.contains("thrall"), "{unnamed}");
    let unnamed = parse("/enroll", &ctx()).expect_err("the name is required");
    assert!(unnamed.contains("common name"), "{unnamed}");
    let unfocused = parse(
        "/enroll phone-1",
        &crate::boundary::line::Context::default(),
    )
    .expect_err("a device is seated in a workspace, and none is focused");
    assert!(unfocused.contains("no workspace in context"), "{unfocused}");
}
