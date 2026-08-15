//! [`AppModel`]'s half of the §8.5 **search**: the ask, the landed answer, and
//! the one thing a result is for — going there.
//!
//! The window's seat is asynchronous and the other two are not, and that is not
//! two implementations: all three end in [`search::run`](crate::search::run) —
//! the window's by way of the wire since bl-44e9, the other two in place. A
//! frame derives nothing (§7.2), so here the query is *handed over* and the
//! frame renders whatever answer has landed — the same contract it already has
//! with the derivation worker's snapshot. `yog gesture` and the deposit
//! consumer are already off-frame and simply run it.
//!
//! **Opening a hit is not a new navigation path.** Every [`Address`] resolves
//! to the existing selection ([`focus_workspace`](AppModel::focus_workspace),
//! [`focus_agent`](AppModel::focus_agent), [`select_tab`](AppModel::select_tab)),
//! so a result behaves exactly like clicking the row it names — including the
//! §6 acknowledgement a conversation selection carries.

use super::AppModel;
use crate::keymap::InspectorTab;
use crate::search::{Address, Found};

impl AppModel {
    /// Ask (§8.5). Returns immediately: the searcher takes it, and a search
    /// already running for an older text abandons itself.
    pub fn search(&self, text: &str) {
        self.search.ask(text);
    }

    /// The landed answer — empty until the first search publishes, which is the
    /// general path with no input rather than a state to branch on.
    pub fn found(&self) -> Found {
        self.search.found()
    }

    /// Whether an ask is still outstanding (the "searching…" fact).
    pub fn searching(&self) -> bool {
        self.search.searching()
    }

    /// This instance's ask cell, for the engine to build its
    /// [`Searcher`](crate::search::Searcher) around (REMOTE §9.7, bl-44e9). The
    /// searcher needs a **seat on the wire** now, which is the engine's to mint,
    /// so the model hands over the half it owns and starts no thread — the same
    /// reason [`boot`](AppModel::boot) hands back a `Deriver`.
    pub(crate) fn search_cell(&self) -> crate::state::SearchCell {
        self.search.clone()
    }

    /// Go to a hit: the selection a click on that thing would have made.
    ///
    /// A ball is selected *through its workspace* (§3.5: the focused workspace
    /// is what names the focused ball), so a ball no workspace holds moves
    /// nothing — the row still carries the `(project, id)` every `bl` verb
    /// takes, and inventing a ball-shaped selection to route it into would be a
    /// second navigation model for one case.
    pub fn open(&mut self, at: &Address) {
        match at {
            Address::Ball { project, id } => {
                if let Some(ws) = self.ball_workspace(project, id) {
                    self.focus_workspace(&crate::naming::leaf(&ws));
                }
            }
            Address::Workspace { path } => self.focus_workspace(&crate::naming::leaf(path)),
            Address::Conversation { workspace, agent } => {
                self.focus_agent(workspace, agent);
                self.select_tab(InspectorTab::Transcript);
            }
        }
    }

    /// The workspace holding this ball, per the §3.5 join.
    fn ball_workspace(&self, project: &std::path::Path, id: &str) -> Option<std::path::PathBuf> {
        // A join row addresses by name since bl-b4b5, so the hit's path is
        // named first and the answer resolved back — both directions through
        // the snapshot's one round trip.
        let project = self.snap.project_name(project);
        self.snap
            .join_rows
            .iter()
            .find(|r| r.project == project && r.ball_id == id)
            .and_then(|r| self.snap.ws_path(r.workspace.as_deref()?).ok())
    }
}
