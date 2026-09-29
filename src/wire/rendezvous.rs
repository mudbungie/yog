//! **The rendezvous loop** (REMOTE §13.2–§13.4, bl-4263): the engine's end of
//! the punched wire. It publishes presence hourly, polls the inbox every
//! fifteen seconds, verifies and unseals before it ever emits a SYN, punches
//! from one fixed port, and serves what lands exactly as the listener serves
//! what it accepts — one thread, one `Drop` that joins, the [`Listener`]
//! shape ([`server`](super::server)).
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
//! open; `cycle` is the loop itself and `call` what one opened call draws.

use super::server::{Answerer, Quiet};
use crate::dht::{Config, Udp};
use crate::registry::presence::Presence;
use crate::ui_state::Clock;
use rustls::ServerConfig;
use std::net::IpAddr;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

mod call;
mod cycle;
pub mod item;
pub mod material;
pub mod punch;
mod say;
mod stats;

pub(crate) use punch::Punch;
pub(crate) use say::{Say, stderr};
pub use stats::Standing;
pub(crate) use stats::Stats;

/// The mainline DHT's bootstrap nodes — the caller's fact, resolved on the
/// loop's own thread (a name lookup is a network act and boot is not). The
/// four standard long-lived routers, because from the deployed engine box
/// only one of the first two answered at all (REMOTE §13.7 ruling 3,
/// bl-9408): a silent router should cost a quarter of the roster, not half.
pub(crate) fn mainline() -> Vec<String> {
    vec![
        "router.bittorrent.com:6881".to_owned(),
        "dht.transmissionbt.com:6881".to_owned(),
        "router.utorrent.com:6881".to_owned(),
        "dht.aelitis.com:6881".to_owned(),
    ]
}

/// Every duration the loop keeps — stated defaults (REMOTE §13.7 ruling 3), and a
/// test's to shorten.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Cadence {
    /// How often presence is republished against the DHT's storage decay.
    pub(crate) publish: Duration,
    /// How often the inbox is read.
    pub(crate) poll: Duration,
    /// How long the loop sleeps between looks at the clock.
    pub(crate) tick: Duration,
    /// How long a punch keeps sending SYNs.
    pub(crate) window: Duration,
    /// How a punched connection's silence is read.
    pub(crate) quiet: Quiet,
}

impl Default for Cadence {
    fn default() -> Cadence {
        Cadence {
            publish: Duration::from_hours(1),
            poll: Duration::from_secs(15),
            tick: Duration::from_secs(1),
            window: Duration::from_secs(20),
            quiet: Quiet::held(),
        }
    }
}

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

/// The loop's thread. Owns its join handle and a stop flag; [`Drop`] signals
/// stop and joins, the engine's own shutdown shape (§7.2).
pub(crate) struct Rendezvous {
    stats: Arc<Stats>,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Rendezvous {
    /// Run `ctx`'s loop until dropped, marked active and its ports said. The
    /// one refusal is a seed no keypair comes from, judged here.
    pub(crate) fn spawn(ctx: Ctx) -> Result<Rendezvous, String> {
        let dht_port = ctx.transport.local_addr().map_or(0, |at| at.port());
        let (stats, say) = (Arc::clone(&ctx.stats), Arc::clone(&ctx.say));
        let cycle = cycle::Cycle::new(ctx)?;
        stats.active.store(true, Ordering::Relaxed);
        say(&say::started(cycle.punch_port(), dht_port));
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let handle = std::thread::spawn(move || cycle.run(&flag));
        Ok(Rendezvous {
            stats,
            stop,
            handle: Some(handle),
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
        if let Some(handle) = self.handle.take() {
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
