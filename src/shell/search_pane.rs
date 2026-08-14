//! The window's §8.5 search results — coverage-excluded glue like the rest of
//! `src/shell/*`: the reading is [`crate::search`]'s, the routing is
//! [`AppModel::open`]'s, and this file only paints rows and reports clicks.
//!
//! It has **no RAM of its own**. The pane is a view of the published answer, so
//! it appears when there is one and vanishes when a search with no text clears
//! it — the same relationship every other surface has with the derivation it
//! renders (§7.2). There is no "search mode" to enter or leave: since bl-1ca2
//! the answer is a **center tab focus** ([`super::center`]) that the strip
//! offers exactly while there is an answer, rather than a 220 pt scroller
//! growing out of the composer and pushing the conversation off its own pane.
//!
//! Which is why the answer arrives as an argument: the tab strip has already
//! read it to decide whether to offer the tab at all, and reading it twice per
//! frame would be two clones of the same published value.

use crate::AppModel;
use crate::search::{self, Found};

use super::ShellState;

/// What a result row does when pressed — the §11 discoverability rule.
const OPEN_HINT: &str = "Go to this result: select the thing it names, exactly as clicking it in \
     the roster would. Nothing is changed. No key of its own: Tab reaches the row, \
     Space presses it — and Ctrl+F asks the next `/search`.";

/// Paint the landed answer. A click goes where the hit says and hands the
/// keyboard back (§11 focus discipline: a pointer selection ends with the
/// cursor in the box — and on the Conversation tab, which is where the thing
/// it selected lives).
pub(super) fn pane(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    found: &Found,
    searching: bool,
) {
    if searching {
        ui.weak("searching…");
    }
    let mut open = None;
    egui::ScrollArea::vertical()
        .id_salt("search-hits")
        .show(ui, |ui| {
            // **An answer with no hits is an answer** (bl-648a, QUALITY H2):
            // it says so, in the operator's own needle, and names the way on.
            // Painted above any unreadable notes, because "nothing matched" is
            // the headline and a source that could not be read is the caveat.
            if !searching && found.hits.is_empty() {
                ui.label(search::no_matches(&found.needle));
                ui.weak(search::SEARCHED_EVERYTHING);
            }
            for hit in &found.hits {
                if ui
                    .button(egui::RichText::new(search::label(hit)).monospace())
                    .on_hover_text(OPEN_HINT)
                    .clicked()
                {
                    open = Some(hit.at.clone());
                }
            }
            for note in &found.unreadable {
                ui.weak(format!("unreadable — {note}"));
            }
        });
    if let Some(at) = open {
        model.open(&at);
        super::focus::request(state);
    }
}
