//! The inbox-composer, docked at the bottom of the conversation pane (§11
//! inbox-composer, bl-929d over bl-c038's seat). Coverage-excluded glue: it
//! wires the queue region ([`super::inbox_queue`] — pending deposits above the
//! draft under the derived fold line) and the short-verb buttons to the tested
//! dispatchers ([`crate::actions::verbs`], [`crate::start`], the
//! [`crate::composer`] derivations) and enablement predicates,
//! then asks the model for the on-demand refresh so the operator sees the
//! outcome at once. The verb invocations themselves live in
//! [`super::dispatch`], shared with the §11 key bindings and the row menus.
//!
//! **The queue is the snapshot's** (§11: no frame-time IO): the pending items
//! are the snapshot's own pending listing, gathered by the off-thread
//! enumerate over the watched `inbox/` root — the frame renders it, adds no
//! read, and notices no arrival event. The cut is lernie's: pending is what
//! still sits in `inbox/<id>/`, crossed is what its delivery drain committed;
//! yog holds no membership claim.
//!
//! **The target follows the selection** (§11): a selected agent ⇒ Enter sends
//! a message (the resume gesture); none ⇒ Enter fires a new conversation (a
//! detached prompt into the focused workspace) — one box, one Enter, the
//! S0/S1 gesture. Shift+Enter is the box's own newline (bl-4515), so a draft
//! may span lines before either fires. The ball actions ride beside it; the dir (path-rung) field
//! does **not** — it moved to the §11 birth-config block at the top of the
//! center (bl-7927), because *where the next conversation runs* is a parameter
//! it is born with, not something being composed. One carrier, one fact: this
//! file only reads `actions.path_dir` through [`super::fire`].
//!
//! **The target is named, not identified** (§11, bl-2f30): the `→ message <x>`
//! line and the box's hint are two spellings of one fact — the §3.3 display-name
//! ladder ([`crate::nav::convs::display_name_of`]) over the selection's
//! *conversation root* ([`crate::nav::convs::root_of`]), since the selection may
//! be a descent child while the name belongs to the conversation. The message
//! still targets the selected agent; the raw id survives weak in the center
//! header, never here.
//!
//! No error is ever printed and dropped (INV-2): a spawn failure, a gate
//! failure, and a clean run all leave the same durable `ops.jsonl` line, and the
//! composer derives its ichor-red last-failure banner from it **every frame**
//! ([`AppModel::last_failure`], §7.3) — not at dispatch, which missed a detached
//! driver that died after the launch returned (bl-4895). The draft survives a failure —
//! it is RAM until *sent* (§5.3), so a draft only clears on a clean send.
//!
//! **The box remembers what you said, without storing it** (bl-f908): ↑ at its
//! top row pages back through this conversation's own operator turns, folded
//! here from the two seats already open — the pending listing and the
//! transcript the inspector memoizes per snapshot (§7.2), never a second
//! `messages/` read. The walk itself is [`crate::composer::Recall`]'s.
//!
//! **One box, many drafts** (bl-a69a): the box is one widget, but its buffer is
//! the *target's* — [`crate::actions::Drafts`] keyed by [`DraftKey`], read in
//! and written back every frame. A verb that re-labels itself with the selection
//! must re-key with it too, or a goal typed for a new conversation rides the
//! selection into `→ message <name>` and Enter deposits it on a stranger.

use crate::AppModel;
use crate::actions::DraftKey;
use crate::cli_outbound::Cli;
use crate::start;
use lernie::mint::SplitMix64;

use super::ShellState;
use super::inbox_queue;
use super::verb_row::{VerbCtx, verb_buttons};

/// The inbox-composer at the conversation pane's bottom (§11): the queue
/// region — pending deposits above the draft, under the derived fold line —
/// then the target line, the verb buttons, and the ball actions for the
/// focused workspace. `cap` is half the pane, the fold line's ceiling.
pub fn composer(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    bl: &Cli,
    cap: f32,
) {
    let Some(ws) = model.focused_workspace() else {
        return;
    };
    // The frame's roster in the form a seat can hold (REMOTE §9.4, bl-1eb0):
    // the §3.3 ladder's input for the pending headers' senders, and nothing
    // else — the target's own name rides its seat view below.
    let titles = model.agent_titles();
    // The selection bridge (§11): the conversation list picks the target — at
    // any depth, since bl-fa82 made a member a row of it; the composer never
    // grows its own picker.
    let seat = model.focused_conversation();
    state.actions.selected_branch = seat.as_ref().map(|s| s.agent_id.clone());
    let target = state.actions.selected_branch.clone();
    let mint_seed = state.start.mint_seed;
    // Derived ONCE for both spellings (§3.3, bl-2f30): the selection's
    // conversation root, then the one ladder — both folded into the seat's
    // `name` at the boundary, so this reads it rather than re-deriving it.
    let conv_name = seat.as_ref().map(|s| s.name.clone());

    if conv_name.is_none() {
        // A new conversation: the greyed identity preview (§3.3), stable
        // frame-to-frame off the held mint seed, above the box as ruled — a
        // new-conversation target has no inbox, so nothing ever sits between
        // the fold line and the box here. The work directory is **not** here
        // (bl-7927) — it is a birth *parameter*, not a draft, so it rides the
        // §11 birth-config block at the top with the model line.
        ui.weak(
            start::preview(
                &model.start_bare_inputs(),
                &SplitMix64::from_seed(mint_seed),
            )
            .preview,
        );
    }

    // The queue region (§11 inbox-composer): the target's pending deposits
    // oldest-first, then the draft as the queue's last item, at the derived
    // fold-line height. The draft is the **target's**, not the box's
    // (bl-a69a): the region reads this target's own text in and writes it
    // back, so switching the selection switches drafts. The pending listing
    // is the snapshot's (§5.1 #11) — the message target's inbox; a new
    // conversation has none, which is the same rule at zero items.
    let key = DraftKey::composer(Some(ws.clone()), target.clone());
    let pending = model.focused_pending();
    // What ↑ pages back through (bl-f908): the operator's own turns in this
    // conversation, derived from the two seats already open here — the
    // snapshot's pending listing above and the delivered transcript, which is
    // the inspector's own standing question and therefore **one ask** shared
    // with the chat pane (REMOTE §9.7, bl-13f9), never a second `messages/`
    // read. A new conversation has neither, and a question not yet answered is
    // the same derivation at zero items: the recall fills a round trip after
    // the chat it pages back through, which is the honest empty state and not a
    // case of its own. There is nowhere here to paint a refusal — the composer
    // is a box, not a pane — so one reads as no past turns, exactly as an
    // unanswered frame does.
    let prompts = target
        .as_deref()
        .map(|agent| {
            let tx = super::inspector::transcript(model, &ws, agent)
                .value
                .unwrap_or_default();
            crate::composer::prompts(&pending, &tx)
        })
        .unwrap_or_default();
    let queue = inbox_queue::QueueCtx {
        key: key.clone(),
        agent_id: target.clone(),
        pending,
        prompts,
        hint: match conv_name.as_deref() {
            Some(name) => format!("message {name}"),
            None => "start a conversation".to_owned(),
        },
        cap,
    };
    let before = state.actions.drafts.text(&key);
    let edit = inbox_queue::region(
        ui,
        &mut state.composer,
        &mut state.actions.drafts,
        &queue,
        &titles,
    );
    // A §8.5 line's answer belongs to the line that earned it: edit anything,
    // and what is on screen is about something you are no longer saying.
    if state.actions.drafts.text(&key) != before {
        state.slash = None;
    }
    // The §11 focus hand-off: take the keyboard on the requested frame, through
    // the one mechanism (`super::focus`) — launch, a pointer selection, a send,
    // and a dismissed modal all arrive here as the same bit.
    super::focus::take(state, ui, &edit);
    // Enter alone sends (bl-4515). Bare only: Shift+Enter was the widget's
    // newline (the region's box), and any other modified Enter is deliberately
    // inert here so a combo send (bl-a33d's Ctrl+Enter send-and-interrupt) can
    // land as its own arm without reworking this one.
    let entered =
        edit.has_focus() && ui.input(|i| i.modifiers.is_none() && i.key_pressed(egui::Key::Enter));
    let ctx = VerbCtx {
        ws,
        stoppable: seat.as_ref().is_some_and(|s| s.stoppable),
        stop_children: seat.as_ref().is_some_and(|s| s.stop_children),
        present: seat.as_ref().is_some_and(|s| s.present),
        nudgeable: seat.as_ref().is_some_and(|s| s.nudgeable),
        held: seat.and_then(|s| s.held),
        key: key.clone(),
        text: state.actions.drafts.text(&key),
        conv_name,
        entered,
    };
    verb_buttons(ui, model, state, lernie, bl, &ctx);

    // The line's own answer stays under the box that typed it. A **search**
    // answer does not: it is a whole-center surface, so since bl-1ca2 it is
    // the §11 Search tab focus, not a scroller growing out of the composer.
    super::slash::note_ui(ui, state);

    let join = model.focused_join().cloned();
    super::ball_bar::actions(ui, model, join.as_ref());
    // Derived per frame, never cached at dispatch (§7.3, bl-4895): a detached
    // driver that dies after launch lands in the model on a later sweep. Scoped
    // to this surface's own gestures (bl-48f8) — the message/stop verbs and the
    // bare/path-rung start Enter fires. A ▶ Start that failed in the roster
    // banners there, where it was offered (§11, bl-6ad8), not under a box the
    // operator was not using.
    if let Some(failure) = model.last_failure(crate::opslog::Origin::Conversation) {
        super::banner::failure_banner(ui, model, state, &failure);
    }
}
