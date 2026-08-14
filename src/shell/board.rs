//! The balls section as the **V4 board** (VISION §5 V4, DESIGN §11): the four
//! derived columns, and each row's gate, drones and spend.
//!
//! Coverage-excluded glue, like every other file here. [`AppModel::board`] is
//! the covered derivation — the columns, the gates, the drones and the figures
//! are all decided there — and this lays it out. The per-row affordances stay
//! where they were built ([`super::start_rows`]); the board only decides which
//! one a row earns and what facts hang beneath it.

use super::ShellState;
use super::menus::{BallRef, Target};
use super::start_rows::{ball_id, continue_row, new_ball_form, ready_row};
use crate::AppModel;
use crate::board::Column;
use crate::cli_outbound::Cli;
use crate::nav::menu::Seat;
use crate::start::StartInputs;
use std::collections::HashMap;

/// The balls section, **as the V4 board** (VISION §5 V4): the empty-project
/// hint, then one fold per column — ready / gated / claimed / blocked, each
/// headed by its own derived count — then a per-project new-ball form.
///
/// The affordances did not move altitude, only address: a ready or gated row is
/// still ▶ Start + Assign, a claimed row is still ▶ Continue, and every row
/// still seats the §11 ball-row menu. What the columns add is the rest of
/// [`crate::board::BoardRow`] — the gate that holds a row (and the ball whose
/// close mints it), the drones working a claimed one, and its spend.
///
/// Coverage-excluded glue, as ever: [`AppModel::board`] is the covered
/// derivation and this file only lays it out.
pub fn board(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    bl: &Cli,
) {
    // **This** surface's last failure (§7.3), derived from the model this frame.
    // A failed seed/claim/mint step no longer prints to a stderr nobody reads —
    // it renders here in ichor red, and the composer that would have opened
    // stays closed with the reason shown. Per-frame, not cached at dispatch, so
    // a detached prompt that dies *after* its launch returned still banners once
    // the sweep folds its §8.1 sink in (bl-4895).
    //
    // `Origin::Balls` is the whole of bl-48f8: this fold paints ball ops and
    // ball-rung starts — the gestures it offers — and nothing else. Unfiltered
    // it painted the *global* last failure, so a bare-rung Enter typed into the
    // composer accused the balls section too, and a clean `bl close` here wiped
    // the composer's live banner.
    if let Some(failure) = model.last_failure(crate::opslog::Origin::Balls) {
        super::banner::failure_banner(ui, model, state, &failure);
    }
    // STORIES S3-T5: with zero projects, the paved interim for adding one. The
    // two lines come from the covered view-model (bl-b491); the command gets
    // its own row, **wrapped** rather than truncated — wrapping is bounded by
    // the panel's width, so unlike `Extend` it cannot ratchet the left column
    // wider (§11 bl-9669/bl-ac3d) — and selectable, so it can be copied out.
    if let Some(hint) = model.empty_project_hint() {
        ui.weak(hint.lead);
        ui.add(
            egui::Label::new(egui::RichText::new(hint.command).monospace())
                .wrap()
                .selectable(true),
        )
        .on_hover_text(
            "The command that registers a project with yog — selectable, so you \
             can copy it out and run it in a terminal. No key of its own: Tab reaches \
             it, then copy.",
        );
    }
    columns(ui, model, state, lernie, bl);
    ui.separator();
    // Rows are labelled by the covered [`crate::projects::labels`] fold — the
    // basename, extended only where two projects would read alike (bl-ac3d).
    let projects = model.project_paths();
    for (project, label) in projects.iter().zip(crate::projects::labels(&projects)) {
        new_ball_form(ui, model, state, lernie, bl, project, &label);
    }
}

/// The four columns, each folded open by default and headed by its count. A
/// column with no rows renders nothing at all — an operator with no blocked
/// work should not be told about a blocked column.
fn columns(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    bl: &Cli,
) {
    let board = model.board();
    // **The armed loop's facts, and only when one is armed** (VISION §5 V4
    // item 2). `board.fleet` is empty in every unarmed world, so this loop
    // paints nothing at all and the section is byte-for-byte what it was —
    // V4's burden check, which is why there is no "unarmed" chip to draw.
    for fleet in &board.fleet {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("fleet").strong());
            ui.weak(fleet.label());
        })
        .response
        .on_hover_text(format!(
            "An armed loop on this workspace, taking ready balls from {}. \
             It claims and starts; it releases a claim whose conversations have \
             gone quiet past the lease; it never stops anything that is running. \
             Every spawn and reap is a row on the ops trail.",
            fleet.project.display()
        ));
        // The ceiling renders where it will bind: on the next spawn.
        if let Some(refusal) = &fleet.ceiling {
            ui.colored_label(crate::theme::ICHOR, refusal);
        }
    }
    // The affordances, addressed by ball id: a row renders the one its own
    // column earns, and a row the planner cannot enter renders as a read.
    let affordances = Affordances {
        starts: keyed(model.startable()),
        resumes: keyed(model.resumable()),
    };
    for column in Column::ALL {
        let rows = board.in_column(column);
        if rows.is_empty() {
            continue;
        }
        egui::CollapsingHeader::new(format!("{} ({})", column.word(), rows.len()))
            .id_salt(column.word())
            .default_open(true)
            .show(ui, |ui| {
                for row in rows {
                    board_row(ui, model, state, lernie, bl, &row, &affordances);
                }
            })
            .header_response
            .on_hover_text(column_hint(column));
    }
}

/// What each column means, in the operator's words — the gated one especially,
/// since it is the column that is not a `bl list` status.
fn column_hint(column: Column) -> &'static str {
    match column {
        Column::Ready => "claimable now: unclaimed, with every dependency resolved.",
        Column::Gated => {
            "claimable, but not deliverable: a close-gate is still open. You can \
                          start the work; the ball cannot close until the ball named on the row \
                          closes first."
        }
        Column::Claimed => {
            "someone holds it. The rows beneath name the conversations working \
                            it and what they have spent."
        }
        Column::Blocked => "not claimable yet: a dependency is still open.",
    }
}

/// The planner's entry points, indexed by the ball each enters — the join
/// between the board's rows and the start flow. Held together because a row
/// asks both questions at once ("can I start it, can I resume it").
struct Affordances {
    starts: HashMap<String, StartInputs>,
    resumes: HashMap<String, StartInputs>,
}

fn keyed(inputs: Vec<StartInputs>) -> HashMap<String, StartInputs> {
    inputs
        .into_iter()
        .filter_map(|i| Some((ball_id(&i.payload)?, i)))
        .collect()
}

/// One board row: its affordance (or a plain read), then the facts the row
/// carries — its gate, its drones, its spend and, for an epic, its rollup.
fn board_row(
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
        _ => read_row(ui, model, state, lernie, bl, row),
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
    bl: &Cli,
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
    super::menus::attach(&label, seat, &target, model, state, lernie, bl);
}
