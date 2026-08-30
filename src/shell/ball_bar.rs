//! The ball action row of the composer (§8.2/§11): the focused ball's
//! Close / Release, split from [`super::input_bar`] per §12's 300-line
//! budget. Coverage-excluded glue — the enablement predicates and the `bl`
//! dispatchers it wires are covered in `actions`.
//!
//! It is also **the one body of each §8.2 ball verb**, written by (project,
//! ball, claimant) rather than by "the focused row": [`close_ball`],
//! [`release_ball`] and [`assign_ball`]. The composer's buttons
//! and the §11 `c`/`r` keys reach them through the focus; the §11 ball-row
//! context menu (`super::menus`) reaches them with the row under the pointer.
//! One implementation per gesture, whichever hand fires it.
//!
//! **Every verb here crosses the wire** (REMOTE §9.8, bl-4841): it posts the
//! gesture and holds no receipt, because none was ever read — the durable record
//! is the ball verb's own `ops.jsonl` line and the pane re-reads the store the
//! §7.1 routing refreshes. The `litany`/`bl` pair went with the dispatch: a
//! posted act carries the gesture and nothing else, the binaries being the
//! engine's.

use crate::AppModel;
use crate::actions::{close_enabled, unclaim_enabled};
use crate::boundary::Action;
use crate::nav::BoundBall;

/// `Close` (§8.2): the delivery, said in what it actually does to the repo.
const CLOSE_HINT: &str = "Deliver this ball (`bl close`): folds `main` into its worktree, runs the \
     project's pre-commit gate, squashes the work onto the target branch, and \
     removes the worktree. A failing gate aborts and leaves the ball claimed (c).";

/// `Release` (§8.2): the claim drops, the commits stay.
const RELEASE_HINT: &str = "Let this ball go (`bl unclaim`): the workspace stops holding it and anyone \
     can claim it again. Nothing already committed in its worktree is lost (r).";

/// The focused ball's Close / Release, gated by its §3.5 join state (§8.2).
/// Every `bl` verb stamps `--as` the ball's **bound workspace name** (its
/// claimant, which the answered row carries as `owner`) — the §3.2 ownership
/// line, never the operator `$USER`.
///
/// `ball` is the first row of the focused workspace's landed listing
/// (REMOTE §9.7, bl-b4b5) — a selection out of an answer this frame already
/// holds, so the row is one ask rather than a second read of the window's own
/// snapshot.
pub(super) fn actions(ui: &mut egui::Ui, model: &mut AppModel, ball: Option<&BoundBall>) {
    let Some(row) = ball else {
        return;
    };
    ui.horizontal(|ui| {
        ui.label(format!("ball {}", row.id));
        if ui
            .add_enabled(close_enabled(row.state), egui::Button::new("Close"))
            .on_hover_text(CLOSE_HINT)
            .on_disabled_hover_text("only the workspace holding a ball can close it")
            .clicked()
        {
            close_row(model, row);
        }
        if ui
            .add_enabled(unclaim_enabled(row.state), egui::Button::new("Release"))
            .on_hover_text(RELEASE_HINT)
            .on_disabled_hover_text(
                "this workspace is not holding the ball, so it has none to let go",
            )
            .clicked()
        {
            release_row(model, row);
        }
    });
}

/// `bl close <id> --as <owner>` for one answered ball row (§8.2) — the Close
/// button's body, shared with the §11 `c` binding so the gesture has one
/// implementation.
fn close_row(model: &mut AppModel, row: &BoundBall) {
    close_ball(model, &row.project, &row.id, &row.owner);
}

/// `bl unclaim <id> --as <owner>` for one answered ball row (§8.2) — the
/// Release button's body, shared with the §11 `r` binding.
fn release_row(model: &mut AppModel, row: &BoundBall) {
    release_ball(model, &row.project, &row.id, &row.owner);
}

/// `bl close <id> --as <owner>` (§8.2) named outright — the one body the Close
/// button, the `c` binding and the §11 ball-row menu all reach (`super::menus`),
/// which is why the menu can never mean something else by "close".
pub(super) fn close_ball(model: &mut AppModel, project: &str, id: &str, owner: &str) {
    super::act::fire(
        model,
        &Action::Close {
            project: project.to_owned(),
            id: id.to_owned(),
            name: owner.to_owned(),
        },
    );
}

/// `bl unclaim <id> --as <owner>` (§8.2) — Release's one body, shared with the
/// `r` binding and the ball-row menu.
pub(super) fn release_ball(model: &mut AppModel, project: &str, id: &str, owner: &str) {
    super::act::fire(
        model,
        &Action::Release {
            project: project.to_owned(),
            id: id.to_owned(),
            name: owner.to_owned(),
        },
    );
}

/// `bl claim <id> --as <to>` (§8.2) — Assign's one body: the ready ball row's
/// `assign → <workspace>` button ([`super::start_pane`]) and the ball-row menu.
pub(super) fn assign_ball(model: &mut AppModel, project: &str, id: &str, to: &str) {
    super::act::fire(
        model,
        &Action::Assign {
            project: project.to_owned(),
            id: id.to_owned(),
            name: to.to_owned(),
        },
    );
}

/// Release the **focused conversation's bound ball** (§8.2): the §11 `r`
/// binding, refused exactly where the button is disabled.
pub(super) fn release_focused(model: &mut AppModel) {
    let Some(row) = focused_ball(model) else {
        return;
    };
    if unclaim_enabled(row.state) {
        release_row(model, &row);
    }
}

/// Close the **focused conversation's bound ball** (§8.2): the §11 `c` binding.
/// Aims at the same row the button does and honours [`close_enabled`], so it is
/// refused exactly where the button is disabled.
pub(super) fn close_focused(model: &mut AppModel) {
    let Some(row) = focused_ball(model) else {
        return;
    };
    if close_enabled(row.state) {
        close_row(model, &row);
    }
}

/// The row both key bindings act on: the **first** of the focused workspace's
/// landed ball rows, which is the very row [`actions`] paints above the
/// composer.
///
/// **Read off the landed answer, never awaited at the click** (REMOTE §9.7,
/// bl-b4b5). The question is already standing — the ball row painted this
/// frame declared it — so this is a map read, and a key pressed before the
/// first answer simply does nothing, exactly as a key pressed with no ball
/// bound does. The authoritative gate is the engine's, re-derived at fire.
fn focused_ball(model: &mut AppModel) -> Option<BoundBall> {
    super::chrome::focused_balls(model).first().cloned()
}
