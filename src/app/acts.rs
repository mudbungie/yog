//! **The frame's act half** (REMOTE §1.2, §9.8; bl-4841): a gesture sent over
//! the wire, and the receipt that lands frames later.
//!
//! The read half declares a standing question and paints what came back
//! ([`wire::link`](crate::wire::link)); this is the same shape pointed the other
//! way. A click **posts** an act and holds a [`Ticket`]; the
//! [`Poster`](crate::wire::poster::Poster) dials off-frame; the receipt lands
//! under that ticket and whoever kept it reads it. The frame never waits on a
//! socket in either direction, which is the invariant the whole read path was
//! built to keep and the reason an act could not simply be re-pointed.
//!
//! **The aftermath belongs to the receipt, not to the click.** A dispatched verb
//! used to name the root it touched the instant it returned, because it *had*
//! returned — the frame had just run it. Over the wire the act has not happened
//! yet at the moment of clicking, so the root is resolved then (against the
//! snapshot the operator clicked on) and marked dirty when the receipt says the
//! engine is done with it. One place, for every act: which substrate a gesture
//! touched is a fact about the gesture, and re-deciding it per call site is how
//! three copies of that match came to exist.

use crate::boundary::{Action, Gesture, codec};
use crate::wire::link::Landed;
use crate::wire::post::{Post, Ticket};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// What the frame has fired and not yet heard back about: the wire's own end,
/// and what to re-derive when each act lands.
///
/// The map holds one entry per act in flight and is emptied by the receipt, so
/// its size is the number of gestures the engine currently owes an answer for.
#[derive(Default)]
pub struct Acts {
    post: Post,
    roots: BTreeMap<Ticket, Vec<PathBuf>>,
}

impl crate::AppModel {
    /// Take the engine's end of the act path (REMOTE §9.8). Handed over rather
    /// than taken at [`boot`](Self::boot) for
    /// [`adopt_wire`](Self::adopt_wire)'s reason exactly: the model owns no
    /// thread and mints no handle the engine is the one owner of.
    pub fn adopt_post(&mut self, post: Post) {
        self.acts.post = post;
    }

    /// **Fire one gesture over the wire** (REMOTE §1.2) and hand back the
    /// ticket its receipt will land under. Never blocks and never dials — the
    /// poster does both, off-frame — so a surface built on this paints the act
    /// as in flight and the receipt whenever it arrives.
    pub fn post_act(&mut self, action: &Action) -> Ticket {
        let roots = self.act_roots(action);
        let envelope = codec::encode(&Gesture::Act(action.clone()));
        let ticket = self.acts.post.send(&envelope);
        self.acts.roots.insert(ticket, roots);
        ticket
    }

    /// Take one act's receipt, if it has landed. Spent by the read: the caller
    /// that held the ticket holds it no longer.
    pub fn act_receipt(&mut self, ticket: Ticket) -> Option<Landed> {
        self.acts.post.receipt(ticket)
    }

    /// One frame's act duty (§7.2): take the receipts that landed and mark each
    /// act's roots dirty now that the engine has finished with it. One channel
    /// drain and, with nothing in flight, nothing else — which is every frame
    /// but the handful after a click.
    pub(super) fn settle_acts(&mut self) {
        for ticket in self.acts.post.settle() {
            if let Some(roots) = self.acts.roots.remove(&ticket) {
                self.mark_dirty(roots);
            }
        }
    }

    /// Which roots an act's effect lands in (§7.1) — read from the **action**,
    /// which is the one thing that knows what it touched, and resolved at the
    /// fire against the enumeration the operator clicked against.
    ///
    /// Two, because a gesture writes in two places and the §7.1 roots are
    /// disjoint. Its **substrate** root is the project a ball verb names, or
    /// the yog state root every `lernie` verb's `ops.jsonl` line lands under.
    /// Its **workspace** is whatever the boundary's own address table answers
    /// ([`Action::workspace`]), when the enumeration holds that name.
    ///
    /// **The workspace half is bl-18e8.** A `Message` names no project, so a
    /// send used to request the yog state root alone — and the deposit that
    /// creates the mail-on-tail state therefore never asked for the
    /// re-derivation that clears it, leaving the §7.3/§13.3 banners' liveness
    /// half waiting on an fs event and a whole sweep. The act that changed a
    /// workspace is the one thing that knows it changed: naming it here is
    /// what makes the deposit schedule its own catch-up.
    fn act_roots(&self, action: &Action) -> Vec<PathBuf> {
        let substrate = action
            .project()
            .and_then(|name| self.derivation().project_path(&name).ok())
            .unwrap_or_else(|| self.roots.yog_state.clone());
        let workspace = action
            .workspace()
            .and_then(|name| self.derivation().ws_path(&name).ok());
        std::iter::once(substrate).chain(workspace).collect()
    }
}

#[cfg(test)]
mod tests;
