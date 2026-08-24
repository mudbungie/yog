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
/// **What a SIGTERM means to a running yog** (§8.5, bl-269a): the catch, the
/// flag both faces consult, and the windowless face's loop — which ends by
/// dropping this engine, since the drop already IS the stop.
pub mod stop;
/// The engine's hand-overs to a window's off-frame threads (REMOTE §1.2, §9.8)
/// — split out of this file at §12's budget when the act path landed (bl-4841).
pub mod window;

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
    /// The REMOTE §9.5 wire listener (bl-b6fa) — `None` only where the mint
    /// itself failed, since bl-ae05 made an unprovisioned box found its own
    /// loopback material rather than go without a listener. Held so it lives as
    /// long as the engine and stops when it drops, and **read** by
    /// [`asker`](Self::asker): the window dials the port this actually bound.
    wire: Option<crate::wire::server::Listener>,
    _sentry: Sentry,
    _pilot: Pilot,
    /// The asker's half of the window's read path (REMOTE §1.2, bl-ae05),
    /// minted here beside the listener because both ends of a loopback wire are
    /// this one assembly's. Taken by [`asker`](Self::asker) — a window takes it
    /// and a `yog serve` never does, which is the whole difference between the
    /// two faces here.
    wire_end: Option<crate::wire::link::LinkEnd>,
    /// **The same, once per §8.2 entry** (bl-670c): what each entry channel is,
    /// and the end its own asker answers on. Minted in one act with the model's
    /// half ([`channels::compose`](crate::wire::channels::compose)), so the
    /// pairing is the composition's rather than a join two lists keep in step.
    ///
    /// Taken by [`window_wire`](Self::window_wire) and **not gated**: a second
    /// call already answers `None` on the loopback channel's own end, which is
    /// the one wire a window cannot be a window without, so a second gate here
    /// would be the same fact with a second home. A `yog serve` never takes
    /// them: a headless face is nobody's window and is a client of no other box.
    entry_ends: Vec<crate::wire::channels::EntryEnd>,
    /// The follow lane's engine-side end (REMOTE §3, bl-73e7), minted and taken
    /// on the read path's own terms — one lane per engine, and a `yog serve`
    /// never takes it.
    lane_end: Option<crate::wire::lane::TailEnd>,
    /// The poster's half of the window's **act** path (REMOTE §9.8, bl-4841),
    /// minted beside the read path's end and taken the same way — by
    /// [`poster`](Self::poster), which a window calls and `yog serve` never
    /// does.
    post_end: Option<crate::wire::post::Outbox>,
    /// The face's wake hook, kept so the asker can wake a window when an answer
    /// lands — the same reason the follower holds one.
    repaint: Arc<dyn Repaint>,
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
        // Which clients hold a live connection right now (REMOTE §5, bl-4e08):
        // one handle, minted here because the listener fills it while every
        // answer reads it. RAM by ruling: presence changes with every network
        // blip, so it never reaches the world. **The model no longer holds a
        // copy** (bl-ae05): the frame reads presence off a `Reply` like any
        // other client does, so the second reader went with the second read
        // path.
        let presence = crate::registry::presence::Presence::default();
        // The window's read path (REMOTE §1.2 as ruled, bl-ae05): the frame's
        // half goes to the model, the asker's half is held for whichever face
        // takes it. Minted unconditionally — a `yog serve` simply never asks
        // for the other end, and a model whose link nobody answers is the same
        // code path as a surface whose answer has not landed yet.
        let (link, wire_end) = crate::wire::link::pair();
        // …and one channel per §8.2 entry beside it (bl-028a), composed here
        // because the engine is what holds the world. Zero entries is the local
        // channel alone — the general path with empty inputs. Since bl-670c the
        // composition hands back the far end of every entry channel too, so a
        // window can put an asker on each: one thread, one seat and one slice
        // per channel, which is what makes an unreachable entry cost only its
        // own rows.
        let (channels, entry_ends) = crate::wire::channels::compose(world, link);
        model.adopt_wire(channels);
        // The follow lane's two ends (REMOTE §3, bl-73e7), minted here for the
        // read path's reason exactly. The §7.2 live tail used to be a follower
        // thread on this engine writing into the model's own RAM; it is a held
        // wire read now, so what the engine mints is a channel pair and the
        // face takes the far end or does not.
        let (tail, lane_end) = crate::wire::lane::pair();
        model.adopt_tail(tail);
        // The act path's two ends (REMOTE §9.8, bl-4841), minted here for the
        // read path's reason exactly: both ends of a loopback wire belong to
        // this one assembly, and a face takes the far one or does not.
        let (post, post_end) = crate::wire::post::pair();
        model.adopt_post(post);
        // The §8.5 gestures-inbox consumer: both faces are one consumer surface,
        // so a deposit converges whichever is up (I0).
        let intake = Arc::new(ConsumerCtx {
            lernie: Cli::resolve_in_world(Binary::Lernie, overrides),
            bl: Cli::resolve_in_world(Binary::Bl, overrides),
            state_root: world.yog_state_root(),
            home: world.home_dir(),
            yog_data_root: world.yog_data_root(),
            balls_state_root: model.balls_state_root(),
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
        });
        let consumer = Consumer::spawn(Arc::clone(&intake));
        // The REMOTE §9.5 wire listener (bl-b6fa), beside the consumer and for
        // its reason exactly: a seat must reach whichever face is up, so the
        // channel rides the ENGINE and not a face. It is the same intake — the
        // context above, handed to a connection instead of to a poll — so the
        // wire adds no verb and no second dispatch. A refusal is said twice,
        // once per face (bl-dc14): stderr for `yog serve`, and the model for a
        // window — whose every read and act crosses this wire, so it must
        // paint the refusal (`shell::refusal`) instead of opening inert.
        let wire = match crate::wire::listen(
            world,
            Arc::new(crate::wire::intake::Intake::new(intake))
                as Arc<dyn crate::wire::server::Answerer>,
            presence,
        ) {
            Ok(listener) => Some(listener),
            Err(reason) => {
                eprintln!("yog: wire: {reason}");
                model.refuse_wire(reason);
                None
            }
        };
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
                yog_binary: crate::cli_outbound::self_exe().unwrap_or_default(),
                world: world.clone(),
                home: world.home_dir(),
                yog_data_root: world.yog_data_root(),
                balls_state_root: model.balls_state_root(),
                // Replaced per tick by what the worker has published.
                snapshot: crate::state::latest_snapshot(&model.snapshot_cell()),
                caller: crate::boundary::dispatch::Caller::default(),
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
            wire,
            _sentry: sentry,
            _pilot: pilot,
            wire_end: Some(wire_end),
            entry_ends,
            lane_end: Some(lane_end),
            post_end: Some(post_end),
            repaint,
        }
    }
}

#[cfg(test)]
mod tests;
