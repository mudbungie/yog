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
use crate::git_tree::Agent;

/// What an empty inbox **is** (QUALITY H2: "an empty region says what it is and
/// names the paved path in full"). The old wording was a parenthesised
/// `(no deposits)` and nothing else — true, and no use to an operator who has
/// never seen mail arrive.
pub(crate) const NO_DEPOSITS: &str = "no deposits — nothing is waiting for this agent to read.";

/// …and how one ever gets here: the two ways a deposit is written (§2.11 — the
/// operator's own message to a working agent, and a subagent's result message)
/// and the verb on this tab that delivers one already queued.
pub(crate) const HOW_MAIL_ARRIVES: &str = "mail arrives when you message this agent while it works, or a subagent \
     reports back; Scan delivers any still queued.";

/// Render the inbox listing, oldest-first. An empty inbox names itself and the
/// paved path ([`empty`]) — the common quiescent case, not an error. `raw` is the §11 Raw toggle:
/// each deposit file's name and its bytes exactly as they sit on disk —
/// envelope included — instead of the parsed view that drops it.
///
/// `agents` is the frame's roster, the §3.3 ladder's input for each deposit's
/// sender ([`super::header_line`], bl-b6d0) — borrowed at paint time rather
/// than carried on the view-model, so nothing here holds a second copy of a
/// snapshot fact.
pub fn render(ui: &mut egui::Ui, entries: &[InboxEntry], agents: &[Agent], raw: bool) {
    // §11 tail idiom: the listing is oldest-first, so the newest deposit is the
    // bottom row and the view sits on it — a lone deposit at the bottom edge,
    // a backlog scrolled to it. **An empty inbox is not asking that question**
    // (bl-71fc): there is no newest content, so the anchor's top pad had been
    // pushing a one-line absence ~450 pt down an otherwise blank pane. The
    // anchor is the answer to "is my bottom row my newest content?", and with
    // no rows the honest answer is no.
    crate::tail::scroll(ui, !entries.is_empty(), |ui| {
        if entries.is_empty() {
            empty(ui);
            return;
        }
        for entry in entries {
            if raw {
                render_raw(ui, entry);
            } else {
                render_deposit(ui, &entry.deposit, agents);
            }
            ui.separator();
        }
    });
}

/// The empty inbox, top-anchored: what the region is, then the paved path.
fn empty(ui: &mut egui::Ui) {
    ui.label(NO_DEPOSITS);
    ui.weak(HOW_MAIL_ARRIVES);
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
fn render_deposit(ui: &mut egui::Ui, deposit: &Deposit, agents: &[Agent]) {
    // Brazen — the same hue as the tree's `✉n` badge, so "pending mail"
    // reads as one signal across altitudes; the wording is the shared
    // [`super::header_line`], one home for all three seats of §5.1 #11.
    ui.label(
        egui::RichText::new(super::header_line(deposit, agents))
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
