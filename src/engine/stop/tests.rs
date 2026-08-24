//! The loop, on a flag this test owns.
//!
//! **The process-wide flag is not touched here.** It has exactly one owner in
//! the whole suite — `engine::serve::tests`, which drives the face that reads
//! it — because a static two tests may raise is a static they can race, and
//! `cargo test` runs them in parallel. One owner is cheaper than a lock and is
//! the same discipline the linked lernie applies to its own §2.9 flag.

use super::*;
use std::sync::Arc;
use std::time::Instant;

/// The waiting arm: [`park_until`] does not return while its flag is down, and
/// returns once it is raised from elsewhere.
#[test]
fn park_until_waits_for_its_flag_and_then_returns() {
    let flag = Arc::new(AtomicBool::new(false));
    let raiser = Arc::clone(&flag);
    let hand = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(20));
        raiser.store(true, Ordering::SeqCst);
    });
    let started = Instant::now();
    park_until(&flag);
    assert!(
        started.elapsed() >= Duration::from_millis(20),
        "it returned before anything raised the flag"
    );
    hand.join().unwrap();
}
