//! The §11 conversation seat's own spelling (REMOTE §9.4, bl-1eb0) — both
//! directions of [`AgentView`], cut off the reply roster at §12's budget on the
//! seam [`search`](super::search) and [`queue`](super::queue) already take: one
//! payload whose rows are its own vocabulary.
//!
//! The one token table this reply owns is the §6 **mark** set. The §5.1 agent
//! state and the §11 flight class are the conversation list's already
//! ([`rows`](super::rows)), read from there rather than restated — a second
//! spelling of one vocabulary is the drift the round-trip test exists to catch.
//!
//! Absent-not-null throughout: an unmarked conversation carries no `marks` key
//! and one at rest no `flight`, because a reader must never have to tell an
//! empty list from a fact the encoder declined to state.

use serde_json::{Map, Value, json};

use crate::boundary::answer::agent::AgentView;
use crate::boundary::codec::fields::{bool_of, opt, opt_val, pick, str_of, strings_of};
use crate::control::hold::Held;
use crate::git_tree::AgentMark;

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
    if !view.marks.is_empty() {
        let marks: Vec<&str> = view.marks.iter().copied().map(mark_token).collect();
        map.insert("marks".to_owned(), json!(marks));
    }
    if let Some(flight) = view.flight {
        map.insert("flight".to_owned(), json!(flight_token(flight)));
    }
    if let Some(held) = &view.held {
        // lernie's own three keys — the blob's spelling, not a second one
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
        marks,
        held: opt_val(o, "held", held_of)?,
        flight: opt(o, "flight", |o, k| pick(o, k, &FLIGHTS))?,
        present: bool_of(o, "present")?,
        nudgeable: bool_of(o, "nudgeable")?,
        stoppable: bool_of(o, "stoppable")?,
        stop_children: bool_of(o, "stop_children")?,
    })
}

/// The parked invocation, read back on lernie's own three keys.
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
