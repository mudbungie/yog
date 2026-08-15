//! Top-level app state and the render entry points.
//!
//! [`Args`] is the CLI surface; [`AppModel`] is what a **frame** owns, and since
//! bl-ee0a that is deliberately very little: the `ui.json` document ([`UiState`]
//! — durable pins/collapsed/seen, written through at the gesture), the
//! per-instance [`Focus`] (RAM, §13.1), and an `Arc<`[`Snapshot`]`>` — the
//! latest *completed* derivation. Rendering reads that snapshot and nothing
//! else.
//!
//! **The frame derives nothing** (§7.2). Enumeration, the watchers, the dirty
//! routing, both sweeps, every `GitTree` re-derivation, the ball and ops
//! fetches and the liveness probes all live on [`Deriver`], which one
//! [`Worker`](derive::worker::Worker) thread drives. The frame's per-frame duty
//! is [`AppModel::refresh`]: take the newest snapshot if there is one, adopt an
//! externally-changed `ui.json`, hold the §6 acknowledgement. When the frame
//! *causes* a change (a dispatched verb, a toggled filter) it says so by
//! marking the affected root dirty — the same vocabulary the watchers use, so
//! there is one path into the derivation and no request channel beside it.
//!
//! The impl is split for the 300-line budget: [`derive`] holds the worker's
//! pass, [`focus`] the tab-bar / conversation / attention / seen-acknowledgement
//! surface, [`view`] the read surface a frame paints. This root stays
//! declaration-light so the `pub mod` list carries no coverable `impl` header
//! (llvm-cov phantom).

mod balls;
pub mod cadence;
mod deletes;
mod derive;
mod dirty;
mod drift;
mod echo;
mod focus;
mod grace;
mod knobs;
mod line;
mod live;
mod memo;
mod ops;
mod panels;
mod roots;
mod search;
mod seat;
mod snapshot;
mod spend;
mod view;

use crate::binding::Workspace;
use crate::fs_watcher::RootKind;
use crate::keymap::InspectorTab;
use crate::projects::runner::BlRunner;
use crate::state::{
    DirtySet, SearchCell, SnapshotCell, TailCell, latest_snapshot, new_snapshot_cell,
};
use crate::ui_state::{Clock, UiState};
use crate::watch::Mark;
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;

pub use self::cadence::Cadence;
pub use self::derive::Deriver;
pub use self::derive::worker::Worker;
pub use self::grace::WoundGrace;
pub use self::live::{FollowThread, Follower, LiveTail};
pub(crate) use self::memo::SnapMemo;
pub use self::roots::Roots;
pub use self::snapshot::{Growth, Snapshot};

#[derive(Parser, Debug, Clone, PartialEq, Eq)]
#[command(version, about = "yog: egui frontend for lernie loops")]
pub struct Args {
    /// Workspace to focus at startup (overrides the derived next-attention
    /// focus). Absent ⇒ the first attention-bearing workspace, else the
    /// first (§4.1 startup-focus derivation).
    #[arg(long)]
    pub workspace: Option<PathBuf>,
}

/// The focused viewport position — **per-instance RAM** (§13.1: focus is
/// *which data you look at*, not data; it re-derives at startup and is never
/// mirrored). A focused workspace drives the center panel; a focused agent is
/// the inspector target and the seen-acknowledgement subject (§6); `tab` is the
/// selected §11 Altitude-2 inspector tab (sticky across focus changes).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Focus {
    pub ws: Option<PathBuf>,
    pub agent: Option<String>,
    pub tab: InspectorTab,
}

/// What one frame owns (§15 Y11, §7.2 as rewritten by bl-ee0a). The render
/// source is [`snap`](AppModel::snap); everything that keeps it fresh belongs to
/// [`Deriver`] on another thread.
pub struct AppModel {
    roots: Roots,
    ui: UiState,
    focus: Focus,
    /// **What rendering reads** (§7.2): the latest completed derivation with
    /// the pending echo folded in. Identical to [`derived`](Self::derived) —
    /// the same pointer, allocating nothing — whenever nothing is pending,
    /// which is nearly always.
    pub(crate) snap: Arc<Snapshot>,
    /// **What gestures read**: the worker's derivation, untouched. The §7.2
    /// partition — paint reads the fold, gestures read the derivation — so
    /// nothing yog *does*, and nothing a headless reader is told, is ever
    /// decided by a fact that is only optimistic.
    derived: Arc<Snapshot>,
    /// Which echo [`snap`](Self::snap) was folded from — the fold's own memo.
    /// The rendered `Arc` must be stable while its inputs are, or `SnapMemo`
    /// rebuilds the transcript every frame (§7.2, bl-e90a).
    folded: Option<echo::Echo>,
    /// Where the worker publishes; read once per frame, never per accessor.
    cell: SnapshotCell,
    /// How the frame tells the worker it changed something (§7.2).
    dirty: DirtySet,
    /// The injected time source (§7.2), shared with the worker — the frame's
    /// one use is snapshot age.
    clock: Arc<dyn Clock>,
    /// `$USER` fallback for the claim identity (§4.1) — read from the env
    /// snapshot at the shell boundary, never live here (the xdg discipline).
    identity_user: Option<String>,
    /// The §8.5 search hand-off, shared with this instance's
    /// [`Searcher`](crate::search::Searcher): the frame asks through it and
    /// renders the answer that has landed, never running a search itself
    /// (§7.2 — the frame derives nothing).
    search: SearchCell,
    /// The §3.4 start claim **and the pending echo it carries** (§7.2,
    /// bl-915e): the workspace, the target, the operator's own text and the
    /// landed-message baseline it reconciles against
    /// ([`focus::await_conversation`]). One value, not two — it names the
    /// conversation a fire started, paints what the operator typed, and is
    /// retired by the single predicate that also spends the focus. Per-instance
    /// RAM (§13.1), like the focus it becomes.
    started: Option<echo::Echo>,
    /// The §7.2 **live tail** hand-off (bl-54f7): the frame asks this for the
    /// focused conversation every refresh and folds whatever the
    /// [`Follower`] has published. Purely display, under the
    /// in-memory carve-out — and a **dead end**, which is what the absence of
    /// any accessor beside [`refresh`](Self::refresh)'s own fold enforces.
    tail: TailCell,
    /// Which tail [`snap`](Self::snap) was folded from — the fold's other memo,
    /// beside [`folded`](Self::folded) and for the same reason.
    followed: Option<Arc<LiveTail>>,
    /// Which clients hold a live wire connection right now (REMOTE §5,
    /// bl-4e08) — the listener's own RAM, held by handle so the §11 clients
    /// section paints the flap. Default until the engine hands the real one
    /// over: a model with no engine behind it has no connections, which is the
    /// same posture as a box with no wire.
    presence: crate::registry::presence::Presence,
}

impl AppModel {
    /// Build the frame's model **and** the worker's [`Deriver`], taking the
    /// first derivation synchronously so the window opens on real content and
    /// the §4.1 startup focus has a roster to derive from.
    ///
    /// Returned as a pair rather than spawned here: the thread is the caller's
    /// (`main.rs` hands it an egui repaint hook; a test drives `step()` by
    /// hand), and a model that spawned its own thread could not be exercised
    /// deterministically. This is the same shape the watch bridge always had.
    pub fn boot(
        roots: Roots,
        initial_focus: Option<PathBuf>,
        clock: Arc<dyn Clock>,
        balls: Box<dyn BlRunner>,
        user: Option<String>,
    ) -> (Self, Deriver) {
        let ui = UiState::open(roots.ui_json());
        let dirty = DirtySet::default();
        let cell = new_snapshot_cell(Arc::new(Snapshot::empty(clock.now())));
        let mut deriver = Deriver::new(
            roots.clone(),
            Arc::clone(&clock),
            balls,
            dirty.clone(),
            Arc::clone(&cell),
        );
        deriver.boot();
        let derived = latest_snapshot(&cell);
        let mut model = Self {
            snap: Arc::clone(&derived),
            derived,
            folded: None,
            roots,
            ui,
            focus: Focus::default(),
            cell,
            dirty,
            clock,
            identity_user: user,
            search: SearchCell::default(),
            started: None,
            tail: TailCell::default(),
            followed: None,
            presence: crate::registry::presence::Presence::default(),
        };
        model.focus = model.startup_focus(initial_focus, &Arc::clone(&model.snap).workspaces);
        (model, deriver)
    }

    /// One frame's whole model duty (§7.2): take the newest completed snapshot
    /// if the worker published one, adopt an externally-changed `ui.json` from
    /// it, and hold the §6 acknowledgement. Returns whether the render source
    /// moved.
    ///
    /// It never blocks on a derivation — the only wait is the pointer swap in
    /// [`crate::state`] — and it never *starts* one. A frame that arrives
    /// mid-pass renders the previous snapshot, which is the whole point.
    pub fn refresh(&mut self) -> bool {
        let latest = latest_snapshot(&self.cell);
        let landed = !Arc::ptr_eq(&self.derived, &latest);
        if landed {
            self.derived = latest;
            self.adopt_ui();
        }
        // A fired start focuses the conversation it started (§3.4), the first
        // frame whose roster carries its root, and the pending echo it carries
        // retires on the same predicate (§7.2). Over the roster, not the
        // pointer swap — so it is asked every frame, and free with nothing
        // pending.
        self.adopt_started();
        // Tell the follower which conversation is on screen and take whatever
        // it has folded since the last frame (§7.2 live tail). Both are one
        // lock and one compare, which is what a frame is allowed to cost.
        crate::state::follow(&self.tail, self.followed_subject());
        let followed = crate::state::taken_tail(&self.tail);
        // The one fold of derivation + the non-derived facts (§7.2), run only
        // when one of its inputs moved so the rendered `Arc` stays stable under
        // `SnapMemo`.
        if landed || self.started != self.folded || followed != self.followed {
            self.folded = self.started.clone();
            self.followed = followed;
            self.snap = echo::compose(
                &self.derived,
                self.started.as_ref(),
                self.followed.as_deref(),
            );
        }
        // The §6 ack is a state, not a gesture (bl-aa1f): re-stamp the focused
        // agent's evidence every frame, so a signal that landed on the
        // conversation the operator is reading is already seen. Free — §4.1
        // elides a write whose bytes are unchanged.
        self.ack_focused();
        landed
    }

    /// Adopt an external `ui.json` change the worker read for us (§4.1, I5):
    /// unless it is our own echo (content-hash match), wholesale-adopt it — the
    /// converging seen/pins path both instances share.
    fn adopt_ui(&mut self) {
        if let Some(bytes) = self.derived.ui_bytes.clone()
            && !self.ui.is_echo(&bytes)
        {
            self.ui.adopt(&bytes);
        }
    }

    /// Tell the worker a root changed (§7.2). The frame's *only* outbound
    /// signal: a dispatched verb names the root it touched and the worker's
    /// ordinary routing does the rest, so there is no second path in.
    pub(crate) fn mark_dirty<I: IntoIterator<Item = PathBuf>>(&self, roots: I) {
        self.dirty
            .mark_all(roots.into_iter().map(|r| (r, Mark::Watch)));
    }
}

/// The roots the model watches (§7.1): every enumerated workspace, the three
/// enumeration roots (the flat names root, lernie workspaces + replays), and the
/// yog state root. Missing roots are tolerated —
/// [`WatchSet::reconcile`](crate::watch::WatchSet::reconcile) skips one that fails to arm.
fn desired_watches(roots: &Roots, workspaces: &[Workspace]) -> Vec<(PathBuf, RootKind)> {
    let mut desired = vec![
        (roots.names(), RootKind::NamesRoot),
        (roots.workspaces(), RootKind::WorkspacesRoot),
        (roots.replays(), RootKind::WorkspacesRoot),
        (roots.yog_state.clone(), RootKind::YogState),
        (roots.balls_clones.clone(), RootKind::BallsClones),
    ];
    desired.extend(
        workspaces
            .iter()
            .map(|w| (w.path.clone(), RootKind::Workspace)),
    );
    desired
}

#[cfg(test)]
mod tests;
