//! **One row on the glass** — the chrome line, the fold toggle, the inline
//! preview, the expanded body, and the Raw view of a whole entry. Split from
//! [`super`] at §12's budget on the seam the projection is already cut along
//! ([`crate::transcript::rows::project`] ↔ its `build`): the list is what a
//! transcript becomes, this is what one row is made of.
//!
//! The fold toggle is the jsonview exception (§11 widget-split discipline): a
//! self-contained widget whose interaction is *intrinsic* owns its tested
//! click, exercised under a simulated-pointer render test, rather than
//! smearing the flip across the excluded shell.

use std::collections::HashSet;

use crate::jsonview::{GLYPH_COLLAPSED, GLYPH_EXPANDED, toggle_path};
use crate::theme;
use crate::transcript::{Entry, Fold, Row, Tone};

/// Leaf marker for a row with nothing to fold — keeps its prefix aligned
/// under the toggles above it (jsonview's alignment convention).
const NO_FOLD_MARK: &str = "·";

/// Verbatim backing bytes under a filename header — the Raw view of any entry.
/// Both seats are [`whole`]: Raw is the escape from a parse, and bytes cut at
/// the pane edge are not the bytes.
pub(super) fn render_raw(ui: &mut egui::Ui, entry: &Entry) {
    whole(ui, |ui| {
        ui.monospace(&entry.name);
        ui.monospace(String::from_utf8_lossy(&entry.raw).to_string());
    });
}

/// **Show `body` entire** — the seat that must not truncate, stated locally
/// (bl-7654).
///
/// `shell::row::bounded` puts §11 rule 1's `Truncate` at every bounded panel
/// root, the centre included, and a `Ui` inherits its parent's style — so an
/// expanded payload laid in the centre was a *one-line* galley ending in `…`,
/// 67 of a 400-character answer at 420x320. The fold's whole promise is that
/// turning the triangle shows the payload, and rule 1's own body names this
/// exemption: *"a seat that must not truncate (a wrapped prose block) can
/// still say so locally"*. Taken here, deliberately, in a scope: no ambient
/// default is set, so the rows around it keep truncating, and the seat depends
/// on no ambient state either — in a seat that declares none, egui's
/// horizontal default is `Extend` and the pane's clip rect slices the run
/// mid-glyph with no ellipsis at all.
fn whole<R>(ui: &mut egui::Ui, body: impl FnOnce(&mut egui::Ui) -> R) -> R {
    ui.scope(|ui| {
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
        body(ui)
    })
    .inner
}

/// One chrome line of a row — stripe, toggle, label, inline preview — with §11
/// rule 1 stated at the row rather than inherited: chrome is one line and
/// elides rather than running off the pane, wherever the transcript is seated.
fn chrome(ui: &mut egui::Ui, body: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
        body(ui);
    });
}

/// What the prefix stands for, hovered (bl-2335) — the §11 glyph doctrine
/// applied to a name: the glance gets the agent, the hover gets the config fact
/// behind it. A label that says everything it has to say hovers nothing.
fn hover(row: &Row, label: egui::Response) {
    if !row.hover.is_empty() {
        label.on_hover_text(&row.hover);
    }
}

/// A row: the speaker's own line where somebody is speaking, then the payload
/// line — toggle, machinery label, and the payload inline as the preview while
/// contracted — with the full body below while expanded. A row whose payload
/// already fits the line has an empty body and shows it inline always, so
/// nothing is hidden behind a fold that would reveal nothing. A turn's
/// aggregate row has no payload of its own: what its fold reveals is the step
/// rows the projection puts after it.
///
/// **The speaker is not the output** (bl-f3fc, operator ruling). A speaking row
/// wore its label in the body's own ink on the body's own line, so the eye had
/// nothing to separate who spoke from what they said. The label now takes the
/// line above the payload and the role's hue — `theme::role_badge`, the one
/// mapping the stripe and the pending queue already read, so no hue is minted.
/// Machinery rows (`role: None` — thinking, tool calls, results, raw bytes, the
/// turn aggregate) are one line still: nobody is speaking, so there is no
/// speaker to set apart, and their tone-painted prefix keeps its seat.
///
/// **One fact decides three things** (bl-7654). `row.body.is_empty()` — the
/// projection's own spelling of *"is there anything behind this?"* — says
/// whether the inline seat is the payload entire or a stand-in for a body the
/// `▶` reveals, and the seat then wraps-or-truncates, paints solid-or-faded,
/// and wears a toggle-or-the-mark on that one answer. The operator's ruling is
/// that the three never disagree: **anything hidden is hidden behind a
/// triangle**, so a run that ends in `…` always has one beside it, and a run
/// with no triangle beside it is whole.
pub(super) fn render_row(ui: &mut egui::Ui, row: &Row, folds: &mut HashSet<String>) {
    let inline = row.fold == Fold::Steps || row.body.is_empty() || !row.expanded;
    // Is the inline seat standing in for something? `Fold::Steps` is not
    // abridged prose — an aggregate has no payload of its own and its preview
    // is empty — so this is exactly "a body exists and is folded away".
    let abridged = inline && !row.body.is_empty();
    if let Some(role) = row.role {
        chrome(ui, |ui| {
            // The §11 role stripe (bl-3acb): who this row speaks for, at the
            // left edge, from the one mapping the label now reads too.
            theme::role_stripe(ui, Some(role));
            let label = ui.colored_label(theme::role_badge(role).0, &row.prefix);
            hover(row, label);
        });
    }
    chrome(ui, |ui| {
        // Every line allocates the stripe seat, so the toggles stay aligned
        // down the whole chat; the hue is spent once, on the line that names
        // the speaker.
        theme::role_stripe(ui, None);
        // The toggle rides the payload line — the line it folds — so a run
        // marked as cut always has its triangle on its own laid band.
        toggle(ui, row, folds);
        if row.role.is_none() {
            let label = paint(ui, row.tone, &row.prefix);
            hover(row, label);
        }
        if inline && !row.preview.is_empty() {
            preview(ui, &row.preview, abridged);
        }
    });
    if !inline {
        whole(ui, |ui| ui.label(&row.body));
    }
}

/// The inline payload seat. `abridged` is the row's one fact: a body is folded
/// away behind the `▶` beside this run.
///
/// **The fade means "there is more behind this", so it may only appear where
/// there is** (bl-7654). `ui.weak` was unconditional, which painted a payload
/// shown *whole* exactly as faded as a preview standing in for hidden text —
/// the surface saying the opposite of the truth precisely where it told the
/// whole truth. Inverted: complete content reads complete.
///
/// **Solidity, not a tone.** [`Tone`] is the projection's statement about what
/// kind of thing a row *is*, derived from the entry's bytes; abridgement is a
/// fact about this seat in this frame, and it flips when the operator turns
/// the triangle — so a row would have to wear two tones for one payload, which
/// is a category error. `theme::tone_solidity` is the palette's answer for
/// exactly this ("a statement that is not yet a statement"): one number, the
/// same 0.55 the §7.2 pending echo and the inbox queue spend, dimming the seat
/// rather than repainting it into a parallel palette.
fn preview(ui: &mut egui::Ui, text: &str, abridged: bool) {
    if !abridged {
        whole(ui, |ui| ui.label(text));
        return;
    }
    ui.scope(|ui| {
        ui.set_opacity(theme::tone_solidity(Tone::Weak));
        ui.label(text);
    });
}

/// The disclosure toggle, or the alignment mark when there is nothing to fold.
/// An aggregate row always folds — its steps are what it opens onto.
fn toggle(ui: &mut egui::Ui, row: &Row, folds: &mut HashSet<String>) {
    if row.fold == Fold::Payload && row.body.is_empty() {
        ui.monospace(NO_FOLD_MARK);
        return;
    }
    let glyph = if row.expanded {
        GLYPH_EXPANDED
    } else {
        GLYPH_COLLAPSED
    };
    // §11 discoverability: the glyph passes on convention, the hover says what
    // this particular fold holds.
    let hit = ui
        .add(egui::Label::new(egui::RichText::new(glyph).monospace()).sense(egui::Sense::click()))
        .on_hover_text(
            "Fold this row open or shut — open shows its whole text, shut shows the \
             first line. The two checkboxes above set which kinds open by default. No key \
             of its own: Tab reaches it, Space presses it.",
        );
    if hit.clicked() {
        toggle_path(folds, &row.key);
    }
}

/// Paint `text` in the row's tone, handing back the label so the caller can
/// hang a hover on it. An in-flight tool call pulses and pulls a near-term
/// repaint, exactly as the git_tree tool indicator does.
fn paint(ui: &mut egui::Ui, tone: Tone, text: &str) -> egui::Response {
    match tone {
        Tone::Plain => ui.label(text),
        Tone::Weak => ui.weak(text),
        Tone::Good => ui.colored_label(theme::HYDRA, text),
        Tone::Bad => ui.colored_label(theme::ICHOR, text),
        Tone::Live => ui.colored_label(theme::SPECTRE, text),
        Tone::InFlight => {
            let time = ui.ctx().input(|i| i.time);
            ui.ctx().request_repaint_after(theme::PULSE_REPAINT_DELAY);
            // Spectral, not the §11 tools-class hydra: inside a transcript the
            // contrast that matters is running-vs-finished, and hydra is
            // already the finished-ok result row two lines down.
            ui.colored_label(theme::pulse(theme::SPECTRE, time), text)
        }
    }
}
