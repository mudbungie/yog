//! Which **workspace** a §8.2 ball verb names (DESIGN §3.1, §3.2, §8.2): the
//! Assign destination, the Move picker, and the Move picker minus the current
//! holder. Split from [`super`] per §12's 300-line budget — the ball fetch and
//! the §3.5 join live there, the verbs' target names here.
//!
//! Each is a pure read over the enumerated workspaces + the focus, so the shell
//! paints a name it never decides: the claimant rider (§8.2) stamps `--as` a
//! workspace name, and there is exactly one derivation of each.

use super::AppModel;
use crate::binding::WorkspaceKind;

impl AppModel {
    /// The focused workspace's name (§3.1): the **target** an Assign
    /// (`bl claim <id> --as <name>`) or a Move's claim stamps (§8.2/§3.2).
    /// `None` when no workspace is focused (the affordance is then withheld).
    ///
    /// **The focus verbatim since bl-7407** — it holds the name — where it was
    /// a leaf derivation off a held path. The derivation did not move, it
    /// dissolved: one fact, one home.
    pub fn focused_ws_name(&self) -> Option<String> {
        self.focus.ws.clone()
    }

    /// The local **named** workspaces' names (§3.1), the Move affordance's target
    /// picker (§8.2): where a bound ball can be re-homed. Foreign/replay workspaces
    /// carry no yog identity, so they are not move targets.
    pub fn workspace_names(&self) -> Vec<String> {
        self.snap
            .workspaces
            .iter()
            .filter_map(|w| match &w.kind {
                WorkspaceKind::Named { name } => Some(name.clone()),
                _ => None,
            })
            .collect()
    }

    /// Where a bound ball can be re-homed (§8.2 Move): [`Self::workspace_names`]
    /// minus the workspace that already holds it. One rule for the composer's
    /// `move to:` buttons and the §11 ball-row menu's destination submenu, so the
    /// visible carrier and its accelerator can never offer different destinations
    /// — and neither ever offers a move to where the ball already is.
    pub fn move_targets(&self, owner: &str) -> Vec<String> {
        self.workspace_names()
            .into_iter()
            .filter(|n| n != owner)
            .collect()
    }
}
