//! The conversation row's **cells** (§11 altitude 0): the depth elbow, the
//! state badge that is the row's whole prefix, and the subagent field. Split
//! from [`super`] at §12's budget on the seam the module doc already draws —
//! that file assembles a row and wires its gestures, each of these paints one
//! seat and carries the §11 rule that decides its shape.

use crate::AppModel;
use crate::jsonview::{GLYPH_COLLAPSED, GLYPH_EXPANDED};
use crate::nav::convs::ConvRow;
use crate::theme;

use super::super::ShellState;

/// The row's **whole prefix** (§11, bl-8257): the state badge, and nothing
/// else, on every row without a condition.
///
/// This completes bl-b9e3's rule — the title's left edge is wherever the prefix
/// ends, so every *conditional* element before it moves the name column on the
/// rows that have it — which three elements still broke. All three now ride
/// right, the §10 `?` included.
///
/// The `?` was expected to keep its seat, as a *suffix* qualifying the badge
/// rather than a mark of its own, paid for with a monospace slot allocated on
/// every row. That was built and **measured out**: the slot costs one character
/// of every title to avoid movement on a condition §10 makes rare, and with it
/// in, `acceptance::unfold`'s ambiguity guard reddened — three sibling rows
/// painted the same elided head. §11 carries the full reasoning; what is left
/// here is why this function is one line long.
pub(super) fn state_cell(ui: &mut egui::Ui, row: &ConvRow) {
    let (glyph, color, phrase) = theme::state_badge(row.state);
    // §11 glyph doctrine: the badge is the glance layer over a *stated*
    // state — the row is width-bound (bl-9669), so the words ride hover.
    ui.colored_label(color, glyph).on_hover_text(phrase);
}

/// The §11 **reply elbow** and the indent it rides on — the operator's "little
/// chat-reply line" (bl-fa82). One galley, so the indent and the glyph move
/// together and a depth's title edge is one number rather than two widgets'
/// sum. Nothing at depth 0: a root row is what this list always painted.
pub(super) fn elbow(ui: &mut egui::Ui, depth: usize) {
    if depth > 0 {
        ui.monospace(format!(
            "{}{}",
            "  ".repeat(depth.saturating_sub(1)),
            theme::ELBOW
        ));
    }
}

/// The §11 **subagent field** (bl-fa82), replacing the bare `(N)` member count:
/// the crate's disclosure arrow — `▶` shut, `▼` open — and the two numbers the
/// operator asked for, **direct** then **total**. Painted only where there is a
/// descent to open, exactly as the count was.
///
/// It is an interactive control, so its hover says what each number means and
/// how to press it without the mouse (§11 discoverability). The click flips one
/// id in the shell's expanded set through the crate's one disclosure toggle;
/// the field itself holds no state and asks no question the list has not
/// already answered.
pub(super) fn subagent_field(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    ws: &std::path::Path,
    row: &ConvRow,
) {
    if !row.has_children() {
        return;
    }
    let open = state.expanded.contains(&row.root_id);
    let glyph = if open {
        GLYPH_EXPANDED
    } else {
        GLYPH_COLLAPSED
    };
    let (direct, total) = (row.direct, row.total());
    let hit = ui
        .add(
            egui::Label::new(
                egui::RichText::new(format!("{glyph} {direct}/{total}"))
                    .monospace()
                    .weak(),
            )
            .sense(egui::Sense::click()),
        )
        .on_hover_text(format!(
            "subagents — {direct} dispatched by this agent itself, {total} under it \
             altogether at any depth. Click to {} its children as rows in this list; \
             → unfolds the selected row, ← folds it shut, and ← on a child pages the \
             selection up to its parent.",
            if open { "fold away" } else { "unfold" }
        ));
    if hit.clicked() {
        super::super::focus::toggle_row(model, state, ws, &row.root_id);
    }
}
