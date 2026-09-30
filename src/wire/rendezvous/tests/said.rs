//! **What the loop says** (bl-355c), arm by arm, through the bench's injected
//! sink — and the counters `/doctor` hands a seat. Each test reads the lines
//! an operator would read off the engine's stderr.

use super::super::item::Presence as Published;
use super::bench::{bench, loopback, until};
use super::{Punch, WAIT};
use crate::dht::tests::fake::Mood;
use std::sync::atomic::Ordering::Relaxed;
use std::time::Duration;

const P: &str = "yog: rendezvous:";

#[test]
fn the_loop_says_it_started_and_what_it_published() {
    let b = bench(Mood::Answer, None);
    assert!(until(
        || b.said(&format!("{P} presence published")) >= 1,
        WAIT
    ));
    let heard = b.heard();
    let first = heard.first().expect("a first line");
    assert!(
        first.starts_with(&format!(
            "{P} started — punch port {}, DHT socket port ",
            b.port
        )),
        "{first}"
    );
    let published = heard
        .iter()
        .find(|l| l.contains("published"))
        .expect("said");
    assert!(published.contains("1 endpoint(s), 1 ack(s)"), "{published}");
    assert!(b.engine.stats().standing().active);
}

#[test]
fn a_dark_commons_says_each_publish_and_one_poll_failure_per_streak() {
    let b = bench(Mood::Silent, None);
    let stats = b.engine.stats();
    assert!(until(|| stats.publish_failures.load(Relaxed) >= 1, WAIT));
    b.clock.advance(Duration::from_hours(1));
    assert!(until(|| stats.publish_failures.load(Relaxed) >= 2, WAIT));
    assert!(stats.poll_failures.load(Relaxed) >= 2);
    assert_eq!(b.said(&format!("{P} presence not published")), 2);
    assert_eq!(
        b.said(&format!("{P} inbox poll failed")),
        1,
        "{:?}",
        b.heard()
    );
}

#[test]
fn an_item_that_will_not_verify_is_said_once_by_its_seq() {
    let b = bench(Mood::Answer, None);
    let stats = b.engine.stats();
    assert!(until(|| stats.polls.load(Relaxed) >= 1, WAIT));
    assert!(b.write_inbox(4, b"sealed under nobody's key".to_vec()) >= 1);
    b.clock.advance(Duration::from_secs(15));
    assert!(until(|| stats.polls.load(Relaxed) >= 2, WAIT));
    b.clock.advance(Duration::from_secs(15));
    assert!(until(|| stats.polls.load(Relaxed) >= 3, WAIT));
    let line =
        format!("{P} inbox item seq 4 did not verify under the pairing's seal key — no punch");
    assert_eq!(b.said(&line), 1, "{:?}", b.heard());
    assert_eq!(stats.standing().last_poll_unix, 30);
}

#[test]
fn an_item_that_verifies_and_is_not_a_call_is_said_as_such() {
    let b = bench(Mood::Answer, None);
    let stats = b.engine.stats();
    assert!(until(|| stats.polls.load(Relaxed) >= 1, WAIT));
    let sealed = Published {
        endpoints: vec![loopback(1)],
    }
    .seal(&b.pairing.seal_key())
    .expect("seal");
    assert!(b.write_inbox(5, sealed) >= 1);
    b.clock.advance(Duration::from_secs(15));
    let line = format!("{P} inbox item seq 5 verified but is not a call — no punch");
    assert!(until(|| b.said(&line) == 1, WAIT), "{:?}", b.heard());
}

#[test]
fn a_call_says_opened_landed_ended_and_then_seen_once() {
    let b = bench(Mood::Answer, None);
    let stats = b.engine.stats();
    assert!(until(|| stats.polls.load(Relaxed) >= 1, WAIT));
    let client = Punch::bind(0).expect("client port");
    assert!(b.write_inbox(1, b.call(7, client.port())) >= 1);
    b.clock.advance(Duration::from_secs(15));
    let mut streams = client.punch(vec![], Duration::from_secs(5));
    drop(b.ask_over(streams.pop().expect("punched")));
    drop(streams);
    let opened = format!("{P} call nonce 7 opened — 1 endpoint(s) (1 v4) — punch started");
    let landed = format!("{P} punch for nonce 7 landed ");
    let ended = format!("{P} served stream for nonce 7 ended");
    assert!(until(|| b.said(&ended) >= 1, WAIT), "{:?}", b.heard());
    let heard = b.heard();
    let at = |head: &str| heard.iter().position(|l| l.starts_with(head));
    assert!(at(&opened) < at(&landed), "opened is said first: {heard:?}");
    let line = heard
        .iter()
        .find(|l| l.starts_with(&landed))
        .expect("landed");
    assert!(line.ends_with("v4)"), "{line}");
    for _ in 0..2 {
        b.clock.advance(Duration::from_secs(15));
    }
    assert!(until(|| stats.polls.load(Relaxed) >= 3, WAIT));
    assert_eq!(
        b.said(&format!("{P} call nonce 7 already punched — no punch")),
        1
    );
    let standing = stats.standing();
    assert_eq!((standing.calls, standing.punched), (1, 1));
    assert!(standing.served >= 1);
}

#[test]
fn a_punched_call_read_between_quiet_polls_is_said_seen_once() {
    // The commons shows the stale call, then nothing, then the call again
    // (bl-1633): a quiet poll between them must not make it news twice.
    let b = bench(Mood::Blink, None);
    let stats = b.engine.stats();
    assert!(until(|| stats.polls.load(Relaxed) >= 1, WAIT));
    let closed = Punch::bind(0).expect("a port").port();
    assert!(b.write_inbox(1, b.call(9, closed)) >= 1);
    for polls in 2..=6 {
        b.clock.advance(Duration::from_secs(15));
        assert!(until(|| stats.polls.load(Relaxed) >= polls, WAIT));
    }
    let heard = b.heard();
    let opened = format!("{P} call nonce 9 opened");
    assert_eq!(b.said(&opened), 1, "{heard:?}");
    let seen = format!("{P} call nonce 9 already punched — no punch");
    assert_eq!(b.said(&seen), 1, "{heard:?}");
    assert_eq!(stats.standing().calls, 1);
}

#[test]
fn a_punch_toward_nobody_says_it_expired() {
    let b = bench(Mood::Answer, None);
    let stats = b.engine.stats();
    assert!(until(|| stats.polls.load(Relaxed) >= 1, WAIT));
    let closed = Punch::bind(0).expect("a port").port();
    assert!(b.write_inbox(1, b.call(8, closed)) >= 1);
    b.clock.advance(Duration::from_secs(15));
    let expired = format!("{P} punch for nonce 8 expired after 3s with no stream");
    assert!(until(|| b.said(&expired) == 1, WAIT), "{:?}", b.heard());
    assert_eq!(stats.standing().punched, 0);
}

#[test]
fn a_dropped_loop_is_no_longer_active() {
    let b = bench(Mood::Answer, None);
    let stats = b.engine.stats();
    assert!(stats.standing().active);
    drop(b.engine);
    assert!(!stats.standing().active);
}
