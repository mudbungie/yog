//! The offer family's envelopes (REMOTE §15.1; bl-9ea6): four requests and the
//! four replies they earn, each spelled once and read back by its inverse.
//! Written beside the vocabulary rather than in the boundary's codec, as
//! [`login::wire`](crate::login::wire) is: the roster there holds the one line
//! that names each, and the spelling is this family's own.

use super::{Answer, Row, State, Verb};
use crate::boundary::codec::fields::{i64_of, list_of, opt_str_of, str_of};
use crate::boundary::codec::grade_of;
use serde_json::{Map, Value, json};

/// The four request ops.
pub(crate) const OFFER: &str = "offer";
pub(crate) const OFFERS: &str = "offers";
pub(crate) const APPROVE: &str = "approve";
pub(crate) const DECLINE: &str = "decline";

/// The four reply kinds. `offers` is both the op and the kind it earns.
pub(crate) const OFFERED: &str = "offered";
pub(crate) const APPROVED: &str = "approved";
pub(crate) const DECLINED: &str = "declined";

/// One act as its request envelope.
pub(crate) fn encode(verb: &Verb) -> Value {
    match verb {
        Verb::Mint {
            workspace,
            name,
            grade,
        } => json!({ "op": OFFER, "workspace": workspace, "name": name,
                     "grade": grade.word() }),
        Verb::List { workspace } => json!({ "op": OFFERS, "workspace": workspace }),
        Verb::Approve { workspace, name } => {
            json!({ "op": APPROVE, "workspace": workspace, "name": name })
        }
        Verb::Decline { workspace, name } => {
            json!({ "op": DECLINE, "workspace": workspace, "name": name })
        }
    }
}

/// The inverse, by the op the roster matched. Every field is required, the
/// grade included: a default would be a promotion or a demotion nobody typed.
pub(crate) fn decode(op: &str, o: &Map<String, Value>) -> Result<Verb, String> {
    let workspace = str_of(o, "workspace")?;
    Ok(match op {
        OFFER => Verb::Mint {
            workspace,
            name: str_of(o, "name")?,
            grade: grade_of(&str_of(o, "grade")?)?,
        },
        OFFERS => Verb::List { workspace },
        APPROVE => Verb::Approve {
            workspace,
            name: str_of(o, "name")?,
        },
        _ => Verb::Decline {
            workspace,
            name: str_of(o, "name")?,
        },
    })
}

/// One answer as its reply envelope. A row's `word` is **absent** until a
/// device has claimed: a reader must not have to tell "no word yet" from an
/// empty one.
pub(crate) fn reply(answer: &Answer) -> Value {
    match answer {
        Answer::Offered { url, expires } => {
            json!({ "ok": true, "kind": OFFERED, "url": url, "expires": expires })
        }
        Answer::Offers(rows) => {
            json!({ "ok": true, "kind": OFFERS, "rows": rows.iter().map(row).collect::<Vec<_>>() })
        }
        Answer::Approved => json!({ "ok": true, "kind": APPROVED }),
        Answer::Declined => json!({ "ok": true, "kind": DECLINED }),
    }
}

fn row(row: &Row) -> Value {
    let mut map = Map::new();
    map.insert("name".to_owned(), json!(row.name));
    map.insert("grade".to_owned(), json!(row.grade.word()));
    map.insert("state".to_owned(), json!(row.state.word()));
    map.insert("expires".to_owned(), json!(row.expires));
    if let Some(word) = &row.word {
        map.insert("word".to_owned(), json!(word));
    }
    Value::Object(map)
}

/// The inverse of [`reply`], or `None` for a kind that is not this family's.
pub(crate) fn answer_of(kind: &str, o: &Map<String, Value>) -> Option<Result<Answer, String>> {
    Some(match kind {
        OFFERED => offered(o),
        OFFERS => list_of(o, "rows", row_of).map(Answer::Offers),
        APPROVED => Ok(Answer::Approved),
        DECLINED => Ok(Answer::Declined),
        _ => return None,
    })
}

fn offered(o: &Map<String, Value>) -> Result<Answer, String> {
    Ok(Answer::Offered {
        url: str_of(o, "url")?,
        expires: i64_of(o, "expires")?,
    })
}

fn row_of(v: &Value) -> Result<Row, String> {
    let o = v.as_object().ok_or("offers: a row is not an object")?;
    let state = str_of(o, "state")?;
    Ok(Row {
        name: str_of(o, "name")?,
        grade: grade_of(&str_of(o, "grade")?)?,
        state: State::of(&state).ok_or_else(|| format!("unknown offer state {state:?}"))?,
        expires: i64_of(o, "expires")?,
        word: opt_str_of(o, "word")?,
    })
}
