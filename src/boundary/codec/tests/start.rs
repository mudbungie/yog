//! The §8.1 start family's own enum tables (`codec::start`): every payload rung
//! a prepare carries, every join state an existing ball is spelled with, every
//! origin a prompt is stamped with, and the start envelope's answer to a
//! gesture that is not one. Its own file at §12's cap, on the seam the codec
//! itself is cut along.

use super::{p, rt};
use crate::boundary::codec::decode;
use crate::boundary::codec::start::encode_start;
use crate::boundary::{Action, Gesture};
use crate::opslog::Origin;
use crate::projects::join::JoinState;
use crate::start::{BallSpec, Payload, Prepared};

#[test]
fn every_payload_rung_round_trips_inside_prepare() {
    for payload in [
        Payload::Bare,
        Payload::Path { dir: p("/work") },
        Payload::Ball {
            project: "proj".into(),
            ball: BallSpec::New {
                title: "t".into(),
                body: "b".into(),
            },
        },
    ] {
        rt(Gesture::Act(Action::Prepare {
            workspace: "ws".into(),
            payload,
        }));
    }
}

#[test]
fn every_join_state_round_trips_inside_an_existing_ball() {
    for join in [
        JoinState::ReadyStartable,
        JoinState::Blocked,
        JoinState::Bound,
        JoinState::ClaimedElsewhere,
        JoinState::Delivered,
        JoinState::UnassignedWorkspace,
        JoinState::OrphanedProject,
    ] {
        rt(Gesture::Act(Action::Prepare {
            workspace: "ws".into(),
            payload: Payload::Ball {
                project: "proj".into(),
                ball: BallSpec::Existing {
                    id: "bl-9".into(),
                    title: "t".into(),
                    body: "b".into(),
                    join,
                    tags: Vec::new(),
                },
            },
        }));
    }
}

/// Every origin **and both states of the §3.3 typed binding** (bl-6654): a
/// bound rung and the bare rung's `None` are two values of one field, so both
/// have to survive the wire — `null` decoding as "bind nothing" is the whole
/// reason the bare rung can be deposited back as the gesture it was.
#[test]
fn every_origin_round_trips_inside_a_prompt() {
    for origin in [Origin::Balls, Origin::Conversation, Origin::World] {
        for binding in [None, Some(p("/target"))] {
            // Both halves of the §3.3 seed (bl-1747): a seat that predicted a
            // name carries it, and one that predicted none carries `None` —
            // absent and present are different facts, so both cross.
            for seed in [None, Some(0xc0df)] {
                rt(Gesture::Act(Action::Prompt {
                    prepared: Prepared {
                        workspace: "ws".into(),
                        binding: binding.clone(),
                        goal: "the goal".into(),
                        origin,
                        lineage: None,
                    },
                    goal: "edited goal".into(),
                    seed,
                }));
            }
        }
    }
}

/// **The start family's envelope builder answers `null` to anything else**
/// (bl-1747). Its one caller is the action table's `Prepare | Prompt` arm, so
/// nothing else can reach it in production — but a fallback nobody can reach is
/// still one somebody could widen that arm onto, and `null` is the honest
/// answer: an envelope with no `op`, which decode refuses by name. Pinned here
/// rather than assumed, so the pair and its fallback are read together.
#[test]
fn the_start_envelope_answers_null_to_anything_but_its_own_two() {
    let value = encode_start(&Action::Ack);
    assert!(value.is_null(), "not a start gesture, so not an envelope");
    assert!(decode(&value).is_err(), "and decode refuses it");
}
