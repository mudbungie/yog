//! The inbox-composer's queue region (§11 inbox-composer, bl-929d): the
//! pending deposits above the typed draft, under the derived fold line.
//! Coverage-excluded glue — every decision is [`crate::composer`]'s (the row
//! projection, the fold-line height, the snap) and [`crate::tail`]'s (the
//! anchor and the measurement); this file only paints them.
//!
//! The region's top edge — the panel boundary's rule — **is** the fold line:
//! its height is [`SnapState::desired`], the settled content measurement
//! (pending rows + the draft's wrapped height) capped at half the pane, so an
//! item landing pushes the line up one row and typing wraps it up the same
//! way, with no stored height and no draggable boundary (§11 rule 3). Inside,
//! [`crate::tail::scroll`] anchors the queue on its bottom edge — the input is
//! the queue's last item — scrolls it past the cap, and during a snap seats
//! the shrunken content on the floor while the line eases down over it.
//!
//! The pending rows are here; the draft box that is the queue's last item —
//! the recall keys, the caret gate, the Shift+Enter contract — is
//! `inbox_queue/draft`.
//!
//! [SnapState::desired]: crate::composer::SnapState::desired

use crate::actions::{DraftKey, Drafts};
use crate::composer::{self, ComposerRam, QueueRow};
use crate::inboxview::InboxEntry;
use crate::jsonview::{GLYPH_COLLAPSED, GLYPH_EXPANDED, toggle_path};
use crate::theme;

mod draft;

/// What a pending row's fold arrow reveals (§11 discoverability).
const FOLD_HOVER: &str = "Fold this pending message open or shut. Either way it stays in the \
     inbox: everything below the line enters the next prompt when delivery drains it. \
     No key of its own: Tab reaches it, Space presses it.";

/// What the composer paints, bundled (owned, per the no-named-lifetimes rule)
/// to keep the region under the argument cap.
pub(super) struct QueueCtx {
    /// The draft this composer is composing (bl-a69a).
    pub key: DraftKey,
    /// The message target's agent id — `None` for a new conversation, which
    /// has no inbox and therefore a queue of zero items (the general path).
    pub agent_id: Option<String>,
    /// The target's pending deposits, from the snapshot (§5.1 #11) — never a
    /// frame-time read.
    pub pending: Vec<InboxEntry>,
    /// The box's greyed hint — the target line's twin spelling (bl-2f30).
    pub hint: String,
    /// What the operator has already said to this target, newest first — the
    /// derived recall history (bl-f908), never a stored list.
    pub prompts: Vec<String>,
    /// Half the pane: the fold line's ceiling (§11 rule 3).
    pub cap: f32,
}

/// Paint the queue region — pending rows oldest-first, then the input as the
/// queue's last item — at the derived fold-line height, and return the input
/// box's response for the caller's focus/Enter wiring.
///
/// `titles` is the frame's roster in the form a seat can hold (bl-1eb0): the
/// §3.3 ladder the pending headers' senders ride (bl-b6d0), a paint-time input
/// borrowed rather than copied into [`QueueCtx`], which carries what this
/// region *paints*.
pub(super) fn region(
    ui: &mut egui::Ui,
    ram: &mut ComposerRam,
    drafts: &mut Drafts,
    ctx: &QueueCtx,
    titles: &crate::nav::convs::Titles,
) -> egui::Response {
    // Split the RAM into its independent facts up front: the body closure
    // below holds the folds while the box holds the recall, and one `&mut`
    // over the whole bundle would make them borrow each other.
    let ComposerRam {
        folds,
        snap,
        recall,
        caret,
    } = ram;
    let now = ui.input(|i| i.time);
    snap.observe(&ctx.key, ctx.pending.len(), now);
    let desired = snap.desired(ctx.cap, now);
    let settled = snap.settled();
    let agent = ctx.agent_id.clone().unwrap_or_default();
    let queue = composer::rows(&agent, &ctx.pending, titles, folds);
    // The region's rect is allocated **exactly** at the derived height, and
    // the queue paints into a child bounded to it: an explicit allocation is
    // what lets the panel above shrink as well as grow (a `set_max_height`
    // scope ratchets — egui keeps the larger stale extent), so the fold line
    // rides the content both ways.
    let (rect, _space) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), desired),
        egui::Sense::hover(),
    );
    let mut region = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    let mut body = |ui: &mut egui::Ui| {
        for row in &queue {
            pending_row(ui, row, folds);
        }
        draft::input_box(ui, recall, caret, drafts, ctx)
    };
    // Content past the region (the cap engaged) scrolls, tail-anchored — the
    // §11 tail idiom from its one home. Content within it lays out directly,
    // seated on the region's bottom edge by an explicit top pad — the snap's
    // descending headroom, zero in the steady state (the region *is* its
    // content height, so nothing is ever hidden and there is nothing to
    // scroll).
    let (edit, painted) = if settled > desired + 0.5 {
        crate::tail::scroll(&mut region, true, body)
    } else {
        region.add_space((desired - settled).max(0.0));
        let top = region.cursor().top();
        let edit = body(&mut region);
        (edit, (region.min_rect().bottom() - top).max(0.0))
    };
    snap.settle(painted);
    if snap.active(now) {
        ui.ctx().request_repaint();
    }
    edit
}

/// One pending line (§11 transcript density idiom): the jsonview fold arrow,
/// the brazen `✉ from · at` header — the same signal as the `✉n` badge and the
/// Inbox tab, one derivation seen thrice — and the first line while folded, the
/// whole body below while open. Only the input has no arrow.
fn pending_row(ui: &mut egui::Ui, row: &QueueRow, folds: &mut std::collections::HashSet<String>) {
    // Faded while the deposit is only §7.2's pending echo, solid the moment the
    // derivation makes it a statement (§11, bl-915e). Opacity over the whole
    // row rather than a second hue per element: the row already wears the
    // colours it will keep, so brightening is this same row at full strength.
    ui.scope(|ui| {
        ui.set_opacity(theme::tone_solidity(row.tone));
        row_body(ui, row, folds);
    });
}

/// The row itself, inside its tone scope.
fn row_body(ui: &mut egui::Ui, row: &QueueRow, folds: &mut std::collections::HashSet<String>) {
    ui.horizontal(|ui| {
        // One line, always: overflow truncates at the pane's edge rather than
        // wrapping, which would grow the row (§11 rule 1 on the other axis).
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
        // The §11 role stripe (bl-3acb): the same role identity the row will
        // wear once delivered, from the one mapping the transcript reads.
        theme::role_stripe(ui, Some(row.role));
        let glyph = if row.expanded {
            GLYPH_EXPANDED
        } else {
            GLYPH_COLLAPSED
        };
        let hit = ui
            .add(
                egui::Label::new(egui::RichText::new(glyph).monospace())
                    .sense(egui::Sense::click()),
            )
            .on_hover_text(FOLD_HOVER);
        if hit.clicked() {
            toggle_path(folds, &row.key);
        }
        ui.colored_label(theme::BRAZEN, &row.header);
        if !row.expanded {
            ui.weak(&row.preview);
        }
    });
    if row.expanded {
        // The opened body **wraps** (bl-5410). The row above truncates because
        // it is a row; this is the whole message the fold exists to show, and
        // the panel's rule-1 `Truncate` would keep only its first line — so
        // opening the fold would paint one line where the shut row already
        // painted one, and the gesture would do nothing at all.
        ui.scope(|ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
            ui.label(&row.body);
        });
    }
}
