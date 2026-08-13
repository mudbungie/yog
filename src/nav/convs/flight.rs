//! What kind of work is in flight in a conversation (DESIGN §5.1 #28, §7.2,
//! §11) — the one derivation all three live-activity seats read: the list
//! row's pulsing name, the altitude-1 header's chip, and the bottom in-flight
//! strip ([`strip`], bl-905f), which adds the live characteristics to the same
//! answer rather than asking the question again.
//!
//! Three classes, and a conversation shows exactly **one**: the operator's
//! priority is `inference > tools > subagents`. They deliberately overlap —
//! a dispatched child that is streaming lights all three — because the
//! priority *is* the answer to "several at once", not a defect to design
//! around. What the operator wants to see is the most immediate thing
//! happening, and a model call is more immediate than the tool it will call
//! or the child it dispatched.
//!
//! Everything here is a **query over the §5.1 snapshot** — the agent states
//! (#9: the executor flock plus the open `response.json` fd), the latest step's
//! tool records (#10: `input.json` landed, `output.json` absent) and the two
//! **structural starts** those same records carry (`Agent::call_start_unix`,
//! `ToolCall::start_unix`). No flag is stored and none could be: yog observes
//! neither start nor end, only the disk at this tick — but the world's own
//! records mark when each call began, so the start is read, never remembered.
//!
//! **The two overlapping classes are folded, never re-derived.** Whether a
//! member is in a model call and whether it is running a tool are the per-agent
//! [`Doing`](super::doing::Doing) fact (§5.1 #28b), so this file asks that one
//! question and applies the priority to its answers. The live-driver guard on a
//! running tool lives there with it.

use super::doing::{Doing, doing};
use crate::git_tree::{Agent, ToolCallState};

/// The one live-activity class a conversation displays (§11). The variant
/// order *is* the priority order; [`flight`] resolves by it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flight {
    /// A model call is streaming right now — any member is `InFlight` (§5.1 #9).
    Inference,
    /// A tool is executing — a running member holds a tool call whose
    /// `output.json` has not landed (§5.1 #10).
    Tools,
    /// A dispatched child is running — any non-root member holds a driver.
    Subagents,
}

/// The class in flight over one conversation's subtree, `members` in §2.3
/// descent order (**root first** — the subagent rung is "not the root").
/// `None` when nothing is in flight, which is what makes an idle window
/// schedule no repaint at all (§7.2).
pub fn flight(members: &[&Agent]) -> Option<Flight> {
    if members.iter().any(|a| doing(a).is_model_call()) {
        return Some(Flight::Inference);
    }
    if members.iter().any(|a| doing(a) == Doing::Tools) {
        return Some(Flight::Tools);
    }
    if members.iter().skip(1).any(|a| super::running(a.state)) {
        return Some(Flight::Subagents);
    }
    None
}

/// The same question for a conversation named by its root (the §11 altitude-1
/// pane, which holds agents rather than a row). Empty — an id that roots
/// nothing here — is nothing in flight.
pub fn conversation_flight(agents: &[Agent], root_id: &str) -> Option<Flight> {
    let subtree = super::members(agents, root_id);
    let members: Vec<&Agent> = subtree.iter().filter_map(|r| agents.get(r.index)).collect();
    flight(&members)
}

/// What the §11 **bottom in-flight strip** paints (bl-905f) — the third seat of
/// this one derivation, and the only one that carries characteristics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlightStrip {
    /// The class, resolved by [`flight`] — never re-decided here.
    pub class: Flight,
    /// The live characteristics, ` · `-joined: everything the class's own words
    /// ([`crate::theme::flight_badge`], the seat's other half) do not say —
    /// ending in the compact elapsed for whichever class carries a structural
    /// start.
    pub facts: String,
}

/// One class's contribution to the strip: what it states, and **when the thing
/// it names began** — `None` for a class no world record marks a start for
/// (§5.1 #28: an omitted segment is lawful, an invented one is not).
struct Facts {
    says: String,
    start_unix: Option<i64>,
}

/// What the strip *is*, for the hover every seat owes the operator (bl-68ac).
pub const STRIP_HOVER: &str = "what is in flight in the open conversation, right now — the strip is absent whenever \
     nothing is";

/// The strip for the conversation rooted at `root_id`, or `None` when nothing
/// is in flight there — which is what makes an idle window paint no strip and
/// schedule no repaint (§7.2).
///
/// **Elapsed comes off the structure, never a stored flag** (bl-9dfb,
/// overruling bl-905f — DESIGN §5.1 #28, §11). yog still observes no
/// start, but the world's records mark one: the model call's `request.json` and
/// the tool call's `input.json` are each written once, immediately before the
/// call they open, and never rewritten. Those stamps ride in on the snapshot;
/// `now_unix` is the caller's wall clock (the shell mints it, as it does for the
/// list's ages), so the segment ticks per frame with nothing stored anywhere.
/// The subagents class carries no start and shows none — see
/// [`children_facts`].
pub fn strip(agents: &[Agent], root_id: &str, now_unix: i64) -> Option<FlightStrip> {
    let subtree = super::members(agents, root_id);
    let members: Vec<&Agent> = subtree.iter().filter_map(|r| agents.get(r.index)).collect();
    let class = flight(&members)?;
    let facts = match class {
        Flight::Inference => inference_facts(&members),
        Flight::Tools => tool_facts(&members),
        Flight::Subagents => children_facts(&members),
    };
    Some(FlightStrip {
        class,
        facts: with_elapsed(facts, now_unix),
    })
}

/// The strip's line: the class's characteristics, then ` · <elapsed>` when it
/// carries a start. The label is [`super::age_label`]'s — the list row's age and
/// the strip's elapsed are one spelling of "how long", reused rather than
/// restated, so `42s`/`7m` cannot drift between the two seats.
fn with_elapsed(facts: Facts, now_unix: i64) -> String {
    match facts.start_unix {
        Some(start) => format!("{} · {}", facts.says, super::age_label(now_unix - start)),
        None => facts.says,
    }
}

/// `<who> · <n> chars streamed` — how much of the answer has landed in the live
/// tail, off the snapshot's own `Agent::stream` (no render-path disk read).
/// Zero is the general path with an empty tail, not a case: a call whose first
/// token has not arrived reads `0 chars streamed`, which is the true thing to
/// say about it. The start is the streaming member's own `call_start_unix`.
fn inference_facts(members: &[&Agent]) -> Facts {
    let who = members.iter().find(|a| doing(a).is_model_call()).copied();
    let chars = who
        .and_then(|a| a.stream.text.as_ref())
        .map_or(0, |t| t.chars().count());
    Facts {
        says: format!(
            "{} · {} streamed",
            name_of(who),
            plural(chars, "char", "chars")
        ),
        start_unix: who.and_then(|a| a.call_start_unix),
    }
}

/// `<who> · <tool>` — the running tool's name off the step record that already
/// decided the class (§5.1 #10). A record with no parsable name drops the
/// segment: `toolu_01abc…` names nothing to an operator. The start is that same
/// record's stamp, so the name and the elapsed are one call's.
fn tool_facts(members: &[&Agent]) -> Facts {
    let who = members.iter().find(|a| doing(a) == Doing::Tools).copied();
    let call = who.and_then(|a| {
        a.tool_calls
            .iter()
            .find(|c| c.state == ToolCallState::InFlight)
    });
    Facts {
        says: match call.and_then(|c| c.name.clone()) {
            Some(tool) => format!("{} · {tool}", name_of(who)),
            None => name_of(who),
        },
        start_unix: call.and_then(|c| c.start_unix),
    }
}

/// `<n> children running` — the count is the characteristic here; a subtree
/// working on the operator's behalf has no single agent to name.
///
/// **No elapsed, and none is honest.** This class is by construction the window
/// in which a child holds a driver while running *neither* a model call nor a
/// tool — the between-steps gap. There is no call to time. The child's dispatch
/// commit was the candidate and it fails the test the other two pass: a start
/// must be tied one-to-one to the thing in flight, and a dispatch commit is
/// written once per branch while the driver over it may be its third run (a stop
/// plus a `lernie advance` resume), so "dispatched 40m ago" would be printed in
/// the slot that everywhere else means "running for". A count of three children
/// has no one dispatch to quote in any case.
fn children_facts(members: &[&Agent]) -> Facts {
    let n = members
        .iter()
        .skip(1)
        .filter(|a| super::running(a.state))
        .count();
    Facts {
        says: format!("{} running", plural(n, "child", "children")),
        start_unix: None,
    }
}

/// What to call the member doing the work — [`super::member_title`]'s §3.3
/// ladder over its own rungs, so the strip names an agent exactly as the row
/// title, the centre header, the descent-tree member row and the §3.6 dialog do.
fn name_of(agent: Option<&Agent>) -> String {
    agent.map(super::member_title).unwrap_or_default()
}

/// A count and its noun, so no seat prints "1 chars" or "1 childs".
fn plural(n: usize, one: &str, many: &str) -> String {
    match n {
        1 => format!("1 {one}"),
        _ => format!("{n} {many}"),
    }
}
