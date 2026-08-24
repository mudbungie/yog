//! One channel's two directions, its refusal and its slice — driven over a
//! [`Link`] pair, which is the whole of what §8.2 needs to be tested: *slices
//! are values*, so a second engine is never assembled here.
//!
//! The fixtures live here and the beats are split at §12's pre-split band along
//! the seam the module itself has: [`mapping`] is the leaf↔host-name rewrite in
//! both directions, [`slice`] is what the channel holds and what it refuses.

/// The mapping, both directions.
mod mapping;
/// The slice: the rows a channel holds, and the sentence it answers with.
mod slice;

use super::{Channel, Origin, RosterRow, claimed};
use crate::boundary::reply::{Reply, Workspaces, WsRow};
use crate::boundary::{Gesture, Query, codec};
use crate::wire::entries::Entry;
use crate::wire::link::{Link, LinkEnd};
use serde_json::Value;

/// An entry naming `leaf` here and `host` there, with material that read
/// clean.
fn entry(leaf: &str, host: &str) -> Entry {
    Entry {
        leaf: leaf.to_owned(),
        workspace: host.to_owned(),
        // Material this rung never dials: the slice is a value, and the seat
        // that would open it is bl-670c's.
        channel: Ok(crate::wire::material::Material {
            address: "host:1".to_owned(),
            anchors: std::path::PathBuf::new(),
            chain: std::path::PathBuf::new(),
            key: std::path::PathBuf::new(),
        }),
    }
}

/// The channel and the far end nobody has attached a thread to — the asker's
/// seat, played by the test.
fn wired(held: Entry) -> (Channel, LinkEnd) {
    let (link, end) = crate::wire::link::pair();
    (Channel::entry(&held, link), end)
}

/// One workspace question, encoded the way `shell::wire` encodes it.
fn about(workspace: &str) -> Value {
    codec::encode(&Gesture::Ask(Query::WorkspaceBalls {
        workspace: workspace.to_owned(),
    }))
}

/// A roster answer naming `names`.
fn listing(names: &[&str]) -> Reply {
    Reply::Workspaces(Workspaces {
        rows: names.iter().map(|n| row(n)).collect(),
        stale: None,
        growth: None,
    })
}

fn row(workspace: &str) -> WsRow {
    WsRow {
        workspace: workspace.to_owned(),
        agents: 3,
        ..claimed(workspace)
    }
}

/// One frame on this channel: declare the slice ([`Channel::rows`] is itself
/// an ask) and `asking` beside it, then settle. A question the next frame stops
/// declaring has its answer dropped, so a test that lands one must keep
/// standing on it — which is the frame loop, exactly.
fn frame(channel: &mut Channel, asking: &[Value]) {
    channel.rows();
    for question in asking {
        channel.ask(question);
    }
    channel.settle();
}

/// The asker's own pass, minus the socket: land `reply` for everything the
/// channel declared, between two frames that keep declaring it.
fn answer(channel: &mut Channel, end: &mut LinkEnd, asking: &[Value], reply: &Reply) {
    frame(channel, asking);
    for question in end.standing() {
        end.publish(&question, Ok(reply.clone()));
    }
    frame(channel, asking);
}
