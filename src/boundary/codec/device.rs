//! **The envelope of a REMOTE device's own act** — the sign-in (REMOTE §8.3,
//! bl-c285) — its own family file on the seam
//! [`action::device`](crate::boundary::action::device) already draws for the
//! prose, split out of the roster at §12's cap (bl-b680), beside the one grade
//! vocabulary the offer family reads.

use serde_json::{Map, Value, json};

use super::{Action, str_of};

/// The sign-in act's op token (bl-c285), named once for both directions and
/// for the line that types it.
pub(crate) const LOGIN: &str = "login";

/// The §8.3 sign-in (REMOTE §8.3, bl-c285): the wall it runs in and the
/// provider row it signs into, and nothing else — the flow is the row's own
/// capability, never a field a seat may spell (DESIGN §8.3 rule 1).
pub(super) fn encode_login(workspace: &str, provider: &str) -> Value {
    json!({ "op": LOGIN, "workspace": workspace, "provider": provider })
}

/// The inverse. Both halves required (REMOTE §8.3): a sign-in that guessed
/// either would write a credential into the wrong sphere, or into the right
/// one for a row nobody named.
pub(super) fn decode(o: &Map<String, Value>) -> Result<Action, String> {
    Ok(Action::Login {
        workspace: str_of(o, "workspace")?,
        provider: str_of(o, "provider")?,
    })
}

/// One grade word read back, or the refusal naming the token (bl-f4e3) — the
/// registry's own table, spent by the offer's envelope and by its line, so
/// the two serializations share one vocabulary rather than each carrying a
/// copy.
pub(crate) fn grade_of(word: &str) -> Result<crate::registry::Grade, String> {
    crate::registry::Grade::of(word).ok_or_else(|| format!("unknown grade {word:?}"))
}
