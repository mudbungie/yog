//! The attempt's envelope (VISION V2, bl-dc0c): `{"op":"fork", …}`.
//!
//! Its own module for the same reason the §9 config family has one — the
//! envelope carries a **list** (`skills`), and reading a list strictly needs a
//! reader the scalar fields do not: absence is the empty list, but a present
//! field of the wrong shape is a refusal, like every other mistyped value.
//!
//! **There is no cohort envelope.** A fan is N of these, so the shape carries
//! no count and no group id: there is nothing to name ([`crate::fork`]).

use serde_json::{Map, Value, json};

use crate::boundary::Action;
use crate::fork::Attempt;

use super::str_of;

/// Encode one attempt.
pub(super) fn encode(workspace: &str, parent: &str, attempt: &Attempt, goal: &str) -> Value {
    json!({ "op": "fork", "workspace": workspace, "parent": parent,
            "from": attempt.from, "role": attempt.role, "skills": attempt.skills,
            "goal": goal })
}

/// Decode one attempt, strictly.
pub(super) fn decode(o: &Map<String, Value>) -> Result<Action, String> {
    Ok(Action::Fork {
        workspace: str_of(o, "workspace")?,
        parent: str_of(o, "parent")?,
        attempt: Attempt {
            from: str_of(o, "from")?,
            role: str_of(o, "role")?,
            skills: words_of(o, "skills")?,
        },
        goal: str_of(o, "goal")?,
    })
}

/// An optional array-of-strings field, read as a list. Absent is the empty
/// list — an attempt that pins nothing says so by saying nothing — but a
/// present field that is not an array of strings is a refusal, because a
/// gesture is an instruction and a mistyped one must not be half-obeyed.
fn words_of(obj: &Map<String, Value>, key: &str) -> Result<Vec<String>, String> {
    let Some(value) = obj.get(key) else {
        return Ok(Vec::new());
    };
    let wrong = || format!("field {key:?} must be an array of strings");
    value
        .as_array()
        .ok_or_else(wrong)?
        .iter()
        .map(|item| item.as_str().map(str::to_owned).ok_or_else(wrong))
        .collect()
}
