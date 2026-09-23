//! **The envelopes of a REMOTE device's own acts** — enrollment (REMOTE §1.4
//! as amended, §8.4; bl-f4e3) and the sign-in (REMOTE §8.3, bl-c285) — their
//! own family file on the seam [`action::device`](crate::boundary::action::device)
//! already draws for the prose, split out of the roster at §12's cap (bl-b680).
//! Both are bodies rather than rows — an enrollment assembles four keys and an
//! optional fifth — and the roster still names each, so there is exactly one
//! place to learn which act is said where.

use serde_json::{Map, Value, json};

use super::{Action, opt_str_of, str_of};

/// The sign-in act's op token (bl-c285), named once for both directions and
/// for the line that types it.
pub(crate) const LOGIN: &str = "login";

/// Enrollment's op token, named once so the envelope, the line and the help
/// page cannot spell it three ways (REMOTE §1.4 as amended, bl-f4e3).
pub(crate) const ENROLL: &str = "enroll";

/// REMOTE §1.4's enrollment (bl-f4e3): the workspace it seats the new client
/// in, the common name its certificate will carry, and the grade — in the one
/// grade vocabulary `Grade::word`/`of` spell both ways. `address` rides only
/// when the operator stated one (bl-fec6): absent is "the address this engine
/// wrote for itself", which the executor reads, and a null would be a second
/// spelling of the same absence.
pub(super) fn encode_enroll(request: &crate::registry::enroll::Request) -> Value {
    let mut map = Map::new();
    map.insert("op".to_owned(), json!(ENROLL));
    map.insert("workspace".to_owned(), json!(request.workspace));
    map.insert("name".to_owned(), json!(request.name));
    map.insert("grade".to_owned(), json!(request.grade.word()));
    if let Some(address) = &request.address {
        map.insert("address".to_owned(), json!(address));
    }
    Value::Object(map)
}

/// The §8.3 sign-in (REMOTE §8.3, bl-c285): the wall it runs in and the
/// provider row it signs into, and nothing else — the flow is the row's own
/// capability, never a field a seat may spell (DESIGN §8.3 rule 1).
pub(super) fn encode_login(workspace: &str, provider: &str) -> Value {
    json!({ "op": LOGIN, "workspace": workspace, "provider": provider })
}

/// The inverse of both, by the op the roster matched.
pub(super) fn decode(op: &str, o: &Map<String, Value>) -> Result<Action, String> {
    if op == LOGIN {
        // Both halves required (REMOTE §8.3): a sign-in that guessed either
        // would write a credential into the wrong sphere, or into the right
        // one for a row nobody named.
        return Ok(Action::Login {
            workspace: str_of(o, "workspace")?,
            provider: str_of(o, "provider")?,
        });
    }
    // REMOTE §1.4's enrollment (bl-f4e3). Every field is required, the grade
    // included: a default here would be a promotion or a demotion nobody
    // typed, and §4.2 forbids the first outright.
    Ok(Action::Enroll(crate::registry::enroll::Request {
        workspace: str_of(o, "workspace")?,
        name: str_of(o, "name")?,
        grade: grade_of(&str_of(o, "grade")?)?,
        // …and the one OPTIONAL field (bl-fec6): the address the device will
        // dial, absent when the engine's own is right. A present field must
        // still be a string, so a mistyped one refuses here rather than
        // reaching a QR.
        address: opt_str_of(o, "address")?,
    }))
}

/// One grade word read back, or the refusal naming the token (bl-f4e3) — the
/// registry's own table, spent here and by the line, so the two serializations
/// share one vocabulary rather than each carrying a copy.
pub(crate) fn grade_of(word: &str) -> Result<crate::registry::Grade, String> {
    crate::registry::Grade::of(word).ok_or_else(|| format!("unknown grade {word:?}"))
}
