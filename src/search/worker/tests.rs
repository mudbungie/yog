//! The searcher as a **wire client** (REMOTE §9.7, bl-44e9): it asks the engine
//! and publishes what came back, three arms and no fourth — the answer, the
//! refusal carried as an unreadable source, and the reply of another kind that
//! is a codec defect rather than a state.
//!
//! Driven against a real listener over loopback mTLS, like the asker's own
//! tests: a fake answerer at the far end, so what is under test is this thread's
//! decode and its publish protocol rather than the boundary's derivation.

use super::{SearchCell, Searcher};
use crate::registry::presence::Presence;
use crate::search::{Found, Hit};
use crate::test_support::wire::{EPHEMERAL, material, mint};
use crate::wire::material::Role;
use crate::wire::server::{Answerer, Listener};
use serde_json::{Value, json};
use std::sync::Arc;
use tempfile::TempDir;

/// An engine that answers with a fixed stream — the asker's own fixture.
struct Says(Vec<Value>);

impl Answerer for Says {
    fn answer(
        &self,
        _peer: &crate::registry::Peer,
        _request: Value,
    ) -> Box<dyn Iterator<Item = Value>> {
        Box::new(self.0.clone().into_iter())
    }
}

/// A bound listener answering `says`, and a searcher seated on it.
fn wired(tmp: &TempDir, says: Vec<Value>) -> (Listener, Searcher, SearchCell) {
    mint(tmp.path());
    let listener = Listener::bind(
        &material(tmp.path(), Role::Server, EPHEMERAL),
        Arc::new(Says(says)),
        Presence::default(),
    )
    .expect("bind");
    let seat = crate::wire::client::Seat::open(&material(
        tmp.path(),
        Role::Window,
        &crate::wire::loopback(&listener.address()),
    ))
    .expect("seat");
    let asks = SearchCell::default();
    (
        listener,
        Searcher::new(crate::wire::dial::Dial::of(seat), asks.clone()),
        asks,
    )
}

/// One `search` reply body carrying a single ball hit.
fn answers(needle: &str) -> Vec<Value> {
    vec![json!({
        "ok": true, "kind": "search", "needle": needle,
        "rows": [{"at": "ball", "project": "/dev/yog", "id": "bl-1",
                  "field": "name", "offset": 0, "excerpt": needle}],
        "unreadable": [],
    })]
}

/// The whole leg: nothing asked answers nothing, an ask crosses the wire, and
/// what the engine said is what the frame reads.
#[test]
fn the_searcher_asks_the_engine_and_publishes_what_came_back() {
    let tmp = TempDir::new().expect("tmp");
    let (_listener, searcher, asks) = wired(&tmp, answers("kraken"));
    assert!(!searcher.pass(), "nothing asked, nothing answered");

    asks.ask("kraken");
    assert!(searcher.pass(), "the pass answered the ask");
    let found = asks.found();
    assert_eq!(found.needle, "kraken");
    assert!(
        matches!(found.hits.as_slice(), [Hit { .. }]),
        "the engine's own row reached the cell: {found:?}"
    );
    assert!(!asks.searching());
}

/// **A refusal is an unreadable source.** The engine can say no, and
/// `Found::unreadable` is already "each unreadable source, named with why" — so
/// the reason is painted where a mangled transcript's would be, and a refused
/// search never reads as *no matches*.
#[test]
fn a_refusal_lands_as_the_reason_and_never_as_an_empty_answer() {
    let tmp = TempDir::new().expect("tmp");
    let (_listener, searcher, asks) = wired(
        &tmp,
        vec![json!({"ok": false, "error": "no such workspace: gone"})],
    );
    asks.ask("kraken");
    assert!(searcher.pass());
    let found = asks.found();
    assert_eq!(
        found.needle, "kraken",
        "the answer still knows its question"
    );
    assert!(found.hits.is_empty());
    assert_eq!(found.unreadable, vec!["no such workspace: gone".to_owned()]);
    assert!(!found.is_empty(), "a refusal is content, not an empty pane");
}

/// A reply of another kind is a codec that has drifted from the query it
/// answers — a defect the round-trip tests are the witness for, not a state — so
/// nothing is invented for it: no hits, and nothing named unreadable.
///
/// **The needle survives it** (bl-670c). The answer knows its question from the
/// *ask* rather than from a reply, which is what a union of several channels'
/// answers needs: one host answering the wrong kind must not erase the search
/// every other host answered.
#[test]
fn a_reply_of_another_kind_invents_nothing() {
    let tmp = TempDir::new().expect("tmp");
    let (_listener, searcher, asks) =
        wired(&tmp, vec![json!({"ok": true, "kind": "balls", "rows": []})]);
    asks.ask("kraken");
    assert!(searcher.pass());
    assert_eq!(
        asks.found(),
        Found {
            needle: "kraken".to_owned(),
            ..Found::default()
        }
    );
}

/// The thread is the one thing only a real thread can test (the `Consumer`
/// pattern): spawn it, ask, and see the answer land — then drop it, which is
/// the shutdown path.
#[test]
fn the_searcher_thread_answers_an_ask_and_stops_on_drop() {
    let tmp = TempDir::new().expect("tmp");
    let (_listener, searcher, asks) = wired(&tmp, answers("bl-thread"));
    let thread = searcher.start();
    asks.ask("bl-thread");
    for _ in 0..200 {
        if !asks.searching() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert_eq!(asks.found().hits.len(), 1, "the thread answered");
    drop(thread);
}
