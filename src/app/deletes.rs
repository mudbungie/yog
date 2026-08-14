//! [`AppModel`]'s hand-off to the §3.6 unmaking: derive the confirmation, gate it,
//! run the plan, converge (DESIGN §3.6, §8.2).
//!
//! One effectful entry ([`AppModel::delete_workspace`]) and one derivation
//! ([`AppModel::delete_confirmation`]) the dialog and both §11 carriers read. The
//! gate is re-derived **at fire time**, not trusted from the dialog: a driver may
//! have woken while the confirmation sat open, and the §3.6 rule is fail-closed.

use super::AppModel;
use crate::cli_outbound::Cli;
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

    /// Fire the §3.6 unmaking on `ws`, armed by `typed` (the operator's retyped
    /// workspace name). Refuses — attempting nothing — when the workspace is not
    /// yog's to delete, when any conversation is live, or when the typed name
    /// does not match. On success the workspace set, the balls and the ops tail
    /// re-derive at once, and a focus that pointed at the dead workspace moves to
    /// the §4.1 startup derivation rather than a gone directory.
    pub fn delete_workspace(
        &mut self,
        lernie: &Cli,
        bl: &Cli,
        ws: &Path,
        typed: &str,
        ts: &str,
    ) -> Result<(), String> {
        let deps = self.boundary_deps(lernie, bl);
        let action = crate::boundary::Action::DeleteWorkspace {
            workspace: self.snap.ws_name(ws),
            typed: typed.to_owned(),
        };
        let result = crate::boundary::dispatch::dispatch(&deps, &mut self.ui, ts, &action);
        self.after_delete(ws);
        result.map(|_| ())
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

    /// Fire the §3.6 agent delete on `ws`/`root` through the boundary
    /// chokepoint. `typed` arms the subtree form (`--children`) by re-stating
    /// the conversation's name; unarmed, the bare verb fires and lernie's own
    /// declines ride back. A refusal or a declined verb is the `Err` — its
    /// stderr verbatim, already the durable ops row — and a clean removal
    /// moves a focus that pointed into the dead subtree back to the
    /// conversation list's general path.
    pub fn delete_agent(
        &mut self,
        lernie: &Cli,
        bl: &Cli,
        ws: &Path,
        root: &str,
        typed: &str,
        ts: &str,
    ) -> Result<(), String> {
        let deps = self.boundary_deps(lernie, bl);
        let action = crate::boundary::Action::DeleteAgent {
            workspace: self.snap.ws_name(ws),
            agent: root.to_owned(),
            typed: typed.to_owned(),
        };
        let result = crate::boundary::dispatch::dispatch(&deps, &mut self.ui, ts, &action);
        self.after_delete_agent(ws, root);
        match result? {
            crate::boundary::reply::Reply::Outcome(o) if !o.ok() => {
                let stderr = o.stderr.trim().to_owned();
                if stderr.is_empty() {
                    Err(format!("lernie delete failed (exit {})", o.exit))
                } else {
                    Err(stderr)
                }
            }
            _ => Ok(()),
        }
    }

    /// Converge after an agent-delete attempt: the ops tail changed (the
    /// yog-state root, the ordinary lernie-verb aftermath), and a focus inside
    /// the deleted subtree — the root or a `<root>-*` descendant — clears
    /// rather than pointing at a gone branch. The workspace's own tree
    /// re-derives through its standing watch root (§7.1).
    fn after_delete_agent(&mut self, ws: &Path, root: &str) {
        self.mark_dirty([self.roots.yog_state.clone()]);
        let inside = |a: &str| a == root || a.starts_with(&format!("{root}-"));
        if self.focus.ws.as_deref() == Some(ws) && self.focus.agent.as_deref().is_some_and(inside) {
            self.focus.agent = None;
        }
    }

    /// Converge after an unmaking attempt: name the roots the unmaking changed —
    /// the names root (the removed dir leaves the roster and the watch set), the
    /// clones root (the releases changed balls) and the yog-state root (the ops
    /// tail) — so the worker re-derives them on its next pass, ahead of the
    /// watch. Runs after a **failed** attempt too: the releases that did land are
    /// already real. Focus is the frame's own, and moves here.
    fn after_delete(&mut self, ws: &Path) {
        self.mark_dirty([
            self.roots.names(),
            self.roots.balls_clones.clone(),
            self.roots.yog_state.clone(),
        ]);
        if self.focus.ws.as_deref() == Some(ws) {
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
