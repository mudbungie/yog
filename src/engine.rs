//! **The engine both faces run** (VISION §5 V5.1, DESIGN §8.5): the model, the
//! derivation worker, the watch bridge and the gesture consumer, assembled
//! once from a composed world.
//!
//! V5's claim is verbatim that *"headless mode is the same binary, minus the
//! window"*, and V5.4 that *"nothing here is a second implementation."* This
//! module is where that stops being an intention. Before it, `main.rs` carried
//! the assembly twice — once inside the eframe closure and once in the
//! windowless arm — in the one file `tarpaulin.toml` excludes, so the two
//! copies were free to drift and no test could notice. Now there is one
//! [`Engine::boot`] and two callers, and what a face adds beside it is exactly
//! what the window is: an event loop to repaint ([`Repaint`]), a search thread
//! ([`AppModel::searcher`] — the windowless face needs none, since every
//! headless seat already answers a search off-frame), and the §5.3 RAM
//! surfaces a pointer needs.
//!
//! The engine spawns six threads and **no frame** — which is the §7.2
//! invariant stated from the other side: everything yog does other than paint
//! happens here, so nothing that runs long has a frame to block.

use crate::AppModel;
use crate::app::{FollowThread, Roots, Worker};
use crate::boundary::consumer::{Consumer, ConsumerCtx};
use crate::boundary::dispatch::Deps;
use crate::cli_outbound::{Binary, Cli};
use crate::config_edit;
use crate::fleet::{Pilot, PilotCtx};
use crate::monitor::{BzCaller, Sentry, SentryCtx};
use crate::projects::runner::BlStore;
use crate::ui_state::Clock;
use crate::watch::{Bridge, Repaint};
use crate::xdg::Env;
use std::path::PathBuf;
use std::sync::Arc;

/// What one running yog *is*, minus its face. The model is public because a
/// face renders it; the three threads are held only so they live as long as
/// the engine and are stopped and joined when it drops (each owns that shape
/// itself — §7.2).
pub struct Engine {
    pub model: AppModel,
    _bridge: Bridge,
    _worker: Worker,
    _consumer: Consumer,
    /// The REMOTE §9.5 wire listener (bl-b6fa) — `None` on a box with no
    /// certificates provisioned, which is every box until an operator performs
    /// the out-of-channel act (REMOTE §1.4). Held only so it lives as long as
    /// the engine and stops when it drops.
    _wire: Option<crate::wire::server::Listener>,
    _sentry: Sentry,
    _pilot: Pilot,
    _follower: FollowThread,
}

impl Engine {
    /// Boot the engine into `world` (already composed, §16.2) with `overrides`
    /// standing on every child spawn. `initial_focus` is the `--workspace`
    /// argument the window takes and the windowless face has no use for;
    /// `repaint` is the face's wake hook — [`EguiRepaint`](crate::watch::EguiRepaint)
    /// at a window, [`NoRepaint`](crate::watch::NoRepaint) without one.
    ///
    /// The §5.2 startup sweep runs here rather than at either caller: dropping
    /// stale scratch is the *engine's* housekeeping, and its wall clock is the
    /// injected [`Clock`] every other timestamp already comes from — so a test
    /// advances it like anything else. It is **both** of §5.2's transient
    /// artifacts off one clock read: the scripted-editor staging dirs, and
    /// (bl-e47c) the I3 temps left in the destination directories yog writes
    /// through — the half the doc had promised since I3 and nobody had written.
    pub fn boot(
        world: &Env,
        overrides: &[(String, String)],
        initial_focus: Option<PathBuf>,
        clock: Arc<dyn Clock>,
        repaint: Arc<dyn Repaint>,
    ) -> Self {
        let now_secs = clock.stamp().parse().unwrap_or(0);
        config_edit::branch::edit::sweep_staging(&world.yog_stage_root(), now_secs);
        crate::scratch::sweep(&crate::scratch::dirs(world), now_secs);
        let roots = Roots::of(world);
        // Ball reads are IN-PROCESS (§16.7 W8): balls' own layout over the world
        // env resolves the nested store checkout, and the `bl` Cli rides along
        // only for the one history-served read, which spawns `yog bl …`.
        let balls = Box::new(BlStore::new(
            world.balls_layout(),
            Cli::resolve_in_world(Binary::Bl, overrides),
        ));
        // `boot` takes the first derivation synchronously — every workspace
        // enumerated and snapshotted, the watches armed, the startup focus
        // derived — and hands back the `Deriver` the worker then owns forever.
        let (mut model, deriver) = AppModel::boot(
            roots,
            initial_focus,
            Arc::clone(&clock),
            balls,
            world.user(),
        );
        let bridge = Bridge::spawn(deriver.watchset_handle(), deriver.dirty_handle());
        let worker = Worker::spawn(deriver, Arc::clone(&repaint));
        // The §7.2 live tail (bl-54f7): the focused conversation's open
        // `response.json`, followed at frame cadence. It rides the engine
        // beside the worker because a *reader* of one file is not a face's
        // concern — the windowless seat simply has a `NoRepaint` to wake.
        let follower = model.follower().spawn(repaint);
        // Which clients hold a live connection right now (REMOTE §5, bl-4e08):
        // one handle, minted here because the listener fills it while every
        // answer — and the frame's own clients section — reads it. RAM by
        // ruling: presence changes with every network blip, so it never
        // reaches the world.
        let presence = crate::registry::presence::Presence::default();
        model.adopt_presence(presence.clone());
        // The §8.5 gestures-inbox consumer: both faces are one consumer surface,
        // so a deposit converges whichever is up (I0).
        let intake = Arc::new(ConsumerCtx {
            lernie: Cli::resolve_in_world(Binary::Lernie, overrides),
            bl: Cli::resolve_in_world(Binary::Bl, overrides),
            state_root: world.yog_state_root(),
            home: world.home_dir(),
            yog_data_root: world.yog_data_root(),
            balls_state_root: model.balls_state_root(),
            yog_binary: std::env::current_exe().unwrap_or_default(),
            world: world.clone(),
            ui_path: model.ui_json_path(),
            cell: model.snapshot_cell(),
            clock: Arc::clone(&clock),
            presence: presence.clone(),
            // The routing leg's mailbox (REMOTE §5, bl-024b), minted here
            // beside presence and for its reason: the listener's connections
            // drain it while the deposit inbox's callers fill it, so one
            // handle, held by the one context both intakes answer through.
            mailbox: crate::registry::mailbox::Mailbox::default(),
        });
        let consumer = Consumer::spawn(Arc::clone(&intake));
        // The REMOTE §9.5 wire listener (bl-b6fa), beside the consumer and for
        // its reason exactly: a seat must reach whichever face is up, so the
        // channel rides the ENGINE and not a face. It is the same intake — the
        // context above, handed to a connection instead of to a poll — so the
        // wire adds no verb and no second dispatch. With no material
        // provisioned there is no listener and nothing said about it.
        let wire = crate::wire::listen(
            world,
            Arc::new(crate::wire::intake::Intake::new(intake))
                as Arc<dyn crate::wire::server::Answerer>,
            presence,
        );
        // The VISION §4.9 alignment monitor's level trigger. Spawned
        // unconditionally and free when unarmed: with no `cadence.yaml` monitor
        // entry a tick finds no workspace to check and makes no call. It rides
        // the engine rather than a face for the same reason the consumer does —
        // arming is a fact of the world, not of the seat.
        let pilot_clock = Arc::clone(&clock);
        let sentry = Sentry::spawn(SentryCtx {
            state_root: world.yog_state_root(),
            cell: model.snapshot_cell(),
            clock,
            caller: Box::new(BzCaller::new(world.clone())),
        });
        // The VISION §4.3 armed loop's level trigger, beside the sentry and
        // free for the same reason: with no `cadence.yaml` fleet entry a tick
        // reads the published snapshot, finds nothing armed and returns before
        // it builds a board or opens a file. Arming is a fact of the world, not
        // of the seat, so it rides the engine and both faces run it.
        let pilot = Pilot::spawn(PilotCtx {
            deps: Deps {
                lernie: Cli::resolve_in_world(Binary::Lernie, overrides),
                bl: Cli::resolve_in_world(Binary::Bl, overrides),
                state_root: world.yog_state_root(),
                yog_binary: std::env::current_exe().unwrap_or_default(),
                world: world.clone(),
                home: world.home_dir(),
                yog_data_root: world.yog_data_root(),
                balls_state_root: model.balls_state_root(),
                // Both are replaced per tick — the snapshot by what the worker
                // has published, the seed by that tick's own stamp.
                snapshot: crate::state::latest_snapshot(&model.snapshot_cell()),
                caller: crate::boundary::dispatch::Caller::default(),
                mint_seed: 0,
            },
            cell: model.snapshot_cell(),
            clock: pilot_clock,
            ui_path: model.ui_json_path(),
        });
        Self {
            model,
            _bridge: bridge,
            _worker: worker,
            _consumer: consumer,
            _wire: wire,
            _sentry: sentry,
            _pilot: pilot,
            _follower: follower,
        }
    }
}

#[cfg(test)]
mod tests;
