//! **The punch port's one acceptor** (REMOTE §13.3's standing acceptor,
//! bl-5276): every stream that lands on the engine's punch port, at any time,
//! is served.
//!
//! A punch window used to be the only reader of its own listeners, so a
//! stream reaching the punch port through a still-live mapping (REMOTE §13.3:
//! never a re-punch after the engine's mapping has expired) completed the TCP
//! handshake in the kernel and was never read: a connected socket with no
//! opening frame. Now one thread accepts for the whole run and hands every
//! stream to the same `serve` a punch feeds. mTLS authenticates it there
//! (REMOTE §5, fail-closed), so a stream nobody called for is exactly as
//! safe as one the front door accepts.
//!
//! **One acceptor, with a hint** rather than two loops polling one socket.
//! A live window tells this loop which addresses it punches toward
//! ([`Toward`]); a stream from one of them is that window's, handed over so
//! the call still says what landed for it and stops its SYNs. Every other
//! stream — or one its window no longer takes — is served here, counted as
//! `accepted` beside `served`, and said once per burst: streams landing
//! within [`LINGER`] of the last are one arrival, as a punch's are.

use super::call::Answer;
use super::punch::LINGER;
use super::say;
use crate::wire::server::serve;
use std::net::{IpAddr, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, SyncSender};
use std::time::{Duration, Instant};

/// How often the listeners are looked at — a latency knob on shutdown and
/// on an arrival, never on correctness: the backlog holds a stream until
/// the next look.
const ACCEPT_POLL: Duration = Duration::from_millis(20);

/// One live window: whom it punches toward, until when, and where a stream
/// from them goes.
pub(super) struct Toward {
    pub(super) ips: Vec<IpAddr>,
    pub(super) until: Instant,
    pub(super) tx: SyncSender<TcpStream>,
}

impl Answer {
    /// Accept on the punch port until `stop`, serving what no window takes.
    pub(super) fn accept(self, hints: &Receiver<Toward>, stop: &AtomicBool) {
        let mut toward: Vec<Toward> = Vec::new();
        let mut last: Option<Instant> = None;
        while !stop.load(Ordering::Relaxed) {
            let landed = self.punch.accept();
            // Hints read AFTER the accept: a window that told us before its
            // stream was accepted is always seen by the time we hand it.
            toward.extend(hints.try_iter());
            toward.retain(|hint| Instant::now() < hint.until);
            for stream in landed {
                if let Some(stream) = hand(&toward, stream) {
                    let peer = stream.peer_addr().ok().map(|at| at.ip());
                    if last.is_none_or(|at| at.elapsed() > LINGER) {
                        (self.say)(&say::accepted(peer));
                    }
                    last = Some(Instant::now());
                    self.serve(stream, peer);
                }
            }
            std::thread::sleep(ACCEPT_POLL);
        }
    }

    /// Serve one accepted stream on a thread of its own, saying a refusal.
    fn serve(&self, stream: TcpStream, peer: Option<IpAddr>) {
        self.stats.accepted.fetch_add(1, Ordering::Relaxed);
        self.stats.served.fetch_add(1, Ordering::Relaxed);
        let answer = self.clone();
        std::thread::spawn(move || {
            let (tls, quiet) = (&answer.tls, answer.quiet);
            if !serve(
                stream,
                tls,
                answer.answerer.as_ref(),
                &answer.presence,
                quiet,
            ) {
                (answer.say)(&say::refused(peer));
            }
        });
    }
}

/// Hand `stream` to the window punching toward its peer — or back, where
/// none is or the one that was has stopped taking. The hand-off blocks until
/// the window's next look (its channel holds nothing), which is one poll.
fn hand(toward: &[Toward], stream: TcpStream) -> Option<TcpStream> {
    let ip = stream.peer_addr().ok().map(|at| at.ip());
    let mut stream = stream;
    for hint in toward
        .iter()
        .filter(|h| ip.is_some_and(|ip| h.ips.contains(&ip)))
    {
        match hint.tx.send(stream) {
            Ok(()) => return None,
            Err(back) => stream = back.0,
        }
    }
    Some(stream)
}
