//! The V4 board reply's encoders (§8.5, VISION §5 V4): one row, and the §3.5
//! figure shape it carries twice — its own and, for an epic, its rollup.

use serde_json::{Map, Value, json};

use crate::board::BoardRow;
use crate::spend::{Attribution, Figure};

use super::super::codec::join_token;

/// The decoders, beside the encoders they undo (bl-7067).
pub(super) mod decode;

/// One V4 board row. The column is the answer's headline fact, so it leads;
/// `state` rides beside it because the two say different things (§3.5 binding
/// vs. the ladder). A figure encodes as its money string plus its raw
/// micro-USD, so a reader can render or arithmetic without re-deriving either.
pub(super) fn board_row(row: &BoardRow) -> Value {
    let mut map = Map::new();
    map.insert("id".to_owned(), json!(row.id));
    map.insert("column".to_owned(), json!(row.column.word()));
    map.insert("state".to_owned(), json!(join_token(row.state)));
    map.insert("title".to_owned(), json!(row.title));
    map.insert("priority".to_owned(), json!(row.priority));
    map.insert("project".to_owned(), json!(row.project));
    if let Some(ws) = &row.workspace {
        map.insert("workspace".to_owned(), json!(ws));
    }
    if let Some(claimant) = &row.claimant {
        map.insert("claimant".to_owned(), json!(claimant));
    }
    if let Some(parent) = &row.parent {
        map.insert("parent".to_owned(), json!(parent));
    }
    map.insert(
        "gates".to_owned(),
        json!(
            row.gates
                .iter()
                .map(|g| json!({ "id": g.id, "title": g.title, "mints": "close" }))
                .collect::<Vec<Value>>()
        ),
    );
    map.insert(
        "drones".to_owned(),
        json!(
            row.drones
                .iter()
                .map(|d| json!({ "root_id": d.root_id, "name": d.name }))
                .collect::<Vec<Value>>()
        ),
    );
    if let Some(figure) = &row.spend {
        map.insert("spend".to_owned(), figure_value(figure));
    }
    if let Some(figure) = &row.rollup {
        map.insert("rollup".to_owned(), figure_value(figure));
    }
    Value::Object(map)
}

/// One armed loop's facts (VISION §5 V4 item 2), all derived: the cap and its
/// project from the config entry, the count from the rows above, the tick from
/// the clock, the last act from the trail, the lease from the entry, and the
/// ceiling from §3.5's own gate asked over this workspace's spend.
///
/// **The tick is a period, not a countdown.** A level-triggered loop has no
/// phase on disk and must not grow one to render a clock face: what it can
/// truthfully say is how long its period is and how long ago it last changed
/// the world, and `last_act_secs_ago` is absent — not zero — for a loop that
/// never has.
pub(super) fn fleet_facts(facts: &crate::fleet::Facts) -> Value {
    let mut map = Map::new();
    // The §3.1 workspace leaf and the §5.1 #1 project name (REMOTE §8.1,
    // bl-ef16) — the same two words `board_row` above already spells, under the
    // same two keys the gestures take.
    map.insert("workspace".to_owned(), json!(facts.workspace));
    map.insert("project".to_owned(), json!(facts.project));
    map.insert("cap".to_owned(), json!(facts.cap));
    map.insert("count".to_owned(), json!(facts.count));
    map.insert("room".to_owned(), json!(facts.has_room()));
    map.insert("tick_secs".to_owned(), json!(facts.tick.as_secs()));
    if let Some(lease) = facts.lease {
        map.insert("lease_secs".to_owned(), json!(lease.as_secs()));
    }
    if let Some(secs) = facts.since_act {
        map.insert("last_act_secs_ago".to_owned(), json!(secs));
    }
    if let Some(ceiling) = &facts.ceiling {
        map.insert("ceiling".to_owned(), json!(ceiling));
    }
    map.insert("label".to_owned(), json!(facts.label()));
    Value::Object(map)
}

/// One §3.5 figure: tokens always, money when the price table has rates, and
/// the granularity clause when the figure is honest at less than it claims.
///
/// **Two shapes widened for bl-7067**, both because the answer could not be
/// read back as the answer that was given. `tokens` was the derived total
/// alone, which four ARCH §6 counters do not fit in; it is now the same object
/// the Steps rows already spell ([`spend_value`](crate::steps_view::wire::spend_value)),
/// total included, so nothing a reader had is lost. `attribution` was the
/// rendered clause alone — and `Conversations(1)` renders as no clause at all,
/// so "one stamped conversation" and "workspace-wide" were the same absence.
/// It is now the classification, with the clause riding beside it exactly as
/// `usd` rides beside `micro_usd`: derived text next to the fact it derives
/// from, never instead of it.
pub(crate) fn figure_value(figure: &Figure) -> Value {
    let mut map = Map::new();
    map.insert(
        "tokens".to_owned(),
        crate::steps_view::wire::spend_value(&figure.tokens),
    );
    if let Some(cost) = figure.cost {
        map.insert("usd".to_owned(), json!(cost.usd()));
        map.insert("micro_usd".to_owned(), json!(cost.micro_usd));
        map.insert("unpriced_tokens".to_owned(), json!(cost.unpriced_tokens));
    }
    map.insert("attribution".to_owned(), attribution_value(figure));
    Value::Object(map)
}

/// The §3.5 attribution: what the figure sums over, and the clause it says out
/// loud when that is more than its seat already claims.
fn attribution_value(figure: &Figure) -> Value {
    let mut map = Map::new();
    match figure.attribution {
        Attribution::Conversations(count) => {
            map.insert("kind".to_owned(), json!("conversations"));
            map.insert("count".to_owned(), json!(count));
        }
        Attribution::Workspace => {
            map.insert("kind".to_owned(), json!("workspace"));
        }
    }
    if let Some(note) = figure.attribution.note() {
        map.insert("label".to_owned(), json!(note.label));
    }
    Value::Object(map)
}
