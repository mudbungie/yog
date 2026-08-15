//! REMOTE §5's tool-host presentation, in the deposit envelope (bl-4e08) —
//! beside the other families' spellings for their reason exactly: a family's
//! grammar lives in its own file, and the action roster stays a roster.
//!
//! **The element spelling is not here.** It is
//! [`registry::tools`](crate::registry::tools), the same encoder the stored
//! document spends, because a presented set and a stored one written twice
//! would drift within a week (§8.5's single-source discipline).
//!
//! **No client field**, and that is the gesture (REMOTE §5): the identity a set
//! lands under is the intake's, and one on the wire would let a connection
//! write another client's set.

use serde_json::{Map, Value, json};

use crate::boundary::Action;
use crate::registry::tools;

/// The op token, named once so the encoder's word and the decoder's arm are
/// one fact.
pub(super) const ADVERTISE: &str = "advertise";

/// One presentation as its envelope.
pub(super) fn encode(set: &[tools::Tool]) -> Value {
    json!({ "op": ADVERTISE, "tools": tools::encode(set) })
}

/// Read one back. `tools` is required: a presentation with no set is not the
/// empty set — it is an envelope that failed to say anything.
pub(super) fn decode(o: &Map<String, Value>) -> Result<Action, String> {
    let set = o.get("tools").ok_or("advertise: missing tools")?;
    Ok(Action::Advertise {
        tools: tools::decode(set)?,
    })
}
