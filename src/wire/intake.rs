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
    /// One request, one reply frame — **unless the read is follow-class**
    /// (bl-73e7), in which case it is the frames that read produces, written as
    /// they are produced. Two arms and no third: [`follow`](ConsumerCtx::follow)
    /// answers `None` for every request that is not one, and for a follow whose
    /// address does not resolve — which then earns the ordinary refusal, in one
    /// frame, from the one function that words it.
    ///
    /// **This is where the wire's trust grade is spent** (REMOTE §4, bl-8bbc):
    /// [`answer_as`](ConsumerCtx::answer_as) is
    /// [`answer`](ConsumerCtx::answer) with a client identity, and the identity
    /// is the whole difference between the two intakes. The inbox's callers are
    /// the world's own residents and are unscoped (§3); a connection is a
    /// caller from another trust domain and sees its registrations. A held read
    /// spends it at the same moment and on the same terms — a connection that
    /// stays open is still one request.
    fn answer(
        &self,
        client: &crate::registry::Client,
        request: Value,
    ) -> Box<dyn Iterator<Item = Value>> {
        match self.ctx.follow(client, &request) {
            Some(frames) => frames,
            None => Box::new(std::iter::once(self.ctx.answer_as(client, &request))),
        }
    }
}

#[cfg(test)]
mod tests;
