//! **S12-T5 three-spellings** (the envelope half): the attempt round-trips as
//! itself, its list field reads strictly, and there is no cohort envelope to
//! test — a fan is N of these.

use super::{p, rt};
use crate::boundary::codec::decode;
use crate::boundary::{Action, Gesture};
use crate::fork::Attempt;
use serde_json::json;

fn attempt(from: &str, skills: Vec<String>) -> Attempt {
    Attempt {
        from: from.to_owned(),
        role: "worker".to_owned(),
        skills,
    }
}

/// One attempt, with and without skills — and the goal verbatim through the
/// envelope, spacing and newlines included.
#[test]
fn the_attempt_round_trips_with_and_without_skills() {
    rt(Gesture::Act(Action::Fork {
        workspace: p("/ws"),
        parent: "c-1".into(),
        attempt: attempt("aaaa1111", vec!["bash".into(), "read_file".into()]),
        goal: "try it  the other way\nagain".into(),
    }));
    rt(Gesture::Act(Action::Fork {
        workspace: p("/ws"),
        parent: "c-1".into(),
        attempt: attempt("config/strict", Vec::new()),
        goal: "g".into(),
    }));
}

/// A fork envelope with no `skills` field carries no skills: absence is a
/// value here, because "this attempt pins nothing" is the ordinary case and
/// saying it with an empty array would be ceremony.
#[test]
fn a_fork_without_a_skills_field_pins_nothing() {
    use serde_json::json;
    assert_eq!(
        decode(&json!({"op": "fork", "workspace": "/ws", "parent": "c-1",
                       "from": "aaaa1111", "role": "worker", "goal": "g"})),
        Ok(Gesture::Act(Action::Fork {
            workspace: p("/ws"),
            parent: "c-1".into(),
            attempt: crate::fork::Attempt {
                from: "aaaa1111".into(),
                role: "worker".into(),
                skills: Vec::new(),
            },
            goal: "g".into(),
        }))
    );
}

/// The list field reads strictly: absent is none, but a present field of the
/// wrong shape refuses rather than being half-obeyed.
#[test]
fn a_malformed_attempt_refuses_with_a_reason() {
    let cases: Vec<(serde_json::Value, &str)> = vec![
        (
            json!({"op": "fork", "workspace": "/ws", "parent": "c", "role": "worker",
                   "goal": "g"}),
            "field \"from\"",
        ),
        (
            json!({"op": "fork", "workspace": "/ws", "parent": "c", "from": "r",
                   "role": "worker", "goal": "g", "skills": "bash"}),
            "field \"skills\" must be an array of strings",
        ),
        (
            json!({"op": "fork", "workspace": "/ws", "parent": "c", "from": "r",
                   "role": "worker", "goal": "g", "skills": [7]}),
            "field \"skills\" must be an array of strings",
        ),
    ];
    for (envelope, needle) in cases {
        let refusal = decode(&envelope).expect_err("must refuse");
        assert!(refusal.contains(needle), "{envelope} refused {refusal:?}");
    }
}
