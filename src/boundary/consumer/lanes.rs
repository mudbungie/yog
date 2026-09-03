//! **The follow-class door** (REMOTE §3, §10, §14.1) — the one arm of the
//! intake whose answer is a frame *sequence* rather than a value, split from
//! [`super`] at §12's budget on the seam the answer's own shape draws.
//!
//! Three reads reach it and they share everything but their subject: `Follow`
//! is one conversation's live tail (bl-73e7), `LoginTail` is one sign-in's
//! output (REMOTE §8.3, bl-c285), and `Attention` is what needs the operator
//! (REMOTE §14.1, bl-09aa). All three are answered *once* by
//! [`answer`](crate::boundary::answer::answer) at every other intake — the tail
//! as of now, the standing as of now, the queue as of now, which is the general
//! path with one frame — and held open only here, because this is the only
//! intake that owns a connection to hold.
//!
//! **The third one names nothing, and that costs no branch** (REMOTE §14.1): a
//! read that names no workspace resolves to nothing and no arm reads it, which
//! is the resolution [`answer`](crate::boundary::answer::answer) already
//! performs the same way — the general path with no input rather than a case of
//! its own.

use serde_json::Value;
use std::sync::Arc;

use super::ConsumerCtx;
use crate::boundary::reply::Reply;
use crate::boundary::{Gesture, Query};

/// Which lane a follow-class read opens, with the one thing that read names
/// beyond its workspace. It exists so the address resolution below can be
/// written once, ahead of the three constructors, exactly as the two
/// chokepoints resolve ahead of their tables — and so no arm is unreachable.
enum Subject {
    /// The conversation whose live tail is followed.
    Tail(String),
    /// The provider row whose sign-in is followed.
    Login(String),
    /// What needs the operator, under this asker's scope. It names no
    /// workspace: the queue is world-wide and the scope is the narrowing.
    Attention,
}

impl ConsumerCtx {
    /// **The same gesture, answered as a stream** (REMOTE §3, bl-73e7) — `Some`
    /// only for a query whose answer is a *sequence*, and only once its address
    /// has resolved under this client's scope.
    ///
    /// `None` is the whole of "this is not a follow-class read, or it is one
    /// nobody can answer", and it is deliberately the same `None` for both: an
    /// unresolvable workspace and an unknown conversation fall back to
    /// [`answer_as`](ConsumerCtx::answer_as), which refuses in the resolver's
    /// own words and in one frame. The intake needs no second refusal path, and
    /// a seat cannot tell a refused follow from any other refused read.
    ///
    /// The scope is spent HERE, at connect, exactly as it is for a one-frame
    /// answer — the identity is per request (REMOTE §4) and a held read is one
    /// request. What the stream then re-reads per look is the state of a
    /// conversation, of a sign-in, or of the world under the registrations this
    /// caller was already authorized for.
    ///
    /// **The attention lane can never be a read nobody can answer** (REMOTE
    /// §14.1): it addresses nothing, so there is nothing to resolve, and a seat
    /// registered in no workspace is answered a lane whose every frame is empty
    /// — REMOTE §4's absence, said as a stream — rather than a refusal.
    ///
    /// **A foot never reaches the lane** (REMOTE §4.2, bl-7ff3): a follow-class
    /// read is not one of the three gestures its grade admits, so this answers
    /// `None` for it and [`answer_as`](ConsumerCtx::answer_as) words the refusal
    /// — which is the same fall-through an unresolvable address already takes.
    pub fn follow(
        &self,
        peer: &crate::registry::Peer,
        request: &Value,
    ) -> Option<Box<dyn Iterator<Item = Value>>> {
        let Ok(gesture) = crate::boundary::codec::decode(request) else {
            return None;
        };
        if !peer.grade.admits(&gesture) {
            return None;
        }
        let (named, subject) = match gesture {
            Gesture::Ask(Query::Follow { workspace, agent }) => {
                (Some(workspace), Subject::Tail(agent))
            }
            Gesture::Ask(Query::LoginTail {
                workspace,
                provider,
            }) => (Some(workspace), Subject::Login(provider)),
            Gesture::Ask(Query::Attention) => (None, Subject::Attention),
            _ => return None,
        };
        let client = &peer.client;
        let scope = crate::registry::registered(&self.state_root, client);
        let (deps, _, _) = self.deps(client, Some(&scope));
        let ws = match named {
            Some(name) => deps.snapshot.ws_path(&name).ok()?,
            None => std::path::PathBuf::new(),
        };
        let frames: Box<dyn Iterator<Item = Reply>> = match subject {
            Subject::Tail(agent) => {
                let agent =
                    crate::boundary::address::resolve_agent(&deps.snapshot, &ws, Some(agent))
                        .ok()?;
                Box::new(crate::boundary::follow::Follow::new(
                    self.cell.clone(),
                    ws,
                    agent,
                ))
            }
            Subject::Login(provider) => Box::new(crate::boundary::login::Lane::new(
                deps.caller.logins.clone(),
                ws,
                provider,
            )),
            Subject::Attention => Box::new(crate::boundary::attend::Attend::new(
                self.cell.clone(),
                scope,
                self.ui_path.clone(),
                crate::registry::pane(&self.state_root, client),
                Arc::clone(&self.clock),
            )),
        };
        Some(Box::new(
            frames.map(|reply| crate::boundary::reply::encode(&reply)),
        ))
    }
}
