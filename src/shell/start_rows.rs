//! The start affordances of the balls section (§11): the ▶ Start / Assign rows
//! for ready balls, ▶ Continue for bound ones, and the per-project new-ball
//! form. Every one of them ends in [`super::start_pane::run_prepare`], which is
//! where the flow they enter — prepare, editable goal, fire — lives.
//! Coverage-excluded glue: the startable set, the plan and the orchestration are
//! tested in `AppModel` / `crate::start`; this file only wires widgets.
//!
//! Since bl-9dd4 the section they sit in is [`super::board`]'s: it decides
//! which affordance a row earns from that row's derived column, and calls the
//! bodies here.

use super::ShellState;
use super::menus::{BallRef, Target};
use super::start_pane::run_prepare;
use crate::AppModel;
use crate::cli_outbound::Cli;
use crate::nav::menu::Seat;
use crate::projects::join::JoinState;
use crate::start::{BallSpec, Payload, StartInputs};
use std::path::Path;

/// ▶ Start (§8.1 ball rung): claim, then the editable goal — nothing runs yet.
const START_HINT: &str = "Claim this ball for the focused workspace and open its goal for editing. \
     Nothing is sent to a model until you press Send. (s) starts the top ready row, \
     and `/prepare ball` says it on one line.";

/// `assign → <ws>` (§8.2): the claim without the conversation.
const ASSIGN_HINT: &str = "Claim this ball for the focused workspace and stop there — no worktree \
     is prepared, no conversation starts, no model call is spent. Typed, it is \
     `/assign [id]`.";

/// ▶ Continue (§8.1 addendum): a second conversation on a ball already held.
const CONTINUE_HINT: &str = "Open a fresh start goal on a ball this workspace already holds — a new \
     conversation against the same task. Nothing is sent until Send; (s) opens the top \
     row, and `/prepare ball` says it on one line.";

/// One ready ball: ▶ Start (the ball rung — claim + composer) and, when a
/// workspace is focused, Assign it there (`bl claim <id> --as <target>`, §8.2)
/// without starting a conversation. The ▶ button also seats the §11 ball-row
/// accelerator menu, whose Assign entry fires exactly this row's button body.
pub(super) fn ready_row(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    bl: &Cli,
    inputs: StartInputs,
) {
    let ball = ball_ref(&inputs.payload);
    let to = model.focused_ws_name();
    let label = format!("▶ {}", start_label(&inputs));
    // `assign → <ws>` is the row's trailing **control** and is laid first
    // (§11 rule 1b, [`super::row::control_last`]): after the greedy ▶ label —
    // `<id>: <title>`, arbitrarily long — it was handed the leftovers and
    // rendered as `assig…`, or as a bare `…` two title characters later
    // (bl-bc06). The ▶ label truncates in its place, which costs the tail of a
    // title while keeping the verb glyph, the id, and the hover.
    let assign_to = ball.as_ref().and(to.clone());
    let (start_button, assign_clicked) = super::row::control_last(
        ui,
        |ui| ui.button(label).on_hover_text(START_HINT),
        |ui| {
            assign_to.is_some_and(|to| {
                ui.button(format!("assign → {to}"))
                    .on_hover_text(ASSIGN_HINT)
                    .clicked()
            })
        },
    );
    if start_button.clicked() {
        run_prepare(model, state, lernie, bl, inputs);
        return;
    }
    let Some((project, id, join)) = ball else {
        return;
    };
    if assign_clicked && let Some(to) = &to {
        super::ball_bar::assign_ball(
            model,
            lernie,
            bl,
            &model.snap.project_path(&project).unwrap_or_default(),
            &id,
            to,
        );
    }
    let seat = Seat::BallRow {
        state: join,
        assign_to: to,
        // A ready ball is claimed by nobody, so there is nothing to re-home;
        // `move_enabled` refuses it anyway (§3.5).
        move_to: Vec::new(),
    };
    let target = Target::Ball(BallRef {
        // The menu target is a repo the seat acts in, so the name comes back
        // through the one mapping (REMOTE §8); a project this snapshot does not
        // enumerate resolves to nothing and the verbs refuse where they always
        // did.
        project: model.snap.project_path(&project).unwrap_or_default(),
        id,
        owner: String::new(),
    });
    super::menus::attach(&start_button, seat, &target, model, state, lernie, bl);
}

/// One bound ball: ▶ Continue `<id>: <title>` into the ball's **own** claimant
/// workspace (§8.1 resume, addendum), seating the §11 ball-row menu — Move /
/// Release / Close — on the button.
///
/// This is a bound ball's **only** roster row (bl-abbe). It used to render
/// twice: here, and again below the new-ball form as a bare grey id with no
/// title, state or verb. The bare one is gone
/// ([`AppModel::roster_ball_rows`]) and its verbs moved here, so the surviving
/// row is the full one.
pub(super) fn continue_row(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    bl: &Cli,
    inputs: StartInputs,
) {
    let ball =
        ball_ref(&inputs.payload).and_then(|(_, id, _)| model.bound_ball(&inputs.workspace, &id));
    let button = ui
        .button(format!("▶ Continue {}", start_label(&inputs)))
        .on_hover_text(CONTINUE_HINT);
    if button.clicked() {
        run_prepare(model, state, lernie, bl, inputs);
        return;
    }
    let Some(ball) = ball else {
        return;
    };
    let seat = Seat::BallRow {
        state: ball.state,
        assign_to: model.focused_ws_name(),
        move_to: model.move_targets(&ball.owner),
    };
    let target = Target::Ball(BallRef {
        project: ball.project,
        id: ball.id,
        owner: ball.owner,
    });
    super::menus::attach(&button, seat, &target, model, state, lernie, bl);
}

/// The (project, id, join state) of an existing-ball payload — the ball an
/// Assign or the §11 ball-row menu acts on; `None` for a new-ball payload
/// (nothing to assign yet).
/// The id of an existing-ball payload — the key the board indexes its
/// affordances by. Split from [`ball_ref`] because a key needs no repo, and
/// resolving one to build a key would be a lookup the caller cannot use.
pub(super) fn ball_id(payload: &Payload) -> Option<String> {
    match payload {
        Payload::Ball {
            ball: BallSpec::Existing { id, .. },
            ..
        } => Some(id.clone()),
        _ => None,
    }
}

pub(super) fn ball_ref(payload: &Payload) -> Option<(String, String, JoinState)> {
    match payload {
        Payload::Ball {
            project,
            ball: BallSpec::Existing { id, join, .. },
        } => Some((project.clone(), id.clone(), *join)),
        _ => None,
    }
}

/// The Start-button label for a ball-rung entry: `<id>: <title>` (existing) or
/// the title (new). Bare/path rungs are not offered here (they are the input
/// bar's Enter and Z4's picker).
fn start_label(inputs: &StartInputs) -> String {
    match &inputs.payload {
        Payload::Ball {
            ball: BallSpec::Existing { id, title, .. },
            ..
        } => format!("{id}: {title}"),
        Payload::Ball {
            ball: BallSpec::New { title, .. },
            ..
        } => title.clone(),
        Payload::Bare => "(new conversation)".to_owned(),
        Payload::Path { dir } => dir.display().to_string(),
    }
}

/// A per-project new-ball form (§8.1): title + body RAM drafts and a
/// Create-&-Start button that mints the ball and enters the start flow.
///
/// Headed by the project's `label`, not its path (§11, bl-ac3d): an
/// `egui::CollapsingHeader` lays its text `TextWrapMode::Extend` whatever the
/// panel's own wrap mode says, so this one row escaped the bl-9669 truncation
/// and sized the whole left column to an absolute path. The full path stays on
/// hover, and the header is keyed by the **path** rather than by its text, so
/// two projects can never share fold state however their labels elide.
pub(super) fn new_ball_form(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    bl: &Cli,
    project: &Path,
    label: &str,
) {
    let (mut title, mut body) = state
        .start
        .new_ball
        .get(project)
        .cloned()
        .unwrap_or_default();
    let mut create = false;
    egui::CollapsingHeader::new(format!("+ new ball · {label}"))
        .id_salt(project)
        .show(ui, |ui| {
            // Hints from the covered [`crate::actions::new_ball_hints`]
            // (bl-b2ed) — empty, the two boxes were indistinguishable.
            let hints = crate::actions::new_ball_hints();
            ui.add(egui::TextEdit::singleline(&mut title).hint_text(hints.title))
                .on_hover_text(
                    "The new ball's title — the one line it is listed by. Typed, it is \
                     the words before any flag in `/create <title…>`.",
                );
            ui.add(egui::TextEdit::multiline(&mut body).hint_text(hints.body))
                .on_hover_text(
                    "The new ball's body — the task written out as the agent will read \
                     it. Typed, it is `/create <title…> --body <text…>`.",
                );
            create = ui
                .add_enabled(
                    crate::actions::create_ball_enabled(&title),
                    egui::Button::new("Create & Start"),
                )
                .on_hover_text(
                    "File this ball in the project (`bl create`), claim it for the focused \
                     workspace, and open its goal. Nothing is sent until Send. Typed, it \
                     is `/create <title…>` then `/prepare ball`.",
                )
                .on_disabled_hover_text("give the ball a title first")
                .clicked();
        })
        .header_response
        .on_hover_text(format!(
            "Fold open a form for filing a brand-new ball in {}. No key of its own: Tab \
             reaches it, Space presses it — and `/create <title…>` files one without \
             the form.",
            project.display()
        ));
    if create {
        let inputs = model.new_ball_inputs(project, &title, &body);
        state.start.new_ball.remove(project);
        run_prepare(model, state, lernie, bl, inputs);
    } else {
        state
            .start
            .new_ball
            .insert(project.to_path_buf(), (title, body));
    }
}
