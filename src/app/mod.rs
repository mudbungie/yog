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

/// The frame's act half (REMOTE §9.8, bl-4841): a gesture posted over the wire
/// and the receipt that lands frames later.
mod acts;
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
/// The §3.4 raise claim (REMOTE §9.7 class 2, bl-7407): the wall a landed start
/// founded, held until the derivation reads it.
mod raise;
/// One frame's model duty (§7.2) — split out of this root at §12's budget when
/// the act path landed (bl-4841).
mod refresh;
mod roots;
mod search;
mod seat;
mod snapshot;
mod view;

use crate::binding::Workspace;
use crate::fs_watcher::RootKind;
use crate::keymap::InspectorTab;
use crate::projects::runner::BlRunner;
use crate::state::{DirtySet, SearchCell, SnapshotCell, latest_snapshot, new_snapshot_cell};
use crate::ui_state::{Clock, UiState};
use crate::watch::Mark;
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;

pub use self::cadence::Cadence;
pub use self::derive::Deriver;
pub use self::derive::worker::Worker;
/// The §7.2 instrumentation lines the `workspaces` answer carries (bl-b4b5):
/// how late the derivation is, and what grew in it. Re-exported here because
/// the boundary says them and this module derives them — one wording, read at
/// the chokepoint rather than restated for a seat.
pub use self::drift::stale_label;
pub use self::grace::WoundGrace;
pub(crate) use self::memo::SnapMemo;
pub use self::roots::Roots;
/// The §3.1 enumeration standing in for the derivation's cached copy at the
/// boundary's intake (bl-6c9e) — re-exported here because the engine builds a
/// gesture's environment with it and this module owns the derivation.
pub(crate) use self::snapshot::addressable;
pub use self::snapshot::growth_label;
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
///
/// **`ws` is the §3.1 NAME, not a path** (REMOTE §9.7 class 2, bl-7407): the
/// wire spelling, so what this window is looking at is said the same way a
/// gesture addresses it and a reply answers it. A path is resolved at the doors
/// that need one — [`AppModel::focused_workspace`], the pin key, the §3.6
/// delete seat — off the one enumeration
/// ([`Snapshot::ws_path`](snapshot::Snapshot::ws_path)), never joined per paint
/// against a second table.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Focus {
    pub ws: Option<String>,
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
    /// The §3.4 **raise claim** (REMOTE §9.7 class 2, bl-7407): the wall a
    /// landed start just founded, held until the derivation enumerates it. The
    /// start claim's own shape one noun up — an optimistic claim on a thing yog
    /// made, held by what yog knows about it, retired by the derivation showing
    /// it ([`AppModel::adopt_raised`]) and folded into the painted snapshot at the
    /// one seam the echo is ([`echo::compose`]). Without it the focus names a
    /// workspace no enumeration carries for one derivation, and the composer's
    /// bare Enter resolves into the *previous* wall (bl-9acf).
    raised: Option<PathBuf>,
    /// Which raise [`snap`](Self::snap) was folded from — the fold's third
    /// memo, beside [`folded`](Self::folded) and for its reason exactly.
    folded_raise: Option<PathBuf>,
    /// **The window's follow lane** (REMOTE §3, DESIGN §7.2; bl-73e7): the
    /// conversation whose live tail this frame is watching, and the newest fold
    /// that has arrived for it. Default until the engine hands over the live
    /// end — a window with no lane simply paints the tail the pull
    /// `Query::Transcript` folds, which is the same code path and the same
    /// content, one ask period behind.
    lane: crate::wire::lane::Tail,
    /// **The window's read path over the wire** (REMOTE §1.2 as executed,
    /// bl-ae05): the standing questions this frame declares and the decoded
    /// replies that have landed for them. Default until the engine hands over
    /// the live end — a model with no listener behind it asks into a link
    /// nobody answers, which is the same posture, and the same code path, as a
    /// surface whose answer has not arrived yet.
    wire: crate::wire::link::Link,
    /// **Why this window has no wire, when it has none** (bl-dc14): the
    /// engine's refusal — a bind another process beat it to, a mint the box
    /// cannot perform — held so the frame paints it (`shell::refusal`) instead
    /// of controls that only look actionable. `None` is a wired window.
    wire_refusal: Option<String>,
    /// **The window's act path over the wire** (REMOTE §1.2, §9.8; bl-4841):
    /// what this frame has fired and not yet heard back about. Default until
    /// the engine hands over the live end, for [`wire`](Self::wire)'s reason
    /// exactly — a window with nothing behind it earns the sentence saying so
    /// rather than a branch.
    acts: acts::Acts,
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
        let cell = new_snapshot_cell(Arc::new(Snapshot::empty(clock.unix())));
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
            raised: None,
            folded_raise: None,
            lane: crate::wire::lane::Tail::default(),
            wire: crate::wire::link::Link::default(),
            wire_refusal: None,
            acts: acts::Acts::default(),
        };
        model.focus = model.startup_focus(initial_focus, &Arc::clone(&model.snap).workspaces);
        (model, deriver)
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
