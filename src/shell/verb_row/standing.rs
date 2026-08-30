//! The composer's **standing** verbs (§8.2), split off [`super`] at §12's
//! budget on the seam the row itself is laid out along: Message, New prompt and
//! Interrupt all spend the draft — they take what is in the box and send it —
//! while Nudge, the §8.6 hold answer and Stop take no text at all and act on
//! the selected conversation exactly as it already stands. A textless verb is
//! gated by the seat's own view of the conversation (REMOTE §9.4, bl-1eb0) and
//! never by the box, which is why the two families arm on different facts.

use crate::AppModel;

use super::{ShellState, VerbCtx};

/// `Stop` (§8.2): the driver dies, the work stays.
const STOP_HINT: &str = "Kill the driver running this conversation (`litany stop`). Everything it \
     has already committed is kept; you can message it again afterwards (x).";

/// **Nudge** (§8.2, bl-9bef): run the model on this conversation as it stands.
/// It sits beside Message rather than replacing it because the two say
/// different things — Message adds a turn, this one adds nothing and is exactly
/// what a first turn that died before reaching the model needs: sign in, press
/// it, and the same conversation continues with the goal it was born with.
///
/// It is deliberately **independent of the draft**: a nudge takes no text, so
/// arming it on the box would make an empty composer mean "cannot re-dispatch".
pub(super) fn nudge_control(ui: &mut egui::Ui, model: &mut AppModel, ctx: &VerbCtx, agent: &str) {
    let on = ctx.nudgeable;
    if ui
        .add_enabled(on, egui::Button::new("Nudge"))
        .on_hover_text(
            "Run the model on this conversation as it already stands — no new message, no \
             goal retyped (`litany advance`). It is the re-dispatch for a first turn that \
             died before it reached the model: sign in, press this, and the same \
             conversation carries on. Typed, it is `/nudge`.",
        )
        .on_disabled_hover_text(
            "something is already running this conversation — a nudge would find the \
             driver's lease taken and do nothing",
        )
        .clicked()
    {
        crate::shell::dispatch::nudge(model, &ctx.ws, agent);
    }
}

/// The §8.6 hold answer, offered **only while the selected conversation is
/// actually parked** — a park is a fact the snapshot carries, so the controls
/// appear with it and vanish with it rather than sitting inert. Two buttons and
/// no modal: attended and unattended are one flow (VISION §4.11 item 5), and
/// `hold` — keep it parked — needs no button, because not pressing either one
/// *is* keeping it parked; the line spells it for the case where pinning the
/// park across a policy edit is the point.
///
/// Neither button stops anything: a decline is the model's own in-band tool
/// result, which it reads and steps past (litany bl-b98d — a stop here would
/// wedge the branch for good).
pub(super) fn hold_controls(ui: &mut egui::Ui, model: &mut AppModel, ctx: &VerbCtx, agent: &str) {
    use crate::control::judge::Ruling;
    let Some(held) = ctx.held.as_ref() else {
        return;
    };
    // The control's own sentence — the tool, what it was about to do, the class
    // it landed in and the evidence — beside the buttons that answer it, so the
    // decision and its grounds are never two places.
    ui.weak("⏸ held").on_hover_text(held.reason.clone());
    if ui
        .button("Approve")
        .on_hover_text(
            "Let this one parked tool call through and drive the conversation on. \
             The approval is scoped to this exact call — the next one is judged afresh. \
             Typed, it is `/answer pass`.",
        )
        .clicked()
    {
        crate::shell::dispatch::answer_hold(model, &ctx.ws, agent, Ruling::Pass);
    }
    if ui
        .button("Decline")
        .on_hover_text(
            "Refuse this one parked tool call and drive the conversation on. The model \
             is told why and carries on from there — nothing is stopped or killed. \
             Typed, it is `/answer refuse`.",
        )
        .clicked()
    {
        crate::shell::dispatch::answer_hold(model, &ctx.ws, agent, Ruling::Refuse);
    }
}

/// Stop (+ the children cascade when descendants exist) for the selected agent.
/// Both gates come off the §11 seat's own view (REMOTE §9.4, bl-1eb0) rather
/// than being re-derived here against a roster; the caller reaches this only
/// with a selection, which is what those gates are about.
pub(super) fn stop_controls(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    ctx: &VerbCtx,
) {
    if ctx.stop_children {
        ui.checkbox(&mut state.actions.stop_children, "children")
            .on_hover_text(
                "Stop the agents this conversation spawned too, not only the one at \
                 its root. Typed, it is `/stop children`.",
            );
    }
    if ui
        .add_enabled(ctx.stoppable, egui::Button::new("Stop"))
        .on_hover_text(STOP_HINT)
        .on_disabled_hover_text(
            "nothing is running on this conversation — there is no driver to kill",
        )
        .clicked()
    {
        crate::shell::dispatch::stop_selected(model, state);
    }
}
