//! The conversation's **settings rows** (§11 bottom accessories, bl-2e18):
//! the config-shaped rows of altitude 1, docked at the bottom of the
//! conversation pane below the composer.
//!
//! The ruling: every setting for a conversation moves to the bottom of the
//! surface instead of the top. The altitude-1 header is
//! the identity line and nothing else ([`super::workspace`]); what the
//! conversation *runs on* — the §9.4 model line and the §3.5 budget-spent
//! figures — reads at the bottom of the conversation, and the transcript leads
//! uninterrupted.
//!
//! **bl-2e18's ordering clause is superseded** (bl-58e4). That ball seated
//! these rows *"between the goal box and the in-flight strip, so what the
//! conversation runs on reads beside where the operator talks to it"* — the
//! adjacency to the box was the stated reason. The later ruling: the work
//! directory, the budget, the context and the model selection must not sit
//! between the input bar and the chat — those elements belong below the input
//! box, not above it. Those four
//! elements are exactly this band — the §9.4 model line, the §3.5 spend
//! figures, the §5.1 #35 context line, and the work directory the birth block
//! below carries — so the adjacency is what the ruling overrides: these rows
//! are read occasionally, and putting them between the chat and the box the
//! operator types into pushed the two things that ARE read together apart.
//!
//! The seat is therefore the **bottom edge of the pane**, below the goal box.
//! Everything else about bl-2e18 stands: the ruling that these rows
//! belong at the bottom of the conversation rather than on its header, and the
//! internal order of the rows, which that ball also fixed. The in-flight strip
//! keeps its own seat hard against the chat tail (bl-905f) — it is not one of
//! the four elements named, and the ruling does not reach it.
//!
//! **An empty selection is the same seat, not an empty one.** With no
//! conversation selected the rows are the §11 birth-config block
//! ([`super::birth`]): these rows answer *what is this conversation running
//! on*, and with nothing selected the same question is *what would one started
//! now run on* (bl-824e, re-seated here with the settings it mirrors). One
//! branch on the selection, never a second surface.
//!
//! **The seat is bounded at half the pane** — the fold line's own ceiling
//! (§11 inbox-composer), one idiom for "an accessory may not eat the pane it
//! hangs off". The §9.4 picker expands inline at the model line, so the region
//! is exactly the kind that grows without limit, and an over-subscribed bottom
//! stack is the QUALITY G4 defect bl-9551 filed. Past the cap the rows scroll.
//!
//! Coverage-excluded shell glue like the rest of `src/shell/*`: every fact it
//! paints is composed and tested elsewhere (`crate::model_pick`,
//! `crate::spend`, `AppModel`).

use super::ShellState;
use crate::AppModel;
use crate::boundary::answer::agent::AgentView;
use crate::cli_outbound::Cli;
use crate::spend;
use std::path::Path;

/// Paint the seat as the conversation pane's own bottom panel. **Created first
/// of the stack, so it holds the pane's bottom edge** — below the goal box,
/// which is where the band-order ruling puts these four elements. `cap` is the
/// band's ceiling, already divided by the pane ([`crate::layout::share`]): the
/// stack's arithmetic lives in one place ([`super::pane`]) and each band is
/// handed its answer, so the row this band holds back for the box below it is
/// decided where the order it follows from is written.
pub(super) fn render(
    ui: &mut egui::Ui,
    cap: f32,
    model: &mut AppModel,
    state: &mut ShellState,
    bz: &Cli,
) {
    let Some(ws) = model.focused_workspace() else {
        return;
    };
    egui::TopBottomPanel::bottom("conversation-settings").show_inside(ui, |ui| {
        // The panel is sized by its content, and a `ScrollArea` sizes itself by
        // what is *available* — which inside a content-sized panel is last
        // frame's height. Left alone the two lock each other at whatever the
        // first frame happened to be, and an opened picker paints into a clip
        // rect two rows tall. Handing the ui the cap outright breaks the loop:
        // the region grows with its content up to half the pane, then scrolls.
        ui.set_max_height(cap);
        egui::ScrollArea::vertical()
            .id_salt("conversation-settings")
            .max_height(cap)
            .show(ui, |ui| match model.focused_conversation() {
                Some(seat) => conversation(ui, model, state, &ws, &seat, bz),
                None => super::birth::block(ui, model, state, &ws, bz),
            });
    });
}

/// The rows of a selected conversation: the §3.5 spend figures — one per bound
/// workspace ball, then the conversation's own — and the §9.4 model line with
/// the picker it opens.
fn conversation(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    ws: &Path,
    agent: &AgentView,
    bz: &Cli,
) {
    // The §3.5 per-ball figure, one row per bound ball (§3.2's claimant join):
    // the Usage fold joined with the price table, each carrying the
    // granularity it is honest at — a ball claimed mid-conversation attributes
    // workspace-wide, and the row says so. Each row names its own ball, so the
    // figures read as themselves down here, away from the header's ball line.
    for ball in model.ws_balls(ws) {
        ui.horizontal(|ui| {
            ui.weak(format!("{}:", ball.id));
            spend::render(ui, &model.ball_spend(ws, &ball.id));
        });
    }
    let root = agent.root.as_str();
    spend::render(ui, &model.conversation_spend(ws, root));
    // The §5.1 #35 context figure, directly under the spend it is not: the
    // budget line above sums the whole descent's burn, this one states how full
    // *this* conversation's context is right now. Absent — no step, no model, or
    // no declared window — it paints nothing at all rather than a placeholder.
    if let Some(full) = model.conversation_context(ws, root) {
        crate::context::render::render(ui, &full);
    }
    // The model row (§9.4): *what am I talking to, and how do I change it* is
    // asked while looking at a conversation, so it is answered on the
    // conversation surface rather than in the Config tab — and since bl-cd2a it
    // is answered by the two dropdowns themselves, not by a sentence with a
    // button. They show and write the **workspace default**: the branch tip a
    // pick advances, for the next conversation. This one stays frozen on the
    // commit it forked off, which the row's hover says and — when the two have
    // parted — a clause beside them names, with the two exits it earns (§9.4
    // drift): `lernie retarget`, which moves this conversation onto the current
    // config and keeps its history (bl-2d19), and the composer's own
    // new-conversation verb, focused, which starts over instead (bl-9786).
    //
    // The workspace's config-lineage tip (§7 snapshot, `HEAD` → `config/default`)
    // is the "workspace default" half of that drift.
    let config_tip = model.config_tip();
    if super::model_pick::conversation_seat(ui, model, state, ws, agent, config_tip.as_ref(), bz) {
        super::keys::new_conversation(model, state);
    }
}
