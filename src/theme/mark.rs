//! The mark on screen (DESIGN §11) — two seats, one painter.
//!
//! [`wordmark`] is the mark **at rest** beside the name: identity, in the
//! empty-workspace placeholder. [`live_mark`] is the same mark as the open
//! conversation's **telemetry**, on its §11 altitude-1 headline row — one
//! circle per agent, hue = what that agent is doing right now (§5.1 #28b). The
//! eye is the conversation's root; the nine node circles are its subagents in
//! §2.3 descent order.
//!
//! **The name rides the seat, not the mark.** The wordmark's "yog" is chrome
//! branding a window; inside a conversation it says nothing, so the live seat
//! is the bare mark and its hover. One mark on screen at a time, and it is
//! always the live one — the same glyph inert in one corner and live in another
//! would be one picture meaning two things.
//!
//! **Rest is not a case.** An idle agent, and a seat with no agent in it, are
//! both hydra green — which is the mark yog has always painted. So an operator
//! with nothing running is looking at the logo, and the logo is the empty
//! reading of the telemetry rather than a separate picture of it.
//!
//! **The hue never carries the fact alone** (§11 glyph doctrine). The mark
//! hovers a worded roster: every seat named as every other seat names an agent
//! (§3.3), what it is doing in `doing_badge`'s words, and — when a conversation
//! has more subagents than the mark has circles — how many are not shown. A cap
//! that says nothing is a cap that lies about coverage.
//!
//! Nothing here animates, and nothing here asks for a repaint: the mark's hues
//! change exactly when the snapshot does, and a new snapshot already brings a
//! frame with it (§7.2). An idle window still paints once and sleeps.

use super::{GATE, TAGLINE, doing_badge, icon};
use crate::nav::convs::Seat;

/// The mark's on-screen edge in points — the operator's "medium": large
/// enough that the tangent-circles mark reads as itself, small enough not to
/// inflate the top bar it leads.
const MARK_PT: f32 = 28.0;

/// What the live mark says when nothing is open — the roster's honest empty.
const NOTHING_OPEN: &str = "The mark carries what your agents are doing: the eye is the conversation you \
     have open, the outer circles its subagents. Nothing is open, so it rests.";

/// The wordmark: the mark at rest, then "yog" in gate violet. The
/// empty-workspace placeholder's seat (§11 altitude 1) — identity, with no
/// agent's business on it.
pub fn wordmark(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        draw(ui, &icon::Tints::rest());
        ui.add_space(3.0);
        ui.heading(egui::RichText::new("yog").color(GATE).strong());
    });
}

/// The **live** mark (§11 altitude 1, the conversation's headline row): the same
/// mark with one circle per agent, and the roster on hover. `seats` is
/// [`crate::nav::convs::seats`]'s list — the eye first, then subagents in
/// descent order; empty means no conversation is open, and the mark rests. In
/// practice the seat is only reached with one open, since the pane returns
/// before its header when nothing is selected.
pub fn live_mark(ui: &mut egui::Ui, seats: &[Seat]) {
    draw(ui, &tints(seats)).on_hover_text(roster(seats));
}

/// Allocate a [`MARK_PT`] square and paint the mark into it.
fn draw(ui: &mut egui::Ui, tints: &icon::Tints) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(MARK_PT, MARK_PT), egui::Sense::hover());
    icon::paint(ui.painter(), rect, tints);
    response
}

/// The seats' hues, laid onto the mark's circles: the first seat is the eye,
/// the rest fill the node circles in order. Seats past the mark's circle count
/// are dropped **here and stated in the roster** — the geometry is the cap, so
/// this is where it can be applied and nowhere earlier.
fn tints(seats: &[Seat]) -> icon::Tints {
    let hue = |seat: &Seat| doing_badge(seat.doing).0;
    let mut out = icon::Tints::rest();
    let mut riders = seats.iter();
    if let Some(eye) = riders.next() {
        out.eye = hue(eye);
    }
    for (node, seat) in out.nodes.iter_mut().zip(riders) {
        *node = hue(seat);
    }
    out
}

/// What the mark says in words: the tagline, then one line per seat — the
/// agent's name and what it is doing — and the overflow note when a
/// conversation has more subagents than the mark has circles.
fn roster(seats: &[Seat]) -> String {
    let Some((eye, children)) = seats.split_first() else {
        return NOTHING_OPEN.to_owned();
    };
    let line = |seat: &Seat| format!("{} — {}", seat.name, doing_badge(seat.doing).1);
    let seen = icon::NODE_SEATS;
    let mut said = vec![format!("yog — {TAGLINE}"), String::new(), line(eye)];
    said.extend(
        children
            .iter()
            .take(seen)
            .map(|s| format!("  ↳ {}", line(s))),
    );
    if children.len() > seen {
        said.push(String::new());
        said.push(format!(
            "the mark has {seen} circles for subagents; {seen} of {} are shown",
            children.len()
        ));
    }
    said.join("\n")
}

#[cfg(test)]
mod tests;
