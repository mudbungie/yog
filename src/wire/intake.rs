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

/// The server's [`Answerer`](super::listener::Answerer): the deposit consumer's
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

impl super::listener::Answerer for Intake {
    /// One request, one reply frame — **unless the read is follow-class**
    /// (bl-73e7), in which case it is the frames that read produces, written as
    /// they are produced. Two arms and no third: [`follow`](ConsumerCtx::follow)
    /// answers `None` for every request that is not one, and for a follow whose
    /// address does not resolve — which then earns the ordinary refusal, in one
    /// frame, from the one function that words it.
    ///
    /// **The certificate's grade is spent one level down** (REMOTE §4.2,
    /// bl-7ff3), where the identity already is: a foot's follow-class read is
    /// not one of the three gestures it may send, so `follow` answers `None`
    /// for it and the refusal arrives through the same one function that words
    /// every other one. No second refusal path here, and nothing to forget.
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
        peer: &crate::registry::Peer,
        request: Value,
    ) -> Box<dyn Iterator<Item = Value>> {
        // **This client is speaking, so it was seen** (REMOTE §5 as amended,
        // bl-d542): one stamp per request, at the one door every connection's
        // bytes pass through, ahead of both arms so a follow-class read that
        // parks for thirty seconds is stamped when it arrives rather than when
        // it ends.
        self.ctx.seen(peer);
        // **A leaf an offer issued and nobody approved is answered nothing**
        // (REMOTE §15.1 step 4, bl-9ea6) — ahead of both arms, and ahead of
        // the one gesture a client registered nowhere may otherwise make:
        // founding a workspace, which would seat it as the creator. Speaking
        // is still what proves the offer, which is why the stamp above comes
        // first.
        let root = &self.ctx.state_root;
        if crate::registry::offer::store::held(root, &peer.client) {
            let refusal = crate::registry::offer::store::refusal(&peer.client);
            return Box::new(std::iter::once(crate::boundary::reply::refusal(&refusal)));
        }
        match self.ctx.follow(peer, &request) {
            Some(frames) => frames,
            None => Box::new(std::iter::once(self.ctx.answer_as(peer, &request))),
        }
    }
    /// **The registration door's one request** (REMOTE §15.2, bl-9ea6): a
    /// certless peer's code and body, handed to the claim with this server's
    /// own roots and clock. It meets no codec and no dispatch.
    fn register(&self, code: &str, body: &[u8]) -> Option<Value> {
        crate::registry::offer::claim::claim(
            &self.ctx.state_root,
            &crate::wire::material::dir(&self.ctx.world),
            self.ctx.clock.unix(),
            code,
            body,
        )
    }
}

#[cfg(test)]
mod tests;
