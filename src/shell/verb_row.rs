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
//!
//! Here are the verbs that **spend the draft** — Message, New prompt,
//! Interrupt. The ones that take no text at all and act on the conversation as
//! it already stands — Nudge, the §8.6 hold answer, Stop — are `verb_row/standing`.

use crate::AppModel;
use crate::actions::{DraftKey, message_enabled, new_prompt_enabled};
use crate::boundary::line;
use crate::cli_outbound::Cli;
use std::path::PathBuf;

use super::ShellState;

mod standing;

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
    /// Ctrl+Enter, the box's third Enter (bl-a33d): send-and-interrupt. Its own
    /// bit beside `entered` rather than a modifier read here — the box owns the
    /// key family, and this row only spends what it decided.
    pub interrupted: bool,
}

/// Send (Enter or click; message when an agent is selected, else new
/// conversation) and Stop (+children) for a selected live agent. Scan lives on
/// the Inbox tab ([`super::inspector::tabs_and_content`]), not here — it flushes
/// the workspace's inbox rather than acting on the composer's target.
pub(super) fn verb_buttons(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    litany: &Cli,
    bl: &Cli,
    ctx: &VerbCtx,
) {
    // The line seat (§8.5): a drafted command re-labels the one button and takes
    // Enter, whatever the selection is — the gesture says its own target.
    if super::slash::seat(ui, model, state, litany, bl, ctx) {
        return;
    }
    let selected = state.actions.selected_branch.clone();
    // **Whether a start may be fired at all** (§3.4, bl-56c6): not while one is
    // still in flight. The invariant is `acting::start::staging`'s — this is the
    // same fact painted, so the operator sees a control that is not offered
    // rather than pressing one that does nothing.
    let quiet = state.acting.is_none();
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
                     driver so it reads it (`litany message`). Enter sends it; typed \
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
                // The draft is RAM until *sent*: it clears on a clean deposit
                // and no sooner (§5.3), which over the wire is a fact the
                // **receipt** carries — so the clear rides the ticket this post
                // holds (REMOTE §9.8, bl-1747) rather than a reply read here.
                super::dispatch::message(model, state, &ctx.key, &ctx.ws, agent, &said);
            }
            interrupt_control(ui, model, state, ctx, agent, &said, msg_on);
            standing::nudge_control(ui, model, ctx, agent);
            standing::hold_controls(ui, model, ctx, agent);
            standing::stop_controls(ui, model, state, ctx);
        } else {
            // Armed by both halves (bl-6191): something to say, and a work
            // directory the start can actually run in. The birth block's field
            // carries the red sentence saying which one refused.
            let on = quiet && new_prompt_enabled(&ctx.text, &state.actions.path_dir);
            let send = ui
                .add_enabled(on, egui::Button::new("New prompt"))
                .on_hover_text(
                    "Start a new conversation in this workspace from this text — a \
                     detached `litany prompt` that keeps running whatever yog does. \
                     Enter starts it; typed whole, it is `/prompt <goal…>`.",
                )
                .on_disabled_hover_text(
                    "type something first — an empty prompt starts nothing; and a start \
                     already in flight has to land before a second one may be fired",
                )
                .clicked();
            if on && (send || ctx.entered) {
                super::focus::request(state);
                // The bare / path rung (§3.4): Enter → prompt into the focused
                // workspace. A non-empty dir fires the path rung; the whole
                // flow rides the one planner (§8.1), read back below.
                // Only the draft clears (§5.3), and only **this** target's
                // (bl-a69a) — carried on the ticket, because the launch is two
                // posted acts and the box empties when the second one lands.
                // The work directory survives a send: it is a birth parameter
                // the block states, not a message being composed, and an
                // operator working in one tree starts their next conversation
                // there too (bl-7927).
                super::fire::fire_start(model, state, &ctx.key, &said);
            }
        }
    });
}

/// **Send and interrupt** (§8.2, bl-a33d): the button beside Message, and the
/// click half of Ctrl+Enter. Armed by the **same** gate Message is — something
/// to say, and a conversation the world carries — and deliberately *not* by
/// `stoppable`: a stop with nothing in flight is declined in band and the text
/// still lands, so the operator never has to know whether a driver is up before
/// deciding to cut in. That is one gate for two buttons rather than a second
/// rule to keep in step.
fn interrupt_control(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    ctx: &VerbCtx,
    agent: &str,
    said: &str,
    armed: bool,
) {
    let clicked = ui
        .add_enabled(armed, egui::Button::new("Interrupt"))
        .on_hover_text(
            "Cut this conversation off mid-work and send it this text instead: it stops what is \
             running and then deposits the message (`litany stop`, then `litany message`), and \
             the deposit is what starts it going again. Work already committed is kept, and a \
             tool call cut off mid-flight is reported to the model as having produced no result. \
             Ctrl+Enter does the same; typed whole, it is `/interrupt <text…>`.",
        )
        .on_disabled_hover_text("type something first — an interrupt with nothing to say is a stop")
        .clicked();
    if armed && (clicked || ctx.interrupted) {
        // A send keeps the keyboard, exactly as Message's does (§11 focus
        // discipline): the whole point of this gesture is the next thing you
        // type.
        super::focus::request(state);
        super::dispatch::interrupt(model, state, &ctx.key, &ctx.ws, agent, said);
    }
}
