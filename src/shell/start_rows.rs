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
//!
//! Here are the rows over a ball that already exists; the form that files one
//! that does not is `start_rows/new_ball`.

use super::ShellState;
use super::menus::{BallRef, Target};
use super::start_pane::run_prepare;
use crate::AppModel;
use crate::cli_outbound::Cli;
use crate::nav::menu::Seat;
use crate::projects::join::JoinState;
use crate::start::{BallSpec, Payload, StartInputs};
mod new_ball;
pub(super) use new_ball::new_ball_form;

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
    litany: &Cli,
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
        run_prepare(model, state, inputs);
        return;
    }
    let Some((project, id, join)) = ball else {
        return;
    };
    if assign_clicked && let Some(to) = &to {
        super::ball_bar::assign_ball(model, &project, &id, to);
    }
    let seat = Seat::BallRow {
        state: join,
        assign_to: to,
    };
    let target = Target::Ball(BallRef {
        project,
        id,
        owner: String::new(),
    });
    super::menus::attach(&start_button, seat, &target, model, state, litany);
}

/// One bound ball: ▶ Continue `<id>: <title>` into the ball's **own** claimant
/// workspace (§8.1 resume, addendum), seating the §11 ball-row menu — Release /
/// Close — on the button.
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
    litany: &Cli,
    inputs: StartInputs,
) {
    // The ball's **own** claimant workspace, not the focused one (a resume
    // reaches a wall the operator is not looking at), so the listing is asked
    // for that workspace by name — `nav::balls::bound` is then a selection out
    // of it, never a second derivation (REMOTE §9.7, bl-b4b5).
    let ws = model.snap.ws_name(&inputs.workspace);
    let rows = super::chrome::balls(model, &ws).value.unwrap_or_default();
    let ball =
        ball_ref(&inputs.payload).and_then(|(_, id, _)| crate::nav::balls::bound(&rows, &id));
    let button = ui
        .button(format!("▶ Continue {}", start_label(&inputs)))
        .on_hover_text(CONTINUE_HINT);
    if button.clicked() {
        run_prepare(model, state, inputs);
        return;
    }
    let Some(ball) = ball else {
        return;
    };
    let seat = Seat::BallRow {
        state: ball.state,
        assign_to: model.focused_ws_name(),
    };
    let target = Target::Ball(BallRef {
        project: ball.project,
        id: ball.id,
        owner: ball.owner,
    });
    super::menus::attach(&button, seat, &target, model, state, litany);
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
