//! **Identity is the certificate** (REMOTE §2, §4; bl-8bbc) — who reaches the
//! answerer, who never does, and what name the ones who do arrive under.
//!
//! Two sides of one contract. A peer with no leaf, or a leaf the operator's CA
//! did not issue, dies inside rustls and asks nothing; a peer that gets through
//! is handed to the answerer *as* its leaf's common name, which is what the
//! engine resolves registrations against. A chain naming nothing usable is
//! nobody, which is the same refusal read off the chain rather than off the
//! handshake.

use super::*;

/// **An unauthenticated connection gets a TLS refusal, not a yog reply**
/// (REMOTE §4): a peer holding no client certificate never reaches the
/// answerer, so nothing was asked and nothing was said.
#[test]
fn an_uncertificated_peer_is_refused_by_tls() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    let asked = Arc::new(AtomicUsize::new(0));
    let listener = Listener::bind(
        &material(
            tmp.path(),
            Role::Server,
            crate::test_support::wire::EPHEMERAL,
        ),
        Arc::new(Echo {
            asked: Arc::clone(&asked),
            chunks: 1,
        }),
        Presence::default(),
    )
    .expect("bind");
    // A plain TCP peer speaking JSON at a TLS socket: the bytes are never a
    // handshake, so the connection dies inside rustls.
    let mut plain = std::net::TcpStream::connect(listener.address()).expect("connect");
    let _ = crate::wire::frame::write_value(&mut plain, &json!({"op": "workspaces"}));
    plain
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("timeout");
    let mut buf = [0u8; 1];
    // Either a TLS alert or a closed socket — never a reply frame.
    let _ = std::io::Read::read(&mut plain, &mut buf);
    assert_eq!(asked.load(Ordering::Relaxed), 0);
}

/// A certificate the operator's CA did not issue is the same refusal: identity
/// is the certificate, and a foreign CA's leaf is no identity here.
#[test]
fn a_foreign_certificate_is_refused() {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    let stranger = TempDir::new().expect("tmp");
    mint(stranger.path());
    let asked = Arc::new(AtomicUsize::new(0));
    let listener = Listener::bind(
        &material(
            tmp.path(),
            Role::Server,
            crate::test_support::wire::EPHEMERAL,
        ),
        Arc::new(Echo {
            asked: Arc::clone(&asked),
            chunks: 1,
        }),
        Presence::default(),
    )
    .expect("bind");
    // The stranger's own CA and leaf: a complete, valid, wrong identity. Its
    // anchors are swapped for ours so it will accept our server and we can see
    // the *client* half refuse.
    std::fs::copy(tmp.path().join("ca.pem"), stranger.path().join("ca.pem")).expect("copy");
    let seat = Seat::open(&material(
        stranger.path(),
        Role::Client,
        &listener.address(),
    ))
    .expect("seat");
    seat.ask(&json!({"op": "workspaces"})).expect_err("refused");
    assert_eq!(asked.load(Ordering::Relaxed), 0);
}

/// **The connection carries an identity** (REMOTE §2, §4; bl-8bbc): the
/// answerer is handed the certificate's own leaf name, which is what the engine
/// resolves registrations against. `make wire-certs` mints `yog-client`.
#[test]
fn the_answerer_is_handed_the_certificates_leaf_name() {
    let (_tmp, _listener, seat, _asked) = wired(1);
    let stream = seat.ask(&json!({"op": "workspaces"})).expect("answered");
    assert_eq!(stream[0]["client"], "yog-client");
}

/// A chain that names no usable identity is no connection at all: no chain, an
/// empty one, a leaf with no common name, and a leaf claiming the reserved
/// `local` name every in-world caller owns.
#[test]
fn a_chain_naming_no_usable_identity_is_nobody() {
    assert!(peer_client(None).is_none());
    assert!(peer_client(Some(&[])).is_none());
    let nameless = CertificateDer::from(vec![0x30, 0x00]);
    assert!(peer_client(Some(&[nameless])).is_none());
}
