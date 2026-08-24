//! The roster composed across the set (§8.2): local first, then the entries in
//! leaf order, every row wearing the channel it came from — and the zero-entry
//! shape, which must be byte for byte what a window did before §8.2 existed.

use super::*;

/// **Zero entries behaves byte for byte as it always did** (§8.2's migration
/// clause): the union is one channel's slice, every name goes where it always
/// went, and nothing extra is asked.
#[test]
fn a_window_holding_no_entry_is_one_channel() {
    let (link, mut end) = crate::wire::link::pair();
    let mut set = Channels::of(link);
    assert_eq!(set.ask(&about("home")), None, "nothing has landed yet");
    frame(&mut set, &[about("home")]);
    assert_eq!(
        end.standing(),
        vec![about("home"), roster()],
        "the roster and the routed question, both on the one channel"
    );
    assert!(
        set.awaiting(),
        "declared this frame, answered on no frame yet"
    );
    answer(
        &mut set,
        &mut end,
        &[about("home")],
        &listing(&["home", "ops"]),
    );
    assert!(!set.awaiting(), "and answered on the next");
    assert_eq!(names(&mut set), vec!["home".to_owned(), "ops".to_owned()]);
    assert_eq!(
        set.roster().first().map(|r| r.origin.clone()),
        Some(Origin::Local)
    );
}

/// **The roster is the union** (§8.2): local first, then the entries in leaf
/// order, every row wearing the channel it came from.
#[test]
fn the_roster_unions_every_channels_slice() {
    let (local, mut local_end) = crate::wire::link::pair();
    let (remote, mut remote_end) = crate::wire::link::pair();
    let filled = Channel::entry(entry("cobalt", "home"), remote);
    let empty = Channel::entry(entry("zinc", "zinc"), Link::default());
    let mut set = Channels::held(local, vec![filled, empty]);
    answer(&mut set, &mut remote_end, &[], &listing(&["home"]));
    answer(&mut set, &mut local_end, &[], &listing(&["ops"]));
    assert_eq!(
        set.roster()
            .into_iter()
            .map(|r| (r.row.workspace, r.origin.label()))
            .collect::<Vec<_>>(),
        vec![
            ("ops".to_owned(), None),
            ("cobalt".to_owned(), Some("cobalt".to_owned())),
            ("zinc".to_owned(), Some("zinc".to_owned())),
        ],
        "a workspace is a workspace; which engine hosts it is painted on it — \
         and an entry whose slice is still empty wears its leaf regardless"
    );
}

/// A composed world's entries become channels — `compose` reading the same
/// directory `entries` does, in leaf order, and a box with none composing the
/// local channel alone.
#[test]
fn compose_reads_the_worlds_entries() {
    let tmp = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    assert_eq!(
        names(&mut Channels::compose(&world, Link::default())),
        Vec::<String>::new(),
        "no entries directory is zero entries, which is every box before §8.2"
    );
    let root = crate::wire::material::dir(&world).join(ENTRIES);
    for leaf in ["zinc", "cobalt"] {
        std::fs::create_dir_all(root.join(leaf)).expect("mkdir");
    }
    assert_eq!(
        names(&mut Channels::compose(&world, Link::default())),
        vec!["cobalt".to_owned(), "zinc".to_owned()],
        "in leaf order, each holding its own name before anything is asked"
    );
}

/// A model that has adopted nothing holds the local channel alone, on a link
/// nobody answers — the same code path as an answer that has not arrived.
#[test]
fn the_default_set_is_one_channel_nobody_answers() {
    let mut set = Channels::default();
    assert_eq!(set.ask(&about("home")), None);
    assert_eq!(names(&mut set), Vec::<String>::new());
}
