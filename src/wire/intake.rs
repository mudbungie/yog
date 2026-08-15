//! The wire's half of the **one** intake (REMOTE §3, §9.5; bl-b6fa).
//!
//! There is one thing in yog that turns a gesture envelope into a reply
//! envelope, and it is
//! [`ConsumerCtx::answer`](crate::boundary::consumer::ConsumerCtx::answer) —
//! build a fresh [`Deps`](crate::boundary::dispatch::Deps) over the latest
//! published snapshot, open the §4.1 `ui.json`, decode with the codec, run
//! `dispatch`/`answer`, encode the reply. The gestures inbox drives it on a
//! poll; a connection drives it on a request. **Two doors, one room** — which
//! is why the wire adds no verb and cannot: it never sees an `Action` or a
//! `Query`, only bytes on their way to the same chokepoint.

use crate::boundary::consumer::ConsumerCtx;
use serde_json::Value;
use std::sync::Arc;

/// The engine's [`Answerer`](super::server::Answerer): the deposit consumer's
/// context, shared rather than copied.
pub struct Intake {
    ctx: Arc<ConsumerCtx>,
}

impl Intake {
    /// Wrap the context the gestures-inbox consumer is already driving.
    pub fn new(ctx: Arc<ConsumerCtx>) -> Self {
        Self { ctx }
    }
}

impl super::server::Answerer for Intake {
    /// One request, one reply frame. A follow-class read would be the same
    /// call returning more of them (see [`frame`](super::frame)); no `Query` is
    /// follow-class today, so this is the whole of it.
    ///
    /// **This is where the wire's trust grade is spent** (REMOTE §4, bl-8bbc):
    /// [`answer_as`](ConsumerCtx::answer_as) is
    /// [`answer`](ConsumerCtx::answer) with a client identity, and the identity
    /// is the whole difference between the two intakes. The inbox's callers are
    /// the world's own residents and are unscoped (§3); a connection is a
    /// caller from another trust domain and sees its registrations.
    fn answer(&self, client: &crate::registry::Client, request: Value) -> Vec<Value> {
        vec![self.ctx.answer_as(client, &request)]
    }
}

#[cfg(test)]
mod tests;
