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
use crate::keymap::CenterTab;

use super::ShellState;

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
/// **A config-kind failure gets a remedy, not only a Dismiss** (bl-dd7f,
/// ruled at bl-9b52). Dismiss puts the sentence down; it does not fix the
/// file, and for a dispatch through a provider row brazen cannot resolve the
/// file is the whole answer. So the classification
/// ([`crate::config_edit::fault`]) pairs its sentence with the control that
/// opens the §9.1 raw-TOML editor — exactly the shape an auth-kind step
/// failure has had since bl-8e34, where the affordance is Login.
///
/// Additive, and in that order: brazen's and litany's own words stay verbatim
/// on top (INV-2 / §7.3), the remedy sits under them, and Dismiss stays where
/// it was — a *wrong* classification must never become the only thing on
/// screen (§8.3 rule 5's own clause).
pub(super) fn failure_banner(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    failure: &crate::opslog::SurfaceFailure,
) {
    ui.colored_label(crate::theme::ICHOR, format!("⚠ {}", failure.argv));
    if !failure.stderr_tail.is_empty() {
        ui.colored_label(crate::theme::ICHOR, &failure.stderr_tail);
    }
    if let Some(remedy) = crate::config_edit::fault::config_remedy(&failure.stderr_tail) {
        // The reason is painted, never hidden behind the button's hover
        // (bl-402f): a control whose reason needs a mouseover is the mystery
        // no-op. The hover says what pressing it does — one home, the tab's own
        // ([`CenterTab::focus_hover`]), so this seat grows no phrasing of its own.
        ui.weak(remedy);
        if ui
            .button(CenterTab::Config.label())
            .on_hover_text(CenterTab::Config.focus_hover())
            .clicked()
        {
            super::focus::center(state, CenterTab::Config);
        }
    }
    if ui
        .small_button(crate::opslog::operator::ACK_LABEL)
        .on_hover_text(crate::opslog::operator::ACK_HOVER)
        .clicked()
    {
        model.ack_failures();
    }
}
