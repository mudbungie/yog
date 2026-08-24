//! Composing the set from a world: which entries become channels, in what
//! order, and that each one's slice and its asker's end are minted as a pair.

use super::compose;
use crate::wire::channel::RosterRow;
use crate::wire::entries::ENTRIES;
use crate::wire::link::Link;
use tempfile::TempDir;

/// A composed world's entries become channels — `compose` reading the same
/// directory `entries` does, in leaf order, and a box with none composing the
/// local channel alone.
#[test]
fn compose_reads_the_worlds_entries() {
    let tmp = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    let (mut set, ends) = compose(&world, Link::default());
    assert_eq!(
        names(&mut set),
        Vec::<String>::new(),
        "no entries directory is zero entries, which is every box before §8.2"
    );
    assert!(ends.is_empty(), "and no channel for an asker to answer on");

    let root = crate::wire::material::dir(&world).join(ENTRIES);
    for leaf in ["zinc", "cobalt"] {
        std::fs::create_dir_all(root.join(leaf)).expect("mkdir");
    }
    let (mut set, ends) = compose(&world, Link::default());
    assert_eq!(
        names(&mut set),
        vec!["cobalt".to_owned(), "zinc".to_owned()],
        "in leaf order, each holding its own name before anything is asked"
    );
    assert_eq!(
        ends.iter()
            .map(|e| e.entry.leaf.clone())
            .collect::<Vec<_>>(),
        vec!["cobalt".to_owned(), "zinc".to_owned()],
        "one end per entry channel, in the order the channels were composed"
    );
}

/// **The two halves are one pair, not two lists** — an answer published on the
/// end `compose` handed back lands on the slice of the channel it was minted
/// with, which is the whole of what the pairing has to promise.
#[test]
fn each_end_answers_its_own_channels_slice() {
    let tmp = TempDir::new().expect("tmp");
    let world = crate::test_support::world_under(tmp.path());
    let root = crate::wire::material::dir(&world).join(ENTRIES);
    for leaf in ["cobalt", "zinc"] {
        provision(&root.join(leaf));
    }
    let (mut set, mut ends) = compose(&world, Link::default());
    frame(&mut set);
    let listed = crate::boundary::reply::Reply::Workspaces(crate::boundary::reply::Workspaces {
        rows: vec![row("cobalt")],
        stale: None,
        growth: None,
    });
    let cobalt = &mut ends[0];
    for question in cobalt.end.standing() {
        cobalt.end.publish(&question, Ok(listed.clone()));
    }
    frame(&mut set);
    let attention: Vec<usize> = set
        .roster()
        .into_iter()
        .filter(|r: &RosterRow| r.row.workspace == "cobalt")
        .map(|r| r.row.attention)
        .collect();
    assert_eq!(
        attention,
        vec![7],
        "cobalt's row is the one its own end answered — zinc still wears the \
         claim row nothing has filled"
    );
}

/// The four files `Role::Client` reads. Their bytes are never parsed here —
/// nothing below dials — so what this states is only that the entry is whole.
fn provision(dir: &std::path::Path) {
    std::fs::create_dir_all(dir).expect("mkdir");
    for name in [crate::wire::material::ANCHORS, "client.pem", "client.key"] {
        std::fs::write(dir.join(name), "-----PEM-----\n").expect("write");
    }
    std::fs::write(dir.join(crate::wire::material::ADDRESS), "127.0.0.1:7737\n").expect("write");
}

/// One frame over the set: declare the union roster, then settle. A question
/// the next frame stops declaring has its answer dropped, so landing one is two
/// frames — the `Link` discipline, exactly as the frame loop keeps it.
fn frame(set: &mut super::Channels) {
    set.roster();
    set.settle();
}

/// The names the union holds, in the order it composes them.
fn names(set: &mut super::Channels) -> Vec<String> {
    set.roster().into_iter().map(|r| r.row.workspace).collect()
}

/// A row an engine answered with, distinguishable from the claim row a channel
/// wears before anything lands.
fn row(workspace: &str) -> crate::boundary::reply::WsRow {
    crate::boundary::reply::WsRow {
        workspace: workspace.to_owned(),
        kind: crate::binding::WorkspaceKind::Foreign,
        attention: 7,
        agents: 0,
        running: false,
        pinned: None,
        config_tip: None,
    }
}
