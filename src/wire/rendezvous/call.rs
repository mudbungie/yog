//! **What one opened call draws** (REMOTE §13.3, bl-355c): the punch on its
//! own thread, every stream that lands served on one more, and each step said.
//! Split from `cycle` at §12's budget on the seam the thread boundary already
//! draws — the loop decides a call is new, this is everything after.

use super::item::Call;
use super::{Punch, Say, Stats, say};
use crate::registry::presence::Presence;
use crate::wire::server::{Answerer, Quiet, serve};
use rustls::ServerConfig;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

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
}

impl Answer {
    /// Punch toward `call`'s endpoints on a thread of its own, and serve
    /// every stream that lands — saying what landed, or that nothing did,
    /// and when each served stream ends. Only the peer's FAMILY is said.
    pub(super) fn spawn(self, call: Call) {
        std::thread::spawn(move || {
            let streams = self.punch.punch(call.endpoints, self.window);
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
}
