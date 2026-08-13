//! [`AppModel`]'s half of the §8.5 **search**: the ask, the landed answer, and
//! the one thing a result is for — going there.
//!
//! The window's seat is asynchronous and the other two are not, and that is not
//! two implementations: all three end in [`search::run`](crate::search::run).
//! A frame derives nothing (§7.2), so here the query is *handed over* and the
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
use crate::search::{Address, Found, Searcher};

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

    /// This instance's [`Searcher`], for the shell to spawn — the model never
    /// starts its own thread, so a test can drive `pass()` by hand (the same
    /// reason [`boot`](AppModel::boot) hands back a `Deriver`).
    pub fn searcher(&self) -> Searcher {
        Searcher::new(self.cell.clone(), self.search.clone())
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
                    self.focus_workspace(&ws);
                }
            }
            Address::Workspace { path } => self.focus_workspace(path),
            Address::Conversation { workspace, agent } => {
                self.focus_agent(workspace, agent);
                self.select_tab(InspectorTab::Transcript);
            }
        }
    }

    /// The workspace holding this ball, per the §3.5 join.
    fn ball_workspace(&self, project: &std::path::Path, id: &str) -> Option<std::path::PathBuf> {
        self.snap
            .join_rows
            .iter()
            .find(|r| r.project == project && r.ball_id == id)
            .and_then(|r| r.workspace.clone())
    }
}
