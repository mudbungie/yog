//! Top-level app state and the render entry points.
//!
//! [`Args`] is the bare binary's own interface; [`AppModel`] is what one
//! running yog holds of its derivation, and since
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
/// **Founding the model, and the one signal it sends the worker** (§7.2) — the
/// pair a caller gets, the first derivation taken synchronously, and
/// [`AppModel::mark_dirty`]. Split off this root at §12's budget on the seam
/// the doc above already draws: what a frame *owns* is declared here, how it is
/// brought into being and how it talks to the worker is there.
mod boot;
pub mod cadence;
mod derive;
mod dirty;
mod drift;
mod grace;
mod roots;
mod snapshot;
/// The model's read surface (§7.2) — split from this root at the cap.
mod view;

use crate::binding::Workspace;
use crate::fs_watcher::RootKind;
use crate::state::{DirtySet, SnapshotCell};
use crate::ui_state::{Clock, UiState};
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
pub use self::roots::Roots;
/// The §3.1 enumeration standing in for the derivation's cached copy at the
/// boundary's intake (bl-6c9e) — re-exported here because the engine builds a
/// gesture's environment with it and this module owns the derivation.
pub(crate) use self::snapshot::addressable;
pub use self::snapshot::growth_label;
pub use self::snapshot::{Growth, Snapshot};

/// **The bare binary's own interface: a name, a version, and no flags**
/// (bl-7942). It carried `--workspace`, the window's startup focus, and the
/// focus went with the window (REMOTE §7). It stays a clap type so `--version`
/// and the top-level `--help` header are rendered from the manifest rather
/// than typed — the roster under that header is [`multiplex::help::COMMANDS`],
/// which is where every verb yog answers to actually lives.
#[derive(Parser, Debug, Clone, PartialEq, Eq)]
#[command(
    version,
    about = "yog: the server for litany loops — the world, the balls and the conversations, \
             behind one control boundary"
)]
pub struct Args {}

/// **What one running yog holds of its own derivation** (§15 Y11, §7.2 as
/// rewritten by bl-ee0a): the roots, the durable `ui.json`, and an
/// `Arc<`[`Snapshot`]`>` — the latest *completed* derivation. Everything that
/// keeps it fresh belongs to [`Deriver`] on another thread.
///
/// It was *what a frame owns* until bl-7942, and the fields that left were all
/// one seat's: the focus, the optimistic echo folded over the derivation for
/// paint, the window's wire channels, its follow lane and its act path. What a
/// seat is looking at, and what it has fired and not yet heard back about, are
/// the seat's own (REMOTE §7, §12) and cross the boundary as gestures.
pub struct AppModel {
    roots: Roots,
    ui: UiState,
    /// The worker's derivation, untouched — the one thing every read is over.
    /// There is no second, optimistically-folded copy any more: the fold was
    /// the paint path's, and §7.2 already said in as many words that *nothing
    /// yog does, and nothing a reader is told, is ever decided by a fact that
    /// is only optimistic*.
    pub(crate) snap: Arc<Snapshot>,
    /// Where the worker publishes.
    cell: SnapshotCell,
    /// The injected time source (§7.2), shared with the worker.
    clock: Arc<dyn Clock>,
    /// How a caller tells the worker it changed something (§7.2).
    dirty: DirtySet,
    /// `$USER` fallback for the claim identity (§4.1) — read from the env
    /// snapshot at the process boundary, never live here (the xdg discipline).
    identity_user: Option<String>,
}

/// The roots the model watches (§7.1): every enumerated workspace, the three
/// enumeration roots (the flat names root, litany workspaces + replays), and the
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
