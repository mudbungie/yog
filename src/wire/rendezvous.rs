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
//! Three files under this root: [`material`] the two minted facts and their
//! derivations, [`item`] the two sealed items, [`punch`] the simultaneous
//! open; `cycle` is the loop itself.

use super::server::{Answerer, Quiet};
use crate::dht::{Config, Transport, Udp};
use crate::registry::presence::Presence;
use crate::ui_state::Clock;
use rustls::ServerConfig;
use std::net::IpAddr;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

mod cycle;
pub mod item;
pub mod material;
pub mod punch;

pub(crate) use punch::Punch;

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
    pub(crate) transport: Box<dyn Transport>,
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
}

/// What the loop has done so far — counters a test reads and nothing prints.
#[derive(Default)]
pub(crate) struct Stats {
    pub(crate) published: AtomicUsize,
    pub(crate) publish_failures: AtomicUsize,
    pub(crate) polls: AtomicUsize,
    pub(crate) poll_failures: AtomicUsize,
    /// Calls that verified, unsealed and were new — each one a punch.
    pub(crate) calls: AtomicUsize,
    /// Streams a punch landed and handed to the serving code.
    pub(crate) served: AtomicUsize,
}

/// The loop's thread. Owns its join handle and a stop flag; [`Drop`] signals
/// stop and joins, the engine's own shutdown shape (§7.2).
pub(crate) struct Rendezvous {
    #[cfg(test)]
    stats: Arc<Stats>,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Rendezvous {
    /// Run `ctx`'s loop until dropped. The one refusal is a seed no keypair
    /// comes from, judged here rather than on the thread.
    pub(crate) fn spawn(ctx: Ctx) -> Result<Rendezvous, String> {
        let stats = Arc::new(Stats::default());
        let cycle = cycle::Cycle::new(ctx, Arc::clone(&stats))?;
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let handle = std::thread::spawn(move || cycle.run(&flag));
        Ok(Rendezvous {
            #[cfg(test)]
            stats,
            stop,
            handle: Some(handle),
        })
    }

    /// The counters, shared with the thread — what the suite watches, since
    /// the loop prints nothing.
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
    }
}

/// The engine's composition: the loop over `dir`'s material at the default
/// cadence, or `None` where `dir` holds no rendezvous material (REMOTE §13.4).
pub(crate) fn start(
    dir: &Path,
    answerer: Arc<dyn Answerer>,
    presence: Presence,
    clock: Arc<dyn Clock>,
    bootstrap: Vec<String>,
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
        transport: Box::new(udp),
        bootstrap,
        config: Config::default(),
        punch: Punch::bind(0)?,
        advertise: punch::local_ips(),
        tls,
        answerer,
        presence,
        clock,
        cadence: Cadence::default(),
    })
    .map(Some)
}

#[cfg(test)]
mod tests;
