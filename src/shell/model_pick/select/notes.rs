//! The sentences above the pair (DESIGN §9.4), and the row they are about —
//! split off [`super`] at §12's cap along the seam it already had: everything
//! below chooses, this one only *says*, and what it says is a standing property
//! of the row the dropdown landed on rather than of any gesture.
//!
//! Three sentences, and they are three different kinds of thing:
//!
//! - the **strand** (bl-dd7f) — the row the conversation actually dispatched
//!   through, which brazen no longer has. Reported because the steering that
//!   replaced it is otherwise silent, and the operator reads the picker as a
//!   report of what ran.
//! - the tool **refusal** (bl-3d22) — a role already sitting on a row whose
//!   protocol declines tools, which `plan` would refuse to write; the operator is
//!   looking at the one control that repairs it.
//! - the context **caveat** (bl-671d) — a dialect that leaves the context size to
//!   the server, which nothing refuses: yog cannot see what the server chose, so
//!   the row stays selectable and the line carries the fact with the remedy on
//!   its hover.
//!
//! Every one of them is the pure half's own words
//! ([`Scoped::strand_note`](crate::model_pick::Scoped::strand_note),
//! [`ProviderRow::tools_blocked`], [`ProviderRow::context_caveat`],
//! [`CONTEXT_REMEDY`](crate::config_edit::brazen::CONTEXT_REMEDY)), never a
//! second phrasing beside the refusal or the plan.

use super::PickerState;
use crate::config_edit::brazen::{CONTEXT_REMEDY, ProviderRow, row_names};
use crate::model_pick::{ModelRow, default_row};

/// Which provider row the pair is scoped to, having said everything standing
/// about it. The resolution and the sentences are one act: the strand exists only
/// as a byproduct of the steering, and the other two are about the row the
/// steering landed on.
///
/// A role stranded on a row brazen dropped is still steered off it, but the row
/// it was stranded on is **named**. Once the operator has picked something
/// themselves there is no strand left to report — the selection is their own
/// answer to it — and `default_row` still steers over the WHOLE table, so a
/// tool-less or caveated row is reported rather than mistaken for one brazen
/// dropped.
pub(super) fn scoped_with_notes(
    ui: &mut egui::Ui,
    picker: &PickerState,
    row: &ModelRow,
    rows: &[ProviderRow],
) -> String {
    let scoped = default_row(&row.provider, &row_names(rows));
    let strand = picker
        .provider
        .is_none()
        .then(|| scoped.strand_note())
        .flatten();
    let provider = picker.provider.clone().unwrap_or(scoped.row);
    if let Some(note) = strand {
        ui.colored_label(crate::theme::ICHOR, note);
    }
    let selected = rows.iter().find(|r| r.name == provider);
    if let Some(why) = selected.and_then(ProviderRow::tools_blocked) {
        ui.colored_label(crate::theme::ICHOR, format!("⚠ {provider} {why}"));
    }
    if let Some(caveat) = selected.and_then(ProviderRow::context_caveat) {
        ui.colored_label(crate::theme::ICHOR, format!("⚠ {provider} {caveat}"))
            .on_hover_text(CONTEXT_REMEDY);
    }
    provider
}
