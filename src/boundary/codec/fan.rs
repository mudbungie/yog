//! The mutating fan's two envelopes (VISION §4.10, bl-8746): `{"op":"fan", …}`
//! and `{"op":"retire", …}`.
//!
//! Its own module for the reason the §9 config family and the attempt have one
//! — both envelopes carry the **optional** `ball`, whose absence is a value (the
//! bare project-repo fan targets the integration branch) rather than a
//! malformed gesture, and `fan` carries a nested `prepared` body besides.
//!
//! **There is still no cohort envelope** ([`crate::fan::cohort`]). `fan` names a
//! count, never a group: N attempts off one pinned target tip are one act
//! because the shared base is what makes them siblings, and the group they form
//! is read back off the trail, not written down here.

use serde_json::{Map, Value, json};

use crate::boundary::Action;

use crate::fan::Obligation;

use super::start::{encode_path, encode_prepared};
use super::{decode_prepared, opt_field, opt_str_of, path_of, str_of, usize_of};

/// The fan's own `op` token — matched in [`super::decode`]'s roster.
pub(super) const FAN: &str = "fan";
/// The retirement's.
pub(super) const RETIRE: &str = "retire";

/// Encode a fan: the prepared start to spend, the obligation, and N.
pub(super) fn encode(
    prepared: &crate::start::Prepared,
    obligation: &Obligation,
    n: usize,
) -> Value {
    let mut map = obligation_map(FAN, obligation);
    map.insert("prepared".to_owned(), encode_prepared(prepared));
    map.insert("n".to_owned(), json!(n));
    Value::Object(map)
}

/// Encode a retirement: the obligation and the opaque handle to retire.
pub(super) fn encode_retire(obligation: &Obligation, handle: &str) -> Value {
    let mut map = obligation_map(RETIRE, obligation);
    map.insert("handle".to_owned(), json!(handle));
    Value::Object(map)
}

/// The half both envelopes share: the op word and the obligation. `ball` stays
/// **absent** rather than null when there is none — the bare project-repo fan
/// says so by saying nothing, exactly as `--body` does.
fn obligation_map(op: &str, obligation: &Obligation) -> Map<String, Value> {
    let mut map = Map::new();
    map.insert("op".to_owned(), json!(op));
    map.insert("project".to_owned(), encode_path(&obligation.project));
    opt_field(&mut map, "ball", obligation.ball.as_ref());
    map
}

/// Decode either envelope, strictly. `op` has already been matched.
pub(super) fn decode(op: &str, o: &Map<String, Value>) -> Result<Action, String> {
    let obligation = Obligation {
        project: path_of(o, "project")?,
        ball: opt_str_of(o, "ball")?,
    };
    if op == RETIRE {
        return Ok(Action::Retire {
            obligation,
            handle: str_of(o, "handle")?,
        });
    }
    Ok(Action::Fan {
        prepared: decode_prepared(o.get("prepared").ok_or("fan: missing prepared")?)?,
        obligation,
        n: usize_of(o, "n")?,
    })
}
