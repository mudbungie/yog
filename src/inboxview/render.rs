//! egui widget: the §11 Altitude-2 Inbox tab — the agent's undelivered
//! deposits (`✉n`), explained.
//!
//! A pure function of the [`InboxEntry`] listing (§2.11): each deposit's
//! framing (sender), its asserted facts (`from` / `deposited_at`, plus
//! `epitaph` / `terminal_ref` on a result message), and its body verbatim —
//! or, under the §11 Raw toggle, the deposit file's own bytes. No click, so
//! the whole tab is a headless shape-walk-tested pure render; the Flush =
//! `lernie scan` action is this tab's own Scan button, wired beside it in the
//! shell's `inspector::tabs_and_content` per-tab controls.

use super::{Deposit, InboxEntry};

/// Render the inbox listing, oldest-first. An empty inbox shows a placeholder
/// (the common quiescent case, not an error). `raw` is the §11 Raw toggle:
/// each deposit file's name and its bytes exactly as they sit on disk —
/// envelope included — instead of the parsed view that drops it.
pub fn render(ui: &mut egui::Ui, entries: &[InboxEntry], raw: bool) {
    // §11 tail idiom: the listing is oldest-first, so the newest deposit is the
    // bottom row and the view sits on it — a lone deposit at the bottom edge,
    // a backlog scrolled to it.
    crate::tail::scroll(ui, true, |ui| {
        if entries.is_empty() {
            ui.label("(no deposits)");
            return;
        }
        for entry in entries {
            if raw {
                render_raw(ui, entry);
            } else {
                render_deposit(ui, &entry.deposit);
            }
            ui.separator();
        }
    });
}

/// Verbatim backing bytes under a filename header — the transcript tab's Raw
/// idiom, one wording of it per tab and no second spelling of the layout.
fn render_raw(ui: &mut egui::Ui, entry: &InboxEntry) {
    ui.monospace(&entry.name);
    ui.monospace(String::from_utf8_lossy(&entry.raw).to_string());
}

/// One deposit: a `from · deposited_at` header, the result-message epitaph +
/// terminal ref when present (§2.6), then the body verbatim. Absent fields
/// read as `?` rather than vanishing, so a hand-edited deposit stays legible.
fn render_deposit(ui: &mut egui::Ui, deposit: &Deposit) {
    // Brazen — the same hue as the tree's `✉n` badge, so "pending mail"
    // reads as one signal across altitudes; the wording is the shared
    // [`super::header_line`], one home for all three seats of §5.1 #11.
    ui.label(
        egui::RichText::new(super::header_line(deposit))
            .color(crate::theme::BRAZEN)
            .strong(),
    );
    if let Some(epitaph) = &deposit.epitaph {
        ui.weak(format!("epitaph: {}", epitaph.label()));
    }
    if let Some(terminal) = &deposit.terminal_ref {
        ui.weak(format!("terminal: {terminal}"));
    }
    ui.label(&deposit.body);
}
