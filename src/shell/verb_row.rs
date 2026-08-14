//! The composer's verb row (§8.2, split from `input_bar` at the §12 cap):
//! the `→ message <name>` target line (bl-2f30), Send — Message to the
//! selected agent or New prompt into the focused workspace — and Stop
//! (+children) for a selected live agent. One body per verb, shared with the
//! §11 key bindings and the row menus through [`super::dispatch`]. Scan lives
//! on the Inbox tab ([`super::inspector::tabs_and_content`]), not here — it
//! flushes the workspace's inbox rather than acting on the composer's target.
//!
//! **A draft that starts with `/` is a command** (§8.5): [`super::slash`] is
//! that seat — no new control (bl-8aab), one re-labelled button.

use crate::AppModel;
use crate::actions::{DraftKey, message_enabled, new_prompt_enabled};
use crate::boundary::line;
use crate::cli_outbound::Cli;
use std::path::PathBuf;

use super::ShellState;

/// `Stop` (§8.2): the driver dies, the work stays.
const STOP_HINT: &str = "Kill the driver running this conversation (`lernie stop`). Everything it \
     has already committed is kept; you can message it again afterwards (x).";

/// The per-frame verb context, bundled (owned — no borrow rides a struct, per
/// the no-named-lifetimes rule) to keep the fns under the argument cap.
pub(super) struct VerbCtx {
    pub ws: PathBuf,
    /// Whether §8.2's `Stop` is offered on the composer's target, and whether
    /// the `+children` cascade rides beside it — the two gates off the §11
    /// seat's own view (REMOTE §9.4, bl-1eb0). They were re-derived here from
    /// the frame's agent set; a seat holding no tree cannot do that, and both
    /// answers were already the boundary's.
    pub stoppable: bool,
    pub stop_children: bool,
    /// Whether the world carries the target at all (§8.2's message gate's
    /// roster half) and whether a nudge is offered on it — the same seat view's
    /// facts, for the same reason.
    pub present: bool,
    pub nudgeable: bool,
    /// The parked invocation the §8.6 answer controls act on (`None` for every
    /// conversation nothing is holding, which is nearly all of them).
    pub held: Option<crate::control::hold::Held>,
    /// The draft this composer is composing (bl-a69a) — so a clean send clears
    /// the target it deposited to and no other.
    pub key: DraftKey,
    pub text: String,
    /// The target's §3.3 display name — the `→ message <name>` line's fact
    /// (bl-2f30), painted at the verb row's head so the fold line directly
    /// bounds the pending queue above it.
    pub conv_name: Option<String>,
    pub entered: bool,
}

/// Send (Enter or click; message when an agent is selected, else new
/// conversation) and Stop (+children) for a selected live agent. Scan lives on
/// the Inbox tab ([`super::inspector::tabs_and_content`]), not here — it flushes
/// the workspace's inbox rather than acting on the composer's target.
pub(super) fn verb_buttons(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    bl: &Cli,
    ctx: &VerbCtx,
) {
    // The line seat (§8.5): a drafted command re-labels the one button and takes
    // Enter, whatever the selection is — the gesture says its own target.
    if super::slash::seat(ui, model, state, lernie, bl, ctx) {
        return;
    }
    let selected = state.actions.selected_branch.clone();
    // `//…` said a slash and meant it (§8.5): the escape is shed here, the one
    // place a draft becomes something a model reads.
    let said = line::unescape(&ctx.text);
    // A **strip of peers** (§11 rule 8, [`super::row::peers`]): every verb here
    // is a control of its own natural width and none of them may be lost, so the
    // row wraps to a second line rather than squeezing the last one. Laid flat
    // it did both of the things rule 8 and rule 1b each forbid — at 420x320
    // `Stop` was clipped to 16 of its 25 points by the pane's edge, and once
    // rule 1 reached the centre the same squeeze truncated it to a bare `…`
    // (bl-5410, the two halves of the same loss to the operator).
    super::row::peers(ui, |ui| {
        // The `→ message <name>` target line (bl-2f30): the box's hint's twin
        // spelling, at the row's head — below the queue, so nothing sits
        // between the fold line and the pending items it bounds.
        if let Some(name) = ctx.conv_name.as_deref() {
            ui.weak(format!("→ message {name}"));
        }
        if let Some(agent) = selected.as_deref() {
            let msg_on = message_enabled(ctx.present, &ctx.text);
            let send = ui
                .add_enabled(msg_on, egui::Button::new("Message"))
                .on_hover_text(
                    "Deposit this text in the selected conversation's inbox and wake its \
                     driver so it reads it (`lernie message`). Enter sends it; typed \
                     whole, it is `/message <text…>`.",
                )
                .on_disabled_hover_text("type something first — an empty message sends nothing")
                .clicked();
            if msg_on && (send || ctx.entered) {
                // A send keeps the keyboard (§11 focus discipline): the click
                // never had it (the multiline box holds it through Enter), so
                // without this the operator's next message starts with a hunt
                // for the box. Asked on the attempt, not on the outcome — a
                // refused send leaves the draft in place to be fixed and
                // re-sent.
                super::focus::request(state);
                // The draft is RAM until *sent*: clear only on a clean
                // deposit (§5.3).
                let cleared = super::dispatch::message(model, lernie, bl, &ctx.ws, agent, &said);
                if cleared {
                    state.actions.drafts.set(ctx.key.clone(), String::new());
                }
            }
            nudge_control(ui, model, lernie, bl, ctx, agent);
            hold_controls(ui, model, lernie, bl, ctx, agent);
            stop_controls(ui, model, state, lernie, bl, ctx);
        } else {
            // Armed by both halves (bl-6191): something to say, and a work
            // directory the start can actually run in. The birth block's field
            // carries the red sentence saying which one refused.
            let on = new_prompt_enabled(&ctx.text, &state.actions.path_dir);
            let send = ui
                .add_enabled(on, egui::Button::new("New prompt"))
                .on_hover_text(
                    "Start a new conversation in this workspace from this text — a \
                     detached `lernie prompt` that keeps running whatever yog does. \
                     Enter starts it; typed whole, it is `/prompt <goal…>`.",
                )
                .on_disabled_hover_text("type something first — an empty prompt starts nothing")
                .clicked();
            if on && (send || ctx.entered) {
                super::focus::request(state);
                // The bare / path rung (§3.4): Enter → prompt into the focused
                // workspace. A non-empty dir fires the path rung; the whole
                // flow rides the one planner (§8.1), read back below.
                if super::fire::fire_start(model, state, lernie, bl, &said) {
                    // Only the draft clears (§5.3), and only **this** target's
                    // (bl-a69a). The work directory survives a send: it is a
                    // birth parameter the block states, not a message being
                    // composed, and an operator working in one tree starts
                    // their next conversation there too (bl-7927).
                    state.actions.drafts.set(ctx.key.clone(), String::new());
                }
            }
        }
    });
}

/// **Nudge** (§8.2, bl-9bef): run the model on this conversation as it stands.
/// It sits beside Message rather than replacing it because the two say
/// different things — Message adds a turn, this one adds nothing and is exactly
/// what a first turn that died before reaching the model needs: sign in, press
/// it, and the same conversation continues with the goal it was born with.
///
/// It is deliberately **independent of the draft**: a nudge takes no text, so
/// arming it on the box would make an empty composer mean "cannot re-dispatch".
fn nudge_control(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    lernie: &Cli,
    bl: &Cli,
    ctx: &VerbCtx,
    agent: &str,
) {
    let on = ctx.nudgeable;
    if ui
        .add_enabled(on, egui::Button::new("Nudge"))
        .on_hover_text(
            "Run the model on this conversation as it already stands — no new message, no \
             goal retyped (`lernie advance`). It is the re-dispatch for a first turn that \
             died before it reached the model: sign in, press this, and the same \
             conversation carries on. Typed, it is `/nudge`.",
        )
        .on_disabled_hover_text(
            "something is already running this conversation — a nudge would find the \
             driver's lease taken and do nothing",
        )
        .clicked()
    {
        super::dispatch::nudge(model, lernie, bl, &ctx.ws, agent);
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
/// result, which it reads and steps past (lernie bl-b98d — a stop here would
/// wedge the branch for good).
fn hold_controls(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    lernie: &Cli,
    bl: &Cli,
    ctx: &VerbCtx,
    agent: &str,
) {
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
        super::dispatch::answer_hold(model, lernie, bl, &ctx.ws, agent, Ruling::Pass);
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
        super::dispatch::answer_hold(model, lernie, bl, &ctx.ws, agent, Ruling::Refuse);
    }
}

/// Stop (+ the children cascade when descendants exist) for the selected agent.
/// Both gates come off the §11 seat's own view (REMOTE §9.4, bl-1eb0) rather
/// than being re-derived here against a roster; the caller reaches this only
/// with a selection, which is what those gates are about.
fn stop_controls(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    bl: &Cli,
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
        super::dispatch::stop_selected(model, state, lernie, bl);
    }
}
