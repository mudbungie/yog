//! The §7.3 **last-failure banner** — the one widget every surface that can
//! fail paints, and the one dismissal that quiets them all.
//!
//! Its own file since bl-e160: [`super`] is the §11 *window assembly* (which
//! panel sits where, in what order), and a banner is a widget a surface paints
//! inside one of them. Two concerns, and the cap made the seam explicit.
//!
//! Excluded shell paint — the view-model it renders (`AppModel::last_failure`,
//! `opslog::operator`) is covered where it is derived.

use crate::AppModel;

/// Paint a surface's last-failure banner (§7.3): the attempted argv and its
/// stderr tail in ichor red (`theme::ICHOR` — never a restated RGB). The
/// originating surface derives the [`SurfaceFailure`] **per frame** from
/// [`AppModel::last_failure`] — never a copy cached at dispatch, which froze the
/// banner at a moment microseconds after the spawn, before a detached driver
/// could die and write its §8.1 sink (bl-4895). The durable fact is the
/// expandable ops-pane row. Excluded shell paint — the view-model it renders is
/// covered.
///
/// **It carries its own dismissal** (bl-c417): the banner used to clear only
/// when a newer *successful* op of the same origin landed, so an operator who
/// read the error and chose not to retry had no way to put it down. Dismiss is
/// the ack gesture ([`AppModel::ack_failures`]) — a §4.2 line, not a widget
/// flag — so it quiets every surface's banner and the §11 chip at once, and a
/// new failure raises them again. Hover explains that (bl-68ac); the words come
/// from `opslog::operator`, the one home both seats read.
pub(super) fn failure_banner(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    failure: &crate::opslog::SurfaceFailure,
) {
    ui.colored_label(crate::theme::ICHOR, format!("⚠ {}", failure.argv));
    if !failure.stderr_tail.is_empty() {
        ui.colored_label(crate::theme::ICHOR, &failure.stderr_tail);
    }
    if ui
        .small_button(crate::opslog::operator::ACK_LABEL)
        .on_hover_text(crate::opslog::operator::ACK_HOVER)
        .clicked()
    {
        model.ack_failures();
    }
}
