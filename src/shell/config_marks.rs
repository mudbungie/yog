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
//! **Both halves cross the wire now** (REMOTE §9.7/§9.8, bl-4841 + bl-f297).
//! Set is posted and paints its receipt — which carries the branch re-read
//! after the write, so the pane states what landed. `Read current` is the
//! **standing question with a latch** §9.8 ruled a click-time read to be: the
//! click turns the latch on, the pane declares [`Query::Marks`] while it paints,
//! and the branch appears when the answer lands one ask-period later. The latch
//! is the workspace it was thrown for, so a focus change stops the question
//! being declared and the asker drops its answer — no unsubscribe, and no way
//! to show one workspace's branch under another's name.
//!
//! **The latch does not fall on the first answer.** A read that turned itself
//! off would be a one-shot with a socket behind it: the same bytes on screen
//! whether the branch moved a second later or not. Left on, the pane's reading
//! stays live for as long as the operator is looking at that workspace, which
//! costs one ask per period and is what makes the write's own confirmation
//! arrive twice by two honest routes rather than once by a lucky one.
//!
//! **The `Cli` pair evaporated with the read** (REMOTE §9.8): the pane had them
//! only to build `boundary_deps` for the in-process answer. A posted act carries
//! the gesture and a standing question carries the query; neither carries this
//! box's verb binaries, so a remote seat could drive this whole pane.
//!
//! **It asks about the focused WORKSPACE, not the focused project** (the
//! per-agent ruling): the branch is the agent's, so a workspace that
//! is bound to no project still has one, which is exactly the
//! launched-then-told-to-work-on-a-project case the pane must serve.

use crate::AppModel;
use crate::boundary::reply::Reply;
use crate::boundary::{Action, Query};
use crate::world::marks;
use std::path::{Path, PathBuf};

/// The tracking-branch field's hint, said once beside both the label and the box.
const BRANCH_HINT: &str = "The balls branch this agent records its tasks on. `balls/tasks` is the \
     project's shared board — the branch an agent is pointed at when it is raised to work an \
     existing project. Anything else is this agent's own task space, which no other agent's \
     churn reaches. Typed, it is `/marks <branch>`.";

/// The pane's RAM state (§3.5): the last-landed branch and the workspace it was
/// read for, the reading latch, the branch input, and the status line. Nothing
/// is held that the space itself answers — the branch on screen is whatever the
/// wire last said.
pub struct MarksPane {
    workspace: Option<PathBuf>,
    branch: String,
    /// **The read's latch** (REMOTE §9.8): the workspace whose branch the
    /// operator asked for, or `None` before anyone asked. Not a bare `bool`,
    /// because the question names a workspace and the focus can move under it —
    /// a latch that could not say which workspace it was thrown for would show
    /// one wall's branch beside another wall's name.
    reading: Option<PathBuf>,
    input: String,
    /// The last gesture's sentence and the ticket its receipt lands under
    /// (REMOTE §9.8, bl-4841). The read and the write share it: one line, and
    /// what it says is whichever the operator last spent.
    act: crate::shell::act::Held,
}

impl MarksPane {
    /// Everything starts empty (read on demand).
    pub fn resolve() -> Self {
        Self {
            workspace: None,
            branch: String::new(),
            reading: None,
            input: String::new(),
            act: crate::shell::act::Held::default(),
        }
    }
}

/// Render the knob for the focused workspace. Reads on demand (never per-frame)
/// and states what landed.
pub fn render(ui: &mut egui::Ui, model: &mut AppModel, pane: &mut MarksPane) {
    ui.heading(
        egui::RichText::new("task branch (this agent's balls space)")
            .color(crate::theme::integration_hue("bl")),
    );
    let Some(workspace) = model.focused_workspace() else {
        ui.weak("focus a workspace to read or amend the branch it tracks on");
        return;
    };
    // The writes below are posted (REMOTE §9.8), so their receipt is folded
    // here, at the top of the pane, on the frame it lands — and the read is a
    // standing question, declared here while the latch is thrown for this
    // workspace, so its answer lands the same way.
    settle(model, pane);
    read_marks(model, pane, &workspace);
    ui.monospace(workspace.display().to_string());
    if pane.workspace.as_ref() == Some(&workspace) && !pane.branch.is_empty() {
        ui.label(format!("branch: {}", pane.branch));
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
            // Throwing the latch is the whole gesture: the declaration is the
            // paint above, and the answer lands one ask-period later.
            pane.reading = Some(workspace.clone());
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
            apply_marks(model, pane, &workspace, marks::SHARED_BRANCH);
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
            if marks::lawful(&named) {
                apply_marks(model, pane, &workspace, &named);
            } else {
                pane.act.say(marks::REFUSAL.to_owned());
            }
        }
    });
    if !pane.act.quiet() {
        ui.weak(pane.act.line());
    }
}

/// Declare the latched read and fold whatever has landed for it — the same
/// [`Query::Marks`] a headless `/marks` reaches (§8.5), asked over the wire
/// (REMOTE §9.7). Nothing is declared until the operator throws the latch, and
/// nothing is declared for a workspace other than the one it was thrown for, so
/// the pane can never paint one wall's branch under another wall's name.
fn read_marks(model: &mut AppModel, pane: &mut MarksPane, workspace: &Path) {
    if pane.reading.as_deref() != Some(workspace) {
        return;
    }
    let query = Query::Marks {
        workspace: model.snap.ws_name(workspace),
    };
    let landed_branch = crate::shell::wire::ask(model, query, |reply| match reply {
        Reply::Marks { branch } => Some(branch),
        _ => None,
    });
    // A refusal is painted, not swallowed: it takes the pane's one status line,
    // which is the only surface this read has.
    if let Some(said) = landed_branch.refused {
        pane.act.say(said);
        return;
    }
    if let Some(branch) = landed_branch.value {
        pane.workspace = Some(workspace.to_path_buf());
        pane.branch = branch;
    }
}

/// Amend the branch through the boundary (§8.5), **posted** (REMOTE §9.8): the
/// variant the click constructs is the one a deposit or a `/marks` line
/// constructs, and its receipt carries the branch re-read afterwards — so the
/// pane paints what landed rather than what it asked for.
fn apply_marks(model: &mut AppModel, pane: &mut MarksPane, workspace: &Path, branch: &str) {
    let action = Action::SetMarks {
        workspace: model.snap.ws_name(workspace),
        branch: branch.to_owned(),
    };
    // The workspace this act is about, held now: the receipt names a branch and
    // the pane keys its reading by workspace, and the focus may have moved by
    // the time it lands.
    pane.workspace = Some(workspace.to_path_buf());
    pane.act
        .fire(model, &action, &format!("tracking on {branch}"));
}

/// Fold whatever the pane's act earned, once, on the frame it arrives. A clean
/// receipt is a [`Reply::Marks`] — the branch **re-read after the write**, the
/// §5.3 receipt discipline — so the pane takes the branch from the answer and
/// keeps the sentence the click wrote.
fn settle(model: &mut AppModel, pane: &mut MarksPane) {
    let Some(landed) = pane.act.landed(model) else {
        return;
    };
    match (crate::shell::act::trouble(&landed), landed) {
        (None, Ok(Reply::Marks { branch })) => {
            pane.act.say(format!("tracking on {branch}"));
            pane.branch = branch;
        }
        (None, Ok(other)) => pane.act.say(format!("unexpected reply: {other:?}")),
        (why, _) => pane.act.say(why.unwrap_or_default()),
    }
}
