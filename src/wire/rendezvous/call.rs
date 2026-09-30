//! **What one opened call draws** (REMOTE §13.3, bl-355c): the punch on its
//! own thread, every stream that lands served on one more, and each step said.
//! Split from `cycle` at §12's budget on the seam the thread boundary already
//! draws — the loop decides a call is new, this is everything after.

use super::accept::Toward;
use super::item::Call;
use super::punch::LINGER;
use super::{Punch, Say, Stats, say};
use crate::registry::presence::Presence;
use crate::wire::server::{Answerer, Quiet, serve};
use rustls::ServerConfig;
use std::net::{SocketAddr, TcpStream};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{self, Sender};
use std::time::{Duration, Instant};

/// Everything a punch and its serving need, cloned per call.
#[derive(Clone)]
pub(super) struct Answer {
    pub(super) punch: Arc<Punch>,
    pub(super) tls: Arc<ServerConfig>,
    pub(super) answerer: Arc<dyn Answerer>,
    pub(super) presence: Presence,
    pub(super) window: Duration,
    pub(super) quiet: Quiet,
    pub(super) stats: Arc<Stats>,
    pub(super) say: Say,
    /// Where a window tells the port's acceptor whom it punches toward.
    pub(super) hints: Sender<Toward>,
}

impl Answer {
    /// Punch toward `call`'s endpoints on a thread of its own, and serve
    /// every stream that lands — saying what landed, or that nothing did,
    /// and when each served stream ends. Only the peer's FAMILY is said.
    pub(super) fn spawn(self, call: Call) {
        std::thread::spawn(move || {
            let streams = self.window(call.endpoints);
            if streams.is_empty() {
                (self.say)(&say::expired(call.nonce, self.window));
                return;
            }
            self.stats.punched.fetch_add(1, Ordering::Relaxed);
            let peers: Vec<_> = streams
                .iter()
                .filter_map(|stream| stream.peer_addr().ok().as_ref().map(SocketAddr::ip))
                .collect();
            (self.say)(&say::landed(call.nonce, &peers));
            for stream in streams {
                self.stats.served.fetch_add(1, Ordering::Relaxed);
                let answer = self.clone();
                std::thread::spawn(move || {
                    serve(
                        stream,
                        &answer.tls,
                        answer.answerer.as_ref(),
                        &answer.presence,
                        answer.quiet,
                    );
                    (answer.say)(&say::ended(call.nonce));
                });
            }
        });
    }

    /// The punch toward `endpoints`, told to the acceptor first so a stream
    /// from one of their addresses is this window's. The hand-off channel
    /// holds nothing (`sync_channel(0)`), so a stream is either taken here or
    /// refused back to the acceptor when this ends — never buffered into a
    /// receiver about to drop.
    fn window(&self, endpoints: Vec<SocketAddr>) -> Vec<TcpStream> {
        let (tx, rx) = mpsc::sync_channel(0);
        let _ = self.hints.send(Toward {
            ips: endpoints.iter().map(SocketAddr::ip).collect(),
            until: Instant::now() + self.window + LINGER,
            tx,
        });
        self.punch
            .toward(endpoints, self.window, &|| rx.try_iter().collect())
    }
}
