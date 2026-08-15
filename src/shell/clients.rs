//! The workspace's **registered clients** section (REMOTE §5, bl-4e08).
//! Coverage-excluded glue beside the balls section it sits under: the rows come
//! from `registry::roster`, the one derivation `Query::Clients` answers, so this
//! file only chooses a glyph and lays the widgets.
//!
//! REMOTE §5: *"The workspace surface renders its registered clients — present
//! or absent — and each one's advertised tools, live: the seat sees the flap;
//! the model's prefix does not."* Both halves are here. **Live** is what the
//! per-derivation memo's key buys: presence moves without a derivation, so the
//! live set is *in* the key and a flap rebuilds exactly the rows it changed
//! (§7.2 — the frame still reads no disk per frame).
//!
//! The section is deliberately a **read**. There is no gesture on it and none
//! is coming: a registration is the operator's own file act on the server
//! (REMOTE §4.1), and an advertisement is the tool host's, arriving over its
//! own connection. Nothing a click here could do would be yog's to do.

use crate::AppModel;
use crate::registry::roster::ClientRow;
use crate::theme;

use super::ShellState;

/// The collapsible clients section (§11), painted beside the balls one and
/// folded by the same persisted §4.1 collapse override. Absent entirely when
/// no client is registered here — an empty section is a question ("is this
/// broken?") on the overwhelming majority of boxes, which have no wire at all.
pub fn section(ui: &mut egui::Ui, model: &mut AppModel, state: &mut ShellState) {
    let Some(ws) = model.focused_workspace().map(std::path::Path::to_path_buf) else {
        return;
    };
    let name = crate::naming::leaf(&ws);
    let rows = state
        .clients
        .read(model.derivation(), (ws, model.live_clients()), &mut || {
            model.clients(&name)
        })
        .clone();
    if rows.is_empty() {
        return;
    }
    let collapsed = model.is_collapsed("clients");
    let arrow = if collapsed { "▶" } else { "▼" };
    if ui
        .selectable_label(false, format!("{arrow} clients"))
        .on_hover_text(
            "show or hide the clients section — the machines registered in this workspace, \
             whether each is connected right now, and the tools each one offers. The same \
             rows are `/clients`.",
        )
        .clicked()
    {
        model.set_collapsed("clients", !collapsed);
    }
    if collapsed {
        return;
    }
    ui.indent("clients", |ui| {
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
