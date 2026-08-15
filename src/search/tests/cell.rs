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
        needle: "kraken".to_owned(),
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
        needle: "shoggoth".to_owned(),
        hits: vec![],
        unreadable: vec!["fresh".to_owned()],
    };
    cell.publish(seq, fresh.clone());
    assert_eq!(cell.found(), fresh);
    assert!(!cell.searching());
    assert_eq!(cell.pending(), None);
}
