//! The alignment monitor family's half of the [`codec`](super) (VISION §4.9,
//! bl-8da1): arm, disarm and flag. Split from the top-level codec per §12's
//! line budget, beside the §9 family's own half; every encoder here is matched
//! by a decoder and every variant round-trips (the §8.5 parity tests).
//!
//! Strict, like the rest of the codec. `arm` carries its model pin as a
//! required field rather than an optional one, because arming with no pin is a
//! different instruction — `disarm` — and a gesture is an instruction, never an
//! observation to be inferred from an absence.

use serde_json::{Map, Value, json};

use crate::monitor::Verb;

use super::{Action, Gesture, encode_path, path_of, str_of};

/// One monitor gesture as its envelope. The `op` is the verb the help table
/// and the line spell, so the three serializations name one thing.
pub(super) fn encode(verb: &Verb) -> Value {
    match verb {
        Verb::Arm { workspace, model } => {
            json!({ "op": "arm", "workspace": encode_path(workspace), "model": model })
        }
        Verb::Disarm { workspace } => {
            json!({ "op": "disarm", "workspace": encode_path(workspace) })
        }
        Verb::Flag {
            workspace,
            agent,
            reason,
        } => json!({ "op": "flag", "workspace": encode_path(workspace),
                     "agent": agent, "reason": reason }),
    }
}

/// The inverse. `op` is already known to be one of the three; anything else
/// never reaches here (the caller's table decides).
pub(super) fn decode(op: &str, o: &Map<String, Value>) -> Result<Gesture, String> {
    let workspace = path_of(o, "workspace")?;
    let verb = match op {
        "arm" => Verb::Arm {
            workspace,
            model: str_of(o, "model")?,
        },
        "disarm" => Verb::Disarm { workspace },
        _ => Verb::Flag {
            workspace,
            agent: str_of(o, "agent")?,
            reason: str_of(o, "reason")?,
        },
    };
    Ok(Gesture::Act(Action::Monitor(verb)))
}
