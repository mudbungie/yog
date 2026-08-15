//! **The acceptance world's `World`** (§11): the populated fixture the tests
//! render, its derivation driven by hand, and — since bl-1747 — **the wire
//! standing behind it**.
//!
//! Split from [`super`] per §12's budget, on the seam the two halves already
//! had: the builder there mints the world's bytes once, this holds what a test
//! then does to it.
//!
//! **Why the wire is here.** Every act the window fires is posted now (REMOTE
//! §1.2, §9.8) and answered by the engine, so a fixture with nothing behind its
//! end of the channel is a window whose every gesture is refused with *"this
//! window has no wire behind it"*. The transport is stood in for and nothing
//! else: the questions and the acts are taken off the same two ends the
//! [`Asker`](crate::wire::asker::Asker) and the
//! [`Poster`](crate::wire::poster::Poster) take theirs from, and each is
//! answered through the one chokepoint the real listener reaches
//! ([`AppModel::answer`], [`crate::boundary::dispatch::dispatch`]) — no second
//! dispatch, which is REMOTE §11's rejection kept.

use super::super::super::ShellState;
use crate::AppModel;
use crate::boundary::{Gesture, codec};
use crate::cli_outbound::Cli;
use crate::git_tree::tests::fixture::Fixture;
use crate::ui_state::UiState;
use crate::wire::link::LinkEnd;
use crate::wire::post::Outbox;
use std::path::PathBuf;
use tempfile::TempDir;

/// A workspace populated across every inspector surface (transcript, steps +
/// tool i/o, inbox), symlinked under the lernie workspaces root so the model
/// enumerates it.
pub(in crate::shell::acceptance) struct World {
    pub(super) _root: TempDir,
    pub(super) fx: Fixture,
    /// Every extra sphere [`World::add_workspace`] created, held only so their
    /// temp dirs outlive the test — the §3.1 wall boundary is unobservable with
    /// one workspace, so a wall drive needs a second.
    pub(super) spheres: Vec<Fixture>,
    pub(in crate::shell::acceptance) model: AppModel,
    /// The §7.2 derivation, driven by hand: in the app a `Worker` thread runs
    /// it, and the frame renders only what it publishes (bl-ee0a).
    pub(super) deriver: crate::app::Deriver,
    pub(in crate::shell::acceptance) state: ShellState,
    pub(in crate::shell::acceptance) ws: PathBuf,
    /// yog's own data root — where the §16 nested world and the §3.1 names root
    /// live. A raise mints under it, so a test that drives one needs to seed the
    /// world here and read the raised sphere back.
    pub(in crate::shell::acceptance) yog_data: PathBuf,
    /// Where a second sphere is symlinked from ([`World::add_workspace`]).
    pub(super) lernie_workspaces: PathBuf,
    /// The frame's end of the read path — the standing questions a painted
    /// surface declared, taken exactly as the asker takes them.
    pub(in crate::shell::acceptance) link: LinkEnd,
    /// The frame's end of the act path — what its gestures posted, taken
    /// exactly as the poster takes them.
    pub(in crate::shell::acceptance) outbox: Outbox,
    /// **The engine's own substrate binaries** — what a posted act actually
    /// spawns. They are the engine's and never a seat's (REMOTE §9.8: a seat
    /// carries the gesture and nothing else), so they live here rather than on
    /// the driver. Deliberately absent by default, on `Screen::new`'s doctrine:
    /// a drive that has not said which fake it wants forks nothing real.
    pub(super) lernie: Cli,
    pub(super) bl: Cli,
}

impl World {
    /// One derivation pass and the frame's take of it — what the smoke test
    /// does whenever it has just changed something on disk. It answers the
    /// acts in flight first: a gesture the frame posted has to have *happened*
    /// before a derivation over the world it changed means anything.
    pub(in crate::shell::acceptance) fn converge(&mut self) {
        self.acts();
        self.deriver.step();
        self.model.refresh();
    }

    /// Point the engine's spawns at this drive's fakes — what
    /// `Screen::with_lernie` is asking for, said once where the acts run.
    pub(in crate::shell::acceptance) fn substrate(&mut self, lernie: &Cli, bl: &Cli) {
        self.lernie = lernie.clone();
        self.bl = bl.clone();
    }

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

    /// Mint a **second sphere** under the same lernie root: another workspace
    /// with one conversation, symlinked where the model enumerates it. Its §3.1
    /// leaf names its own wall (§16.2 as amended), so this is what a wall drive
    /// switches to. Caller converges to fold it in.
    pub(in crate::shell::acceptance) fn add_workspace(
        &mut self,
        name: &str,
        agent: &str,
    ) -> PathBuf {
        let fx = Fixture::new();
        fx.build_agent(agent, name);
        let ws = self.lernie_workspaces.join(name);
        std::os::unix::fs::symlink(&fx.path, &ws).unwrap();
        self.spheres.push(fx);
        ws
    }

    /// Fork a nameless descent child off `parent_id` (§2.3) — the bl-63a1
    /// chained-id shape: no name blob, no goal on disk, no step record, so the
    /// §3.3 ladder bottoms out at its floor. Caller converges to fold it in.
    pub(in crate::shell::acceptance) fn add_child(&self, parent_id: &str, child_id: &str) {
        self.fx.build_child(parent_id, child_id);
    }

    /// A **second root** conversation (§2.3) wearing `name` as its §3.3 name
    /// fact — a second row in the list, so a test can compare two rows of the
    /// same list rather than two worlds. Caller converges to fold it in.
    pub(in crate::shell::acceptance) fn add_root(&self, conv_id: &str, name: &str) {
        self.fx.build_agent(conv_id, name);
        self.fx.name_agent(conv_id, name);
    }

    /// **Advance the workspace's config lineage past every conversation in
    /// it** (§9.4 drift): one ordinary config edit on `config/default`,
    /// carrying `providers_yaml`. Every agent forked the lineage's previous
    /// head, so the governing commit and the tip part the moment this lands —
    /// which is the whole condition the drift clause and its exits are offered
    /// under. Caller converges to fold it in.
    pub(in crate::shell::acceptance) fn advance_config(&self, providers_yaml: &str) {
        self.fx.commit_other("providers.yaml", providers_yaml);
    }

    /// Mark `conv_id` **abandoned** — §6's will-not-retry assertion, the one
    /// gate that suppresses rule 2 (`attention::rest_evidence`). It is how a
    /// fixture gets a settled row bearing **no** attention beside one that
    /// does, without focusing anything and so without spending an ack.
    pub(in crate::shell::acceptance) fn quiet(&self, conv_id: &str) {
        self.fx
            .mark_ref(&format!("refs/lernie/abandoned/{conv_id}"));
    }
}
