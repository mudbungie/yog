//! **The registration door** (REMOTE §15.2; bl-9ea6): what a connection that
//! presents no certificate may say, which is exactly one thing.
//!
//! The listener's client verification is optional, so a peer with no
//! certificate completes the handshake — and is then not the wire. It never
//! reaches [`frame`](super::super::frame), the preface or an
//! [`Answerer::answer`](super::Answerer::answer). It is read as one HTTP/1.1
//! request, `POST /yog/register/<code>` with a small JSON body, answered once
//! and closed. One port, one listener, no second process and no setting.
//!
//! **Hand-rolled, and deliberately tiny.** The whole grammar served is one
//! request line, one header that matters and a body of stated length; anything
//! else is the same bare `404` an unknown code earns, so the parser has no
//! refusal of its own to get wrong and a stranger learns nothing from it.
//!
//! **Bounded by resource, not by rate** (REMOTE §15.1). Forty bits of code
//! against a ten-minute life is beyond any request rate this listener serves,
//! so what is capped is what a stranger can hold: a few connections at once
//! ([`SLOTS`]), a few kilobytes ([`MAX_REQUEST`]), a few seconds
//! ([`DEADLINE`]) — one deadline for the whole exchange, so a peer that
//! dribbles a byte at a time buys no more of it.

use super::Answerer;
use crate::registry::offer::PATH;
use rustls::{ServerConnection, StreamOwned};
use serde_json::Value;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// How many certless connections one listener serves at once. Past it a
/// certless peer is closed with nothing said.
pub(crate) const SLOTS: usize = 8;
/// The most a request may be, head and body together.
pub(crate) const MAX_REQUEST: usize = 8 * 1024;
/// How long a certless peer has, from the end of its handshake, to say its one
/// request and be answered.
pub(crate) const DEADLINE: Duration = Duration::from_secs(5);

/// What the head and the body are parted by.
const BLANK_LINE: &[u8] = b"\r\n\r\n";

type Tls = StreamOwned<ServerConnection, TcpStream>;

/// One listener's count of certless connections in flight.
#[derive(Clone, Default)]
pub(crate) struct Slots(Arc<AtomicUsize>);

/// One held slot, given back when it drops.
pub(crate) struct Slot(Arc<AtomicUsize>);

impl Slots {
    /// Take a slot, or `None` when [`SLOTS`] are already held.
    pub(crate) fn enter(&self) -> Option<Slot> {
        self.0
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |held| {
                (held < SLOTS).then_some(held + 1)
            })
            .ok()?;
        Some(Slot(Arc::clone(&self.0)))
    }
}

impl Drop for Slot {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Serve one certless connection: its one request, its one answer, closed.
pub(crate) fn serve(tls: &mut Tls, answerer: &dyn Answerer, slots: &Slots, patience: Duration) {
    let Some(_slot) = slots.enter() else {
        return;
    };
    let deadline = Instant::now() + patience;
    let answer = request(tls, deadline).and_then(|(code, body)| answerer.register(&code, &body));
    let _ = tls.write_all(response(answer.as_ref()).as_bytes());
    // Half-close and read to the peer's own EOF, so nothing it wrote is unread
    // at the close and the answer is not discarded by a reset
    // ([`hang_up`](super::peer::hang_up)'s reason) — inside the same deadline
    // and the same size cap, so a peer that keeps talking buys neither.
    tls.conn.send_close_notify();
    let _ = tls.flush();
    let _ = tls.sock.shutdown(Shutdown::Write);
    let mut spent = Vec::new();
    while fill(tls, &mut spent, deadline).is_some() {}
}

/// The request's code and body, or `None` for anything that is not exactly
/// `POST /yog/register/<code> HTTP/1.1` with a `Content-Length` it then
/// honours inside [`MAX_REQUEST`] and the deadline.
fn request(tls: &mut Tls, deadline: Instant) -> Option<(String, Vec<u8>)> {
    let mut read = Vec::new();
    let body_at = loop {
        match read.windows(BLANK_LINE.len()).position(|w| w == BLANK_LINE) {
            Some(at) => break at + BLANK_LINE.len(),
            None => fill(tls, &mut read, deadline)?,
        }
    };
    let (code, length) = head(std::str::from_utf8(read.get(..body_at)?).ok()?)?;
    let end = body_at
        .checked_add(length)
        .filter(|end| *end <= MAX_REQUEST)?;
    while read.len() < end {
        fill(tls, &mut read, deadline)?;
    }
    Some((code, read.get(body_at..end)?.to_vec()))
}

/// The code and the stated body length out of a request head.
fn head(head: &str) -> Option<(String, usize)> {
    let mut lines = head.split("\r\n");
    let code = lines
        .next()?
        .strip_prefix("POST ")?
        .strip_suffix(" HTTP/1.1")?
        .strip_prefix(PATH)?;
    let length = lines
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))?
        .1
        .trim()
        .parse()
        .ok()?;
    Some((code.to_owned(), length))
}

/// Read once into `read`, inside the deadline and the size cap. `None` is the
/// peer's EOF, its silence, or more than it may say.
fn fill(tls: &mut Tls, read: &mut Vec<u8>, deadline: Instant) -> Option<()> {
    let left = deadline
        .checked_duration_since(Instant::now())
        .filter(|left| !left.is_zero())?;
    tls.sock.set_read_timeout(Some(left)).ok()?;
    let mut chunk = [0u8; 1024];
    let got = tls.read(&mut chunk).ok().filter(|got| *got > 0)?;
    read.extend_from_slice(chunk.get(..got)?);
    (read.len() <= MAX_REQUEST).then_some(())
}

/// The whole response: the claim's JSON object, or a bare `404`.
fn response(answer: Option<&Value>) -> String {
    let Some(answer) = answer else {
        return "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            .to_owned();
    };
    let body = answer.to_string();
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\
         Connection: close\r\n\r\n{body}",
        body.len()
    )
}

#[cfg(test)]
mod tests;
