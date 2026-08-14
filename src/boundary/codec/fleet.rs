//! The armed loop family's half of the [`codec`](super) (VISION §4.3, bl-66fb):
//! arm and disarm. Split from the top-level codec per §12's line budget, beside
//! the monitor's own half; every encoder here is matched by a decoder and every
//! variant round-trips (the §8.5 parity tests).
//!
//! Strict, like the rest of the codec. `fleet` carries its project and its cap
//! as required fields: they are what the operator must choose, and an arm that
//! inferred either from an absence would be yog spending their money on its own
//! opinion. Disarming is its own `op` rather than a `fleet` with no cap — a
//! gesture is an instruction, never an absence to be read.

use serde_json::{Map, Value, json};

use crate::fleet::Verb;

use super::{Action, Gesture, str_of, usize_of};

/// The two loop `op`s. Named here because the codec, the line and the help
/// table must spell one thing one way.
pub(crate) const ARM: &str = "fleet";
pub(crate) const DISARM: &str = "disband";

/// One loop gesture as its envelope.
pub(super) fn encode(verb: &Verb) -> Value {
    match verb {
        Verb::Arm {
            workspace,
            project,
            cap,
        } => json!({ "op": ARM, "workspace": workspace,
                     "project": project, "cap": cap }),
        Verb::Disarm { workspace } => {
            json!({ "op": DISARM, "workspace": workspace })
        }
    }
}

/// The inverse. `op` is already known to be one of the two; anything else never
/// reaches here (the caller's table decides).
pub(super) fn decode(op: &str, o: &Map<String, Value>) -> Result<Gesture, String> {
    let workspace = str_of(o, "workspace")?;
    let verb = match op {
        ARM => Verb::Arm {
            workspace,
            project: str_of(o, "project")?,
            cap: usize_of(o, "cap")?,
        },
        _ => Verb::Disarm { workspace },
    };
    Ok(Gesture::Act(Action::Fleet(verb)))
}
