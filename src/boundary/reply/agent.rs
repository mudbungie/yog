//! The §11 conversation seat's own spelling (REMOTE §9.4, bl-1eb0) — both
//! directions of [`AgentView`], cut off the reply roster at §12's budget on the
//! seam [`search`](super::search) and [`queue`](super::queue) already take: one
//! payload whose rows are its own vocabulary.
//!
//! The token tables this reply owns are the §6 **mark** set and the §5.1 #28b
//! **doing** set (bl-296f). The §5.1 agent state and the §11 flight class are
//! the conversation list's already ([`rows`](super::rows)), read from there
//! rather than restated — a second spelling of one vocabulary is the drift the
//! round-trip test exists to catch.
//!
//! Absent-not-null throughout: an unmarked conversation carries no `marks` key
//! and one at rest no `flight`, because a reader must never have to tell an
//! empty list from a fact the encoder declined to state.

use serde_json::{Map, Value, json};

use crate::boundary::answer::agent::AgentView;
use crate::boundary::codec::fields::{
    bool_of, list_of, opt, opt_val, pick, str_of, strings_of, u64_of,
};
use crate::context::Fullness;
use crate::control::hold::Held;
use crate::git_tree::AgentMark;
use crate::nav::convs::{Doing, FlightStrip, Seat};

use super::board::decode::figure;
use super::board::figure_value;
use super::rows::decode::{FLIGHTS, state_of};
use super::rows::{flight_token, state_token};

/// The §6 marks in badge order — the table both directions read.
const MARKS: [(&str, AgentMark); 5] = [
    ("notified", AgentMark::Notified),
    ("budget-exhausted", AgentMark::BudgetExhausted),
    ("conflicted", AgentMark::Conflicted),
    ("held", AgentMark::Held),
    ("abandoned", AgentMark::Abandoned),
];

/// The §5.1 #28b per-agent activity vocabulary — the live mark's whole
/// alphabet, and the one place it is spelled for the wire. It is **not**
/// [`FLIGHTS`]: that is the §5.1 #28 class of a *conversation*, and this is one
/// agent's own state, of which the three model-call rungs fold into that
/// class's `inference`.
const DOINGS: [(&str, Doing); 5] = [
    ("waiting", Doing::Waiting),
    ("thinking", Doing::Thinking),
    ("inference", Doing::Inference),
    ("tools", Doing::Tools),
    ("idle", Doing::Idle),
];

/// The whole seat as one object: the identity it echoes back, the §3.3 name and
/// its rung, the tip the config derivations take, the §3.5 state, the §6 marks,
/// the §5.1 #28 flight class, and the two §8.2 gates.
pub(super) fn reply(view: &AgentView) -> Value {
    let mut map = Map::new();
    map.insert("ok".to_owned(), json!(true));
    map.insert("kind".to_owned(), json!("agent"));
    map.insert("agent".to_owned(), json!(view.agent_id));
    map.insert("root".to_owned(), json!(view.root));
    if !view.ancestors.is_empty() {
        map.insert("ancestors".to_owned(), json!(view.ancestors));
    }
    map.insert("display".to_owned(), json!(view.name));
    map.insert("display_only".to_owned(), json!(view.display_only));
    map.insert("tip".to_owned(), json!(view.tip));
    map.insert("state".to_owned(), json!(state_token(view.state)));
    // The *why* beside the state (bl-b43b, §5.1 #9's own shape): the badge set
    // is frozen at four, so a conversation refused at the provider rung comes
    // to rest `stopped` exactly as an operator's own `/stop` does, and this is
    // what tells the two apart on the surface whose whole job is *what it is
    // doing, what may be done to it*.
    map.insert("refused".to_owned(), json!(view.refused));
    if !view.marks.is_empty() {
        let marks: Vec<&str> = view.marks.iter().copied().map(mark_token).collect();
        map.insert("marks".to_owned(), json!(marks));
    }
    if let Some(flight) = view.flight {
        map.insert("flight".to_owned(), json!(flight_token(flight)));
    }
    if let Some(held) = &view.held {
        // litany's own three keys — the blob's spelling, not a second one
        // (`control::hold::parse` reads exactly these).
        map.insert(
            "held".to_owned(),
            json!({
                "tool_use_id": held.tool_use_id,
                "tool": held.tool,
                "reason": held.reason,
            }),
        );
    }
    map.insert("present".to_owned(), json!(view.present));
    map.insert("nudgeable".to_owned(), json!(view.nudgeable));
    map.insert("stoppable".to_owned(), json!(view.stoppable));
    map.insert("stop_children".to_owned(), json!(view.stop_children));
    // The §11 live mark (bl-296f): one entry per agent in the conversation,
    // named and with what it is doing. Absent for the resting mark, for this
    // file's absent-not-null rule.
    if !view.seats.is_empty() {
        let seats: Vec<Value> = view
            .seats
            .iter()
            .map(|seat| json!({ "name": seat.name, "doing": doing_token(seat.doing) }))
            .collect();
        map.insert("seats".to_owned(), json!(seats));
    }
    // The §11 in-flight strip (bl-296f): the class the §5.1 #28 vocabulary
    // above already spells, and the live characteristics as the one rendered
    // run the seat paints. The characteristics cross as **text** rather than as
    // their segments because they are prose assembled by one derivation
    // (`nav::convs::strip`) with per-segment omission rules of its own; a wire
    // spelling of the parts would be a second place that decides what a strip
    // says.
    if let Some(strip) = &view.strip {
        map.insert(
            "strip".to_owned(),
            json!({ "class": flight_token(strip.class), "facts": strip.facts }),
        );
    }
    // The §3.5 figure in the board's own spelling (bl-b4b5) — always present,
    // because a conversation that has spent nothing has spent zero and that is
    // a reading. The §5.1 #35 fullness beside it is absent when nothing
    // measured can be said, which is not the same as a context at 0%.
    map.insert("spend".to_owned(), figure_value(&view.spend));
    if let Some(full) = &view.context {
        map.insert(
            "context".to_owned(),
            json!({
                "model": full.model,
                "prompt_tokens": full.prompt_tokens,
                "window": full.window,
                "percent": full.percent(),
            }),
        );
    }
    Value::Object(map)
}

/// The same object read back, strict: every field this encoder always writes is
/// required, and an unknown mark or state token refuses naming the offender.
pub(super) fn view_of(o: &Map<String, Value>) -> Result<AgentView, String> {
    let marks: Vec<AgentMark> = opt(o, "marks", strings_of)?
        .unwrap_or_default()
        .iter()
        .map(|word| mark_of(word))
        .collect::<Result<_, String>>()?;
    Ok(AgentView {
        agent_id: str_of(o, "agent")?,
        root: str_of(o, "root")?,
        ancestors: opt(o, "ancestors", strings_of)?.unwrap_or_default(),
        name: str_of(o, "display")?,
        display_only: bool_of(o, "display_only")?,
        tip: str_of(o, "tip")?,
        state: state_of(o)?,
        refused: bool_of(o, "refused")?,
        marks,
        held: opt_val(o, "held", held_of)?,
        flight: opt(o, "flight", |o, k| pick(o, k, &FLIGHTS))?,
        present: bool_of(o, "present")?,
        nudgeable: bool_of(o, "nudgeable")?,
        stoppable: bool_of(o, "stoppable")?,
        stop_children: bool_of(o, "stop_children")?,
        seats: opt(o, "seats", |o, k| list_of(o, k, seat_of))?.unwrap_or_default(),
        strip: opt_val(o, "strip", strip_of)?,
        spend: figure(o.get("spend").ok_or("agent: missing spend")?)?,
        context: opt_val(o, "context", context_of)?,
    })
}

/// The §5.1 #35 fullness, read back. `percent` is dropped for the reason the
/// figure's `usd` is: it is [`Fullness::percent`]'s rendering of the two
/// numbers beside it, which ride here in full.
fn context_of(v: &Value) -> Result<Fullness, String> {
    let o = v.as_object().ok_or("context: not an object")?;
    Ok(Fullness {
        model: str_of(o, "model")?,
        prompt_tokens: u64_of(o, "prompt_tokens")?,
        window: u64_of(o, "window")?,
    })
}

/// One live-mark seat, read back on the two keys the encoder writes.
fn seat_of(v: &Value) -> Result<Seat, String> {
    let o = v.as_object().ok_or("seat: not an object")?;
    Ok(Seat {
        name: str_of(o, "name")?,
        doing: pick(o, "doing", &DOINGS)?,
    })
}

/// The in-flight strip, read back on its class and its rendered characteristics.
fn strip_of(v: &Value) -> Result<FlightStrip, String> {
    let o = v.as_object().ok_or("strip: not an object")?;
    Ok(FlightStrip {
        class: pick(o, "class", &FLIGHTS)?,
        facts: str_of(o, "facts")?,
    })
}

fn doing_token(doing: Doing) -> &'static str {
    match doing {
        Doing::Waiting => "waiting",
        Doing::Thinking => "thinking",
        Doing::Inference => "inference",
        Doing::Tools => "tools",
        Doing::Idle => "idle",
    }
}

/// The parked invocation, read back on litany's own three keys.
fn held_of(v: &Value) -> Result<Held, String> {
    let o = v.as_object().ok_or("held: not an object")?;
    Ok(Held {
        tool_use_id: str_of(o, "tool_use_id")?,
        tool: str_of(o, "tool")?,
        reason: str_of(o, "reason")?,
    })
}

fn mark_token(mark: AgentMark) -> &'static str {
    match mark {
        AgentMark::Notified => "notified",
        AgentMark::BudgetExhausted => "budget-exhausted",
        AgentMark::Conflicted => "conflicted",
        AgentMark::Held => "held",
        AgentMark::Abandoned => "abandoned",
    }
}

fn mark_of(word: &str) -> Result<AgentMark, String> {
    MARKS
        .iter()
        .find(|(token, _)| *token == word)
        .map(|(_, mark)| *mark)
        .ok_or_else(|| format!("agent: unknown mark {word:?}"))
}
