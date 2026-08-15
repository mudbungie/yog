//! The §11 Altitude-2 inspector's interaction glue: the tab strip (digit-key /
//! click tab select), the per-tab controls (the Transcript Raw toggle, the
//! Steps selection + drill-in tab), and the build of the focused agent's
//! view-models handed to the tested [`crate::inspector::render`] dispatch.
//!
//! Coverage-excluded like the rest of `shell/*`: every decision it forwards —
//! the tab select ([`AppModel::select_tab`]), the tab dispatch
//! ([`crate::inspector::render`]), and each view-model build — is tested in its
//! owning module. Since bl-13f9 every one of those view-models is a **standing
//! wire question** ([`reads`]) rather than a disk build, so a pulse repaint or
//! a scroll frame declares the set it declared before and asks nothing new; the
//! four per-snapshot memos went with the builds, an answer already being the
//! cached fold refreshed at the asker's cadence. Only the active tab declares
//! its own question — except the steps view and the transcript, which are asked
//! on every tab because the centre's auth/wound banners read the first and the
//! spine (and so the pin) is a function of the second.

use crate::AppModel;
use crate::cli_outbound::Cli;
use crate::keymap::InspectorTab;
use std::path::Path;

mod controls;
mod fork;
mod rail;
mod reads;
mod vms;
mod work;

/// The family's asks, re-exported for the two seats outside this module that
/// declare one: the centre's auth/wound banners (the steps view) and the
/// composer's prompt recall (the transcript, bl-f908). Two callers of one
/// question are **one ask** — the standing set is keyed by the encoded envelope
/// — so neither pays for the other (REMOTE §9.7, bl-13f9).
pub(in crate::shell) use reads::{steps, transcript};

use super::ShellState;

/// Render the inspector tab strip and the selected tab's content for the
/// focused agent. No agent selected ⇒ a prompt; the digit keys and the tab
/// strip both target [`AppModel::select_tab`]. Takes the whole [`ShellState`]
/// (not just its inspector RAM) because the Config tab's foot carries the
/// §3.6 per-conversation danger row, whose dialog lives beside the other
/// modals' state.
pub fn tabs_and_content(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    ws: &Path,
    lernie: &Cli,
) {
    let active = model.inspector_tab();
    // Wrapped, not laid in one line (§11 rule 8, bl-b531): at the documented
    // 420x320 minimum the centre is narrower than six tabs in a row, and egui
    // does not truncate a control that does not fit — it never lays it out, so
    // Config and Work simply ceased to exist. One home: `super::row::peers`.
    super::row::peers(ui, |ui| {
        for tab in InspectorTab::all() {
            if ui
                .selectable_label(tab == active, tab.label())
                .on_hover_text(format!(
                    "{} Press ({}): the strip is (1) to (6) bare, Ctrl+1 to Ctrl+6 \
                     from inside the composer.",
                    tab_hint(tab),
                    tab.digit(),
                ))
                .clicked()
            {
                model.select_tab(tab);
            }
        }
    });
    // The selection as a seat sees it (REMOTE §9.4, bl-1eb0): the id, the tip
    // the §5.1 #17 governing derivation reads, the §3.5 liveness the steps view
    // is keyed on, and the §3.3 speaker — one payload, no agent set.
    let Some(focus) = model.focused_conversation() else {
        ui.weak("select an agent to inspect");
        return;
    };
    // The frame's roster in the form a seat can hold: the §3.3 ladder's input
    // for every Inbox-tab deposit's sender (bl-b6d0).
    let titles = model.agent_titles();
    let (data, refused) = vms::tab_data(active, model, state, ws, &focus);
    // A refusal is painted, not swallowed (REMOTE §9.7, bl-f297): every tab
    // reads over the wire now, so what the engine said is the honest content
    // of this seat — and an empty listing must not stand in for it. Distinct
    // sentences only: the family shares one address, so an unresolvable
    // workspace would otherwise say the same thing five times.
    for said in &refused {
        ui.colored_label(crate::theme::ICHOR, said);
    }
    controls::per_tab_controls(ui, active, model, &mut state.inspector, &data.steps);
    // The V2 fork composer, seated at the pin and nowhere else (bl-dc0c):
    // above the tab content because it belongs to the pin banner's fact, not
    // to whichever tab happens to be open — the pin reaches all four.
    fork::seat(
        ui,
        model,
        &mut state.inspector,
        &fork::Seat {
            ws: ws.to_path_buf(),
            agent_id: focus.agent_id.clone(),
            pin: data.pin.clone(),
        },
    );
    let follow = crate::inspector::render(ui, active, &data, &titles, &mut state.inspector.eph);
    // Following a card is the ordinary selection gesture (§6 acknowledgement),
    // the same one the descent-tree rows spend — so it lands the composer like
    // every other selection (§11 focus discipline) — and the pin is the previous
    // agent's notch index, so it is released with the target it belonged to.
    if let Some(child) = follow {
        state.inspector.eph.notch_sel = None;
        super::focus::conversation(model, state, ws, &child);
    }
    // The §3.6 per-conversation danger row (bl-f17a): the delete verb's
    // visible carrier, at the foot of the settings-shaped tab — mirroring the
    // workspace verb's config-mode danger row.
    if active == InspectorTab::Config {
        super::delete_agent::danger_row(ui, model, state, lernie, ws);
    }
}

/// What each Altitude-2 tab shows, in operator terms — exhaustive over the enum,
/// so a new tab cannot ship without saying what it is (§11 discoverability
/// invariant, the badge-seat pattern applied to a control). This is the tab
/// strip's one seat, so the words live here rather than in `keymap`.
fn tab_hint(tab: InspectorTab) -> &'static str {
    match tab {
        InspectorTab::Transcript => {
            "What was said: one line per delivered message, model reply, thinking \
             block, tool call and tool result, each folding open to its full text."
        }
        InspectorTab::Steps => {
            "One row per model call — how it ended, when, how many attempts and \
             tokens — and a drill-in showing the exact bytes of each record."
        }
        InspectorTab::Inbox => {
            "Messages deposited for this agent that it has not picked up yet. Scan \
             is what delivers them."
        }
        InspectorTab::Files => {
            "This agent's worktree, read-only: its goal, notes, skills and work \
             products, with a preview of whichever file you pick."
        }
        InspectorTab::Config => {
            "The config commit this conversation is frozen on, and what it says. \
             Changing it is the ⚙ Config editors, or the model line above."
        }
        InspectorTab::Work => {
            "What this workspace's agents have actually changed in the project they work \
             in: every ball it holds, the files that ball's branch has touched, and each \
             file's changes."
        }
    }
}
