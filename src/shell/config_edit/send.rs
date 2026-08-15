//! **The lineage pane's write half** (§9.3, REMOTE §9.8; bl-4841): the one
//! gesture this surface fires, and the receipt it earns.
//!
//! Split from [`branch_pane`](super::branch_pane) at DESIGN §12's budget, on the
//! seam the act path draws everywhere: a pane browses and drafts, and firing a
//! gesture — which since bl-4841 means *posting* one and folding its answer
//! frames later — is a different thing from painting one.

use super::super::act;
use super::ConfigState;
use super::branch_pane::reread;
use crate::AppModel;
use crate::boundary::Action;
use crate::boundary::config::ConfigFile;
use std::path::Path;

/// Send the drafted file through the boundary (§8.5), **posted** (REMOTE §9.8):
/// the variant carries the destination — this workspace, this lineage, this
/// path, this origin — and the full staged text, and the chokepoint stages it
/// and drives `lernie config`.
pub(super) fn edit(model: &mut AppModel, config: &mut ConfigState, ws: &Path) {
    let action = Action::ApplyConfig {
        file: ConfigFile::Branch {
            workspace: model.snap.ws_name(ws),
            lineage: config.cb_name.clone(),
            origin: config.cb_origin.clone(),
            path: config.cb_path.clone(),
        },
        text: config.cb_body.clone(),
    };
    config.cb_act.fire(
        model,
        &action,
        "sent — staged and committed by `lernie config`",
    );
}

/// Fold what the send earned, once, on the frame it arrives — and re-read the
/// lineage then, because the pane caused the advance (§7.2) and the advance is
/// what just happened.
pub(super) fn settle(model: &mut AppModel, config: &mut ConfigState, ws: &Path) {
    let Some(landed) = config.cb_act.landed(model) else {
        return;
    };
    if let Some(why) = act::trouble(&landed) {
        let said = format!("{} — ⚠ {why}", config.cb_act.line());
        config.cb_act.say(said);
    }
    reread(config, Some(ws));
}
