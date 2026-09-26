//! One round of the walk, and of `put`: queries out, then one bounded wait
//! for whatever answers. `lookup` and `items` both stand on these two halves.

use super::Dht;
use super::bencode::Dict;
use super::krpc::{self, Message};
use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::time::Instant;

/// The transactions one round is waiting on, by transaction id.
pub(crate) type Pending = BTreeMap<Vec<u8>, SocketAddr>;

impl Dht {
    /// Send one query and, if the send itself went, remember the transaction.
    pub(crate) fn ask(&mut self, pending: &mut Pending, addr: SocketAddr, q: &str, args: Dict) {
        let tid = self.next_tid();
        let datagram = krpc::query(&tid, &self.id, q, args);
        if self.transport.send(addr, &datagram).is_ok() {
            pending.insert(tid, addr);
        }
    }

    /// One round: read datagrams until every pending transaction has answered
    /// or the round's deadline passes. Only an answer to a transaction this
    /// round sent reaches `on`; everything else on the socket is noise.
    pub(crate) fn collect(
        &mut self,
        pending: &mut Pending,
        on: &mut dyn FnMut(SocketAddr, Message),
    ) -> Result<(), String> {
        let deadline = Instant::now() + self.config.round;
        while !pending.is_empty() {
            let wait = deadline.saturating_duration_since(Instant::now());
            if wait.is_zero() {
                break;
            }
            let Some((_, bytes)) = self
                .transport
                .recv(wait)
                .map_err(|e| format!("DHT socket: {e}"))?
            else {
                break;
            };
            let Some(message) = krpc::parse(&bytes) else {
                continue;
            };
            let (Message::Reply { tid, .. } | Message::Error { tid, .. }) = &message;
            if let Some(addr) = pending.remove(tid) {
                on(addr, message);
            }
        }
        Ok(())
    }
}
