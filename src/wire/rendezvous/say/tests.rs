//! The lines themselves: each says counts, seqs, nonces and families, and no
//! line built from addresses ever carries one.

use super::*;
use std::net::{Ipv4Addr, Ipv6Addr};

fn endpoints() -> Vec<SocketAddr> {
    vec![
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 7)), 7737),
        SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 7738),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 7739),
    ]
}

#[test]
fn every_line_is_the_house_shape_and_names_no_address() {
    let ips: Vec<IpAddr> = endpoints().iter().map(SocketAddr::ip).collect();
    let lines = [
        started(7737, 6881),
        published(1_700_000_000, 3, 2),
        not_published(),
        poll_failed(),
        unverified(4),
        unopened(5),
        seen(7),
        opened(7, &endpoints()),
        landed(7, &ips),
        expired(7, Duration::from_secs(20)),
        ended(7),
    ];
    for line in &lines {
        assert!(line.starts_with("yog: rendezvous: "), "{line}");
        for addr in ["203.0.113", "127.0.0.1", "::1"] {
            assert!(!line.contains(addr), "{line} names {addr}");
        }
    }
    assert_eq!(
        opened(7, &endpoints()),
        "yog: rendezvous: call nonce 7 opened — 3 endpoint(s) (1 v6, 2 v4) — punch started"
    );
    assert_eq!(
        published(9, 2, 0),
        "yog: rendezvous: presence published — seq 9, 2 endpoint(s), 0 ack(s)"
    );
}

#[test]
fn families_count_v6_first_and_say_none_for_nothing() {
    assert_eq!(families(&[]), "none");
    assert_eq!(families(&[IpAddr::V6(Ipv6Addr::LOCALHOST)]), "1 v6");
    assert_eq!(families(&[IpAddr::V4(Ipv4Addr::LOCALHOST)]), "1 v4");
}

#[test]
fn the_engine_sink_writes_a_line() {
    stderr()("yog: rendezvous: a line the suite says on stderr");
}
