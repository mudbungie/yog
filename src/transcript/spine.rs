//! egui widget: the step spine drawn **through** the chat (bl-1802, VISION V1).
//!
//! History riding alongside the chat was right, but as its own window it was
//! wrong: every operable commit is a horizontal rule across the chat rather
//! than a panel beside it, and the fork overlay rises when one is clicked.
//!
//! So this is what the `history-rail` side panel used to be, minus the panel.
//! An **operable commit** is a notch with a seat in the chat
//! ([`crate::rail::Place`]) and a `meta.json` commit behind it; its rule is the
//! faint full-width line bl-929d already drew above each commit boundary, now
//! carrying the gesture that used to live on the notch: **clicking a rule pins
//! the whole inspector to that commit, and clicking the pinned one releases
//! it** — one gesture, both directions, no second control to find. The pinned
//! rule burns brazen so the mark is visible in the chat it belongs to, and the
//! §11 fork composer, which is seated on the pin and dies with it, raises
//! itself on the same click — that is the ruling's *"fork overlay"*, with no
//! new mechanism at all.
//!
//! Under a rule hang the children born at that commit: one **cohort** (VISION
//! V2.3) — a `×N` chip and the ancestry its members share stated once, then a
//! column per candidate with its state chip, its own spend and its terminal
//! response. A cohort of one wears no header and is byte-for-byte V1's card,
//! so `N == 1` and `N > 1` walk one path here as everywhere else in the rung.
//!
//! **What the gutter drew that a chat cannot** (VISION V1.3): the two edge
//! strokes, solid for context and dashed for provenance. A rule across a chat
//! has no column to stroke in, and the distinction is already in the card's
//! fork label in words — `from here` / `from <Name>@<oid>` name an ancestry,
//! `from config/<name>` names a clean child that has none. The taxonomy
//! survived; its second rendering did not (the reasoning is `crate::rail`'s
//! module note). Drawing the descent graph as a graph needs a seat that is not
//! the chat, and that is filed as bl-5cf8, not smuggled in here.

use crate::rail::{ChildCard, Cohort, Notch};
use crate::theme;

/// How faint the rule's stroke is: its hue dimmed, so the line reads as
/// structure under the text rather than as a row of its own.
const RULE_FAINT: f32 = 0.45;
/// Vertical room the rule's line claims beside its right-aligned id.
const RULE_HEIGHT: f32 = 6.0;
/// Width a child's card claims under its rule — a glance, not a pane.
const CARD_WIDTH: f32 = 320.0;
/// What an unpinned rule offers (bl-68ac hover doctrine: the seat, never the
/// derivation).
const RULE_HOVER: &str = "a commit boundary — the id is the commit the next turn's model call read. \
     Click to see this conversation as it stood then, and to fork from here. No key of \
     its own: Tab reaches the rule, Space pins it.";
/// What the pinned rule offers: the release, said where the pin was made.
const PINNED_HOVER: &str = "this conversation as it stood here. Click to come back to now — or Tab to the rule \
     and press Space, which is the same release.";
/// What a card's fork label means, said once at the card.
const CARD_HOVER: &str = "An agent dispatched from this point. Its label says what it inherited: \
     `from here` carries this conversation's history, `from config/…` starts clean. Click to open it, \
     or walk the roster onto it with ↑ / ↓.";
/// What a cohort is, for an operator meeting one cold (VISION V2.3).
const COHORT_HOVER: &str = "Candidates tried from this same mark. Nothing groups them but where they were born — \
     compare them side by side and open whichever you want, by click or by the ↑ / ↓ \
     roster walk.";

/// One operable commit's rule, across the chat. `selected` is the caller-owned
/// notch selection (§5.3 viewport ephemera) the click flips.
///
/// A notch with no commit never reaches here: it has nothing to pin to, and
/// [`crate::rail::Rail::rules`] only seats notches the chat has a place for —
/// so the "cannot be picked" arm the gutter needed dissolved with the gutter.
pub(super) fn rule(ui: &mut egui::Ui, index: usize, notch: &Notch, selected: &mut Option<usize>) {
    let pinned = *selected == Some(index);
    let hue = if pinned { theme::BRAZEN } else { theme::ASH };
    let hover = if pinned { PINNED_HOVER } else { RULE_HOVER };
    // The whole line senses the click, id and stroke alike: the operator aims
    // at a rule, not at seven characters of oid.
    let hit = ui
        .horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let id = ui
                    .add(
                        egui::Label::new(egui::RichText::new(notch.short()).monospace().color(hue))
                            .sense(egui::Sense::click()),
                    )
                    .on_hover_text(hover);
                // Same binding discipline as the rule below (bl-914f).
                let size = egui::vec2(ui.available_width(), RULE_HEIGHT);
                let (rect, line) = ui.allocate_exact_size(size, egui::Sense::click());
                // Bound and one-lined rather than a multi-line call: tarpaulin's
                // llvm engine misattributes a call's interior argument lines as
                // uncovered, and which lines it picks moves with the dependency
                // graph — this exact call went CI-red on a lockfile-only change
                // (bl-914f, the steps_view "bound rather than chained" hazard).
                let rule = egui::Stroke::new(1.0, hue.gamma_multiply(RULE_FAINT));
                ui.painter().hline(rect.x_range(), rect.center().y, rule);
                id.clicked() || line.on_hover_text(hover).clicked()
            })
            .inner
        })
        .inner;
    if hit {
        *selected = if pinned { None } else { Some(index) };
    }
}

/// One cohort at its birth rule (VISION V2.3): the ancestry its members share
/// said once, then a column per candidate. A cohort of one wears no header —
/// there is no shared fact worth lifting out of a single column — so it draws
/// exactly the card V1 drew, and nothing here branches on the count to do it.
/// The return is the child a card click asks to open, or `None`.
pub(super) fn cohort(ui: &mut egui::Ui, cohort: &Cohort) -> Option<String> {
    let mut follow = None;
    if cohort.fanned() {
        let shared = cohort.common.clone().unwrap_or_else(|| "mixed".to_owned());
        ui.horizontal(|ui| {
            ui.colored_label(theme::SIGIL, format!("×{}", cohort.members.len()));
            ui.weak(shared);
        })
        .response
        .on_hover_text(COHORT_HOVER);
    }
    for member in &cohort.members {
        if let Some(id) = card(ui, member, cohort.common.is_none() || !cohort.fanned()) {
            follow = Some(id);
        }
    }
    follow
}

/// One candidate's column: the child's name, its fork-point label, state chip,
/// spend and terminal response (VISION V2.3's four side-by-side facts).
/// `own_fork` is false exactly when the cohort's header already said the
/// ancestry — one fact is stated once, at whichever level is its home.
fn card(ui: &mut egui::Ui, card: &ChildCard, own_fork: bool) -> Option<String> {
    let (glyph, hue, phrase) = theme::state_badge(card.state);
    let mut follow = None;
    egui::Frame::group(ui.style())
        .show(ui, |ui| {
            ui.set_width(CARD_WIDTH);
            let clicked = ui
                .add(egui::Label::new(&card.name).sense(egui::Sense::click()))
                .on_hover_text(CARD_HOVER)
                .clicked();
            if clicked {
                follow = Some(card.agent_id.clone());
            }
            if own_fork {
                ui.weak(&card.fork);
            }
            ui.horizontal(|ui| {
                ui.colored_label(hue, glyph).on_hover_text(phrase);
                ui.weak(format!("{} tokens", card.tokens));
            });
            if let Some(tail) = &card.tail {
                ui.colored_label(theme::SPECTRE, tail);
            }
        })
        .response
        .on_hover_text(CARD_HOVER);
    follow
}
