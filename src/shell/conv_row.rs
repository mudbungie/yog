//! One conversation-list row's whole paint (§11 altitude 0): the depth indent
//! and its `↳` elbow, the prefix group, the title, the trailing right-pinned
//! metadata — attention flag, ball badge, **subagent field**, age — and the row
//! menu. Split from [`super::conv_list`] at §12's budget when the unfold landed
//! (bl-fa82); that module is the list frame, this is a row.
//!
//! **Every row fits the panel.** A row that overflows widens the panel's
//! `min_rect`, which egui stores as the panel width — so an overflowing row
//! ratchets the left column wider on every frame it is painted (bl-9669). The
//! trailing metadata is therefore pinned right and the title truncates into
//! what is left.
//!
//! **One row kind, at every depth.** A child of an unfolded row comes through
//! here identically to the root above it: the only difference the depth makes
//! is the indent + elbow ahead of the prefix, which moves the *whole* row and
//! so gives each depth its own title edge — never a conditional element inside
//! the prefix group, which is what would break the name column (bl-b9e3).

use crate::AppModel;
use crate::cli_outbound::Cli;
use crate::nav::convs::{ConvRow, age_label};
use crate::nav::menu::Seat;
use crate::theme;
use std::path::PathBuf;

use super::ShellState;
use super::menus::Target;

/// The per-frame facts every row derives its §11 seat from, bundled and owned
/// (no borrow rides a struct, per the no-named-lifetimes rule): the focused
/// workspace and which agent is selected.
///
/// **The agent snapshot is gone** (REMOTE §9.4, bl-1eb0). It rode here so each
/// row could run the two §8.2 predicates against it; both answers are now
/// fields of the [`ConvRow`] the row is painting, derived once where the row
/// is, so nothing on this surface holds the engine's tree.
pub(super) struct RowCtx {
    ws: PathBuf,
    /// The selected agent — the row that highlights. It is the agent itself and
    /// not its conversation root: since the unfold, a selected member *has* a
    /// row of its own whenever it is visible, and §11's visible-selection
    /// invariant (kept in [`super::conv_list`]) is what makes it always visible.
    selected: Option<String>,
    /// Whether this is one of yog's own named workspaces — the §3.6 scope
    /// every row's delete entry is gated on (one derivation per frame, not
    /// per row).
    named: bool,
}

impl RowCtx {
    /// Gather the frame's row facts once, for `ws` — the focused workspace.
    pub(super) fn of(model: &mut AppModel, ws: PathBuf) -> Self {
        // The §3.6 scope off the landed enumeration (bl-b4b5) — one fold per
        // frame, not per row, and the same answer the tab bar above is built
        // from rather than a second reading of the window's own workspace set.
        let named =
            crate::nav::tabs::is_named(&super::chrome::ws_rows(model), &model.snap.ws_name(&ws));
        Self {
            selected: model.focused_agent_id(),
            named,
            ws,
        }
    }
}

/// The pulsing hue for a row's live-activity class, or `None` at rest — and,
/// when it pulses, the one place this frame asks for another (§7.2: an idle
/// window schedules nothing). Called once per row so the repaint request and
/// the hue are the same decision.
fn flight_hue(ui: &egui::Ui, row: &ConvRow) -> Option<egui::Color32> {
    let (_, hue, _) = theme::flight_badge(row.flight?);
    let time = ui.ctx().input(|i| i.time);
    ui.ctx().request_repaint_after(theme::PULSE_REPAINT_DELAY);
    Some(theme::pulse(hue, time))
}

/// One conversation row: the depth elbow, the state badge, the live-activity
/// chip, first-line preview, and — pinned right — the ball badge, the subagent
/// field, age and the bare attention flag. Clicking focuses **this row's own
/// agent** (the §6 acknowledgement gesture, the same semantics an altitude-1
/// member row has); **right-clicking deliberately does not** — the §11
/// accelerator menu (Stop (+children), Flush) acts on this row while you keep
/// reading another, and focusing would silently acknowledge its attention (§6).
pub(super) fn conversation_row(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    row: &ConvRow,
    ctx: &RowCtx,
) {
    // Faded while the conversation is only §7.2's pending echo — a start yog
    // has fired whose driver has not written a branch — and solid the moment
    // the derivation carries it (§11, bl-915e). Scoped, because the whole list
    // paints into one `Ui`: the tone is this row's, not the rows below it. The
    // row dims as a whole, so brightening is this row at full strength rather
    // than a repaint into a second palette.
    ui.scope(|ui| {
        ui.set_opacity(theme::tone_solidity(row.tone));
        row_body(ui, model, state, lernie, row, ctx);
    });
}

/// The row itself, inside its tone scope.
fn row_body(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    row: &ConvRow,
    ctx: &RowCtx,
) {
    let ws = ctx.ws.as_path();
    ui.horizontal(|ui| {
        elbow(ui, row.depth);
        state_cell(ui, row);
        let pulsing = flight_hue(ui, row);
        let selected = ctx.selected.as_deref() == Some(row.root_id.as_str());
        // The operator's ask: **the name** pulses while anything is in flight
        // here — the title is what the eye is already reading down the column,
        // so it carries the beat and the chip beside it carries the class.
        let title = match pulsing {
            Some(hue) => egui::RichText::new(row.display_name()).color(hue),
            None => egui::RichText::new(row.display_name()),
        };
        // Trailing metadata pinned right, the title filling what is left and
        // truncating there (bl-9669): laid the other way round the title eats
        // the whole width and the metadata overflows — and an overflowing row
        // widens the panel, every frame, without bound.
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // The §6 attention flag, at the row's right edge (bl-b9e3): a
            // **badge, not a tally** — the number is the strip's one global
            // question, and a conditional mark in the prefix group would move
            // the title's left edge and cost the list its name column (§11).
            // Brazen is §11's hue for another agent's doing, which is what
            // arrived here while you weren't looking; the words ride hover
            // because this row is the dense repeating seat.
            if row.attention > 0 {
                ui.colored_label(theme::BRAZEN, "⚑")
                    .on_hover_text(theme::ROW_ATTENTION);
            }
            if let Some(ball) = &row.ball {
                // The conversation's start-flow ball (§3.3), coloured by the
                // §3.5 join.
                super::conv_ball::row_badge(ui, ball);
            }
            // §10's uncertainty, right of the title with every other per-row
            // mark (bl-8257): the liveness probe came back unknown, so the
            // state badge is a framing-only reading. One badge per row means
            // this needs no adjacency to say what it is about.
            if row.uncertain {
                ui.colored_label(theme::SIGIL, "?")
                    .on_hover_text(theme::STATE_UNCERTAIN);
            }
            // The standing alignment verdict (VISION §4.9, rung V6): a rendered
            // fact derived from the ops tail, never a stored flag. Absent unless
            // this workspace is armed *and* a check has landed — the unarmed
            // operator sees exactly what they saw before. It rides the trailing
            // group because it is an **independent** per-row mark (bl-8257): it
            // qualifies nothing beside it, so in the prefix it was pure column
            // drift.
            if let Some(check) = &row.verdict {
                let (glyph, hue, says) = theme::verdict_badge(check.verdict);
                ui.colored_label(hue, glyph).on_hover_text(format!(
                    "{says}\n{}\nchecked at {} by {}",
                    check.reason, check.sha, check.model
                ));
            }
            subagent_field(ui, model, state, ws, row);
            ui.weak(age_label(row.age_secs));
            // The §11 live-activity indicator: one chip for the class in flight
            // (§5.1 #28), pulsing in that class's hue, its words on hover — this
            // row is the dense repeating seat, so it hovers rather than states.
            // Painted last in the trailing group, which seats it immediately
            // right of the title: the chip names the class **the title is
            // pulsing in**, so the two agree and are adjacent, which is what the
            // prefix seat was for before it cost the name column (bl-8257).
            if let (Some(hue), Some(class)) = (pulsing, row.flight) {
                let (mark, _, says) = theme::flight_badge(class);
                ui.colored_label(hue, mark).on_hover_text(says);
            }
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                // §3.3: the row is titled by the display ladder, never the raw
                // id — and the id is still a fact (the branch name and the
                // on-disk key), so it rides the hover. That seat was the
                // altitude-1 member row's until bl-8905 retired it; every row
                // here is the subtree of an agent, so every row hovers the id of
                // the agent it is, at depth 0 exactly as at depth 3.
                let mut label = ui.selectable_label(selected, title).on_hover_text(format!(
                    "{} — open this conversation: the centre shows its transcript and \
                     the composer below aims at it. Right-click for Stop / Flush \
                     without leaving the one you are reading. ↑ / ↓ walks the list \
                     onto it (Ctrl+↑ / Ctrl+↓ from inside the box); → unfolds its \
                     subagents, ← folds them away and, on a child, pages up to its \
                     parent.",
                    row.root_id
                ));
                if row.name_display_only {
                    // The title is the legacy §3.3 rung — prose, not the
                    // lernie name fact — so say it is unaddressable (bl-8068)
                    // before an operator hands it to an agent as a target.
                    label = label.on_hover_text(theme::NAME_DISPLAY_ONLY);
                }
                if label.clicked() {
                    // Opening a conversation by pointer aims the composer at it
                    // and hands over the keyboard (§11 focus discipline).
                    super::focus::conversation(model, state, ws, &row.root_id);
                }
                let seat = Seat::ConversationRow {
                    stoppable: row.stoppable,
                    has_children: row.stop_children,
                    named: ctx.named,
                };
                let target = Target::Conversation {
                    ws: ctx.ws.clone(),
                    agent: row.root_id.clone(),
                };
                super::menus::attach(&label, seat, &target, model, state, lernie);
                // §11: the name is the title, the first payload line rides weak
                // beside it. Empty when the ladder already spent that line as
                // the title, so an unstamped row never says it twice.
                //
                // **And only where the row can pay for it** (§11 rules 1b/1d,
                // bl-0424). This is greedy prose laid AFTER the greedy title,
                // which is rule 1b's inversion in its label-on-label form: a
                // title longer than the row leaves nothing, and the preview is
                // then laid at zero width — a bare `…` that names nothing, and
                // ~20 pt of allocation past the panel's edge, which in a side
                // panel is next frame's panel width. The title is the row's
                // identity and outranks its preview, so the preview is the one
                // that goes.
                let subtitle = row.subtitle();
                if !subtitle.is_empty() && super::row::has_room(ui) {
                    ui.weak(subtitle);
                }
            });
        });
    });
}

mod cells;
use cells::{elbow, state_cell, subagent_field};
