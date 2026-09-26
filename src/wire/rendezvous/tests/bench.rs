//! The bench every loop test stands on: a fake DHT node on loopback, minted
//! material, a fake clock, and the engine's loop over them — plus the
//! client's half of the contract, which the suite is the mirror of.

use super::super::item::Call;
use super::super::material::Pairing;
use super::super::{Cadence, Ctx, Punch, Rendezvous};
use crate::dht::tests::fake::{FakeNode, Mood};
use crate::dht::{Config, Dht, NodeId, Udp};
use crate::registry::Peer;
use crate::registry::presence::Presence;
use crate::test_support::clock::FakeClock;
use crate::test_support::wire::{EPHEMERAL, material, mint};
use crate::wire::material::Role;
use crate::wire::server::{Answerer, Quiet};
use crate::wire::{frame, hello, tls};
use rustls::pki_types::ServerName;
use rustls::{ClientConnection, StreamOwned};
use serde_json::{Value, json};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;

/// Echoes the request inside an envelope.
pub(super) struct Echo;

impl Answerer for Echo {
    fn answer(&self, _peer: &Peer, request: Value) -> Box<dyn Iterator<Item = Value>> {
        Box::new(std::iter::once(json!({"echo": request})))
    }
}

pub(super) struct Bench {
    pub(super) tmp: TempDir,
    /// The node that holds the items — held so its thread outlives the loop.
    _node: FakeNode,
    pub(super) router: FakeNode,
    pub(super) pairing: Pairing,
    pub(super) clock: FakeClock,
    pub(super) port: u16,
    pub(super) engine: Rendezvous,
}

/// A walk that waits milliseconds.
pub(super) fn quick() -> Config {
    Config {
        alpha: 3,
        k: 3,
        round: Duration::from_millis(300),
        max_queries: 64,
    }
}

/// The production cadence with every wait a test can afford.
pub(super) fn cadence() -> Cadence {
    Cadence {
        publish: Duration::from_hours(1),
        poll: Duration::from_secs(15),
        tick: Duration::from_millis(10),
        window: Duration::from_secs(3),
        quiet: Quiet {
            gone: Duration::from_secs(2),
            ping: Some(Duration::from_millis(150)),
        },
    }
}

pub(super) fn loopback(port: u16) -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port)
}

/// Mint everything and stand the loop up over a node of `mood` behind a
/// bootstrap router that answers only `find_node`, as the mainline's do
/// (REMOTE §13.7 ruling 3); `bootstrap` is `None` for the router's address.
/// A silent commons is silent at the router too.
pub(super) fn bench(mood: Mood, bootstrap: Option<Vec<String>>) -> Bench {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    super::super::material::mint(tmp.path()).expect("rendezvous mint");
    let pairing = super::super::material::read_dir(tmp.path())
        .expect("read")
        .expect("minted");
    let mut node = FakeNode::bind(NodeId([1u8; 20]));
    node.serve(vec![], mood, vec![]);
    let mut router = FakeNode::bind(NodeId([0u8; 20]));
    let door = match mood {
        Mood::Silent => Mood::Silent,
        _ => Mood::Router,
    };
    router.serve(vec![node.node()], door, vec![]);
    let clock = FakeClock::new();
    let punch = Punch::bind(0).expect("punch port");
    let port = punch.port();
    let engine = Rendezvous::spawn(Ctx {
        pairing: pairing.clone(),
        transport: Box::new(Udp::bind(loopback(0)).expect("udp")),
        bootstrap: bootstrap.unwrap_or_else(|| vec![router.addr.to_string()]),
        config: quick(),
        punch,
        advertise: vec![IpAddr::V4(Ipv4Addr::LOCALHOST)],
        tls: tls::server_config(&material(tmp.path(), Role::Server, EPHEMERAL)).expect("tls"),
        answerer: Arc::new(Echo),
        presence: Presence::default(),
        clock: clock.arc(),
        cadence: cadence(),
    })
    .expect("spawn");
    Bench {
        tmp,
        _node: node,
        router,
        pairing,
        clock,
        port,
        engine,
    }
}

impl Bench {
    /// A client of the same fake DHT.
    pub(super) fn dht(&self) -> Dht {
        let udp = Udp::bind(loopback(0)).expect("udp");
        Dht::new(Box::new(udp), vec![self.router.addr], quick()).expect("client")
    }

    /// Write one inbox item under the derived key: `value` is what a client
    /// would have sealed, or whatever a hostile one wrote.
    pub(super) fn write_inbox(&self, seq: i64, value: Vec<u8>) -> usize {
        let item = self
            .pairing
            .inbox_keypair()
            .expect("inbox key")
            .sign(self.pairing.inbox_salt(), seq, value)
            .expect("sign");
        self.dht().put(item).expect("stored")
    }

    /// A sealed call at `port` on loopback.
    pub(super) fn call(&self, nonce: u64, port: u16) -> Vec<u8> {
        Call {
            nonce,
            endpoints: vec![loopback(port)],
        }
        .seal(&self.pairing.seal_key())
        .expect("seal")
    }

    /// The seat's mTLS over a punched stream: preface, one ask, its answer.
    pub(super) fn ask_over(&self, tcp: TcpStream) -> StreamOwned<ClientConnection, TcpStream> {
        tcp.set_read_timeout(Some(Duration::from_secs(3)))
            .expect("timeout");
        let config = tls::client_config(&material(self.tmp.path(), Role::Client, "127.0.0.1:1"))
            .expect("client tls");
        let conn = ClientConnection::new(config, ServerName::IpAddress(Ipv4Addr::LOCALHOST.into()))
            .expect("conn");
        let mut tls = StreamOwned::new(conn, tcp);
        hello::state(&mut tls).expect("preface");
        frame::write_value(&mut tls, &json!({"hi": 1})).expect("ask");
        hello::confirm(&mut tls).expect("admitted");
        assert_eq!(
            frame::read_value(&mut tls).expect("answer"),
            Some(json!({"echo": {"hi": 1}}))
        );
        assert_eq!(frame::read_value(&mut tls).expect("end"), None);
        tls
    }
}

/// Wait for `ready`, bounded.
pub(super) fn until(ready: impl Fn() -> bool, wait: Duration) -> bool {
    let deadline = Instant::now() + wait;
    while Instant::now() < deadline {
        if ready() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    ready()
}
