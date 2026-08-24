//! **The frame and its worker, driven by hand** (§7.2) — the two calls a test
//! makes in place of the two threads production runs, split off [`super`] at
//! §12's budget on the seam that file's doc draws: the world a test is *given*
//! is there, what it *does* to that world is here.

use crate::AppModel;

/// The frame and its worker, driven **by hand** (§7.2).
///
/// In production these are two threads: [`Worker`](crate::app::Worker) calls
/// `Deriver::step` forever and the frame calls [`AppModel::refresh`] per paint.
/// A test wants the same two calls in a known order, with no thread and no
/// sleeps, so the rig pairs them and [`tick`](Rig::tick) is one full round —
/// derive, then take. Every timing branch is reached by advancing the injected
/// clock, exactly as before the derivation moved off the frame.
///
/// It derefs to the model so a test asks the *frame* its questions the way the
/// shell does, and reaches `rig.deriver` only for facts the worker owns.
pub(crate) struct Rig {
    pub(crate) model: AppModel,
    pub(crate) deriver: crate::app::Deriver,
}

impl Rig {
    /// One derivation pass followed by one frame's take. Returns whether the
    /// frame's render source moved.
    pub(crate) fn tick(&mut self) -> bool {
        self.deriver.step();
        self.model.refresh()
    }

    /// Publish whatever the worker currently holds, without a pass. The seam
    /// for the Live/InFlight states, which need a held flock no fixture can
    /// take: a test writes the agent row into the worker's tree and this makes
    /// the frame see it, exactly as a real derivation would have.
    pub(crate) fn publish(&mut self) {
        self.deriver.publish();
        self.model.refresh();
    }

    /// The §11 all-collapsed list, asked through the boundary as every seat asks
    /// it (REMOTE §9.7, bl-44e9) — the model holds no conversation accessor any
    /// more, so the rig carries the seat's own door rather than each test
    /// spelling `Query::Conversations` for itself.
    pub(crate) fn conversations(&self, now_unix: i64) -> Vec<crate::nav::convs::ConvRow> {
        crate::test_support::convs::conversations(&self.model, now_unix)
    }

    /// The same answer, folded by a viewport holding `expanded`.
    pub(crate) fn visible_conversations(
        &self,
        now_unix: i64,
        expanded: &std::collections::HashSet<String>,
    ) -> Vec<crate::nav::convs::ConvRow> {
        crate::test_support::convs::visible(&self.model, now_unix, expanded)
    }

    /// The §6 attention-strip total, asked through the boundary (bl-296f) — the
    /// model holds no rollup accessor any more, the top bar folding
    /// `Query::Workspaces` for both the strip and the tabs.
    pub(crate) fn strip_total(&self) -> usize {
        crate::test_support::chrome::strip_total(&self.model)
    }

    /// The §11 workspace tab bar, folded off the same answer.
    pub(crate) fn tab_bar(&self) -> crate::nav::tabs::TabBar {
        crate::test_support::chrome::tab_bar(&self.model)
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
