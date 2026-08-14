//! The §4.10 fan's envelopes: both round-trip, the optional obligation half
//! reads both ways, and a malformed one refuses by name.

use serde_json::json;

use super::{p, rt};
use crate::boundary::codec::decode;
use crate::boundary::{Action, Gesture};
use crate::fan::Obligation;
use crate::opslog::Origin;
use crate::start::Prepared;

fn prepared(binding: Option<&str>) -> Prepared {
    Prepared {
        name: "cobalt-gecko".into(),
        workspace: p("/ws"),
        binding: binding.map(p),
        goal: "Ball bl-1f2a: do it".into(),
        origin: Origin::Balls,
    }
}

fn obligation(ball: Option<&str>) -> Obligation {
    Obligation {
        project: p("/dev/proj"),
        ball: ball.map(str::to_owned),
    }
}

/// A ball fan and the bare project-repo fan are one envelope with and without
/// its `ball` — absence is a value, and it survives the wire as one.
#[test]
fn both_fan_envelopes_round_trip() {
    for ball in [Some("bl-1f2a"), None] {
        rt(Gesture::Act(Action::Fan {
            prepared: prepared(Some("/claim")),
            obligation: obligation(ball),
            n: 3,
        }));
        rt(Gesture::Act(Action::Retire {
            obligation: obligation(ball),
            handle: "at-0badcafe".into(),
        }));
    }
    // N of one is a lawful fan (the ordinary path), and so is N of none.
    for n in [0, 1] {
        rt(Gesture::Act(Action::Fan {
            prepared: prepared(None),
            obligation: obligation(Some("bl-1f2a")),
            n,
        }));
    }
}

/// A fan with no `ball` field fans the project's own integration branch: the
/// bare obligation says so by saying nothing, as `--body` does.
#[test]
fn a_fan_without_a_ball_field_is_the_bare_obligation() {
    let envelope = json!({"op": "fan", "project": "/dev/proj", "n": 2,
                          "prepared": {"name": "cobalt-gecko", "workspace": "/ws",
                                       "binding": null, "goal": "g", "origin": "balls"}});
    assert_eq!(
        decode(&envelope),
        Ok(Gesture::Act(Action::Fan {
            prepared: Prepared {
                goal: "g".into(),
                ..prepared(None)
            },
            obligation: obligation(None),
            n: 2,
        })),
    );
    assert!(
        !encoded(&envelope).contains("\"ball\""),
        "and it re-encodes without the field it never had"
    );
}

/// Re-encode whatever an envelope decodes to, as text.
fn encoded(envelope: &serde_json::Value) -> String {
    let Ok(gesture) = decode(envelope) else {
        return String::new();
    };
    crate::boundary::codec::encode(&gesture).to_string()
}

#[test]
fn a_malformed_fan_or_retirement_refuses_with_a_reason() {
    let prepared = json!({"name": "n", "workspace": "/ws", "binding": null,
                          "goal": "g", "origin": "balls"});
    let cases: Vec<(serde_json::Value, &str)> = vec![
        (
            json!({"op": "fan", "project": "/dev/proj", "prepared": prepared}),
            "field \"n\"",
        ),
        (
            json!({"op": "fan", "project": "/dev/proj", "n": 2}),
            "fan: missing prepared",
        ),
        (
            json!({"op": "fan", "n": 2, "prepared": prepared}),
            "field \"project\"",
        ),
        (
            json!({"op": "fan", "project": "/p", "n": 2, "ball": 7, "prepared": prepared}),
            // `str_of`'s wording since bl-7067 folded every optional reader
            // onto one `opt` — the promise was always that it NAMES the field.
            "field \"ball\"",
        ),
        (
            json!({"op": "fan", "project": "/p", "n": -1, "prepared": prepared}),
            "field \"n\"",
        ),
        (
            json!({"op": "retire", "project": "/dev/proj"}),
            "field \"handle\"",
        ),
    ];
    for (envelope, needle) in cases {
        let refusal = decode(&envelope).expect_err("must refuse");
        assert!(refusal.contains(needle), "{envelope} refused {refusal:?}");
    }
}
