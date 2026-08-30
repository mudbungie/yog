//! The tool-control **wire contract** (DESIGN §8.6, VISION §4.11 item 2) — the
//! two shapes litany's shipped seam speaks, and nothing else.
//!
//! litany's `prompt::tool::control` client (its ARCH *Tool control* section)
//! spawns the configured executable with **no argv**, writes one JSON object on
//! its stdin and reads one JSON verdict off its stdout, requiring exit 0. This
//! module is yog's side of exactly that, deliberately kept to the two types:
//!
//! - [`Request`] — the `tool_use` block verbatim (`id`, `name`, `input`) plus
//!   the calling `role` and `agent_id`. **Unknown fields are ignored**: a later
//!   litany that adds a fact must not brick every tool call yog adjudicates,
//!   and the fields we do read are all required — a missing one is a protocol
//!   break the caller fails closed on.
//! - [`Verdict`] — `pass` (which carries **no** reason: litany's parser rejects
//!   one) or `refuse`/`hold` (each of which **requires** one). The parser on the
//!   far side is a strict `deny_unknown_fields` struct, so the encoding here is
//!   hand-written rather than a tagged enum — one shape, no serde-attribute
//!   distance between what we mean and what lands on the wire.

use serde_json::{Value, json};

/// One invocation to adjudicate, as litany's seam presents it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// `tool_use.id` — provider-unique, and what a hold parks on. It is also
    /// the key a once-answer is scoped to, which is why a once-grant needs no
    /// consumption and cannot race (VISION §4.11 item 6).
    pub id: String,
    /// The tool name the model spelled.
    pub name: String,
    /// `tool_use.input`, verbatim — the object the classifier reads.
    pub input: Value,
    /// The calling agent's role. Grants stay litany's structure (bl-7fc8); the
    /// role is carried for the record, never to narrow a tool name.
    pub role: String,
    /// The calling agent's id — the full hyphenated descent, which is also its
    /// branch name and the key a per-conversation floor matches by prefix.
    pub agent_id: String,
}

impl Request {
    /// Parse one request from the seam's JSON. Hand-written over
    /// [`serde_json::Value`] because yog links `serde_json` and **not** `serde`
    /// (no derive, and a dependency is not added for four fields). Every field
    /// this reads is required: a missing one is a protocol break, `None`, and a
    /// fail-closed exit — never a default that adjudicates the wrong thing.
    /// Fields we do not know are ignored, so a later litany that adds one does
    /// not brick every tool call.
    pub fn parse(raw: &str) -> Option<Request> {
        let value: Value = serde_json::from_str(raw).ok()?;
        let text = |key: &str| value.get(key).and_then(Value::as_str).map(str::to_owned);
        Some(Request {
            id: text("id")?,
            name: text("name")?,
            input: value.get("input")?.clone(),
            role: text("role")?,
            agent_id: text("agent_id")?,
        })
    }

    /// A named string field of [`input`](Self::input), or `""` when absent or
    /// not a string. Total: an off-schema input is not an error here, it is an
    /// invocation whose operands the classifier cannot see — which lands it in
    /// the unmatched (open-world) arm rather than in a panic path.
    pub fn field(&self, key: &str) -> String {
        self.input
            .get(key)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned()
    }
}

/// The control's answer, in litany's three-valued vocabulary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// The invocation proceeds to the executor unchanged.
    Pass,
    /// The invocation never executes; `reason` reaches the model as an in-band
    /// `is_error` tool result it reads and steps past. Never a stop.
    Refuse(String),
    /// The invocation parks before execution and the driver exits; `reason` is
    /// for the operator. Released by re-adjudication at the next drive, which
    /// is why every consult must be side-effect-free.
    Hold(String),
}

impl Verdict {
    /// The verdict as the one JSON line litany parses. A pass carries no
    /// `reason` key at all — the far side rejects `{"verdict":"pass","reason":…}`.
    pub fn json(&self) -> String {
        match self {
            Verdict::Pass => json!({ "verdict": "pass" }).to_string(),
            Verdict::Refuse(reason) => json!({ "verdict": "refuse", "reason": reason }).to_string(),
            Verdict::Hold(reason) => json!({ "verdict": "hold", "reason": reason }).to_string(),
        }
    }
}
