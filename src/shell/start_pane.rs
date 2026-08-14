//! The start flow itself (DESIGN §3.4, §8.1, §11): `prepare` — the
//! mint/create/claim/ensure orchestration — and the editable goal composer whose
//! Send fires the detached prompt. Its entry affordances (the ▶ Start rows, the
//! new-ball form) are [`super::start_rows`]; the tab bar's `new` form and the
//! §11 Enter binding come in through the same two `pub(super)` doors.
//!
//! Coverage-excluded interaction glue: every decision — the startable set, the
//! plan, the mint/create/claim/ensure orchestration ([`start::prepare`]), the
//! detached prompt ([`start::execute_prompt`]) — lives in tested modules
//! (`AppModel`, `crate::start`). This tree only wires widgets. The ▶ Start /
//! Create-&-Start paths route through the ball rung; the bare rung is the input
//! bar's Enter ([`super::input_bar`]); the path rung's picker is Z4's.

use super::{ShellState, StartState};
use crate::AppModel;
use crate::cli_outbound::Cli;
use crate::start::{self, Payload, StartInputs};
use lernie::mint::SplitMix64;
use std::path::PathBuf;

/// Run `prepare` (seed?/new?/create?/claim?/compose) and, on success, open
/// the composer with its editable goal. The run goes through
/// [`AppModel::prepare_start`], so the workspace it resolved is the focused one
/// before the composer opens (§3.4) — the tab bar, the conversation list and the
/// bottom composer all name the start's own workspace. On either outcome refresh the affected
/// project's balls + the ops tail so the mutations and their log lines are
/// visible at once (§8.2). `pub(super)`: the tab bar's `new` form
/// ([`super::new_ws`]) rides the same path.
///
/// **A draft opens only when the rung composed one** (bl-9acf): §3.4's table
/// gives the ball and path rungs a prefill and the bare rung none, and a draft
/// box over nothing is not a lighter version of the flow — it is a second goal
/// box stacked on the docked composer, whose Send fired the identity preamble
/// and nothing else onto the wire. So the raise (the one bare rung that arrives
/// here) hands the keyboard to the composer it just re-aimed, which *is* its
/// goal box (§11: one box, one Enter). One predicate
/// ([`goal_present`](crate::actions::goal_present)), not a rung match — the
/// prefill's blankness is the fact, and the fire sites below read the same one.
pub(super) fn run_prepare(
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    bl: &Cli,
    inputs: StartInputs,
) {
    let project = payload_project(&inputs.payload);
    let ts = super::now_ts();
    let result = model.prepare_start(lernie, bl, &inputs.workspace, &inputs.payload, &ts);
    if let Ok(prepared) = result {
        if crate::actions::goal_present(&prepared.goal) {
            state.start.pending = Some(prepared);
        } else {
            super::focus::request(state);
        }
    }
    match project {
        Some(p) => model.after_bl_verb(&p),
        None => model.after_lernie_verb(),
    }
}

/// The ball rung's project (for the after-verb ball refresh); `None` for the
/// bare/path rungs, which mutate no ball.
fn payload_project(payload: &Payload) -> Option<PathBuf> {
    match payload {
        Payload::Ball { project, .. } => Some(project.clone()),
        Payload::Bare | Payload::Path { .. } => None,
    }
}

/// The editable goal composer (§8.1, §3.3): the greyed name prediction, the
/// editable payload prefill, then Send fires `lernie prompt` detached (the
/// conversation name minted at fire and passed via `--name`, `YOG_NAME`
/// layered; the goal fires verbatim, bl-6920); Cancel drops
/// the draft. The preview draws off the held seed and the target workspace's
/// occupied names — the same two inputs Send re-derives from, so it predicts.
///
/// Dismissing the pane — a clean fire, or Cancel — hands the keyboard back to
/// the message composer beneath it (§11 focus discipline). A failed launch
/// keeps the pane and the edited goal, so it is not a dismissal.
pub fn composer(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    bl: &Cli,
) {
    let mint_seed = state.start.mint_seed;
    let Some(pending) = state.start.pending.as_mut() else {
        return;
    };
    let (mut send, mut cancel) = (false, false);
    ui.weak(start::identity_preview(
        &model.conversation_names(&pending.workspace),
        &SplitMix64::from_seed(mint_seed),
    ));
    ui.label(format!("Start goal → {}", pending.workspace.display()));
    // The goal box fills the pane the operator sized (§4.1 `panels`), less the
    // Send/Cancel row below it. The reservation is the style's own row height
    // (so it scales with the §4.1 zoom) plus a spacing, and it deliberately
    // reserves a little MORE than the row needs: under-filling leaves a few
    // points of blank pane, while over-filling would ratchet the panel taller
    // every frame — the shell pins the panel's height, so only overflow hurts.
    let row = ui.spacing().interact_size.y + 2.0 * ui.spacing().item_spacing.y;
    let box_height = (ui.available_height() - row).max(row);
    egui::ScrollArea::vertical()
        .max_height(box_height)
        .show(ui, |ui| {
            ui.add_sized(
                [ui.available_width(), box_height],
                egui::TextEdit::multiline(&mut pending.goal),
            )
            .on_hover_text(
                "The goal this conversation starts with — the agent's first instruction. \
                 Edit it freely; it is only read when you press Send — or Enter, which \
                 is the same fire. Typed whole, it is `/prompt <goal…>`.",
            );
        });
    // Armed only by a goal that says something (bl-9acf) — the same predicate
    // [`send_pending`] refuses on, so the disabled button and the inert Enter
    // are one rule wearing two faces rather than a check the pointer can dodge.
    let armed = crate::actions::goal_present(&pending.goal);
    ui.horizontal(|ui| {
        send = ui
            .add_enabled(armed, egui::Button::new("Send (detached prompt)"))
            .on_hover_text(
                "Launch the conversation: `lernie prompt`, detached, in the workspace \
                 named above. It keeps running whatever yog does afterwards (Enter).",
            )
            .on_disabled_hover_text(
                "The goal above is empty — say what you want done before this can \
                 launch anything.",
            )
            .clicked();
        cancel = ui
            .button("Cancel")
            .on_hover_text(
                "Drop this goal without launching anything. A ball claimed on the way \
                 here stays claimed. Escape does the same.",
            )
            .clicked();
    });
    if send && send_pending(model, &mut state.start, lernie, bl) {
        super::focus::request(state);
    }
    if cancel {
        state.start.pending = None;
        super::focus::request(state);
    }
}

/// Fire the pending start goal as a detached `lernie prompt` (§8.1) — the Send
/// button's body, shared with the §11 Enter binding. The spawn (success or
/// failure) rode its own ops line; the banner reads it back. A failed launch
/// keeps the composer open with the edited goal so the operator can retry (RAM
/// until sent). Nothing pending is a no-op.
///
/// Returns whether the pane closed — a clean launch. That is the same edge the
/// composer reports, so the §11 focus hand-back reads one fact whichever hand
/// fired it, and neither a retry-able failure nor an empty press moves focus.
///
/// **A blank goal is not a goal** ([`goal_present`](crate::actions::goal_present),
/// bl-9acf): it is taken only if it fires, so a blank draft stays standing with
/// the cursor in it instead of spawning `lernie prompt` with the identity
/// preamble and nothing after it. The guard lives here, not only on the button,
/// because the §11 Enter binding is the other hand on the same trigger.
pub(super) fn send_pending(
    model: &mut AppModel,
    start: &mut StartState,
    lernie: &Cli,
    bl: &Cli,
) -> bool {
    let Some(p) = start
        .pending
        .take_if(|p| crate::actions::goal_present(&p.goal))
    else {
        return false;
    };
    // The boundary's Prompt action (§8.5); the §3.4 start claim — the same
    // one the bare rung's fire makes ([`super::fire`]) — rides its success and
    // retires the same spent seed (bl-28ba): one rule, both hands. A failed
    // launch below keeps the pane, the goal AND the seed — nothing was minted,
    // so that prediction still stands.
    let fired = model.fire_prompt(lernie, bl, &p, &p.goal, start.mint_seed, &super::now_ts());
    model.after_lernie_verb();
    let ok = fired.is_ok();
    if ok {
        start.spend_mint();
    } else {
        start.pending = Some(p);
    }
    ok
}
