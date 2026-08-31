//! **The invocation mailbox** (REMOTE §5, §9 step 7; bl-024b): the engine-side
//! hand-off between an agent's tool call and the machine that runs it.
//!
//! REMOTE §3's routing ruling is that **the ask never inverts**: a tool host
//! rides a follow-class read for its next invocation and posts each capture
//! back as an ordinary act. So the engine never speaks first, and what stands
//! between the two directions is this — a queue per client and a slot per
//! invocation, held in RAM beside the presence refcount ([`super::presence`]).
//!
//! **RAM, for presence's reason exactly.** An invocation in flight is not a
//! fact about the world; it is a fact about this process for the seconds a tool
//! takes to run. Writing it into the world would make a crashed driver's
//! abandoned call durable and put a second, slower bus under a hand-off both
//! ends are already connected to.
//!
//! **Presence is NOT the routing predicate, and that is a finding** (REMOTE §5,
//! amended by bl-024b). §5 imagined an invocation "routed to the client if it
//! is live, refused in-band if not". A tool host holds a connection only while
//! it is *waiting*: it dials per ask (§10), so between polls — and for the
//! whole time it is executing something — it is absent. A presence test would
//! therefore refuse the second call of a busy host, which is the one host that
//! is certainly there. The queue is the predicate instead: an invocation waits
//! in it, and what makes a vanished client visible is the caller's own
//! deadline, in band, never a hang.
//!
//! **This file is the vocabulary; [`slots`] is the map.** Split at §12's
//! per-file budget, on the seam that matters: what a routed invocation *is*
//! (and how it is spelled on the wire) is one subject, and where an in-flight
//! one waits — behind this leg's one lock — is another.

use serde_json::{Value, json};

use crate::boundary::codec::fields::{i64_of, str_of};

/// Where in-flight invocations live — the map, and the one lock this leg has.
mod slots;

pub use slots::Mailbox;

/// **What a driver asks for**: a client's advertised tool, with the model's own
/// arguments. The client is named because the identity is the *addressee* here
/// — unlike the advertisement, whose identity is the intake's (REMOTE §5.1).
///
/// [`Eq`] is written rather than derived for [`Value`]'s reason
/// ([`Tool`](super::tools::Tool)): the grammar has no `NaN` literal, so a
/// schema or an argument object that came through a JSON decoder cannot hold
/// the one value equality is not reflexive over.
#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    /// The client identity that advertised the tool.
    pub client: String,
    /// The advertised name, as that client spells it — never the `<client>_`
    /// prefixed name the model sees (REMOTE §5.2).
    pub tool: String,
    /// The `tool_use.input` JSON, verbatim.
    pub input: Value,
    /// The subject's location, when the call carries one (REMOTE §5's
    /// worktree lane, bl-77be): the conversation's resolved working
    /// directory, honoured at the far end only for an entry its operator
    /// marked `subject_cwd`.
    pub cwd: Option<String>,
}

impl Eq for Call {}

/// **What a tool host is handed**: the engine's handle on this call, and the
/// two facts the far end needs to run it. It carries no client — the read is
/// answered to one identity and a host being told its own name would be a fact
/// it already holds.
#[derive(Debug, Clone, PartialEq)]
pub struct Invocation {
    /// The engine's handle, minted at the post and quoted by the completion.
    ///
    /// **It is also the idempotency key** (bl-e658). An invocation the engine
    /// has no capture for is offered again at the client's next follow-class
    /// read, and a redelivery carries the id it was first handed under — so a
    /// far end that must not run something twice has one stable name to dedupe
    /// on, and needs no field of its own to get it (REMOTE §5.3).
    pub id: String,
    pub tool: String,
    pub input: Value,
    /// The subject's location the call carried, passed to the machine that
    /// runs it (bl-77be).
    pub cwd: Option<String>,
}

impl Eq for Invocation {}

/// **What came back**: litany's own tool contract, one for one (its ARCH §3.3)
/// — bytes on stdout, bytes on stderr, the exit code the verdict. Text rather
/// than bytes because a tool result becomes a model message and a model message
/// is text; the executor transcodes at the one place bytes become an answer
/// (`src/wire/host/exec.rs`), so nothing downstream carries an encoding case.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Capture {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// One completion, as the act that posts it carries it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Completion {
    /// The invocation being answered.
    pub invocation: String,
    pub capture: Capture,
}

/// **The routing leg's two acts, as one family** (bl-024b) — the
/// [`Monitor`](crate::boundary::Action::Monitor) /
/// [`Fleet`](crate::boundary::Action::Fleet) shape, and for their reason: one
/// subject, one mailbox, one pair of ends. Two variants at the boundary would
/// be two rows in every exhaustive table for a hand-off that is one thing read
/// from both sides.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verb {
    /// **A driver queues one call** for the machine that advertised the tool,
    /// and is answered the handle it is known by. It does not wait: the intake
    /// that dispatches it is one thread for the whole world and a tool takes as
    /// long as a tool takes, so the waiting is
    /// [`Query::Capture`](crate::boundary::Query::Capture)'s, on the asker's
    /// own deadline. The client is named because it is the **addressee** —
    /// the opposite of an advertisement, whose identity is the intake's — and
    /// what is checked here is that the named machine advertises the named
    /// tool, which is REMOTE §5's own staleness correction asked where it can
    /// still be answered cheaply.
    Invoke(Call),
    /// **A tool host answers one invocation** with what running it captured.
    /// Only the client an invocation was addressed to may post one, and a
    /// handle addressed to anyone else is **absent** rather than forbidden
    /// (REMOTE §4).
    Complete(Completion),
}

/// The one sentence an unheld handle earns, said the same way at both readers.
pub(super) fn unknown(invocation: &str) -> String {
    format!("no invocation {invocation:?} is in flight; it was answered already or it expired")
}

/// A capture as JSON — the **one** spelling, spent by the completing act, by
/// both replies that carry one and by the client-side executor alike.
pub fn capture_value(capture: &Capture) -> Value {
    json!({ "stdout": capture.stdout, "stderr": capture.stderr,
            "exit_code": capture.exit_code })
}

/// [`capture_value`]'s inverse, strict: an invocation's result is an
/// instruction to a model, so a missing field refuses rather than defaults.
pub fn capture_of(v: &Value) -> Result<Capture, String> {
    let o = v.as_object().ok_or("capture: not a JSON object")?;
    Ok(Capture {
        stdout: str_of(o, "stdout")?,
        stderr: str_of(o, "stderr")?,
        exit_code: exit_of(i64_of(o, "exit_code")?)?,
    })
}

/// The optional subject location (bl-77be), read strictly: absent and null
/// are the ordinary no-location case, and anything but a string refuses —
/// a place a tool will run is an instruction, not an observation.
pub fn cwd_of(o: &serde_json::Map<String, Value>) -> Result<Option<String>, String> {
    match o.get("cwd") {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.clone())),
        Some(_) => Err("field \"cwd\" is not a string".to_owned()),
    }
}

/// An exit code narrowed to what a process can actually have exited with.
fn exit_of(code: i64) -> Result<i32, String> {
    i32::try_from(code).map_err(|_| format!("capture: exit_code {code} out of range"))
}

/// One queued invocation as JSON — the follow-class read's row.
pub fn invocation_value(invocation: &Invocation) -> Value {
    let mut o = json!({ "invocation": invocation.id, "tool": invocation.tool,
            "input": invocation.input });
    if let (Some(cwd), Some(map)) = (&invocation.cwd, o.as_object_mut()) {
        map.insert("cwd".to_owned(), Value::String(cwd.clone()));
    }
    o
}

/// [`invocation_value`]'s inverse, on the same strict terms.
pub fn invocation_of(v: &Value) -> Result<Invocation, String> {
    let o = v.as_object().ok_or("invocation: not a JSON object")?;
    Ok(Invocation {
        id: str_of(o, "invocation")?,
        tool: str_of(o, "tool")?,
        input: o
            .get("input")
            .cloned()
            .ok_or("invocation: missing field \"input\"")?,
        cwd: cwd_of(o)?,
    })
}

#[cfg(test)]
mod tests;
