//! [`AppModel`]'s hand-off to the §3.6 unmaking: derive the confirmation, and
//! converge the frame once the engine says the unmaking happened (DESIGN §3.6,
//! §8.2; REMOTE §9.8).
//!
//! **The firing left** (bl-1747). Two effectful entries ran `dispatch` in
//! process over this window's own `ui.json`; both are `Action::DeleteWorkspace`
//! / `Action::DeleteAgent` posted over the wire now, so the gate — re-derived at
//! fire time and fail-closed, whichever frontend fires — runs where every other
//! seat's does, and the `ui.json` prune is the **engine's** write, adopted back
//! by §7.1 like any external change rather than made here.
//!
//! What stays is the pair a receipt still owes the frame: the derivations both
//! §11 carriers read ([`AppModel::delete_confirmation`],
//! [`AppModel::agent_delete_confirmation`]) and the two convergences a clean
//! removal earns — the roots to re-derive, and a focus that must not point at a
//! gone directory.

use super::AppModel;
use crate::delete::Confirmation;
use std::path::Path;

impl AppModel {
    /// The §3.6 confirmation for `ws` — what dies, what is released, and what is
    /// live. `None` for anything that is **not** one of yog's own named
    /// workspaces (§3.6 scope: foreign workspaces are lernie's retention-governed
    /// territory and replays are read-only), which is also how both carriers
    /// decide whether to offer the verb at all.
    pub fn delete_confirmation(&self, ws: &Path) -> Option<Confirmation> {
        crate::boundary::answer::confirmation_of(&self.snap, ws)
    }

    /// The §3.6 agent-delete confirmation for one conversation (bl-f17a) —
    /// its display name and live members. `None` outside yog's own named
    /// workspaces, which is also how the two §11 carriers (the row menu, the
    /// inspector's danger row) decide whether to offer the verb at all.
    pub fn agent_delete_confirmation(
        &self,
        ws: &Path,
        root: &str,
    ) -> Option<crate::delete::agent::AgentConfirmation> {
        crate::boundary::answer::agent_confirmation_of(&self.snap, ws, root)
    }

    /// Converge after an agent-delete **receipt**: a focus inside the deleted
    /// subtree — the root or a `<root>-*` descendant — clears rather than
    /// pointing at a gone branch. The workspace's own tree re-derives through
    /// its standing watch root (§7.1), and the ops tail through the act's own
    /// root ([`settle_acts`](Self::settle_acts)), so neither is marked twice.
    pub(crate) fn deleted_agent(&mut self, ws: &Path, root: &str) {
        let inside = |a: &str| a == root || a.starts_with(&format!("{root}-"));
        if self.focus.ws.as_deref() == Some(crate::naming::leaf(ws).as_str())
            && self.focus.agent.as_deref().is_some_and(inside)
        {
            self.focus.agent = None;
        }
    }

    /// Converge after an unmaking **receipt**: name the roots the act's own
    /// routing does not — the names root (the removed dir leaves the roster and
    /// the watch set) and the clones root (the releases changed balls) — so the
    /// worker re-derives them on its next pass, ahead of the watch. Focus is the
    /// frame's own, and moves here.
    ///
    /// Run on a **refused** unmaking too, for the reason it always was: the
    /// releases that did land are already real. What decides is that the engine
    /// answered, not what it answered.
    pub(crate) fn deleted_workspace(&mut self, ws: &Path) {
        self.mark_dirty([self.roots.names(), self.roots.balls_clones.clone()]);
        if self.focus.ws.as_deref() == Some(crate::naming::leaf(ws).as_str()) {
            let survivors: Vec<crate::binding::Workspace> = self
                .snap
                .workspaces
                .iter()
                .filter(|w| w.path != ws)
                .cloned()
                .collect();
            self.focus = self.startup_focus(None, &survivors);
        }
    }
}
