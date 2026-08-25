//! The §11 Altitude-2 inspector content — the per-agent tabbed pane's
//! **display**, dispatched by [`InspectorTab`].
//!
//! This is the one tested seam between the selected tab and the landed render
//! functions ([`transcript`](crate::transcript), [`steps_view`](crate::steps_view),
//! [`inboxview`](crate::inboxview), and the config-branch governing view). The
//! shell (coverage-excluded) builds a [`TabData`] for the focused agent — the
//! six view-models built fresh each frame and **moved** in — carries the RAM
//! ephemera beside them (the Raw toggle, the step selection/tab, the jsonview
//! collapse set — §5.3), and forwards the digit-key/click events; the dispatch
//! decision itself lives here so every arm is headless shape-walk tested.
//!
//! The Config tab shows only the agent's **governing config** ("policy frozen
//! at `<short-oid>`", §9.3); the three write surfaces (brazen / lernie-global /
//! config-branch editors) are the roster's Config panel (§11 "Config mode …
//! swaps the center"), interaction glue in the shell.

/// The two owned bundles this pane renders from — the pre-built view-models
/// and the caller-owned RAM ephemera — split off at §12's budget and
/// re-exported here, so a seat still imports one module.
mod data;
pub use data::{Ephemera, TabData};

use crate::config_edit::branch::GoverningConfig;
use crate::files_view;
use crate::inboxview;
use crate::keymap::InspectorTab;
use crate::rail::Pin;
use crate::science;
use crate::steps_view;
use crate::theme;
use crate::transcript;
use crate::workdiff;

/// Render the pin banner and the selected tab's content. `eph` is the
/// caller-owned [`Ephemera`] every intrinsic interaction here writes back to —
/// a jsonview disclosure, a Files or Work row click, a transcript row's fold,
/// a notch pin. The return is the child agent a card click asks to open, which
/// the caller retargets to — the same selection gesture as the §11
/// descent-tree rows.
///
/// **The spine lives in the chat and the release lives on the banner**
/// (bl-1802). The pin reaches all four pinnable tabs (VISION V1.2), so its
/// *release* has to be reachable from each of them; the gesture that raises it
/// is a rule in the Transcript tab, so the banner — which already paints above
/// every pinnable tab — carries the way back. That is one existing gesture
/// given a second seat, not a second control: no new verb, no routing, and the
/// banner's own sentence is true wherever it paints.
///
/// `agents` is the frame's roster — the Inbox tab's one use of it is the §3.3
/// ladder over each deposit's sender (bl-b6d0). It is a parameter rather than a
/// [`TabData`] field because that bundle is the *view-models* this paints; the
/// roster is a snapshot fact the shell already holds, and copying it in would
/// be a second per-frame clone of it.
pub fn render(
    ui: &mut egui::Ui,
    tab: InspectorTab,
    data: &TabData,
    titles: &crate::nav::convs::Titles,
    eph: &mut Ephemera,
) -> Option<String> {
    // The banner claims the tab below it, so it paints only where the pin
    // actually reaches: the Work tab reads the project repo, which no
    // conversation commit indexes yet.
    if let Some(pin) = data.pin.as_ref().filter(|_| tab.pinnable()) {
        pinned_banner(ui, pin, &mut eph.notch_sel);
    }
    render_tab(ui, tab, data, titles, eph)
}

/// What a pin is showing, said outright above whichever tab is open, **and the
/// way out of it**: an operator who switched tabs after pinning must not have
/// to infer why the files look old, nor go hunting for the mark they set in
/// another tab. Clicking the banner releases the pin — the same release
/// clicking the pinned rule performs, seated where the pin is visible.
fn pinned_banner(ui: &mut egui::Ui, pin: &Pin, selected: &mut Option<usize>) {
    const RELEASE: &str = "Release the mark and return every tab to now. No key of its own: \
         Tab reaches it, Space presses it.";
    let click = |ui: &mut egui::Ui, text: egui::RichText| {
        ui.add(egui::Label::new(text).sense(egui::Sense::click()))
            .on_hover_text(RELEASE)
            .clicked()
    };
    let hit = ui
        .horizontal(|ui| {
            let mark = click(
                ui,
                egui::RichText::new(format!("as of {}", pin.short)).color(theme::BRAZEN),
            );
            let words = click(
                ui,
                egui::RichText::new(format!(
                    "— every tab below shows this conversation as it stood then; {} tokens spent \
                     by that point. Click here to come back.",
                    pin.tokens
                ))
                .weak(),
            );
            mark || words
        })
        .inner;
    if hit {
        *selected = None;
    }
}

fn render_tab(
    ui: &mut egui::Ui,
    tab: InspectorTab,
    data: &TabData,
    titles: &crate::nav::convs::Titles,
    eph: &mut Ephemera,
) -> Option<String> {
    // Only the Transcript arm answers with a child to open: it is the tab the
    // step spine is drawn through, so it is the only one holding a card to
    // click. Every other arm's answer is the general path with nothing in it.
    match tab {
        InspectorTab::Transcript => transcript::render(
            ui,
            &data.transcript,
            &transcript::Reading {
                speaker: data.speaker.clone(),
                raw: data.raw,
                auto: data.auto,
            },
            &mut eph.tx_folded,
            &data.rail,
            &mut eph.notch_sel,
        ),
        InspectorTab::Steps => {
            steps_view::render(
                ui,
                &data.steps,
                data.step_sel,
                data.step_detail.as_ref(),
                data.step_tab,
                &mut eph.json_collapsed,
                data.raw,
            );
            None
        }
        InspectorTab::Inbox => {
            inboxview::render(ui, &data.inbox, titles, data.raw);
            None
        }
        InspectorTab::Files => {
            files_view::render(
                ui,
                &data.files,
                data.file_preview.as_ref(),
                &mut eph.files_sel,
            );
            None
        }
        InspectorTab::Config => {
            render_config(ui, data.governing.as_ref());
            None
        }
        InspectorTab::Work => {
            // The fan group card first (bl-77bc), over the same answer the
            // attempt rows below drill into — with no fan it paints nothing.
            science::render::group(ui, &data.science, &mut eph.group);
            let diffs: Vec<workdiff::Attempt> =
                data.science.iter().map(|a| a.diff.clone()).collect();
            workdiff::render(ui, &diffs, data.work_patch.as_ref(), &mut eph.work_sel);
            None
        }
    }
}

/// The Config tab (§11): the focused agent's governing config — "policy frozen
/// at `<short-oid>`", the config commit its branch forks off — plus that
/// commit's file listing. The three config *editors* are the roster Config
/// panel (§9), reached by the shell's link.
fn render_config(ui: &mut egui::Ui, governing: Option<&GoverningConfig>) {
    let Some(gov) = governing else {
        ui.weak("no governing config for this agent");
        return;
    };
    ui.strong(gov.frozen_label());
    if let Some(name) = &gov.branch_name_if_tip_of_one {
        ui.weak(format!("tip of config/{name}"));
    }
    egui::ScrollArea::vertical().show(ui, |ui| {
        for file in &gov.files {
            ui.monospace(file);
        }
    });
}

#[cfg(test)]
mod tests;
