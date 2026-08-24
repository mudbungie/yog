//! egui widget: the §11 Altitude-2 Transcript tab.
//!
//! A scrollable list over the [`rows`] projection — one line per message,
//! per model text block, per thinking block, per tool call, per tool result,
//! per live tail — each folding open to its full payload under a `▶`/`▼`
//! disclosure toggle (§11 glyph doctrine: convention passes; the glyphs are
//! jsonview's, one home for the fold vocabulary). `raw` flips the whole tab
//! to verbatim bytes for every entry (§11: "every tab has a Raw toggle
//! showing verbatim bytes").
//!
//! **The step spine runs through it** (bl-1802). Every operable commit paints
//! as a horizontal rule across the chat, at the row where that model call's
//! reading began, with the children born there hanging under it — the [`spine`]
//! submodule, which is the whole of what the `history-rail` side panel used to
//! be. Where each rule sits and which notch it is arrives derived on the
//! [`Rail`] (`rail::place`), so the line, the pin behind it and the fold it
//! cuts to are one fact rather than three.
//!
//! **What one row is made of is [`row`]**, split off at §12's budget: this file
//! is the scrolling list and where the spine's rules fall in it, that one is the
//! chrome line, the toggle, the preview and the body.

use std::collections::HashSet;

use super::{AutoExpand, Transcript, rows, spine};
use crate::rail::Rail;

mod row;

use row::{render_raw, render_row};

/// How the chat is read: who the model turns ARE (§3.3 display name, bl-2335),
/// whether the Raw toggle is up, and which row classes arrive expanded (the
/// §4.1 knobs). One value because they are one question — *how to project this
/// transcript* — and the tab data already holds all three side by side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reading {
    pub speaker: String,
    pub raw: bool,
    pub auto: AutoExpand,
}

/// Render the transcript. `folds` is the caller's RAM override set (§5.3),
/// which a toggle click mutates; `rail` is the step spine whose notches paint
/// as this chat's rules and whose cards hang under them; `selected` is the
/// caller-owned pinned-notch ephemeron a rule click flips. The return is the
/// child agent a card click asks to open — the same selection gesture the §11
/// descent-tree rows spend.
pub fn render(
    ui: &mut egui::Ui,
    transcript: &Transcript,
    reading: &Reading,
    folds: &mut HashSet<String>,
    rail: &Rail,
    selected: &mut Option<usize>,
) -> Option<String> {
    let mut follow = None;
    // §11 tail idiom, whole: the newest row is the bottom row, so the view sits
    // on the bottom edge whether or not the conversation fills it, and following
    // live output costs nothing. Scrolling up is the deliberate review gesture,
    // and it releases the anchor until the operator scrolls back down.
    crate::tail::scroll(ui, true, |ui| {
        if transcript.entries.is_empty() {
            ui.label("(no messages yet)");
            return;
        }
        if reading.raw {
            for entry in &transcript.entries {
                render_raw(ui, entry);
            }
            return;
        }
        let rules = rail.rules();
        let cohorts = crate::rail::cohorts(rail);
        for row in rows(transcript, &reading.speaker, reading.auto, folds) {
            if let Some(&index) = rules.get(&row.key) {
                // Absence of a commit = no line, unchanged since bl-929d: the
                // in-flight strip owns that interval, and the rule materializes
                // on the ordinary snapshot re-derivation once the commit does.
                if let Some(notch) = rail.notches.get(index).filter(|n| n.commit.is_some()) {
                    spine::rule(ui, index, notch, selected);
                }
                for born_here in cohorts.iter().filter(|c| c.notch == index) {
                    if let Some(id) = spine::cohort(ui, born_here) {
                        follow = Some(id);
                    }
                }
            }
            render_row(ui, &row, folds);
        }
    });
    follow
}
