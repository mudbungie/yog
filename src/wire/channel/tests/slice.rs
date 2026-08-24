//! What one channel holds: the row an entry wears before anything answers, the
//! sentence a half-provisioned one answers with instead, and the local
//! channel — which claims no name and renames nothing.

use super::*;

#[test]
fn an_entry_that_landed_nothing_still_wears_its_leaf() {
    let (mut channel, _end) = wired(entry("cobalt", "home"));
    assert_eq!(
        channel.rows(),
        vec![RosterRow {
            row: claimed("cobalt"),
            origin: Origin::Entry {
                leaf: "cobalt".to_owned(),
                host: "home".to_owned()
            },
        }],
        "the entry IS a workspace this box participates in; it wears a row \
         from the moment it is provisioned, carrying the zeros it honestly has"
    );
    frame(&mut channel, &[]);
    assert!(channel.awaiting(), "and it is still waiting for its facts");
}

#[test]
fn a_half_provisioned_entry_answers_its_own_sentence() {
    let mut channel = Channel::entry(
        &Entry {
            channel: Err("cobalt is an empty entry".to_owned()),
            ..entry("cobalt", "home")
        },
        Link::default(),
    );
    assert_eq!(
        channel.ask(&about("cobalt")),
        Some(Err("cobalt is an empty entry".to_owned())),
        "an entry that exists is the answer to its name even when it cannot \
         be dialled — never a fall-through to another engine"
    );
    assert_eq!(
        channel.rows().len(),
        1,
        "and it still holds its leaf in the roster"
    );
}

#[test]
fn the_local_channel_claims_nothing_and_renames_nothing() {
    let (link, mut end) = crate::wire::link::pair();
    let mut channel = Channel::local(link);
    assert!(!channel.claims("cobalt"), "every unclaimed name goes here");
    assert_eq!(
        channel.ask(&about("cobalt")),
        None,
        "nothing has landed yet"
    );
    answer(&mut channel, &mut end, &[], &listing(&["home"]));
    assert_eq!(
        channel.rows(),
        vec![RosterRow {
            row: row("home"),
            origin: Origin::Local
        }],
        "no claim row: the local channel is not a workspace"
    );
    assert_eq!(Origin::Local.label(), None);
}
