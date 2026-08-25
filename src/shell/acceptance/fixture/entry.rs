//! **The §8.2 entry a fixture world holds** (REMOTE §8.2) — a second channel
//! claiming one leaf, and the host at the far end of it, played by the test.
//!
//! Split from [`super::wire`] at §12's pre-split band on the seam that file's
//! own doc draws: everything there is *this box's* engine answering what the
//! frame said to it, and this is the other box. It is the one fact a
//! single-engine fixture cannot mint on disk — a workspace this window
//! participates in and does not host — so the whole of bl-e349's drive
//! (`acceptance::remote_start`) rests on it.
//!
//! The host answers exactly two things and refuses the rest in its own
//! sentence: its roster slice, because an entry wears a row before it answers
//! anything else (§8.2), and the §8.1 start pair, because that is the gesture
//! under test. Anything wider would be a second engine, which §8.2 says a slice
//! never needs.

use super::world::World;
use crate::boundary::{Gesture, codec};

/// The §3.3 name the entry's host mints for a start it accepted — the host's
/// draw, not this box's, which is the whole point of a routed fire.
pub(in crate::shell::acceptance) const HOST_MINTED: &str = "HostMintedFathom";

impl World {
    /// **Give this window a §8.2 entry** (REMOTE §8.2): a second channel
    /// claiming `leaf`, so the union roster carries a workspace this box does
    /// not host — the one fact a single-engine fixture cannot mint on disk, and
    /// the state `AppModel::start_path` withholds a local path for.
    ///
    /// The whole channel set is re-adopted, the local end included, because
    /// production composes the set once at boot (`Engine::window_wire`) and
    /// there is no attach. Nothing is lost by that: a standing question is
    /// re-declared every frame, which is the asker's own contract.
    pub(in crate::shell::acceptance) fn attach_entry(&mut self, leaf: &str) {
        let (link, link_end) = crate::wire::link::pair();
        let (remote, remote_end) = crate::wire::link::pair();
        let held = crate::wire::entries::Entry {
            leaf: leaf.to_owned(),
            workspace: leaf.to_owned(),
            channel: Ok(crate::wire::material::Material {
                address: "127.0.0.1:7737".to_owned(),
                anchors: std::path::PathBuf::new(),
                chain: std::path::PathBuf::new(),
                key: std::path::PathBuf::new(),
            }),
        };
        let channel = crate::wire::channel::Channel::entry(&held, remote);
        self.model
            .adopt_wire(crate::wire::channels::Channels::held(link, vec![channel]));
        self.link = link_end;
        self.entry = Some((leaf.to_owned(), remote_end));
    }

    /// **The entry's host, answering an act aimed at it** — `None` for every
    /// act this box's own engine owns, which is the zero-entry shape and every
    /// other fixture.
    ///
    /// It answers the §8.1 start pair and refuses the rest in its own sentence:
    /// what a beat here is about is that the pair goes to the host at all, and
    /// a host that answered everything would be a second engine, which §8.2
    /// says a slice never needs.
    pub(super) fn entry_act(
        &self,
        action: &crate::boundary::Action,
    ) -> Option<Result<crate::boundary::reply::Reply, String>> {
        let (leaf, _) = self.entry.as_ref()?;
        if action.workspace().as_deref() != Some(leaf.as_str()) {
            return None;
        }
        Some(match action {
            crate::boundary::Action::Prepare { payload, .. } => Ok(
                crate::boundary::reply::Reply::Prepared(crate::start::Prepared {
                    workspace: leaf.clone(),
                    binding: None,
                    lineage: None,
                    goal: String::new(),
                    origin: payload.origin(),
                }),
            ),
            crate::boundary::Action::Prompt { .. } => Ok(crate::boundary::reply::Reply::Started {
                conversation: HOST_MINTED.to_owned(),
            }),
            _ => Err("this fixture's entry answers only the §8.1 start pair".to_owned()),
        })
    }

    /// **The entry's host, answering its own slice.** It hands back the one
    /// roster row that makes the workspace real — the entry wears a row before
    /// it answers anything else (§8.2) — and refuses every other question in
    /// its own sentence, which is the honest reading of a fixture that holds no
    /// second engine.
    pub(super) fn entry_reads(&mut self) {
        let Some((leaf, end)) = self.entry.as_mut() else {
            return;
        };
        for question in end.standing() {
            let landed = match codec::decode(&question) {
                Ok(Gesture::Ask(crate::boundary::Query::Workspaces)) => Ok(
                    crate::boundary::reply::Reply::Workspaces(crate::boundary::reply::Workspaces {
                        rows: vec![crate::boundary::reply::WsRow {
                            workspace: leaf.clone(),
                            kind: crate::binding::WorkspaceKind::Named { name: leaf.clone() },
                            attention: 0,
                            agents: 0,
                            running: false,
                            pinned: None,
                            config_tip: None,
                        }],
                        stale: None,
                        growth: None,
                    }),
                ),
                _ => Err("this fixture holds no second engine".to_owned()),
            };
            end.publish(&question, landed);
        }
    }

    /// **Answer the outstanding §8.5 search**, the way the
    /// [`Searcher`](crate::search::Searcher) thread does — the same stand-in
    /// this file makes for the asker and the poster, one door over (bl-44e9).
    /// The searcher dials a listener the fixture has none of, so the walk runs
    /// in place over this world's own snapshot, which is what the engine at the
    /// far end would have run.
    pub(in crate::shell::acceptance) fn searches(&mut self) {
        let Some((seq, text)) = self.model.search_cell().pending() else {
            return;
        };
        let snap = self.model.derivation().clone();
        self.model
            .search_cell()
            .publish(seq, crate::search::run(&snap, &text, &|| true));
    }
}
