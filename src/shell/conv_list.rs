//! The conversation list's **frame** (§11 altitude 0): the new-conversation
//! affordance, the flat/grouped organizing toggle, the scroll, and the
//! visible-row iteration both organizing views share. One row's own paint is
//! [`super::conv_row`], split off at §12's budget when the unfold landed
//! (bl-fa82). Like the rest of `shell/*` this is interaction glue over tested
//! `nav`/`AppModel` derivations.
//!
//! **The list is the descent forest, folded.** A row is the subtree rooted at
//! its agent, so what is painted is [`super::convs::visible`] — the wire's
//! answer for the whole forest (REMOTE §9.7) with this seat's expanded set
//! selecting out of it — and with that set empty it is the one-row-per-root list
//! this seat always was, byte for byte. Nothing here decides which rows are
//! visible, and nothing here derives one.

use crate::AppModel;
use crate::cli_outbound::Cli;
use crate::theme;

use super::ShellState;

/// The conversation list (§11): `new conversation`, an organizing-view toggle,
/// then the visible rows — flat by recency (default) or grouped by ball — in
/// the tested sort (last action of any kind, newest first; §11 as amended by
/// bl-cad5). The toggle and the flat/grouped choice are pure `AppModel`/`nav`
/// derivations; this only paints them.
pub(super) fn conversations(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
) {
    let Some(ws) = model.focused_workspace() else {
        ui.weak("no workspace yet — say the word");
        return;
    };
    if ui
        .button("new conversation")
        .on_hover_text("clear the target and type (n)")
        .clicked()
    {
        super::keys::new_conversation(model, state);
    }
    // The organizing-view toggle (§15 Z9): flat by recency | grouped by ball.
    ui.horizontal(|ui| {
        ui.selectable_value(&mut state.group_by_ball, false, "recent")
            .on_hover_text("list every conversation in one run, most recently active first (g)");
        ui.selectable_value(&mut state.group_by_ball, true, "by ball")
            .on_hover_text(
                "group the conversations under the ball each was started for; the ones \
                 started without a ball fall to the end (g)",
            );
    });
    // One ask for the whole forest, once per frame. A refusal is painted rather
    // than swallowed — the wire is how this seat reads now, so being told *no*
    // is the list's honest content (`shell::wire`'s four arms).
    let landed = super::convs::forest(model);
    if let Some(said) = &landed.refused {
        ui.colored_label(theme::ICHOR, said);
        return;
    }
    let forest = landed.value.unwrap_or_default();
    if forest.is_empty() {
        ui.weak("no conversations yet");
        return;
    }
    // §11's visible-selection invariant, kept in ONE place for every gesture
    // that can select — the list's own click, the altitude-1 member rows, a
    // §8.5 address, a start's adoption — rather than at each of their sites
    // (bl-fa82). A selection inside a folded subtree opens exactly the chain
    // above it; a root's chain is empty, so the ordinary case does nothing.
    // Idempotent because the two collapsing gestures carry the selection up to
    // the row they folded, so this can never re-open what was just shut.
    super::focus::reveal_selection(model, state);
    // The rows are folded once, before the loop that may mutate the expanded
    // set: what a frame paints is one answer selected once, not a list re-asked
    // between rows.
    let rows = crate::nav::convs::visible(&forest, &state.expanded);
    let ctx = super::conv_row::RowCtx::of(model, ws);
    egui::ScrollArea::vertical().show(ui, |ui| {
        if state.group_by_ball {
            // The grouping is a partition of the visible rows and asserts no
            // order of its own (§3.5, §15 Z9), so the unfold reaches both
            // organizing views without a second mechanism.
            for group in crate::nav::convs::group::group_by_ball(rows) {
                super::conv_ball::group_header(ui, &group);
                for row in &group.convs {
                    super::conv_row::conversation_row(ui, model, state, lernie, row, &ctx);
                }
            }
        } else {
            for row in &rows {
                super::conv_row::conversation_row(ui, model, state, lernie, row, &ctx);
            }
        }
    });
}
