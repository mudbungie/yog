//! The §8.1 start family's own enum tables (`codec::start`): every payload rung
//! a prepare carries, every join state an existing ball is spelled with, every
//! origin a prompt is stamped with, and the start envelope's answer to a
//! gesture that is not one. Its own file at §12's cap, on the seam the codec
//! itself is cut along.

use super::p;
use crate::boundary::codec::decode;
use crate::boundary::codec::start::encode_start;
use crate::boundary::{Action, Gesture};
use crate::opslog::Origin;
use crate::projects::join::JoinState;
use crate::start::{BallSpec, Payload, Prepared};

/// Every payload rung a prepare carries, every join state an existing ball is
/// spelled with, and every origin a prompt is stamped with — each beside the
/// two states of the §3.3 binding and of the §3.3 seed, because absent and
/// present are different facts and both cross.
pub(super) fn surface() -> Vec<Gesture> {
    let mut out = Vec::new();
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
        out.push(Gesture::Act(Action::Prepare {
            workspace: "ws".into(),
            payload,
        }));
    }
    for join in [
        JoinState::ReadyStartable,
        JoinState::Blocked,
        JoinState::Bound,
        JoinState::ClaimedElsewhere,
        JoinState::Delivered,
        JoinState::UnassignedWorkspace,
        JoinState::OrphanedProject,
    ] {
        out.push(Gesture::Act(Action::Prepare {
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
    for origin in [Origin::Balls, Origin::Conversation, Origin::World] {
        for binding in [None, Some(p("/target"))] {
            for seed in [None, Some(0xc0df)] {
                out.push(Gesture::Act(Action::Prompt {
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
    out
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
