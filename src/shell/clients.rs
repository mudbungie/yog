//! The workspace's **registered clients** section (REMOTE §5, bl-4e08) — and,
//! since bl-ae05, **the first surface yog paints from a wire reply**.
//!
//! REMOTE §1.2 rules the window a client of the boundary over the same wire a
//! remote seat uses, and the operator ruling of 2026-08-14 chose the front
//! door: this section's rows are a `Reply::Clients` that crossed loopback mTLS,
//! was scoped against the window's own registrations, and was decoded by
//! `reply::decode`. Nothing here reads disk, joins presence or holds a memo any
//! more — the fold that built these rows is the engine's, and the frame's whole
//! part is to declare the question and paint what landed.
//!
//! **The frame never waits.** [`AppModel::wire_ask`](crate::AppModel::wire_ask)
//! returns whatever the [`asker`](crate::wire::asker) has landed for this
//! question — `None` until the first answer arrives, then the newest one — so a
//! slow or dead engine costs this section its content and the window nothing.
//! There is no memo because there is nothing to memoize: an answer *is* the
//! cached fold, refreshed at human cadence rather than per derivation, and
//! presence rides it (REMOTE §5's *"the seat sees the flap"*) with no key to
//! invalidate.
//!
//! The section is deliberately a **read**. There is no gesture on it and none
//! is coming: a registration is the operator's own file act on the server
//! (REMOTE §4.1) or the engine's own act for its window, and an advertisement
//! is the tool host's, arriving over its own connection.

use crate::AppModel;
use crate::boundary::reply::Reply;
use crate::boundary::{Gesture, Query, codec};
use crate::registry::roster::ClientRow;
use crate::theme;

/// The collapsible clients section (§11), painted beside the balls one and
/// folded by the same persisted §4.1 collapse override. Absent entirely while
/// the wire has answered nothing and when the answer is empty — an empty
/// section is a question ("is this broken?"), and the one thing worth saying is
/// a refusal, which is said.
pub fn section(ui: &mut egui::Ui, model: &mut AppModel) {
    let Some(ws) = model.focused_workspace() else {
        return;
    };
    let question = codec::encode(&Gesture::Ask(Query::Clients {
        workspace: crate::naming::leaf(&ws),
    }));
    let (rows, refused) = match model.wire_ask(&question) {
        Some(Ok(Reply::Clients(rows))) => (rows, None),
        Some(Err(said)) => (Vec::new(), Some(said)),
        // An answer of another kind is a codec that has drifted from the
        // query it answers, which is a defect rather than a state — and one
        // the round-trip tests are the witness for. Nothing to paint.
        Some(Ok(_)) | None => (Vec::new(), None),
    };
    if rows.is_empty() && refused.is_none() {
        return;
    }
    let collapsed = model.is_collapsed("clients");
    let arrow = if collapsed { "▶" } else { "▼" };
    if ui
        .selectable_label(false, format!("{arrow} clients"))
        .on_hover_text(
            "show or hide the clients section — the machines registered in this workspace, \
             whether each is connected right now, and the tools each one offers. Read over the \
             wire, like every seat's: the same rows are `/clients`.",
        )
        .clicked()
    {
        model.set_collapsed("clients", !collapsed);
    }
    if collapsed {
        return;
    }
    ui.indent("clients", |ui| {
        // A refusal is painted, not swallowed: the wire is how this window
        // reads now, so what it was told is the honest content of the section.
        if let Some(said) = &refused {
            ui.colored_label(theme::ICHOR, said);
        }
        for row in &rows {
            client_row(ui, row);
        }
    });
}

/// One client: a presence dot, its identity, and its advertised tools beneath.
/// Present and absent are both rendered — an absent registered client is a fact
/// about the workspace, and hiding it would make a disconnection look like a
/// revocation.
fn client_row(ui: &mut egui::Ui, row: &ClientRow) {
    let (glyph, hue, says) = if row.present {
        ("●", theme::HYDRA, "connected right now")
    } else {
        ("○", theme::ASH, "registered, not connected")
    };
    ui.horizontal(|ui| {
        ui.colored_label(hue, glyph).on_hover_text(says);
        ui.label(&row.client);
    })
    .response
    .on_hover_text(says);
    ui.indent(&row.client, |ui| {
        // Said outright rather than left blank: "offers nothing" and "has never
        // connected as a tool host" read the same to an operator, and the
        // sentence is the one that is true of both.
        if row.tools.is_empty() {
            ui.weak("no tools advertised");
        }
        for tool in &row.tools {
            ui.weak(&tool.name).on_hover_text(&tool.description);
        }
    });
}
