//! The ball action row of the composer (§8.2/§11): the focused ball's
//! Close / Release / Move, split from [`super::input_bar`] per §12's 300-line
//! budget. Coverage-excluded glue — the enablement predicates and the `bl`
//! dispatchers it wires are covered in `actions`.
//!
//! It is also **the one body of each §8.2 ball verb**, written by (project,
//! ball, claimant) rather than by "the focused row": [`close_ball`],
//! [`release_ball`], [`move_ball`] and [`assign_ball`]. The composer's buttons
//! and the §11 `c`/`r` keys reach them through the focus; the §11 ball-row
//! context menu (`super::menus`) reaches them with the row under the pointer.
//! One implementation per gesture, whichever hand fires it.

use crate::AppModel;
use crate::actions::{close_enabled, move_enabled, unclaim_enabled};
use crate::boundary::Action;
use crate::cli_outbound::Cli;
use crate::projects::join::{JoinRow, owner_name};
use std::path::Path;

/// `Close` (§8.2): the delivery, said in what it actually does to the repo.
const CLOSE_HINT: &str = "Deliver this ball (`bl close`): folds `main` into its worktree, runs the \
     project's pre-commit gate, squashes the work onto the target branch, and \
     removes the worktree. A failing gate aborts and leaves the ball claimed (c).";

/// `Release` (§8.2): the claim drops, the commits stay.
const RELEASE_HINT: &str = "Let this ball go (`bl unclaim`): the workspace stops holding it and anyone \
     can claim it again. Nothing already committed in its worktree is lost (r).";

/// The `move to:` destinations (§8.2): one gesture, two `bl` calls.
const MOVE_HINT: &str = "Re-home this ball to that workspace — released here, claimed there, in \
     one gesture. The destination is a pick, so its keyboard spelling is the line: \
     `/move [id] <to>`.";

/// The focused ball's Close / Release / Move, gated by its §3.5 join state (§8.2).
/// Every `bl` verb stamps `--as` the ball's **bound workspace name** (its claimant,
/// via [`owner_name`]) — the §3.2 ownership line, never the operator `$USER`.
pub(super) fn actions(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    lernie: &Cli,
    bl: &Cli,
    join: Option<&JoinRow>,
) {
    let Some(row) = join else {
        return;
    };
    let owner = owner_name(row);
    // Move targets: the other named workspaces this bound ball can be re-homed to.
    let targets = model.move_targets(&owner);
    ui.horizontal(|ui| {
        ui.label(format!("ball {}", row.ball_id));
        if ui
            .add_enabled(close_enabled(row.state), egui::Button::new("Close"))
            .on_hover_text(CLOSE_HINT)
            .on_disabled_hover_text("only the workspace holding a ball can close it")
            .clicked()
        {
            close_row(model, lernie, bl, row);
        }
        if ui
            .add_enabled(unclaim_enabled(row.state), egui::Button::new("Release"))
            .on_hover_text(RELEASE_HINT)
            .on_disabled_hover_text(
                "this workspace is not holding the ball, so it has none to let go",
            )
            .clicked()
        {
            release_row(model, lernie, bl, row);
        }
    });
    // Move (§8.2): re-home a bound ball to another workspace — one button per
    // target, each an `unclaim --as owner` then `claim --as target`.
    if move_enabled(row.state) && !targets.is_empty() {
        ui.horizontal(|ui| {
            ui.label("move to:").on_hover_text(MOVE_HINT);
            for to in &targets {
                if ui.button(to).on_hover_text(MOVE_HINT).clicked() {
                    move_ball(model, lernie, bl, &row.project, &row.ball_id, &owner, to);
                }
            }
        });
    }
}

/// `bl close <id> --as <owner>` for one join row (§8.2) — the Close button's
/// body, shared with the §11 `c` binding so the gesture has one implementation.
fn close_row(model: &mut AppModel, lernie: &Cli, bl: &Cli, row: &JoinRow) {
    close_ball(
        model,
        lernie,
        bl,
        &row.project,
        &row.ball_id,
        &owner_name(row),
    );
}

/// `bl unclaim <id> --as <owner>` for one join row (§8.2) — the Release
/// button's body, shared with the §11 `r` binding.
fn release_row(model: &mut AppModel, lernie: &Cli, bl: &Cli, row: &JoinRow) {
    release_ball(
        model,
        lernie,
        bl,
        &row.project,
        &row.ball_id,
        &owner_name(row),
    );
}

/// `bl close <id> --as <owner>` (§8.2) named outright — the one body the Close
/// button, the `c` binding and the §11 ball-row menu all reach (`super::menus`),
/// which is why the menu can never mean something else by "close".
pub(super) fn close_ball(
    model: &mut AppModel,
    lernie: &Cli,
    bl: &Cli,
    project: &Path,
    id: &str,
    owner: &str,
) {
    fire(
        model,
        lernie,
        bl,
        project,
        Action::Close {
            project: project.to_path_buf(),
            id: id.to_owned(),
            name: owner.to_owned(),
        },
    );
}

/// `bl unclaim <id> --as <owner>` (§8.2) — Release's one body, shared with the
/// `r` binding and the ball-row menu.
pub(super) fn release_ball(
    model: &mut AppModel,
    lernie: &Cli,
    bl: &Cli,
    project: &Path,
    id: &str,
    owner: &str,
) {
    fire(
        model,
        lernie,
        bl,
        project,
        Action::Release {
            project: project.to_path_buf(),
            id: id.to_owned(),
            name: owner.to_owned(),
        },
    );
}

/// `bl unclaim` then `bl claim --as <to>` (§8.2) — Move's one body, shared with
/// the ball-row menu's destination submenu.
pub(super) fn move_ball(
    model: &mut AppModel,
    lernie: &Cli,
    bl: &Cli,
    project: &Path,
    id: &str,
    owner: &str,
    to: &str,
) {
    fire(
        model,
        lernie,
        bl,
        project,
        Action::Move {
            project: project.to_path_buf(),
            id: id.to_owned(),
            from: owner.to_owned(),
            to: to.to_owned(),
        },
    );
}

/// `bl claim <id> --as <to>` (§8.2) — Assign's one body: the ready ball row's
/// `assign → <workspace>` button ([`super::start_pane`]) and the ball-row menu.
pub(super) fn assign_ball(
    model: &mut AppModel,
    lernie: &Cli,
    bl: &Cli,
    project: &Path,
    id: &str,
    to: &str,
) {
    fire(
        model,
        lernie,
        bl,
        project,
        Action::Assign {
            project: project.to_path_buf(),
            id: id.to_owned(),
            name: to.to_owned(),
        },
    );
}

/// The one dispatch tail every ball verb above shares (§8.5): construct →
/// chokepoint → the `bl`-verb aftermath. The result itself is discarded — the
/// ops line is the durable fact the pane and banner read (INV-2).
fn fire(model: &mut AppModel, lernie: &Cli, bl: &Cli, project: &Path, action: Action) {
    let deps = model.boundary_deps(lernie, bl);
    let _ = model.dispatch(&deps, &super::now_ts(), &action);
    model.after_bl_verb(project);
}

/// Release the **focused conversation's bound ball** (§8.2): the §11 `r`
/// binding, refused exactly where the button is disabled.
pub(super) fn release_focused(model: &mut AppModel, lernie: &Cli, bl: &Cli) {
    let Some(row) = model.focused_join().cloned() else {
        return;
    };
    if unclaim_enabled(row.state) {
        release_row(model, lernie, bl, &row);
    }
}

/// Close the **focused conversation's bound ball** (§8.2): the §11 `c` binding.
/// Re-derives its target from the focus and honours [`close_enabled`], so it is
/// refused exactly where the button is disabled.
pub(super) fn close_focused(model: &mut AppModel, lernie: &Cli, bl: &Cli) {
    let Some(row) = model.focused_join().cloned() else {
        return;
    };
    if close_enabled(row.state) {
        close_row(model, lernie, bl, &row);
    }
}
