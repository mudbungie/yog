//! **The rendezvous loop** (REMOTE §13.2–§13.4, bl-4263): the engine's end of
//! the punched wire. It publishes presence hourly, polls the inbox every
//! fifteen seconds, verifies and unseals before it ever emits a SYN, punches
//! from one fixed port, and serves every stream that lands on that port — in
//! a call's window or at any other time (bl-5276) — exactly as the listener
//! serves what it accepts: two threads, one `Drop` that joins both, the
//! [`Listener`] shape ([`server`](super::server)).
//!
//! **Severable by material** (REMOTE §13.4). An engine whose wire root holds no
//! [`material`] punches nothing, polls nothing and starts no thread: [`start`]
//! answers `None` and the engine is today's listener byte for byte. The mint
//! grows the material only for a box whose `address` is not loopback, so a
//! box only its own seat dials never chatters on the commons at all.
//!
//! **Every duration is a [`Cadence`] field and time is the injected
//! [`Clock`]**, so the suite drives an hour of republishing and a run of
//! polls by advancing a fake clock, against the fake DHT the `dht` corpus
//! already stands up on loopback (REMOTE §13.5). The tick the loop sleeps between
//! looks is real, and short. Nothing here is on a request path: a DHT walk
//! blocks for up to a round per hop, which is why the loop has a thread.
//!
//! **It says what it does** (bl-355c): one stderr line per event, built in
//! `say`, and counters (`stats`) `/doctor` hands a seat as [`Standing`].
//!
//! Three files under this root: [`material`] the two minted facts and their
//! derivations, [`item`] the two sealed items, [`punch`] the simultaneous
//! open; `cycle` is the loop itself, `call` what one opened call draws,
//! `accept` the port's one acceptor, and `cadence` every duration.

use super::server::Answerer;
use crate::dht::{Config, Udp};
use crate::registry::presence::Presence;
use crate::ui_state::Clock;
use rustls::ServerConfig;
use std::net::IpAddr;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;

mod accept;
mod cadence;
mod call;
mod cycle;
pub mod item;
pub mod material;
pub mod punch;
mod say;
mod stats;

pub(crate) use cadence::{Cadence, mainline};
pub(crate) use punch::Punch;
pub(crate) use say::{Say, stderr};
pub use stats::Standing;
pub(crate) use stats::Stats;

/// What the loop is made of, handed over whole to its thread.
pub(crate) struct Ctx {
    pub(crate) pairing: material::Pairing,
    /// The DHT socket — concrete, since its port is said at start.
    pub(crate) transport: Udp,
    pub(crate) bootstrap: Vec<String>,
    pub(crate) config: Config,
    pub(crate) punch: Punch,
    /// The addresses presence names, each at the punch port.
    pub(crate) advertise: Vec<IpAddr>,
    pub(crate) tls: Arc<ServerConfig>,
    pub(crate) answerer: Arc<dyn Answerer>,
    pub(crate) presence: Presence,
    pub(crate) clock: Arc<dyn Clock>,
    pub(crate) cadence: Cadence,
    /// The counters, shared with [`Listening`](crate::wire::Listening).
    pub(crate) stats: Arc<Stats>,
    /// Where the loop's lines go.
    pub(crate) say: Say,
}

/// The loop's two threads — the cycle and the punch port's acceptor
/// (`accept`, bl-5276). Owns their join handles and one stop flag; [`Drop`]
/// signals stop and joins both, the engine's own shutdown shape (§7.2).
pub(crate) struct Rendezvous {
    stats: Arc<Stats>,
    stop: Arc<AtomicBool>,
    handles: Vec<JoinHandle<()>>,
}

impl Rendezvous {
    /// Run `ctx`'s loop and its acceptor until dropped, marked active and
    /// its ports said. The one refusal is a seed no keypair comes from,
    /// judged here.
    pub(crate) fn spawn(ctx: Ctx) -> Result<Rendezvous, String> {
        let dht_port = ctx.transport.local_addr().map_or(0, |at| at.port());
        let (stats, say) = (Arc::clone(&ctx.stats), Arc::clone(&ctx.say));
        let (hints, toward) = std::sync::mpsc::channel();
        let cycle = cycle::Cycle::new(ctx, hints)?;
        let answer = cycle.answer.clone();
        stats.active.store(true, Ordering::Relaxed);
        say(&say::started(cycle.punch_port(), dht_port));
        let stop = Arc::new(AtomicBool::new(false));
        let (flag, also) = (Arc::clone(&stop), Arc::clone(&stop));
        let handles = vec![
            std::thread::spawn(move || cycle.run(&flag)),
            std::thread::spawn(move || answer.accept(&toward, &also)),
        ];
        Ok(Rendezvous {
            stats,
            stop,
            handles,
        })
    }

    /// The counters, shared with the thread — what the suite watches.
    #[cfg(test)]
    pub(crate) fn stats(&self) -> Arc<Stats> {
        Arc::clone(&self.stats)
    }
}

impl Drop for Rendezvous {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        for handle in self.handles.drain(..) {
            let _ = handle.join();
        }
        self.stats.active.store(false, Ordering::Relaxed);
    }
}

/// The engine's composition: the loop over `dir`'s material at the default
/// cadence, counting into `stats` and saying each event on stderr — or `None`
/// where `dir` holds no rendezvous material (REMOTE §13.4).
pub(crate) fn start(
    dir: &Path,
    answerer: Arc<dyn Answerer>,
    presence: Presence,
    clock: Arc<dyn Clock>,
    bootstrap: Vec<String>,
    stats: Arc<Stats>,
) -> Result<Option<Rendezvous>, String> {
    let Some(pairing) = material::read_dir(dir)? else {
        return Ok(None);
    };
    let Some(served) = super::material::read_dir(dir, super::material::Role::Server)? else {
        return Ok(None);
    };
    let tls = super::tls::server_config(&served)?;
    let udp = Udp::bind("0.0.0.0:0".parse().map_err(|e| format!("{e}"))?)
        .map_err(|e| format!("rendezvous: {e}"))?;
    Rendezvous::spawn(Ctx {
        pairing,
        transport: udp,
        bootstrap,
        config: Config::default(),
        punch: Punch::bind(0)?,
        advertise: punch::local_ips(),
        tls,
        answerer,
        presence,
        clock,
        cadence: Cadence::default(),
        stats,
        say: stderr(),
    })
    .map(Some)
}

#[cfg(test)]
mod tests;
