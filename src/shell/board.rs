//! The balls section as the **V4 board** (VISION §5 V4, DESIGN §11): the four
//! derived columns, and each row's gate, drones and spend.
//!
//! Coverage-excluded glue, like every other file here. `crate::board::build` is
//! the covered derivation — the columns, the gates, the drones and the figures
//! are all decided there — and this lays it out. The per-row affordances stay
//! where they were built ([`super::start_rows`]); the board only decides which
//! one a row earns and what facts hang beneath it.
//!
//! **The columns come over the wire** (REMOTE §1.2 and its read-path residual;
//! bl-adcb): the board painted here is a `Reply::Board` that crossed loopback
//! mTLS and was decoded by `reply::decode`, the clients section's shape
//! (bl-ae05) applied to the §11 balls fold. The model's own `board()` went with
//! it — an answer *is* the cached derivation, refreshed at human cadence.
//!
//! **The affordances are not a read and did not move.** `startable` /
//! `resumable` build the composer's fire-time inputs, which the click-glue
//! consumes synchronously — the acts path, and its own ball.

use super::ShellState;
use super::start_rows::new_ball_form;
use crate::AppModel;
use crate::board::Column;

mod rows;

use crate::boundary::Query;
use crate::boundary::reply::Reply;
use crate::cli_outbound::Cli;
use rows::{Affordances, board_row};

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
/// Coverage-excluded glue, as ever: `crate::board::build` is the covered
/// derivation, reached over the wire, and this file only lays it out.
pub fn board(ui: &mut egui::Ui, model: &mut AppModel, state: &mut ShellState, lernie: &Cli) {
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
    columns(ui, model, state, lernie);
    ui.separator();
    // Rows are labelled by the covered [`crate::projects::labels`] fold — the
    // basename, extended only where two projects would read alike (bl-ac3d).
    let projects = model.project_paths();
    for (project, label) in projects.iter().zip(crate::projects::labels(&projects)) {
        new_ball_form(ui, model, state, project, &label);
    }
}

/// The four columns, each folded open by default and headed by its count. A
/// column with no rows renders nothing at all — an operator with no blocked
/// work should not be told about a blocked column.
fn columns(ui: &mut egui::Ui, model: &mut AppModel, state: &mut ShellState, lernie: &Cli) {
    let landed = super::wire::ask(model, Query::Board, |reply| match reply {
        Reply::Board(board) => Some(board),
        _ => None,
    });
    // A refusal is painted, not swallowed: the wire is how this fold reads now,
    // so what the engine said is the section's honest content.
    if let Some(said) = &landed.refused {
        ui.colored_label(crate::theme::ICHOR, said);
    }
    let board = landed.value.unwrap_or_default();
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
    let affordances = Affordances::of(model);
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
                    board_row(ui, model, state, lernie, &row, &affordances);
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
