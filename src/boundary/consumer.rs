//! The thread the gestures inbox is consumed on (§8.5): the boundary's
//! [`consume`](super::consume::consume) pass, driven beside the derivation
//! worker — never on the frame (§7.2: a gesture spawns verbs, and the window
//! must stay live through them). Both run modes spawn it: the GUI window and
//! `yog serve` are one consumer surface, so a deposit converges whichever
//! face is up (I0).
//!
//! The shell is deliberately the [`Worker`](crate::app::Worker) shape: a stop
//! flag, a park loop, a [`Drop`] that joins. All the logic is the pass, which
//! tests drive directly; the thread gets the one test only a real thread can
//! give it.

use crate::state::SnapshotCell;
use crate::ui_state::{Clock, UiState};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

use super::consume::{consume, run_gesture, run_value};
use super::deposit;
use super::dispatch::Deps;
use serde_json::Value;

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
    /// Which clients hold a live wire connection right now (REMOTE §5,
    /// bl-4e08) — the listener's own RAM, shared by handle so the roster read
    /// answers this instant rather than a copy.
    pub presence: crate::registry::presence::Presence,
    /// What is queued for each client and what came back (REMOTE §5, bl-024b)
    /// — the routing leg's own RAM, shared by handle beside the presence map.
    pub mailbox: crate::registry::mailbox::Mailbox,
}

impl ConsumerCtx {
    /// One pass: skip cheaply when the inbox is empty, else consume it against
    /// the environment [`deps`](Self::deps) builds — the latest published
    /// snapshot over the live workspace enumeration — and a freshly-opened
    /// `ui.json` (the §4.1 write-through copy — the frame adopts any change it
    /// makes, §7.1). The empty-inbox skip is ahead of all of it, so a quiet
    /// world costs one directory listing per poll and not two.
    pub fn pass(&self) -> usize {
        if deposit::pending(&self.state_root).is_empty() {
            return 0;
        }
        let (deps, ts, now_unix) = self.deps(&crate::registry::Client::local(), None);
        let mut ui = UiState::open(self.ui_path.clone());
        consume(&deps, &mut ui, &ts, now_unix)
    }

    /// One gesture envelope, answered where a deposit is answered — **for an
    /// in-world caller**, which is unscoped (REMOTE §3: the inbox is the
    /// world's own residents' door, and they hold no certificate).
    pub fn answer(&self, request: &Value) -> Value {
        let (deps, ts, now_unix) = self.deps(&crate::registry::Client::local(), None);
        let mut ui = UiState::open(self.ui_path.clone());
        run_value(&deps, &mut ui, &ts, now_unix, request)
    }

    /// The same gesture, answered **for a wire client** (REMOTE §4, bl-8bbc):
    /// the world narrowed to that client's registrations, and that client's own
    /// pane document (§7) beside the shared one.
    ///
    /// **Auto-registration on create needs no create-detection.** Under scope a
    /// gesture can name only a workspace the client is registered in — or one
    /// it just founded, which is the single case
    /// [`ws_path`](crate::app::Snapshot::ws_path) could not resolve and the
    /// raise founded anyway. So a *successful* answer naming a workspace
    /// outside the scope is, by construction, a creation, and registering it is
    /// the general path rather than a branch: §4's "a workspace created over
    /// the wire auto-registers its creating client", with nothing to detect.
    pub fn answer_as(&self, client: &crate::registry::Client, request: &Value) -> Value {
        let scope = crate::registry::registered(&self.state_root, client);
        let (deps, ts, now_unix) = self.deps(client, Some(&scope));
        let mut ui = UiState::open_at(
            self.ui_path.clone(),
            crate::registry::pane(&self.state_root, client),
        );
        let Ok(gesture) = super::codec::decode(request) else {
            return run_value(&deps, &mut ui, &ts, now_unix, request);
        };
        let named = gesture.workspace();
        let answered = run_gesture(&deps, &mut ui, &ts, now_unix, &gesture);
        if let Some(name) = named
            && answered.get("kind").is_some()
            && !scope.contains(&name)
        {
            let _ = crate::registry::register(&self.state_root, client, &name);
        }
        answered
    }

    /// The per-gesture [`Deps`] every intake builds — freshly against whatever
    /// the worker has published, with this moment's stamp beside it, and with
    /// the **workspace set re-asked of disk** rather than taken off that
    /// derivation (bl-6c9e: birth is a barrier, see below). `scope` is the
    /// REMOTE §4 narrowing: `None` for an in-world caller, the client's
    /// registered workspace names for a connection.
    fn deps(
        &self,
        client: &crate::registry::Client,
        scope: Option<&std::collections::BTreeSet<String>>,
    ) -> (Deps, String, i64) {
        let ts = self.clock.stamp();
        let now_unix: i64 = ts.parse().unwrap_or(0);
        // **The workspace set is asked, not remembered** (bl-6c9e,
        // [`addressable`](crate::app::addressable)): a `Prepare` that founds a
        // wall answers before the worker has enumerated it, so a resolution over
        // the *cached* set refused the very name the last reply made
        // addressable. Three readdirs (§3.1) on this off-frame intake are what
        // make a success reply a barrier for the call after it.
        let published = crate::app::addressable(
            crate::state::latest_snapshot(&self.cell),
            crate::binding::workspaces(&self.yog_data_root, &self.world.lernie_data_root()),
        );
        let deps = Deps {
            lernie: self.lernie.clone(),
            bl: self.bl.clone(),
            state_root: self.state_root.clone(),
            yog_binary: self.yog_binary.clone(),
            world: self.world.clone(),
            home: self.home.clone(),
            yog_data_root: self.yog_data_root.clone(),
            balls_state_root: self.balls_state_root.clone(),
            // **The one filter** (REMOTE §4): scoping is a narrowing of the
            // published derivation, so every enumeration answers the registered
            // set and every resolution refuses an unregistered name in the same
            // words a name nobody founded earns. Absence, never a scope error.
            snapshot: match scope {
                Some(allowed) => Arc::new(published.scoped(allowed)),
                None => published,
            },
            // No held preview to agree with headlessly — any seed is a fair
            // mint draw; the ts keeps successive passes distinct.
            // **Who is asking, and who else is connected** (REMOTE §4, §5).
            // The identity is the intake's — `local` for the world's own
            // residents, the certificate's common name for a connection — and
            // it is what the §5 advertisement lands under.
            caller: crate::boundary::dispatch::Caller {
                client: client.clone(),
                presence: self.presence.clone(),
                mailbox: self.mailbox.clone(),
            },
        };
        (deps, ts, now_unix)
    }
}

/// The consumer thread. Owns its join handle and a stop flag; [`Drop`] signals
/// stop, unparks, and joins — the worker's own shutdown shape (§7.2).
pub struct Consumer {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Consumer {
    /// Run [`ConsumerCtx::pass`] forever, parked between looks. The context is
    /// **shared, not owned**: the §9.5 wire listener answers connections
    /// through the very same one (bl-b6fa).
    pub fn spawn(ctx: Arc<ConsumerCtx>) -> Self {
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
