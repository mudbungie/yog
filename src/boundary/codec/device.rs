//! **The envelope of a REMOTE device's own act** — the sign-in (REMOTE §8.3,
//! bl-c285) — its own family file on the seam
//! [`action::device`](crate::boundary::action::device) already draws for the
//! prose, split out of the roster at §12's cap (bl-b680), beside the one grade
//! vocabulary the offer family reads.

use serde_json::{Map, Value, json};

use super::{Action, opt_str_of, str_of};

/// The sign-in act's op token (bl-c285), named once for both directions and
/// for the line that types it.
pub(crate) const LOGIN: &str = "login";

/// The §8.3 sign-in (REMOTE §8.3, bl-c285): the wall it runs in and the
/// provider row it signs into — the flow is the row's own capability, never a
/// field a seat may spell (DESIGN §8.3 rule 1). `code` (edition 31, bl-0127)
/// is written only when there is one: absent, the act starts the sign-in, and
/// that is the reading every server gave the op before the key existed.
pub(super) fn encode_login(workspace: &str, provider: &str, code: Option<&str>) -> Value {
    let mut act = json!({ "op": LOGIN, "workspace": workspace, "provider": provider });
    if let (Some(code), Some(o)) = (code, act.as_object_mut()) {
        o.insert("code".to_owned(), json!(code));
    }
    act
}

/// The inverse. The pair is required (REMOTE §8.3): a sign-in that guessed
/// either half would write a credential into the wrong sphere, or into the
/// right one for a row nobody named. The code is optional and absent is the
/// start.
pub(super) fn decode(o: &Map<String, Value>) -> Result<Action, String> {
    Ok(Action::Login {
        workspace: str_of(o, "workspace")?,
        provider: str_of(o, "provider")?,
        code: opt_str_of(o, "code")?,
    })
}

/// One grade word read back, or the refusal naming the token (bl-f4e3) — the
/// registry's own table, spent by the offer's envelope and by its line, so
/// the two serializations share one vocabulary rather than each carrying a
/// copy.
pub(crate) fn grade_of(word: &str) -> Result<crate::registry::Grade, String> {
    crate::registry::Grade::of(word).ok_or_else(|| format!("unknown grade {word:?}"))
}
