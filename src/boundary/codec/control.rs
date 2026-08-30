//! The capability family's half of the [`codec`](super) (VISION §4.11, §8.6):
//! the hold answer. Split from the top-level codec beside the §4.9 monitor's
//! own half, per §12's line budget.
//!
//! Strict, like the rest of the codec: the verdict is a required field, and it
//! must be one of the control's three words. There is no default — an answer
//! that guessed its own verdict would be the boundary inventing an operator's
//! decision, which is the one thing a gesture must never do.
//!
//! The `tool_use` id is **not** a field, in either direction. It is derived
//! from `refs/litany/held/<agent>` at fire time (§8.6): a headless caller that
//! had to quote an id would be quoting a fact it read a tick ago, and the
//! whole point of scoping the answer to a provider-unique id is that it names
//! what is parked *now*.

use serde_json::{Map, Value, json};

use crate::control::judge::Ruling;

use super::{Action, Gesture, str_of};

/// One hold answer as its envelope.
pub(super) fn encode(workspace: &str, agent: &str, ruling: Ruling) -> Value {
    json!({ "op": "answer", "workspace": workspace,
            "agent": agent, "verdict": ruling.word() })
}

/// The inverse. `op` is already known to be `answer`; the verdict is checked
/// here, where the refusal can name what was said and what is allowed.
pub(super) fn decode(o: &Map<String, Value>) -> Result<Gesture, String> {
    let workspace = str_of(o, "workspace")?;
    let agent = str_of(o, "agent")?;
    let word = str_of(o, "verdict")?;
    let ruling = Ruling::of(&word)
        .ok_or_else(|| format!("answer: unknown verdict {word:?}; say pass, hold or refuse"))?;
    Ok(Gesture::Act(Action::AnswerHold {
        workspace,
        agent,
        ruling,
    }))
}

/// One floor gesture as its envelope (VISION §4.9's fifth rung). **Two ops for
/// one variant**, exactly as the monitor's arm and disarm are: raising and
/// lowering are two instructions, and a boolean field would make the second one
/// the absence of the first.
pub(super) fn encode_floor(workspace: &str, agent: &str, raised: bool) -> Value {
    let op = if raised { "revoke" } else { "restore" };
    json!({ "op": op, "workspace": workspace, "agent": agent })
}

/// The inverse. `op` is already known to be one of the two; the direction it
/// names is the whole difference between them.
pub(super) fn decode_floor(op: &str, o: &Map<String, Value>) -> Result<Gesture, String> {
    Ok(Gesture::Act(Action::Floor {
        workspace: str_of(o, "workspace")?,
        agent: str_of(o, "agent")?,
        raised: op == "revoke",
    }))
}
