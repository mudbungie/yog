//! One **board row** and the planner entry points it is matched against — the
//! §11 balls fold's per-row half, split from [`super`] at §12's line budget on
//! the seam between *laying out the section* and *painting one ball*.
//!
//! Coverage-excluded glue like the rest of `shell/*`. The row's content is
//! `crate::board::BoardRow`, which arrives over the wire (REMOTE §1.2, bl-adcb);
//! what stays here is the affordance a row earns, which is an act and derived in
//! process still.

use super::super::ShellState;
use super::super::menus::{BallRef, Target};
use super::super::start_rows::{ball_id, continue_row, ready_row};
use crate::AppModel;
use crate::board::Column;
use crate::cli_outbound::Cli;
use crate::nav::menu::Seat;
use crate::start::StartInputs;
use std::collections::HashMap;

/// The planner's entry points, indexed by the ball each enters — the join
/// between the board's rows and the start flow. Held together because a row
/// asks both questions at once ("can I start it, can I resume it").
pub(super) struct Affordances {
    starts: HashMap<String, StartInputs>,
    resumes: HashMap<String, StartInputs>,
}

impl Affordances {
    /// Both questions asked of the model at once. The planner's inputs are
    /// **acts**, not a read: they are what a click fires, so they stay derived
    /// in process while the board itself comes over the wire (bl-adcb).
    pub(super) fn of(model: &AppModel) -> Self {
        Self {
            starts: keyed(model.startable()),
            resumes: keyed(model.resumable()),
        }
    }
}

fn keyed(inputs: Vec<StartInputs>) -> HashMap<String, StartInputs> {
    inputs
        .into_iter()
        .filter_map(|i| Some((ball_id(&i.payload)?, i)))
        .collect()
}

/// One board row: its affordance (or a plain read), then the facts the row
/// carries — its gate, its drones, its spend and, for an epic, its rollup.
pub(super) fn board_row(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    bl: &Cli,
    row: &crate::board::BoardRow,
    affordances: &Affordances,
) {
    match (
        row.column,
        affordances.starts.get(&row.id),
        affordances.resumes.get(&row.id),
    ) {
        (Column::Claimed, _, Some(inputs)) => {
            continue_row(ui, model, state, lernie, bl, inputs.clone());
        }
        (Column::Ready | Column::Gated, Some(inputs), _) => {
            ready_row(ui, model, state, lernie, bl, inputs.clone());
        }
        _ => read_row(ui, model, state, lernie, row),
    }
    ui.indent(("board-facts", row.id.as_str()), |ui| {
        for gate in &row.gates {
            ui.weak(format!("⊣ gate {}: {}", gate.id, gate.title))
                .on_hover_text(
                    "This ball cannot close until that one does. Closing it is what mints \
                     the gate — nothing here releases it.",
                );
        }
        for drone in &row.drones {
            ui.weak(format!("↳ {}", drone.name)).on_hover_text(
                "The conversation working this ball — the same object the conversation list \
                 above shows. Its goal names this ball.",
            );
        }
        if let Some(figure) = &row.spend {
            ui.horizontal(|ui| {
                ui.weak("spend:");
                crate::spend::render(ui, figure);
            });
        }
        if let Some(figure) = &row.rollup {
            ui.horizontal(|ui| {
                ui.weak("epic:");
                crate::spend::render(ui, figure);
            })
            .response
            .on_hover_text(
                "This ball plus its live subtree, summed across every workspace those balls \
                 are claimed in. A closed child leaves the live set, and its spend leaves \
                 this figure with it.",
            );
        }
    });
}

/// A row with no start affordance — blocked, or claimed by someone this
/// machine has no workspace for. Still a full row: id, title, and the §11
/// ball-row menu.
fn read_row(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    row: &crate::board::BoardRow,
) {
    let label = ui
        .add(
            egui::Label::new(egui::RichText::new(format!("{}: {}", row.id, row.title)).weak())
                .sense(egui::Sense::click()),
        )
        .on_hover_text(
            "Right-click for this ball's actions — Close, Release, Move. Each is also \
             a key or a line: (c), (r), `/move [id] <to>`.",
        );
    let seat = Seat::BallRow {
        state: row.state,
        assign_to: model.focused_ws_name(),
        move_to: model.move_targets(row.claimant.as_deref().unwrap_or_default()),
    };
    let target = Target::Ball(BallRef {
        project: row.project.clone(),
        id: row.id.clone(),
        owner: row.claimant.clone().unwrap_or_default(),
    });
    super::super::menus::attach(&label, seat, &target, model, state, lernie);
}
