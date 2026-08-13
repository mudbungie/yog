//! The inspector's **controls** (§11 altitude 2): the per-tab knobs the
//! operator drives — the Transcript Raw toggle and its two density checkboxes,
//! the Steps step-selector and its drill-in tab picker. Split from [`super`] at
//! §12's cap on a real seam: the parent decides *which* view-model a frame
//! builds, this decides *how the operator steers what is shown*, and every
//! control here states what pressing it does (§11 discoverability invariant).
//!
//! Coverage-excluded glue like the rest of `src/shell/*`.

use super::super::InspectorState;
use crate::AppModel;
use crate::cli_outbound::Cli;
use crate::keymap::InspectorTab;
use crate::steps_view::StepsView;

/// The Raw toggle's label — the same words wherever it appears, because it is
/// the same knob (§5.3 ephemera, one `InspectorState::raw` behind all of them).
const RAW_LABEL: &str = "Raw (verbatim bytes)";

/// The Raw toggle's hover — one knob, one sentence (§11 rule 4).
const RAW_HINT: &str = "Show the files behind this tab exactly as they are on disk, \
     unparsed — nothing summarised away. No key of its own: Tab reaches it, Space \
     presses it.";

/// The active tab's controls: the **Raw toggle on every tab that parses a
/// file** — Transcript, Steps, Inbox (bl-1ff1: Files already *is* a bytes
/// preview and Config renders no file's bytes at all, so neither carries one;
/// STORIES.md S7 point 3 says so in those words) — beside the Transcript's two
/// §11 density knobs (durable, `ui.json` §4.1 — unlike the rest here, which is
/// §5.3 ephemera), the Steps step-selector + drill-in tab picker, and the
/// Inbox tab's Scan button (the Flush = `lernie scan` verb, moved off the
/// composer — it flushes the focused workspace's inbox, not this agent's
/// conversation, so it belongs beside the deposits it flushes rather than
/// among the composer's send/stop verbs).
pub(super) fn per_tab_controls(
    ui: &mut egui::Ui,
    tab: InspectorTab,
    model: &mut AppModel,
    inspector: &mut InspectorState,
    steps: &StepsView,
    lernie: &Cli,
    bl: &Cli,
) {
    match tab {
        InspectorTab::Transcript => {
            ui.horizontal(|ui| {
                ui.checkbox(&mut inspector.raw, RAW_LABEL)
                    .on_hover_text(RAW_HINT);
                let mut responses = model.transcript_auto_expand().responses;
                if ui
                    .checkbox(&mut responses, "auto-expand conversation")
                    .on_hover_text(
                        "Open the model's replies to their full text by default. Off, \
                         each is one line you can still fold open. Remembered between \
                         launches. No key of its own: Tab reaches it, Space presses it.",
                    )
                    .changed()
                {
                    model.set_transcript_expand_responses(responses);
                }
                let mut others = model.transcript_auto_expand().others;
                if ui
                    .checkbox(&mut others, "auto-expand machinery")
                    .on_hover_text(
                        "Same, for everything that is not the conversation itself — \
                         thinking, tool calls and their results. Remembered between \
                         launches. No key of its own: Tab reaches it, Space presses it.",
                    )
                    .changed()
                {
                    model.set_transcript_expand_others(others);
                }
            });
        }
        InspectorTab::Steps => {
            ui.checkbox(&mut inspector.raw, RAW_LABEL)
                .on_hover_text(RAW_HINT);
            step_controls(ui, inspector, steps);
        }
        InspectorTab::Inbox => {
            ui.horizontal(|ui| {
                ui.checkbox(&mut inspector.raw, RAW_LABEL)
                    .on_hover_text(RAW_HINT);
                if ui
                    .button("Scan")
                    .on_hover_text(
                        "Run `lernie scan` on this workspace: it writes a died epitaph \
                         for every driver that crashed without one, and delivers inbox \
                         deposits still sitting queued. Nothing is started and nothing \
                         is killed (f).",
                    )
                    .clicked()
                {
                    crate::shell::dispatch::scan_focused(model, lernie, bl);
                }
            });
        }
        _ => {}
    }
}

/// The Steps tab's selection controls: a clickable step seq per row (sets the
/// drill-in target), and the drill-in tab picker (§11 meta/request/staging/
/// response/tools). Selection is RAM ephemera the caller owns.
fn step_controls(ui: &mut egui::Ui, inspector: &mut InspectorState, steps: &StepsView) {
    ui.horizontal(|ui| {
        ui.weak("step:");
        for (i, step) in steps.steps.iter().enumerate() {
            if ui
                .selectable_label(inspector.step_sel == Some(i), &step.seq)
                .on_hover_text(
                    "Drill into this step — its request, staging, response events and \
                     per-tool input/output, byte for byte, below. No key of its own: \
                     Tab reaches it, Space presses it.",
                )
                .clicked()
            {
                inspector.step_sel = Some(i);
            }
            // The §8.3 Login affordance beside an auth-shaped step failure: a
            // prompt-time credential failure surfaces here as derived agent state
            // (§13.3). The row rides the mark when the governing config binds
            // this step's model to one (bl-8e34) — the Toolchain pane is still
            // where the sign-in runs, but it no longer has to be where the
            // operator works out *which* row failed.
            if step.auth_failed.offered() {
                ui.colored_label(crate::theme::ICHOR, step.auth_failed.step_mark());
            }
        }
    });
    if inspector.step_sel.is_none() {
        return;
    }
    ui.horizontal(|ui| {
        // The words are steps_view's (bl-3ffc's record table), read from its one
        // home rather than spelled a second time here (§11).
        for (which, label, hint) in crate::steps_view::RECORDS {
            if ui
                .selectable_label(inspector.step_tab == which, label)
                .on_hover_text(hint)
                .clicked()
            {
                inspector.step_tab = which;
            }
        }
    });
}
