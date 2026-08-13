//! The thread the gestures inbox is consumed on (§8.5): the boundary's
//! [`consume`](super::consume::consume) pass, driven beside the derivation
//! worker — never on the frame (§7.2: a gesture spawns verbs, and the window
//! must stay live through them). Both run modes spawn it: the GUI window and
//! `yog headless` are one consumer surface, so a deposit converges whichever
//! face is up (I0).
//!
//! The shell is deliberately the [`Worker`](crate::app::Worker) shape: a stop
//! flag, a park loop, a [`Drop`] that joins. All the logic is the pass, which
//! tests drive directly; the thread gets the one test only a real thread can
//! give it.

use crate::state::SnapshotCell;
use crate::ui_state::{Clock, UiState, content_hash};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

use super::consume::consume;
use super::deposit;
use super::dispatch::Deps;

/// How often the consumer looks for deposits. A latency knob, not a
/// correctness one: an unconsumed deposit waits, it never rots (I0).
const CONSUMER_POLL: Duration = Duration::from_millis(250);

/// What the consumer thread needs to build a fresh [`Deps`] per pass: the
/// verb binaries and roots (fixed at boot), the snapshot cell the worker
/// publishes to, the durable `ui.json` path, and the clock.
pub struct ConsumerCtx {
    pub lernie: crate::cli_outbound::Cli,
    pub bl: crate::cli_outbound::Cli,
    pub state_root: PathBuf,
    pub home: PathBuf,
    pub yog_data_root: PathBuf,
    pub balls_state_root: PathBuf,
    /// yog's own binary — the `$EDITOR` shim a §9.3 lineage write re-enters.
    pub yog_binary: PathBuf,
    /// The composed world (§16.2) — what the §9 config family folds its
    /// destinations from and asks brazen through (bl-3f46).
    pub world: crate::xdg::Env,
    pub ui_path: PathBuf,
    pub cell: SnapshotCell,
    pub clock: Arc<dyn Clock>,
}

impl ConsumerCtx {
    /// One pass: skip cheaply when the inbox is empty, else consume it against
    /// the latest published snapshot and a freshly-opened `ui.json` (the §4.1
    /// write-through copy — the frame adopts any change it makes, §7.1).
    pub fn pass(&self) -> usize {
        if deposit::pending(&self.state_root).is_empty() {
            return 0;
        }
        let ts = self.clock.stamp();
        let now_unix: i64 = ts.parse().unwrap_or(0);
        let deps = Deps {
            lernie: self.lernie.clone(),
            bl: self.bl.clone(),
            state_root: self.state_root.clone(),
            yog_binary: self.yog_binary.clone(),
            world: self.world.clone(),
            home: self.home.clone(),
            yog_data_root: self.yog_data_root.clone(),
            balls_state_root: self.balls_state_root.clone(),
            snapshot: crate::state::latest_snapshot(&self.cell),
            // No held preview to agree with headlessly — any seed is a fair
            // mint draw; the ts keeps successive passes distinct.
            mint_seed: content_hash(ts.as_bytes()),
        };
        let mut ui = UiState::open(self.ui_path.clone());
        consume(&deps, &mut ui, &ts, now_unix)
    }
}

/// The consumer thread. Owns its join handle and a stop flag; [`Drop`] signals
/// stop, unparks, and joins — the worker's own shutdown shape (§7.2).
pub struct Consumer {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Consumer {
    /// Run [`ConsumerCtx::pass`] forever, parked between looks.
    pub fn spawn(ctx: ConsumerCtx) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let handle = std::thread::spawn(move || {
            while !flag.load(Ordering::Relaxed) {
                ctx.pass();
                std::thread::park_timeout(CONSUMER_POLL);
            }
        });
        Self {
            stop,
            handle: Some(handle),
        }
    }
}

impl Drop for Consumer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            handle.thread().unpark();
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests;
