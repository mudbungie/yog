//! The §11 balls section's own spelling (REMOTE §9.7, bl-b4b5) — both
//! directions of a [`BoundBall`], cut off the reply roster at §12's budget on
//! the seam [`agent`](super::agent) and [`queue`](super::queue) already take:
//! one payload whose rows are its own vocabulary.
//!
//! It owns no token table. The §3.5 join state is
//! [`join_token`](crate::boundary::codec::join_token)'s, and the figure is the
//! V4 board's ([`board::figure_value`](super::board)) — read from there rather
//! than restated, because a second spelling of one figure is exactly the drift
//! the round-trip test exists to catch.

use serde_json::{Map, Value, json};

use crate::boundary::codec::fields::{opt_str_of, str_of};
use crate::boundary::codec::{join_token, parse_join};
use crate::nav::BoundBall;

use super::board::decode::figure;
use super::board::figure_value;

/// One bound ball: what it is, what it says, where its verbs run, and what it
/// has cost. `badge` is absent — not empty — for a state that needs none (a
/// plain Bound row), which is the roster's own reading of "nothing to say".
pub(super) fn bound_ball(ball: &BoundBall) -> Value {
    let mut map = Map::new();
    map.insert("id".to_owned(), json!(ball.id));
    if let Some(badge) = &ball.badge {
        map.insert("badge".to_owned(), json!(badge));
    }
    map.insert("project".to_owned(), json!(ball.project));
    map.insert("owner".to_owned(), json!(ball.owner));
    map.insert("state".to_owned(), json!(join_token(ball.state)));
    map.insert("spend".to_owned(), figure_value(&ball.spend));
    Value::Object(map)
}

/// The same row read back, strict: every field the encoder always writes is
/// required, and an unknown join token refuses naming the offender.
pub(super) fn bound_ball_of(v: &Value) -> Result<BoundBall, String> {
    let o = v.as_object().ok_or("ball row: not an object")?;
    Ok(BoundBall {
        id: str_of(o, "id")?,
        badge: opt_str_of(o, "badge")?,
        project: str_of(o, "project")?,
        owner: str_of(o, "owner")?,
        state: parse_join(&str_of(o, "state")?)?,
        spend: figure(o.get("spend").ok_or("ball row: missing spend")?)?,
    })
}
