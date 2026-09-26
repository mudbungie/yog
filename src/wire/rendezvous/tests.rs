//! The loop end to end against the fake DHT on loopback (REMOTE §13.5):
//! presence published and republished, a call punched and served with pings,
//! and every way the commons can disappoint it.

use super::item::Presence as Published;
use super::*;
use crate::dht::tests::fake::Mood;
use bench::{bench, loopback, until};
use std::sync::atomic::Ordering::Relaxed;
use std::time::Duration;

mod bench;

const WAIT: Duration = Duration::from_secs(10);

#[test]
fn presence_is_published_sealed_and_republished_hourly() {
    let b = bench(Mood::Answer, None);
    let stats = b.engine.stats();
    assert!(until(|| stats.published.load(Relaxed) >= 1, WAIT));
    let keypair = b.pairing.keypair().expect("key");
    let mut dht = b.dht();
    let item = dht
        .get(keypair.public(), b.pairing.presence_salt())
        .expect("walk")
        .expect("published");
    assert_eq!(
        Published::open(&b.pairing.seal_key(), &item.value),
        Some(Published {
            endpoints: vec![loopback(b.port)]
        })
    );
    b.clock.advance(Duration::from_hours(1));
    assert!(until(|| stats.published.load(Relaxed) >= 2, WAIT));
    let again = dht
        .get(keypair.public(), b.pairing.presence_salt())
        .expect("walk")
        .expect("republished");
    assert!(again.seq > item.seq, "{} > {}", again.seq, item.seq);
}

#[test]
fn a_call_in_the_inbox_is_punched_served_and_pinged() {
    let b = bench(Mood::Answer, None);
    let stats = b.engine.stats();
    assert!(until(|| stats.polls.load(Relaxed) >= 1, WAIT));
    assert_eq!(stats.calls.load(Relaxed), 0, "an empty inbox draws nothing");
    let client = Punch::bind(0).expect("client port");
    assert!(b.write_inbox(1, b.call(7, client.port())) >= 1);
    b.clock.advance(Duration::from_secs(15));
    let mut streams = client.punch(vec![loopback(b.port)], Duration::from_secs(5));
    let stream = streams.pop().expect("punched");
    let mut tls = b.ask_over(stream);
    assert_eq!(
        crate::wire::frame::read_value(&mut tls).expect("held"),
        Some(crate::wire::server::peer::ping_frame()),
        "the held connection is pinged through its silence"
    );
    assert!(until(|| stats.served.load(Relaxed) >= 1, WAIT));
    assert_eq!(stats.calls.load(Relaxed), 1);
    b.clock.advance(Duration::from_secs(15));
    assert!(until(|| stats.polls.load(Relaxed) >= 3, WAIT));
    assert_eq!(
        stats.calls.load(Relaxed),
        1,
        "the same nonce is not punched twice"
    );
}

#[test]
fn a_call_that_will_not_unseal_draws_no_syn() {
    let b = bench(Mood::Answer, None);
    let stats = b.engine.stats();
    assert!(until(|| stats.polls.load(Relaxed) >= 1, WAIT));
    assert!(b.write_inbox(1, b"signed, but sealed under nobody's key".to_vec()) >= 1);
    b.clock.advance(Duration::from_secs(15));
    assert!(until(|| stats.polls.load(Relaxed) >= 2, WAIT));
    assert_eq!(stats.calls.load(Relaxed), 0);
    assert_eq!(
        stats.poll_failures.load(Relaxed),
        0,
        "and it is not an error either"
    );
}

#[test]
fn a_dark_commons_is_counted_and_tried_again() {
    let b = bench(Mood::Silent, None);
    let stats = b.engine.stats();
    assert!(until(
        || stats.publish_failures.load(Relaxed) >= 1 && stats.poll_failures.load(Relaxed) >= 1,
        WAIT
    ));
    b.clock.advance(Duration::from_hours(1));
    assert!(until(|| stats.publish_failures.load(Relaxed) >= 2, WAIT));
    assert_eq!(stats.published.load(Relaxed), 0);
}

#[test]
fn a_bootstrap_that_resolves_to_nothing_fails_each_deadline_and_never_walks() {
    let b = bench(Mood::Answer, Some(vec!["nowhere".to_owned()]));
    let stats = b.engine.stats();
    assert!(until(
        || stats.publish_failures.load(Relaxed) >= 1 && stats.poll_failures.load(Relaxed) >= 1,
        WAIT
    ));
    assert_eq!(stats.published.load(Relaxed) + stats.polls.load(Relaxed), 0);
}

#[test]
fn start_composes_only_over_rendezvous_material() {
    let tmp = tempfile::TempDir::new().expect("tmp");
    let node = crate::dht::tests::fake::FakeNode::bind(crate::dht::NodeId([2u8; 20]));
    let compose = || {
        start(
            tmp.path(),
            Arc::new(bench::Echo) as Arc<dyn Answerer>,
            Presence::default(),
            crate::test_support::clock::FakeClock::new().arc(),
            vec![node.addr.to_string()],
        )
    };
    assert!(compose().expect("nothing minted").is_none());
    material::mint(tmp.path()).expect("rendezvous mint");
    assert!(compose().expect("no wire material yet").is_none());
    crate::test_support::wire::mint(tmp.path());
    assert!(compose().expect("composed").is_some());
    std::fs::remove_file(tmp.path().join(material::SALT)).expect("rm");
    assert!(compose().is_err(), "half the material refuses");
}

#[test]
fn the_default_cadence_is_the_stated_one() {
    let cadence = Cadence::default();
    assert_eq!(cadence.poll, Duration::from_secs(15));
    assert_eq!(cadence.publish, Duration::from_hours(1));
    assert_eq!(cadence.quiet, Quiet::held());
    assert_eq!(mainline().len(), 2);
}
