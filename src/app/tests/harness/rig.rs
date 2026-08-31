//! **The model and its worker, driven by hand** (§7.2) — the two calls a test
//! makes in place of the two threads production runs, split off [`super`] at
//! §12's budget on the seam that file's doc draws: the world a test is *given*
//! is there, what it *does* to that world is here.

use crate::AppModel;

/// The frame and its worker, driven **by hand** (§7.2).
///
/// In production the worker thread calls `Deriver::step` forever and every
/// read takes whatever it last published. A test wants the same two calls in a
/// known order, with no thread and no sleeps, so the rig pairs them and
/// [`tick`](Rig::tick) is one full round — derive, then take. Every timing
/// branch is reached by advancing the injected clock.
///
/// It derefs to the model so a test asks its questions the way a gesture does,
/// and reaches `rig.deriver` only for facts the worker owns.
pub(crate) struct Rig {
    pub(crate) model: AppModel,
    pub(crate) deriver: crate::app::Deriver,
}

impl Rig {
    /// One derivation pass followed by one take of what it published. Returns
    /// whether the read source moved.
    pub(crate) fn tick(&mut self) -> bool {
        self.deriver.step();
        self.model.take()
    }

    /// The workspace tab bar, folded off the same answer.
    /// Publish whatever the worker currently holds, without a pass. The seam
    /// for the Live/InFlight states, which need a held flock no fixture can
    /// take: a test writes the agent row into the worker's tree and this makes
    /// every read see it, exactly as a real derivation would have.
    pub(crate) fn publish(&mut self) {
        self.deriver.publish();
        self.model.take();
    }

    pub(crate) fn tab_bar(&self, focused: Option<&str>) -> crate::nav::tabs::TabBar {
        crate::test_support::chrome::tab_bar(&self.model, focused)
    }

    /// The §11 activity chip's counts, folded off the `Query::Ops` answer the
    /// expanded trail paints.
    pub(crate) fn activity(&self) -> crate::opslog::Activity {
        crate::test_support::chrome::activity(&self.model)
    }
}

impl std::ops::Deref for Rig {
    type Target = AppModel;
    fn deref(&self) -> &AppModel {
        &self.model
    }
}

impl std::ops::DerefMut for Rig {
    fn deref_mut(&mut self) -> &mut AppModel {
        &mut self.model
    }
}
