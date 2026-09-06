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
//! what the window is: an event loop to repaint ([`Repaint`]), the four
//! off-frame wire halves ([`Engine::window_wire`] — asker, poster, follow lane
//! and the §8.5 searcher, which the windowless face needs none of, since every
//! headless seat already answers in place and off-frame), and the §5.3 RAM
//! surfaces a pointer needs.
//!
//! The engine spawns five threads and **no frame** — which is the §7.2
//! invariant stated from the other side: everything yog does other than paint
//! happens here, so nothing that runs long has a frame to block.

/// **The windowless face, whole** (§8.5) — `yog serve`, which left `main.rs`
/// for that file's own coverage reason once bl-269a gave its loop an exit.
pub mod serve;
/// **One world has one engine** (§8.5, bl-1d9b): the exclusion a boot takes
/// before it consumes anything, and why it is a lock rather than the bind.
pub mod sole;
/// **What a SIGTERM means to a running yog** (§8.5, bl-269a): the catch, the
/// flag both faces consult, and the windowless face's loop — which ends by
/// dropping this engine, since the drop already IS the stop.
pub mod stop;

use crate::AppModel;
use crate::app::{Roots, Worker};
use crate::boundary::consumer::{Consumer, ConsumerCtx};
use crate::boundary::dispatch::Deps;
use crate::cli_outbound::{Binary, Cli};
use crate::config_edit;
use crate::fleet::{Pilot, PilotCtx};
use crate::monitor::{BzCaller, Sentry, SentryCtx};
use crate::projects::runner::BlStore;
use crate::ui_state::Clock;
use crate::watch::Bridge;
use crate::xdg::Env;
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
    /// The REMOTE §9.5 wire listener (bl-b6fa). **Not an `Option` since
    /// bl-1d9b**: a yog that cannot listen is not an engine — the window that
    /// used to justify carrying on without one left with bl-7942, so the only
    /// thing a wire-less engine could still do was drain the world's gesture
    /// inbox, which is the half-engine that ball is about. A refusal is fatal
    /// and names the address. Held so it lives as long as the engine and stops
    /// when it drops.
    _wire: crate::wire::server::Listener,
    _sentry: Sentry,
    _pilot: Pilot,
    /// **The world's one-engine exclusion** ([`sole`], bl-1d9b), declared last
    /// so it is dropped last: the lock is released only once every thread above
    /// has stopped and joined, so the next engine never overlaps this one.
    _sole: sole::Sole,
}

impl Engine {
    /// Boot the engine into `world` (already composed, §16.2) with `overrides`
    /// standing on every child spawn — **or refuse, in one sentence the caller
    /// prints and exits on** (bl-1d9b).
    ///
    /// Two refusals, and each is the same statement from a different side.
    /// [`sole::take`] is *one world has one engine*: it is taken first, before
    /// a single gesture is consumed, so a second `yog` on a held world is
    /// refused rather than becoming a second consumer of one inbox with a
    /// second `inv-N` namespace behind it. [`wire::listen`](crate::wire::listen)
    /// is *an engine listens*: since bl-7942 there is no window, so every read
    /// and every act crosses that wire (REMOTE §1.2) and a yog without one
    /// answers no seat at all. It is asked **before** the consumer is spawned,
    /// so a boot that will refuse has taken nothing out of the world's inbox.
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
        clock: Arc<dyn Clock>,
    ) -> Result<Self, String> {
        let sole = sole::take(&world.yog_state_root())?;
        let now_secs = clock.stamp().parse().unwrap_or(0);
        config_edit::branch::edit::sweep_staging(&world.yog_stage_root(), now_secs);
        crate::scratch::sweep(&crate::scratch::dirs(world), now_secs);
        let roots = Roots::of(world);
        // Ball reads are IN-PROCESS (§16.7 W8): balls' own layout over the world
        // env resolves the nested store checkout, and the `bl` Cli rides along
        // only for the one history-served read, which spawns `yog bl …`.
        let balls = Box::new(BlStore::new(
            world.clone(),
            Cli::resolve_in_world(Binary::Bl, overrides),
        ));
        // `boot` takes the first derivation synchronously — every workspace
        // enumerated and snapshotted, the watches armed — and hands back the
        // `Deriver` the worker then owns forever.
        let (model, deriver) = AppModel::boot(roots, Arc::clone(&clock), balls, world.user());
        let bridge = Bridge::spawn(deriver.watchset_handle(), deriver.dirty_handle());
        let worker = Worker::spawn(deriver);
        // Which clients hold a live connection right now (REMOTE §5, bl-4e08):
        // one handle, minted here because the listener fills it while every
        // answer reads it. RAM by ruling: presence changes with every network
        // blip, so it never reaches the world. **The model no longer holds a
        // copy** (bl-ae05): the frame reads presence off a `Reply` like any
        // other client does, so the second reader went with the second read
        // path.
        let presence = crate::registry::presence::Presence::default();
        // The §8.5 gestures-inbox consumer: both faces are one consumer surface,
        // so a deposit converges whichever is up (I0).
        let intake = Arc::new(ConsumerCtx {
            litany: Cli::resolve_in_world(Binary::Litany, overrides),
            bl: Cli::resolve_in_world(Binary::Bl, overrides),
            state_root: world.yog_state_root(),
            home: world.home_dir(),
            yog_data_root: world.yog_data_root(),
            yog_binary: crate::cli_outbound::self_exe().unwrap_or_default(),
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
            // The §8.3 sign-in runs (REMOTE §8.3, bl-c285), minted here beside
            // them and over the **world-nested** `bz` every other substrate
            // spawn goes through: the act layers the named workspace's wall on
            // top of it, so the credential lands in that sphere (§16.2).
            logins: crate::login::runs::Runs::of(Cli::resolve_in_world(Binary::Bz, overrides)),
        });
        // The REMOTE §9.5 wire listener (bl-b6fa), ahead of the consumer and
        // for the reason bl-1d9b names: a seat must reach whichever face is up,
        // so the channel rides the ENGINE and not a face. It is the same intake
        // — the context above, handed to a connection instead of to a poll — so
        // the wire adds no verb and no second dispatch.
        //
        // **A refusal is fatal, and it is fatal here so nothing has been
        // consumed yet** (bl-1d9b). It used to be said on stderr while the
        // engine ran on without a wire, on the argument that every deposit
        // still converges through the inbox and only a seat is shut out. That
        // argument died with the window (bl-7942): the seat IS the face, so a
        // wire-less yog is a process that answers nobody and drains the world's
        // gesture inbox while looking healthy — which is worse than no engine,
        // because the operator cannot see it. The refusal is returned rather
        // than printed, since one caller owns the saying and the exit code.
        let wire = crate::wire::listen(
            world,
            Arc::new(crate::wire::intake::Intake::new(Arc::clone(&intake)))
                as Arc<dyn crate::wire::server::Answerer>,
            presence,
        )
        .map_err(|reason| format!("wire: {reason}"))?;
        // **The engine says what it bound** (REMOTE §8, bl-e058). A `:0` in
        // `address` is a request the kernel answers in RAM, and the in-process
        // consumer that used to be told the answer was the window, which left
        // with bl-7942 — so on a self-provisioned box the bound port was
        // knowable only by asking the kernel about the process. A server
        // announcing its endpoint is ordinary, and this is the success arm of a
        // line the refusal already had. It is not a second address file:
        // `address` stays the operator's *request* and its one home (bl-dc14),
        // and this says what that request became on this boot.
        eprintln!("yog: wire: listening on {}", wire.address());
        // The §8.5 gestures-inbox consumer starts only once the boot can no
        // longer refuse: one door is open, so the other may be too.
        let consumer = Consumer::spawn(intake);
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
                litany: Cli::resolve_in_world(Binary::Litany, overrides),
                bl: Cli::resolve_in_world(Binary::Bl, overrides),
                state_root: world.yog_state_root(),
                yog_binary: crate::cli_outbound::self_exe().unwrap_or_default(),
                world: world.clone(),
                home: world.home_dir(),
                yog_data_root: world.yog_data_root(),
                // Replaced per tick by what the worker has published.
                snapshot: crate::state::latest_snapshot(&model.snapshot_cell()),
                caller: crate::boundary::dispatch::Caller::default(),
            },
            cell: model.snapshot_cell(),
            clock: pilot_clock,
            ui_path: model.ui_json_path(),
        });
        Ok(Self {
            model,
            _bridge: bridge,
            _worker: worker,
            _consumer: consumer,
            _wire: wire,
            _sentry: sentry,
            _pilot: pilot,
            _sole: sole,
        })
    }
}

#[cfg(test)]
mod tests;
