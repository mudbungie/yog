//! Which channel a name goes down (§8.2) — `wire::seat::channel`'s rule read
//! from a frame — and the two things said instead of an answer: the collision
//! refusal, and one entry's own sentence.

use super::*;

/// **An entry is the answer to its leaf** — the write path's rule
/// (`wire::seat::channel`) read from a frame: the question goes down that
/// entry's channel carrying the name its host knows.
#[test]
fn a_name_an_entry_holds_goes_down_that_entrys_channel() {
    let (local, mut local_end) = crate::wire::link::pair();
    let (remote, mut remote_end) = crate::wire::link::pair();
    let mut set = Channels::held(
        local,
        vec![Channel::entry(&entry("cobalt", "home"), remote)],
    );
    assert_eq!(set.ask(&about("cobalt")), None);
    frame(&mut set, &[about("cobalt")]);
    assert_eq!(
        remote_end.standing(),
        vec![about("home"), roster()],
        "carried under the host's name, beside the entry's own roster ask"
    );
    assert!(
        !local_end.standing().contains(&about("cobalt")),
        "and never fell through to this box's own engine"
    );
}

/// The other half of the same rule: a name no entry holds — and a question
/// naming no workspace at all — goes where it always went.
#[test]
fn every_other_name_goes_to_this_windows_own_engine() {
    let (local, mut local_end) = crate::wire::link::pair();
    let (remote, mut remote_end) = crate::wire::link::pair();
    let mut set = Channels::held(
        local,
        vec![Channel::entry(&entry("cobalt", "home"), remote)],
    );
    frame(&mut set, &[about("ops")]);
    assert!(local_end.standing().contains(&about("ops")));
    assert_eq!(
        remote_end.standing(),
        vec![roster()],
        "the entry is asked its own roster and nothing else"
    );
}

/// **A collision refuses naming the token** — §8's two-roots-one-leaf rule one
/// namespace up, with §8.2's remedy: rename the entry, never the workspace on
/// its host.
#[test]
fn a_leaf_that_collides_with_a_local_name_refuses() {
    let (local, mut local_end) = crate::wire::link::pair();
    let mut set = Channels::held(
        local,
        vec![Channel::entry(&entry("home", "home"), Link::default())],
    );
    answer(&mut set, &mut local_end, &[], &listing(&["home"]));
    assert_eq!(
        set.ask(&about("home")),
        Some(Err(concat!(
            "ambiguous workspace \"home\": this window's own engine and the ",
            "entry \"home\" hold that name and the union is one namespace — ",
            "rename the entry (`mv` its directory under `workspaces/`), never ",
            "the workspace on its host"
        )
        .to_owned())),
        "one token, one workspace — and the sentence says which token"
    );
    assert_eq!(
        names(&mut set),
        vec!["home".to_owned(), "home".to_owned()],
        "both rows still stand: the roster shows the collision it refuses on"
    );
}

/// **A refusal is one entry's, never the set's** (§8.2). The half-provisioned
/// entry answers its own sentence to its own name, holds its own row, and every
/// other channel's slice stands beside it untouched.
#[test]
fn a_half_provisioned_entry_does_not_disturb_the_rest_of_the_roster() {
    let (local, mut local_end) = crate::wire::link::pair();
    let broken = Entry {
        channel: Err("cobalt is an empty entry: its material is minted on the \
                      host that issued it"
            .to_owned()),
        ..entry("cobalt", "home")
    };
    let mut set = Channels::held(
        local,
        vec![
            Channel::entry(&broken, Link::default()),
            Channel::entry(&entry("zinc", "zinc"), Link::default()),
        ],
    );
    answer(&mut set, &mut local_end, &[], &listing(&["ops"]));
    assert_eq!(
        set.ask(&about("cobalt")),
        Some(Err(
            "cobalt is an empty entry: its material is minted on the \
                  host that issued it"
                .to_owned()
        )),
        "its own sentence, never a fall-through to this box's own engine"
    );
    assert_eq!(
        names(&mut set),
        vec!["ops".to_owned(), "cobalt".to_owned(), "zinc".to_owned()],
        "and the roster is whole: the local slice and every other entry stand"
    );
    assert_eq!(
        set.ask(&about("ops")),
        None,
        "the rest is asked as it always was"
    );
}
