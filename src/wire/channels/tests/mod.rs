//! The union over the channel set: what a name resolves to, what refuses, and
//! the roster composed across slices — every one of them a value, so no engine
//! is assembled and nothing is dialled.
//!
//! The fixtures live here and the beats are split at §12's pre-split band along
//! the seam the module itself has: [`routing`] is which channel a name goes
//! down and what refuses, [`union`] is the roster composed across the set.

/// Where a name goes, and the two sentences that go instead of an answer.
mod routing;
/// The roster composed across every channel's slice.
mod union;

use super::Channels;
use crate::boundary::reply::{Reply, Workspaces, WsRow};
use crate::boundary::{Gesture, Query, codec};
use crate::wire::channel::{Channel, Origin};
use crate::wire::entries::{ENTRIES, Entry};
use crate::wire::link::{Link, LinkEnd};
use serde_json::Value;
use tempfile::TempDir;

/// A provisioned entry naming `leaf` here and `host` there.
fn entry(leaf: &str, host: &str) -> Entry {
    Entry {
        leaf: leaf.to_owned(),
        workspace: host.to_owned(),
        channel: Ok(crate::wire::material::Material {
            address: "127.0.0.1:7737".to_owned(),
            anchors: std::path::PathBuf::new(),
            chain: std::path::PathBuf::new(),
            key: std::path::PathBuf::new(),
        }),
    }
}

fn about(workspace: &str) -> Value {
    codec::encode(&Gesture::Ask(Query::WorkspaceBalls {
        workspace: workspace.to_owned(),
    }))
}

fn roster() -> Value {
    codec::encode(&Gesture::Ask(Query::Workspaces))
}

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
        kind: crate::binding::WorkspaceKind::Foreign,
        attention: 0,
        agents: 0,
        running: false,
        pinned: None,
        config_tip: None,
    }
}

/// One frame over the set: declare the union roster ([`Channels::roster`] is
/// itself an ask, per channel) and `asking` beside it, then settle. A question
/// the next frame stops declaring has its answer dropped, so a test that lands
/// one must keep standing on it — which is the frame loop, exactly.
fn frame(set: &mut Channels, asking: &[Value]) {
    set.roster();
    for question in asking {
        set.ask(question);
    }
    set.settle();
}

/// The asker's own pass on one channel, minus the socket.
fn answer(set: &mut Channels, end: &mut LinkEnd, asking: &[Value], reply: &Reply) {
    frame(set, asking);
    for question in end.standing() {
        end.publish(&question, Ok(reply.clone()));
    }
    frame(set, asking);
}

/// The names the union holds, in the order it composes them.
fn names(set: &mut Channels) -> Vec<String> {
    set.roster().into_iter().map(|r| r.row.workspace).collect()
}
