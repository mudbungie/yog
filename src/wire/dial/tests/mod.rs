//! **Two real engines on one box** (bl-670c): the window's own, and one reached
//! through a §8.2 entry. Every beat below drives a real listener over real mTLS
//! on its own material, because what is under test is *which* engine answered —
//! a claim no stand-in can make.
//!
//! The fixtures are here and the beats are split at §12's pre-split band along
//! the module's own seam: [`routing`] is which channel a gesture goes down and
//! in whose spelling, [`fan`] is the searcher's ask-everyone.

/// Which channel a gesture goes down, and the two names it wears there.
mod fan;
/// Every channel asked, and what a refusal says about which one refused.
mod routing;

use super::Dial;
use crate::boundary::reply::{Reply, Workspaces, WsRow};
use crate::boundary::{Gesture, Query, codec};
use crate::registry::presence::Presence;
use crate::test_support::wire::{EPHEMERAL, NO_LISTENER, material, mint};
use crate::wire::client::Seat;
use crate::wire::entries::Entry;
use crate::wire::material::{Material, Role};
use crate::wire::server::{Answerer, Listener};
use serde_json::Value;
use tempfile::TempDir;

/// An engine that answers with **the name it was asked about, stamped with its
/// own tag** — carried back in a reply the channel boundary does *not* rename,
/// so a beat reads exactly which engine answered and exactly which spelling
/// crossed the wire.
struct Echo(&'static str);

impl Answerer for Echo {
    fn answer(
        &self,
        _peer: &crate::registry::Peer,
        request: Value,
    ) -> Box<dyn Iterator<Item = Value>> {
        let named = request
            .get("workspace")
            .and_then(Value::as_str)
            .unwrap_or("-");
        Box::new(std::iter::once(crate::boundary::reply::encode(
            &Reply::Marks {
                branch: format!("{}:{named}", self.0),
            },
        )))
    }
}

/// An engine that answers every gesture with one workspace row named `0` — the
/// reply the boundary *does* rename, for the inbound direction's beat.
struct Lists(&'static str);

impl Answerer for Lists {
    fn answer(
        &self,
        _peer: &crate::registry::Peer,
        _request: Value,
    ) -> Box<dyn Iterator<Item = Value>> {
        Box::new(std::iter::once(crate::boundary::reply::encode(
            &Reply::Workspaces(Workspaces {
                rows: vec![WsRow {
                    workspace: self.0.to_owned(),
                    kind: crate::binding::WorkspaceKind::Foreign,
                    attention: 0,
                    agents: 0,
                    running: false,
                    pinned: None,
                    config_tip: None,
                }],
                stale: None,
                growth: None,
            }),
        )))
    }
}

/// A minted wire directory with a listener bound on it, answering `answerer`.
/// The `TempDir` rides along because dropping it would take the material with
/// it.
struct Engine {
    tmp: TempDir,
    listener: Listener,
}

fn engine(answerer: std::sync::Arc<dyn Answerer>) -> Engine {
    let tmp = TempDir::new().expect("tmp");
    mint(tmp.path());
    let listener = Listener::bind(
        &material(tmp.path(), Role::Server, EPHEMERAL),
        answerer,
        Presence::default(),
    )
    .expect("bind");
    Engine { tmp, listener }
}

impl Engine {
    /// This engine's material for `role`, aimed at the port it actually bound.
    fn material(&self, role: Role) -> Material {
        material(
            self.tmp.path(),
            role,
            &crate::wire::loopback(&self.listener.address()),
        )
    }

    /// A window seat on this engine — the loopback channel's.
    fn window(&self) -> Seat {
        Seat::open(&self.material(Role::Window)).expect("seat")
    }

    /// A window seat on this engine's material aimed where nothing listens —
    /// the local channel refusing, which is a transport fact and not a
    /// whole-window one (the mint is fine; `Seat::open` never dials).
    fn dead_window(&self) -> Seat {
        Seat::open(&material(self.tmp.path(), Role::Window, NO_LISTENER)).expect("seat")
    }

    /// This engine as an entry named `leaf` here and `host` there.
    fn entry(&self, leaf: &str, host: &str) -> Entry {
        Entry {
            leaf: leaf.to_owned(),
            workspace: host.to_owned(),
            channel: Ok(self.material(Role::Client)),
        }
    }
}

/// An entry that exists and cannot be dialled — the half-provisioned shape,
/// which answers its own sentence in place of every gesture routed to it.
fn broken(leaf: &str) -> Entry {
    Entry {
        leaf: leaf.to_owned(),
        workspace: leaf.to_owned(),
        channel: Err(format!("{leaf} is an empty entry")),
    }
}

/// An entry whose material reads but whose host is not there — a dead tailnet,
/// in the one shape a test can hold still.
fn unreachable(engine: &Engine, leaf: &str) -> Entry {
    Entry {
        leaf: leaf.to_owned(),
        workspace: leaf.to_owned(),
        channel: Ok(material(engine.tmp.path(), Role::Client, NO_LISTENER)),
    }
}

/// A gesture naming `workspace`, and one naming none.
fn about(workspace: &str) -> Value {
    codec::encode(&Gesture::Ask(Query::WorkspaceBalls {
        workspace: workspace.to_owned(),
    }))
}

fn unaddressed() -> Value {
    codec::encode(&Gesture::Ask(Query::Search {
        text: "needle".to_owned(),
    }))
}

/// What an [`Echo`] engine said: `tag:name`, or the refusal instead.
fn said(landed: crate::wire::link::Landed) -> String {
    match landed {
        Ok(Reply::Marks { branch }) => branch,
        Ok(other) => format!("unexpected {other:?}"),
        Err(refusal) => refusal,
    }
}
