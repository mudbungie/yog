//! The §3.5 spend queries on [`AppModel`] — what a frame (or a headless
//! caller) asks for a figure.
//!
//! Both are pure reads **over the held snapshot alone** — the `steps/` walk
//! they used to make on the frame thread is now the worker's, ridden out as
//! `Snapshot::bills` (bl-9dd4), which is what lets the V4 board carry a spend
//! column without a disk read per row per frame (§7.2). Nothing is stored, and
//! asking twice re-derives. The join itself lives in [`crate::spend`]; this is
//! only the part that needs the model's view of the world — which roots stamp a
//! ball, and where the price table is.
//!
//! A child module of `app` for the 300-line budget (§12), like
//! [`super::knobs`] and [`super::balls`].

use std::path::Path;

use super::AppModel;
use crate::spend::{Figure, Prices};

impl AppModel {
    /// The §3.5 price table as `ui.json` currently has it. Empty ⇒ every
    /// figure below renders tokens only.
    pub fn prices(&self) -> Prices {
        self.ui.prices()
    }

    /// One workspace's already-walked bills (§3.5) — the worker's fold, empty
    /// for a workspace no pass has reached yet. The general path with no
    /// inputs, never a bootstrap branch.
    pub fn bills(&self, ws: &Path) -> Vec<crate::budgets::StepBill> {
        self.snap.bills.get(ws).cloned().unwrap_or_default()
    }

    /// One conversation's priced whole-tree figure — the root agent and its
    /// descent (ARCH §6), attributed to itself.
    pub fn conversation_spend(&self, ws: &Path, root_id: &str) -> Figure {
        crate::spend::of_conversation(&self.bills(ws), root_id, &self.prices())
    }

    /// One conversation's **context fullness** (§5.1 #35) — the prompt its root
    /// agent's latest step sent, against the window `models.yaml` declares for
    /// the model that sent it. `None` when nothing measured can be said.
    ///
    /// It sits beside [`conversation_spend`](Self::conversation_spend) because
    /// both are filters over the same held snapshot, and deliberately *apart*
    /// from it in what it answers: spend is the whole descent's cumulative
    /// burn, fullness is this conversation's one current prompt.
    pub fn conversation_context(
        &self,
        ws: &Path,
        root_id: &str,
    ) -> Option<crate::context::Fullness> {
        crate::context::of_conversation(&self.bills(ws), root_id, &self.snap.windows)
    }

    /// A ball's priced figure as this workspace can attribute it (§3.5's
    /// ruling): the conversations whose goal stamps the ball when any does,
    /// else the whole workspace, labelled as such by
    /// [`crate::spend::Attribution`]. No linkage fact is invented for the
    /// unstamped case — the figure widens and says so.
    pub fn ball_spend(&self, ws: &Path, ball_id: &str) -> Figure {
        crate::spend::of_ball(
            &self.bills(ws),
            &self.stamped_roots(ws, ball_id),
            &self.prices(),
        )
    }

    /// Every root in `ws` whose goal stamps `ball_id` (§3.3), deduplicated and
    /// ordered. A stamp is resolved **to its root** before it counts: two
    /// stamps in one descent are one tree, and summing both would bill it
    /// twice.
    ///
    /// `pub(crate)` since bl-9dd4: the board asks it per row, and the drone
    /// rows a claimed ball renders are *this same set* — one derivation
    /// answering "whose spend is this" and "which conversation is on it".
    pub(crate) fn stamped_roots(&self, ws: &Path, ball_id: &str) -> Vec<String> {
        crate::board::stamped_roots(&self.snap.trees, ws, ball_id)
    }

    /// The V4 board (VISION §5 V4) over this instance's snapshot, price table
    /// and ceiling — the frame's delegation to the same [`crate::board::build`]
    /// the §8.5 `Query::Board` answers. One derivation, two serializations.
    ///
    /// The wall clock is this model's injected one (§7.2), so the loop facts'
    /// ages are the same clock every other age on screen is measured against.
    pub fn board(&self) -> crate::board::Board {
        crate::board::build(&self.snap, &self.ui, self.now_unix())
    }
}
