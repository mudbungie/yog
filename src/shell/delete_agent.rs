//! The §3.6 agent-delete surface (bl-f17a): the per-conversation confirmation
//! dialog and its two §11 carriers' shared entry.
//!
//! Coverage-excluded glue like the rest of `shell/*`: the confirmation, the
//! census parse, the dispatch gate and the prune are the tested
//! [`crate::delete::agent`] / [`crate::boundary`] surface; this file paints them.
//!
//! **The delete is posted, not run** (REMOTE §9.8, bl-1747), exactly as its
//! workspace-wide twin is (`super::delete`): the dialog holds a [`Ticket`]
//! beside the sentence it already held, says so while the engine has not
//! answered, closes on a clean receipt and keeps the reason otherwise. The gate
//! is unmoved — re-derived at fire time inside the chokepoint, fail-closed —
//! and the `ui.json` prune is the engine's write, adopted back by §7.1.
//!
//! **One dialog, two doors.** The visible carrier is [`danger_row`] — the
//! worded, ichor `delete this conversation…` row at the foot of the
//! inspector's Config tab (the per-conversation settings surface, mirroring
//! the workspace verb's config-mode danger row). The accelerator is the
//! conversation row's context menu (`super::menus`). Both call [`open`], so
//! the menu reaches the *dialog*, never past it (§11 doctrine), and **no key
//! opens it or arms it** (§3.6). Escape dismisses it (`super::modal`).
//!
//! **The dialog enumerates from the substrate's own census** — `litany delete
//! --children --dry-run`, fetched once at open ([`open`]): the descendants by
//! name and the pending-deposit count come off litany's `DeleteReport`, never
//! a yog re-derivation. The arming scales with the blast radius (§3.6 as
//! amended): a leaf takes a plain explicit confirm — the row the operator is
//! already pointing at, its name in the title — while a subtree takes the
//! typed name, which is also the only thing that fires `--children`.
//!
//! This file is the doors, the RAM and the census they fill; what they open is
//! `delete_agent/window`.

use crate::AppModel;
use crate::cli_outbound::Cli;
use crate::delete::agent::Census;
use crate::theme;
use crate::wire::post::Ticket;
use std::path::{Path, PathBuf};

use super::ShellState;

mod window;
pub(super) use window::dialog;

/// The dialog's RAM (§5.3's unsubmitted-input carve-out): the conversation
/// under confirmation, the census fetched at open, the typed arming name and
/// the last refusal to render. Nothing here is durable.
#[derive(Default)]
pub struct DeleteAgentState {
    pub target: Option<(PathBuf, String)>,
    pub census: Option<Census>,
    pub typed: String,
    pub error: String,
    /// The posted delete this dialog is waiting on (REMOTE §9.8): `Some`
    /// disarms the button and marks the line, and the receipt it names either
    /// closes the dialog or leaves its reason here.
    pub ticket: Option<Ticket>,
}

/// Open the confirmation on `ws`/`root` — the one entry **both** carriers
/// call. The census spawn is the open's own cost (a short dry run, one
/// explicit gesture — never per frame).
pub(super) fn open(state: &mut ShellState, litany: &Cli, ws: &Path, root: &str) {
    let (census, error) = match crate::delete::agent::census(litany, ws, root) {
        Ok(census) => (Some(census), String::new()),
        Err(e) => (None, e),
    };
    state.delete_agent = DeleteAgentState {
        target: Some((ws.to_path_buf(), root.to_owned())),
        census,
        error,
        ..DeleteAgentState::default()
    };
}

/// The visible carrier (§11 roster, conversation-row seat): the worded, ichor
/// row at the foot of the inspector's Config tab, aimed at the focused
/// agent's conversation root. Rendered only where the verb exists — a
/// yog-named workspace (§3.6 scope), exactly what a `None` confirmation says.
pub(super) fn danger_row(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    litany: &Cli,
    ws: &Path,
) {
    // The §2.3 descent root, off the seat's own view of its selection
    // (REMOTE §9.4, bl-1eb0) — the gate is the conversation's, not the member's.
    // Since bl-48ae that view is a selection out of the landed forest, so the
    // row this click arms is aimed at the conversation the operator is looking
    // at rather than at whichever one an answer had caught up to.
    let Some(root) = super::seat::selection(model).map(|seat| seat.root) else {
        return;
    };
    // The §3.6 scope, off the landed enumeration (bl-b4b5): the verb exists for
    // yog's own named workspaces and nowhere else.
    let name = model.snap.ws_name(ws);
    if !crate::nav::tabs::is_named(&super::chrome::ws_rows(model), &name) {
        return;
    }
    ui.separator();
    if ui
        .button(egui::RichText::new("delete this conversation…").color(theme::ICHOR))
        .on_hover_text(
            "removes the agent, its children and their pending inbox — irrecoverable. \
             Typed, it is `/delete-agent [typed name]`.",
        )
        .clicked()
    {
        open(state, litany, ws, &root);
    }
}
