//! Both items round-trip through the seal, and everything that is not one
//! of them opens to nothing.

use super::*;

const KEY: [u8; 32] = [9u8; 32];

fn endpoints() -> Vec<SocketAddr> {
    vec![
        "127.0.0.1:7737".parse().expect("v4"),
        "[::1]:7738".parse().expect("v6"),
    ]
}

#[test]
fn presence_round_trips_and_is_opaque() {
    let presence = Presence {
        endpoints: endpoints(),
    };
    let sealed = presence.seal(&KEY).expect("seal");
    assert_eq!(Presence::open(&KEY, &sealed), Some(presence));
    assert!(
        sealed.len() < 100,
        "well under BEP 44's cap: {}",
        sealed.len()
    );
    assert!(
        !sealed.windows(4).any(|w| w == [127, 0, 0, 1]),
        "the address is not readable on the commons"
    );
    assert_eq!(
        Presence::open(&[8u8; 32], &sealed),
        None,
        "another key opens nothing"
    );
}

#[test]
fn a_call_round_trips_with_its_nonce() {
    let call = Call {
        nonce: 0x0102_0304_0506_0708,
        endpoints: endpoints(),
    };
    let sealed = call.seal(&KEY).expect("seal");
    assert_eq!(Call::open(&KEY, &sealed), Some(call));
}

#[test]
fn an_empty_list_is_an_item_too() {
    let presence = Presence {
        endpoints: Vec::new(),
    };
    let sealed = presence.seal(&KEY).expect("seal");
    assert_eq!(Presence::open(&KEY, &sealed), Some(presence));
}

#[test]
fn a_sealed_item_of_one_kind_is_not_the_other() {
    let presence = Presence {
        endpoints: endpoints(),
    };
    let sealed = presence.seal(&KEY).expect("seal");
    assert_eq!(Call::open(&KEY, &sealed), None, "no nonce to read");
    let call = Call {
        nonce: 1,
        endpoints: endpoints(),
    };
    let sealed = call.seal(&KEY).expect("seal");
    assert_eq!(
        Presence::open(&KEY, &sealed),
        None,
        "a count byte of 0 then bytes left over"
    );
}

#[test]
fn what_will_not_decode_is_nothing() {
    assert_eq!(Presence::open(&KEY, b"short"), None, "no nonce");
    assert_eq!(Presence::open(&KEY, &[0u8; 40]), None, "no tag");
    for plain in [
        vec![],
        vec![1],
        vec![1, 5, 0, 0],
        vec![1, 4, 1, 2, 3],
        vec![1, 4, 1, 2, 3, 4, 0],
        vec![1, 6, 0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0],
    ] {
        let sealed = seal(&KEY, &plain).expect("seal");
        assert_eq!(Presence::open(&KEY, &sealed), None, "{plain:?}");
    }
    let sealed = seal(&KEY, &[0, 0, 0, 0, 0, 0, 0, 1, 0]).expect("seal");
    assert_eq!(
        Call::open(&KEY, &sealed),
        Some(Call {
            nonce: 1,
            endpoints: Vec::new()
        })
    );
}

#[test]
fn the_list_is_bounded_at_a_byte() {
    let many: Vec<SocketAddr> = (0..300u16)
        .map(|port| SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port))
        .collect();
    let (decoded, rest) = decode(&encode(&many)).expect("decodes");
    assert_eq!(decoded.len(), 255);
    assert!(rest.is_empty());
}
