//! **The engine's hand-overs to a window** (REMOTE §1.2 as ruled 2026-08-14,
//! §9.8): the two off-frame threads that make the local window a wire client of
//! the engine it just booted, and the one seat they are both minted from.
//!
//! Two roles in one process and one boundary between them — which is REMOTE
//! §8's *one world, one engine* ruling kept intact. The window boots the engine
//! it serves, exactly as before, and then talks to it over nothing but the wire:
//! it is a client of `127.0.0.1` at the port the listener actually bound, so the
//! address is handed over in RAM rather than read back out of a file, and a `:0`
//! in `address` is a request nobody has to resolve twice.
//!
//! All three ends are **taken**, not shared: a second call answers `None`.
//! There is one asker **per channel** (REMOTE §8.2, bl-670c), one poster and one
//! follow lane per engine, and for the act path that is load bearing — an act
//! must be sent exactly once.
//!
//! **The window is a client of every channel it holds** (§8.2), so what this
//! module hands over is that set seen from the engine's end ([`channels`]): an
//! [`asker`](crate::wire::asker) per channel — one thread, one seat, one slice,
//! failing only itself — and one [`Dial`](crate::wire::dial::Dial) each for the
//! three threads that route rather than stand.

/// **The window's channel set, seated** (REMOTE §8.2, bl-670c) — one asker per
/// channel, and the dial the routing threads share the shape of. Pre-split from
/// this file at §12's band, where the act path split it once before.
mod channels;

use super::Engine;
use crate::wire::dial::Dial;
use crate::xdg::Env;
use std::sync::Arc;

/// **All of a window's off-frame halves**, taken together because they are one
/// hand-over and one lifetime: the [`asker`](crate::wire::asker) polling the
/// standing reads at human cadence, the [`poster`](crate::wire::poster) sending
/// what the frame fires, and — since bl-73e7 — the [`lane`](crate::wire::lane)
/// holding the live tail open at the rate the model writes it.
///
/// Held by the face so they live as long as the window. They stop differently
/// and each in its own right: the asker is a poll, so its handle signals, unparks
/// and joins; the poster is parked on a channel, so it ends when the model's
/// outbox drops and its handle is only a way to wait for that.
pub struct WindowWire {
    /// **One per channel** (§8.2, bl-670c), the loopback engine's first and then
    /// one per entry in leaf order. A `Vec` rather than a field and a list,
    /// because from here they are interchangeable: each is one thread holding
    /// one seat and one slice, and stopping them is stopping each of them.
    _askers: Vec<crate::wire::asker::AskerThread>,
    _poster: std::thread::JoinHandle<()>,
    /// The §8.5 searcher, here since bl-44e9 because its read crosses the wire
    /// too (REMOTE §9.7) — a third half of one hand-over, on the same seat mint
    /// and the same failure condition. Its own thread, and §9.7 ruled that it
    /// stays one: a search walks every transcript in the world, so riding the
    /// asker would make a once-per-ask walk a 2 Hz one and put it in front of
    /// every other surface's answer.
    _searcher: crate::search::SearchThread,
    /// The **follow lane** (REMOTE §3, bl-73e7) — the fourth half, and the one
    /// that exists because the asker's pass is serial: a read held open on the
    /// live tail would stall every other surface for its whole duration, so it
    /// gets a connection and a thread of its own. Its handle signals and
    /// unparks but does not join, for the reason
    /// [`LaneThread`](crate::wire::lane::LaneThread) states.
    _lane: crate::wire::lane::LaneThread,
}

impl Engine {
    /// Spawn both halves for a window. `None` on a box whose mint failed — the
    /// same condition for both, there being one seat behind them — and on a
    /// second call, the two ends being taken rather than shared.
    ///
    /// **A `None` is never silent** (bl-dc14): a seat that cannot open puts
    /// its reason on the model, where the frame paints it instead of controls
    /// (`shell::refusal`) — the model keeps the *first* reason, so the boot's
    /// own bind refusal outranks the "no listener" that follows from it. A
    /// seat that CAN open but whose ends are already taken records nothing:
    /// a taken end means a wired window exists, and this call is the second.
    pub fn window_wire(&mut self, world: &Env) -> Option<WindowWire> {
        if let Err(reason) = self.window_seat(world) {
            self.model.refuse_wire(reason);
            return None;
        }
        // The entry channels' far ends, taken with everything else this window
        // takes once. What each of them *is* stays behind as `entries`, because
        // the three routing threads each seat themselves on it. Ungated: the
        // loopback channel's end is what a second call already fails on.
        let held = std::mem::take(&mut self.ends.entries);
        let entries: Vec<crate::wire::entries::Entry> =
            held.iter().map(|one| one.entry.clone()).collect();
        Some(WindowWire {
            _askers: self.askers(world, held)?,
            _poster: self.poster(self.dial(world, &entries)?)?.start(),
            _searcher: self.searcher(self.dial(world, &entries)?)?.start(),
            _lane: self.lane(self.dial(world, &entries)?)?.start(),
        })
    }

    /// **The window's asker** (REMOTE §1.2 as ruled 2026-08-14, bl-ae05): a
    /// seat on this engine's own listener, over loopback, presenting the window
    /// leaf.
    ///
    /// Two roles in one process and one boundary between them — which is §8's
    /// *one world, one engine* ruling kept intact. The window boots the engine
    /// it serves, exactly as before, and then talks to it over nothing but the
    /// wire: it is a client of `127.0.0.1` at the port the listener actually
    /// bound, so the address is handed over in RAM rather than read back out of
    /// a file, and a `:0` in `address` is a request nobody has to resolve
    /// twice.
    ///
    /// `None` when this box got no listener up or has no window leaf — both
    /// meaning a broken mint, which [`listen`](crate::wire::listen) has already
    /// said so about on stderr. It takes the link end, so a second call answers
    /// `None`: there is one asker per engine.
    pub fn asker(&mut self, world: &Env) -> Option<crate::wire::asker::Asker> {
        Some(crate::wire::asker::Asker::new(
            self.window_seat(world).ok()?,
            self.ends.link.take()?,
            self.model.snapshot_cell(),
            world.yog_state_root(),
            Arc::clone(&self.repaint),
        ))
    }

    /// **The window's poster** (REMOTE §9.8, bl-4841): the asker's twin on the
    /// write side, on its own seat and its own thread.
    ///
    /// A second `Dial` rather than a shared one, because the threads dial
    /// independently and a seat is a configuration and an address (REMOTE §6) —
    /// nothing is held to share, one channel or twenty. It takes the outbox end,
    /// so there is one poster per engine for the reason there is one asker per
    /// channel: an act must be sent exactly once, and two posters draining one
    /// queue would be two windows' worth of gestures with no way to tell whose
    /// is whose. **Routing does not touch that** (§8.2): it picks which channel
    /// the one send goes down.
    pub fn poster(&mut self, dial: Dial) -> Option<crate::wire::poster::Poster> {
        Some(crate::wire::poster::Poster::new(
            dial,
            self.ends.post.take()?,
            Arc::clone(&self.repaint),
        ))
    }

    /// **The window's searcher** (§8.5; REMOTE §9.7, bl-44e9): the third thread
    /// on the same mint, asking `Query::Search` over the wire instead of walking
    /// this instance's own snapshot.
    ///
    /// It takes nothing, unlike the two above: the ask cell is a value the model
    /// shares rather than an end handed over once, because a search has no
    /// exactly-once obligation — a superseded answer is discarded on publish.
    /// It **fans out** over every channel and unions what lands (§8.2), which
    /// is the one read that is not routed: a search names no workspace, so
    /// there is nothing to resolve and every host is asked.
    pub fn searcher(&mut self, dial: Dial) -> Option<crate::search::Searcher> {
        Some(crate::search::Searcher::new(dial, self.model.search_cell()))
    }

    /// **The window's follow lane** (REMOTE §3, §10; bl-73e7): a third seat, on
    /// a connection it holds rather than one it drops.
    ///
    /// Its own seat for the poster's reason exactly — the threads dial
    /// independently and a seat is a configuration and an address (REMOTE §6),
    /// so there is nothing to share. It takes the lane end, so there is one
    /// lane per engine: two lanes on one conversation would be two held reads
    /// of one tail, and REMOTE §10's whole argument for a held connection is
    /// that there is exactly one surface whose rate needs it.
    /// It **resolves** rather than fans out (§8.2): one conversation is focused,
    /// so the lane is dialled at whichever channel hosts that conversation's
    /// workspace and a subject that moves across the boundary is the re-ask the
    /// lane already performs whenever a subject moves.
    pub fn lane(&mut self, dial: Dial) -> Option<crate::wire::lane::Lane> {
        Some(crate::wire::lane::Lane::new(
            dial,
            self.ends.tail.take()?,
            Arc::clone(&self.repaint),
        ))
    }

    /// A seat on this engine's own listener presenting the window leaf — the
    /// one mint both off-frame threads take theirs from. The `Err` is why a
    /// box has none (bl-dc14): no listener (whose cause the boot already
    /// recorded, and outranks this derived sentence), no window leaf, or a
    /// seat the material cannot open.
    pub(crate) fn window_seat(&self, world: &Env) -> Result<crate::wire::client::Seat, String> {
        let Some(wire) = self.wire.as_ref() else {
            return Err("this engine has no listener".to_owned());
        };
        let bound = wire.address();
        let material = crate::wire::material::read(world, crate::wire::material::Role::Window)?
            .ok_or_else(|| {
                format!(
                    "no window leaf at {} — run `{}`",
                    crate::wire::material::dir(world).display(),
                    crate::wire::material::REMEDY
                )
            })?;
        crate::wire::client::Seat::open(&crate::wire::material::Material {
            address: crate::wire::loopback(&bound),
            ..material
        })
    }
}
