//! The search hand-off (§8.5): the cell's supersede/publish protocol, and the
//! one test only a real thread can give the searcher.

use super::*;

/// The searcher's whole protocol: an ask is answered once, a second ask
/// supersedes the first, and an answer for a superseded seq is discarded.
#[test]
fn the_cell_answers_the_current_question_and_discards_a_superseded_one() {
    let cell = SearchCell::default();
    assert!(!cell.searching());
    assert_eq!(cell.pending(), None);

    cell.ask("kraken");
    assert!(cell.searching());
    let (seq, text) = cell.pending().unwrap();
    assert_eq!(text, "kraken");

    cell.ask("shoggoth");
    let stale = Found {
        hits: vec![],
        unreadable: vec!["stale".to_owned()],
    };
    cell.publish(seq, stale);
    assert_eq!(
        cell.found(),
        Found::default(),
        "a superseded run publishes nothing"
    );

    let (seq, text) = cell.pending().unwrap();
    assert_eq!(text, "shoggoth");
    let fresh = Found {
        hits: vec![],
        unreadable: vec!["fresh".to_owned()],
    };
    cell.publish(seq, fresh.clone());
    assert_eq!(cell.found(), fresh);
    assert!(!cell.searching());
    assert_eq!(cell.pending(), None);
}

/// The thread is the one thing only a real thread can test (the `Consumer`
/// pattern): spawn it, ask, and see the answer land — then drop it, which is
/// the shutdown path.
#[test]
fn the_searcher_thread_answers_an_ask_and_stops_on_drop() {
    let ws = PathBuf::from("/w/x");
    let snap = world(&ws, vec![], vec![ball("bl-thread", "t", "body")], vec![]);
    let cell = crate::state::new_snapshot_cell(std::sync::Arc::new(snap));
    let asks = SearchCell::default();
    let searcher = Searcher::new(cell, asks.clone());
    assert!(!searcher.pass(), "nothing asked, nothing answered");
    let thread = searcher.spawn();
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
