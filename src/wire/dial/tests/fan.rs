//! The searcher's ask-everyone: every channel in order, each answer handed over
//! as it lands, and a refusal that says which channel refused.

use super::*;

/// **Fanned out and unioned** (§8.2): the local channel first, then the entries
/// in the order they were composed — one answer per channel, and no gesture
/// rewritten, a search naming no workspace.
#[test]
fn a_search_asks_every_channel_local_first() {
    let here = engine(std::sync::Arc::new(Echo("local")));
    let there = engine(std::sync::Arc::new(Echo("host")));
    let dial = Dial::compose(
        &[there.entry("cobalt", "home"), there.entry("zinc", "zinc")],
        here.window(),
    );
    let mut answers = Vec::new();
    dial.fanned(&unaddressed(), &mut |landed| {
        answers.push(said(landed));
        true
    });
    assert_eq!(
        answers,
        vec![
            "local:-".to_owned(),
            "host:-".to_owned(),
            "host:-".to_owned()
        ],
        "every channel asked, in channel order, the envelope crossing as written"
    );
}

/// **A union that says only *connect refused* cannot say which host.** An
/// entry's refusal is named with the entry that gave it; the local channel's is
/// not, an unattributed sentence having always meant this window's own engine.
/// The routed calls need neither, their answer landing on a slice that already
/// wears its origin.
#[test]
fn a_fanned_refusal_names_the_channel_that_refused() {
    let here = engine(std::sync::Arc::new(Echo("local")));
    let dial = Dial::compose(&[broken("cobalt")], here.window());
    let mut answers = Vec::new();
    dial.fanned(&unaddressed(), &mut |landed| {
        answers.push(said(landed));
        true
    });
    assert_eq!(
        answers,
        vec![
            "local:-".to_owned(),
            "the entry \"cobalt\": cobalt is an empty entry".to_owned()
        ],
        "and the local channel answered regardless"
    );
}

/// **The fan stops when the caller says so** — how a superseded search abandons
/// without a cancel the boundary would have to carry.
#[test]
fn the_fan_ends_when_the_caller_stops_wanting_it() {
    let here = engine(std::sync::Arc::new(Echo("local")));
    let there = engine(std::sync::Arc::new(Echo("host")));
    let dial = Dial::compose(&[there.entry("cobalt", "home")], here.window());
    let mut answers = Vec::new();
    dial.fanned(&unaddressed(), &mut |landed| {
        answers.push(said(landed));
        false
    });
    assert_eq!(
        answers,
        vec!["local:-".to_owned()],
        "one channel asked, and the entry never dialled"
    );
}

/// **The local channel's refusal carries no attribution**, because an
/// unattributed sentence has always meant this window's own engine — and a box
/// holding no entry must read byte for byte what it read before §8.2.
#[test]
fn the_local_channels_refusal_is_never_attributed() {
    let here = engine(std::sync::Arc::new(Echo("local")));
    let dial = Dial::compose(&[broken("cobalt")], here.dead_window());
    let mut answers = Vec::new();
    dial.fanned(&unaddressed(), &mut |landed| {
        answers.push(said(landed));
        true
    });
    assert!(
        answers[0].starts_with("connect 127.0.0.1:1"),
        "the transport's own sentence, unprefixed: {}",
        answers[0]
    );
    assert_eq!(
        answers[1], "the entry \"cobalt\": cobalt is an empty entry",
        "while the entry beside it says which entry it was"
    );
}
