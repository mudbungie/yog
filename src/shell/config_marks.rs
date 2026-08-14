//! **The agent's tracking-branch pane** (§16.3) — a thin egui section of config
//! mode. Coverage-excluded glue (`src/shell/*`, tarpaulin.toml): the VM
//! ([`crate::world::marks`]) carries all logic and is headless-tested; this only
//! wires controls to it. Split from [`super::config_edit`] so both stay under
//! the 300-line cap.
//!
//! Both halves are the boundary's own: `Read current` constructs
//! [`Query::Marks`](crate::boundary::Query::Marks) and Set constructs
//! [`Action::SetMarks`](crate::boundary::Action::SetMarks) — the same variants a
//! headless `/marks` reaches (bl-3f46, bl-0164), one implementation, two
//! serializations.
//!
//! **It asks about the focused WORKSPACE, not the focused project** (the
//! per-agent ruling): the branch is the agent's, so a workspace that
//! is bound to no project still has one, which is exactly the
//! launched-then-told-to-work-on-a-project case the pane must serve.

use crate::AppModel;
use crate::boundary::reply::Reply;
use crate::boundary::{Action, Query};
use crate::cli_outbound::Cli;
use crate::world::marks;
use std::path::{Path, PathBuf};

/// The tracking-branch field's hint, said once beside both the label and the box.
const BRANCH_HINT: &str = "The balls branch this agent records its tasks on. `balls/tasks` is the \
     project's shared board — the branch an agent is pointed at when it is raised to work an \
     existing project. Anything else is this agent's own task space, which no other agent's \
     churn reaches. Typed, it is `/marks <branch>`.";

/// The pane's RAM state (§3.5): the last-read branch and the workspace it was
/// read for, the branch input, and the status line. Nothing is held that the
/// space itself answers — both halves ask at the gesture, never per frame.
pub struct MarksPane {
    workspace: Option<PathBuf>,
    branch: String,
    input: String,
    space: String,
    status: String,
}

impl MarksPane {
    /// Everything starts empty (read on demand).
    pub fn resolve() -> Self {
        Self {
            workspace: None,
            branch: String::new(),
            input: String::new(),
            space: String::new(),
            status: String::new(),
        }
    }
}

/// Render the knob for the focused workspace. Reads on demand (never per-frame)
/// and states what landed.
pub fn render(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    pane: &mut MarksPane,
    lernie: &Cli,
    bl: &Cli,
) {
    ui.heading(
        egui::RichText::new("task branch (this agent's balls space)")
            .color(crate::theme::integration_hue("bl")),
    );
    let Some(workspace) = model.focused_workspace().map(Path::to_path_buf) else {
        ui.weak("focus a workspace to read or amend the branch it tracks on");
        return;
    };
    ui.monospace(workspace.display().to_string());
    if pane.workspace.as_ref() == Some(&workspace) && !pane.branch.is_empty() {
        ui.label(format!("branch: {}", pane.branch));
        ui.weak(format!("space: {}", pane.space));
    }
    ui.weak(
        "each agent tracks on a branch of its own; subagents inherit the space they were \
         dispatched from",
    );
    // §11 rule 8 (bl-7414): two verbs of their own natural width, neither of
    // which may be dropped, so the pair wraps to a second line in a narrow pane
    // instead of running off it — and instead of ratcheting the seat's own
    // `max_rect`, which is what cut the §9.5 sentence below with no ellipsis.
    crate::shell::row::peers(ui, |ui| {
        if ui
            .button("Read current")
            .on_hover_text(
                "Ask which branch this agent's tasks are recorded on right now. \
                 Changes nothing. Typed, it is `/marks` with no branch.",
            )
            .clicked()
        {
            pane.status = read_marks(model, pane, lernie, bl, &workspace);
        }
        if ui
            .button("Point at the project's board")
            .on_hover_text(
                "Track on `balls/tasks` — the project's shared board, where a claim this \
                 agent makes is the same claim `bl list` shows. Typed, it is \
                 `/marks balls/tasks`.",
            )
            .clicked()
        {
            pane.status = apply_marks(model, pane, lernie, bl, &workspace, marks::SHARED_BRANCH);
        }
    });
    // The same rule with a field in it: the branch name takes the remainder of
    // its own line rather than egui's fixed 280 pt `text_edit_width`, and the
    // verb wraps below rather than off (bl-7414).
    crate::shell::row::peers(ui, |ui| {
        ui.label("branch:").on_hover_text(BRANCH_HINT);
        ui.add(egui::TextEdit::singleline(&mut pane.input).desired_width(f32::INFINITY))
            .on_hover_text(BRANCH_HINT);
        if ui
            .button("Set branch")
            .on_hover_text(
                "Point this agent's task space at the branch named beside it. Typed, it \
                 is `/marks <branch>`.",
            )
            .clicked()
        {
            let named = pane.input.trim().to_owned();
            pane.status = if marks::lawful(&named) {
                apply_marks(model, pane, lernie, bl, &workspace, &named)
            } else {
                marks::REFUSAL.to_owned()
            };
        }
    });
    if !pane.status.is_empty() {
        ui.weak(&pane.status);
    }
}

/// Read the focused workspace's branch into the pane (keyed by workspace so a
/// focus change never shows a stale one) — through the boundary (§8.5): the
/// same [`Query::Marks`] a headless `/marks` reaches.
fn read_marks(
    model: &AppModel,
    pane: &mut MarksPane,
    lernie: &Cli,
    bl: &Cli,
    workspace: &Path,
) -> String {
    let deps = model.boundary_deps(lernie, bl);
    let query = Query::Marks {
        workspace: model.snap.ws_name(workspace),
    };
    match model.answer(&deps, &query, super::now_unix()) {
        Ok(reply) => landed(pane, workspace, &reply),
        Err(e) => e,
    }
}

/// Amend the branch through the boundary (§8.5): the variant the click
/// constructs is the one a deposit or a `/marks` line constructs, and the reply
/// carries the branch re-read afterwards — so the pane paints what landed.
fn apply_marks(
    model: &mut AppModel,
    pane: &mut MarksPane,
    lernie: &Cli,
    bl: &Cli,
    workspace: &Path,
    branch: &str,
) -> String {
    let deps = model.boundary_deps(lernie, bl);
    let action = Action::SetMarks {
        workspace: model.snap.ws_name(workspace),
        branch: branch.to_owned(),
    };
    match model.dispatch(&deps, &super::now_ts(), &action) {
        Ok(reply) => format!("applied — {}", landed(pane, workspace, &reply)),
        Err(e) => e,
    }
}

/// Fold one [`Reply::Marks`] into the pane and say it in a line. Any other reply
/// is a boundary contract break and says so rather than rendering nothing.
fn landed(pane: &mut MarksPane, workspace: &Path, reply: &Reply) -> String {
    let Reply::Marks { branch, space } = reply else {
        return format!("unexpected reply: {reply:?}");
    };
    pane.workspace = Some(workspace.to_path_buf());
    pane.branch.clone_from(branch);
    pane.space = space.display().to_string();
    format!("tracking on {branch}")
}
