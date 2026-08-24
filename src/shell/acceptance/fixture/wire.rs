//! **The transport stood in for** (REMOTE §1.2, §9.8, extended to reads by
//! bl-44e9): the frame's standing questions answered, the acts it posted
//! answered, and the fixed point both settle to.
//!
//! Split from [`super::world`] at §12's budget on the seam that file's own doc
//! argues at length and [`super::follow`] already cut once: the `World` there is
//! the populated fixture a test drives and mutates, this is the engine that
//! answers what the frame said to it. Every act the window fires is posted now,
//! so a fixture with nothing behind its end of the channel is a window whose
//! every gesture is refused with *"this window has no wire behind it"*. The
//! transport is stood in for and nothing else: the questions and the acts are
//! taken off the same two ends the [`Asker`](crate::wire::asker::Asker) and the
//! [`Poster`](crate::wire::poster::Poster) take theirs from, and each is
//! answered through the one chokepoint the real listener reaches
//! ([`AppModel::answer`](crate::AppModel::answer),
//! [`crate::boundary::dispatch::dispatch`]) — no second dispatch, which is
//! REMOTE §11's rejection kept.

use super::world::World;
use crate::boundary::{Gesture, codec};
use crate::ui_state::UiState;

impl World {
    /// **Answer every act the frame has posted**, through the chokepoint the
    /// engine's listener reaches, over a `ui.json` opened fresh per gesture —
    /// answer 3's ordering (*the engine writes and the window adopts*) paid in
    /// full, so a test sees the same write-then-adopt a window does.
    ///
    /// `true` when it answered anything, which is what lets a driver settle a
    /// gesture whose receipt posts the next act (the §8.1 start family is two).
    pub(in crate::shell::acceptance) fn acts(&mut self) -> bool {
        let deps = self.model.boundary_deps(&self.lernie, &self.bl);
        let ts = crate::shell::now_ts();
        let mut answered = false;
        while let Some((ticket, envelope)) = self.outbox.try_next() {
            answered = true;
            let landed = match codec::decode(&envelope) {
                Ok(Gesture::Act(action)) => {
                    let mut ui = UiState::open(self.model.ui_json_path());
                    crate::boundary::dispatch::dispatch(&deps, &mut ui, &ts, &action)
                }
                Ok(Gesture::Ask(_)) => Err("the act path carries no reads".to_owned()),
                Err(said) => Err(said),
            };
            self.outbox.publish(ticket, landed);
        }
        answered
    }

    /// **Answer every standing question the frame declared** — the read half,
    /// through [`AppModel::answer`], the same chokepoint `ConsumerCtx::answer_as`
    /// reaches over the socket. Deliberately unscoped: the fixture registers no
    /// client, so there is no registration to narrow against.
    ///
    /// Whether the frame is still waiting on any of them is
    /// [`AppModel::awaiting`]'s answer, never this call's: every call here
    /// answers the whole standing set, so "I answered something" is true
    /// forever and could end no loop.
    pub(in crate::shell::acceptance) fn reads(&mut self) {
        let deps = self.model.boundary_deps(&self.lernie, &self.bl);
        let now = crate::shell::now_unix();
        for question in self.link.standing() {
            let landed = match codec::decode(&question) {
                Ok(Gesture::Ask(query)) => self.model.answer(&deps, &query, now),
                Ok(Gesture::Act(_)) => Err("the read path carries no acts".to_owned()),
                Err(said) => Err(said),
            };
            self.link.publish(&question, landed);
        }
    }

    /// **Answer the outstanding §8.5 search**, the way the
    /// [`Searcher`](crate::search::Searcher) thread does — the same stand-in
    /// this file makes for the asker and the poster, one door over (bl-44e9).
    /// The searcher dials a listener the fixture has none of, so the walk runs
    /// in place over this world's own snapshot, which is what the engine at the
    /// far end would have run.
    pub(in crate::shell::acceptance) fn searches(&mut self) {
        let Some((seq, text)) = self.model.search_cell().pending() else {
            return;
        };
        let snap = self.model.derivation().clone();
        self.model
            .search_cell()
            .publish(seq, crate::search::run(&snap, &text, &|| true));
    }

    /// **One frame's whole wire duty** (REMOTE §9.8's harness ruling, extended
    /// to reads by bl-44e9): hand the standing questions over and take what
    /// landed ([`AppModel::refresh`], which is the frame duty the app runs once
    /// per `update`), answer the reads, answer the acts.
    ///
    /// A bespoke driver calls it between its own frames; [`Screen`] does not,
    /// because it settles to a fixed point and needs the two halves apart.
    ///
    /// [`Screen`]: crate::shell::acceptance::screen::Screen
    pub(in crate::shell::acceptance) fn settle(&mut self) {
        self.model.refresh();
        self.reads();
        self.follows();
        self.acts();
        self.searches();
    }

    /// **Settle the wire to a fixed point**, painting `frame` between passes
    /// (REMOTE §9.8's harness ruling as bl-44e9 extended it to reads; the one
    /// definition of it since bl-13f9, when a second driver needed it too).
    ///
    /// A landed **answer** reaches the frame on the refresh after the frame that
    /// kept its question standing — a `Link` may never be settled twice without
    /// a frame between, or the second settle declares nothing and drops the lot
    /// — so a read costs two passes where a **receipt** costs one. It terminates
    /// for the acts' own reason (only a receipt can post an act, and nothing
    /// posts one unprompted) and for the reads' equivalent: a surface's
    /// questions are a function of state, and a frame that changed no state
    /// declares the set it declared before. A **chain** — the §11 inspector's
    /// step drill-in, whose sequence name is picked out of the step list that
    /// landed — is why counting passes was never enough.
    pub(in crate::shell::acceptance) fn drain(&mut self, frame: &mut dyn FnMut(&mut Self)) {
        loop {
            self.model.refresh();
            let waiting = self.model.awaiting();
            self.reads();
            self.follows();
            let acted = self.acts();
            self.searches();
            if !waiting && !acted {
                return;
            }
            // A landed **answer** reaches the frame on the refresh after the
            // frame that kept its question standing — a `Link` may never be
            // settled twice without a frame between, or the second settle
            // declares nothing and drops the lot — so a read costs two passes
            // where a **receipt** costs one. Paying both is the cheaper half
            // idling.
            frame(self);
            self.model.refresh();
            frame(self);
        }
    }
}
