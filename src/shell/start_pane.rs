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

use super::ShellState;
use crate::AppModel;
use crate::start::{self, StartInputs};
use litany::mint::SplitMix64;

/// Post `prepare` (seed?/new?/create?/claim?/compose). Its **receipt** focuses
/// the workspace it resolved (§3.4) and opens the composer on the goal it
/// composed — both frames later, since the act crosses the wire (REMOTE §9.8,
/// bl-1747), and both in one place ([`super::acting::start`]) so a rung cannot
/// forget one of them. The affected project's balls and the ops tail re-derive
/// off the same receipt, through the act's own root, so nothing is marked here
/// either. `pub(super)`: the tab bar's `new` form ([`super::new_ws`]) rides the
/// same path.
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
pub(super) fn run_prepare(model: &mut AppModel, state: &mut ShellState, inputs: StartInputs) {
    super::acting::start::prepare(model, state, &inputs);
}

/// The editable goal composer (§8.1, §3.3): the greyed name prediction, the
/// editable payload prefill, then Send fires `litany prompt` detached (the
/// conversation name minted at fire and passed via `--name`, `YOG_NAME`
/// layered; the goal fires verbatim, bl-6920); Cancel drops
/// the draft. The preview draws off the held seed and the target workspace's
/// occupied names — the same two inputs Send re-derives from, so it predicts.
///
/// Dismissing the pane — a clean fire, or Cancel — hands the keyboard back to
/// the message composer beneath it (§11 focus discipline). A failed launch
/// keeps the pane and the edited goal, so it is not a dismissal.
///
/// **The pending start is taken for the body and put back at the end** — one
/// borrow of `state`, not a live one held across the paint, which is what lets
/// the §8.1 provider gate ([`super::start_login::gate`], bl-1fd0) and §11's one
/// focus door ([`super::focus::take`]) both take `state` inside. The take is
/// invisible from outside: nothing between the two lines returns, and the only
/// frame the field is `None` is this one, mid-call.
///
/// **This box is a composer like the other two** (§11: one box, one Enter), so
/// it consumes the focus request rather than leaving the keyboard nowhere — it
/// IS the goal box while it is up, and the message composer beneath it is not
/// painted at all.
pub fn composer(ui: &mut egui::Ui, model: &mut AppModel, state: &mut ShellState) {
    let mint_seed = state.start.mint_seed;
    // The start's FIRST rung (§8.1, bl-1fd0), whose roster is docked as its own
    // band above this box ([`super::pane`]). Only the verb's half of it is
    // read here: what the target wall can run, and therefore whether Send is a
    // fire or a sentence.
    let gate = super::start_login::gate(model, state);
    let Some(mut pending) = state.start.pending.take() else {
        return;
    };
    let (mut send, mut cancel) = (false, false);
    // The §3.3 occupied set off the landed forest (bl-b4b5): every member's own
    // stored name, which is what the mint may not re-use. A frame the engine
    // has not answered predicts against an empty set — the same reading a
    // workspace with no conversations gives, and the mint's own `--name` is
    // re-derived at fire where the refusal actually lives.
    let names = crate::nav::convs::names_in_rows(
        &super::convs::of(model, pending.workspace.clone())
            .value
            .unwrap_or_default(),
    );
    ui.weak(start::identity_preview(
        &names,
        &SplitMix64::from_seed(mint_seed),
    ));
    ui.label(format!("Start goal → {}", pending.workspace));
    let mut fan_n = state.start.fan_n.max(1);
    // The §3.8 fan's N picker (bl-77bc), offered only where a fan can aim: the
    // focused ball's delivery obligation, the same context read `/fan` makes.
    // ×1 is the ordinary single start — one code path, counted.
    let ctx = model.line_context();
    if ctx.project.is_some() && ctx.ball.is_some() {
        const N_HOVER: &str = "How many isolated candidates to fire on this ball's target. Each writes in \
             its own attempt worktree off one shared base; delivering one advances the \
             ball's branch and the rest must rework. ×1 is the ordinary single start. \
             Typed, it is `/fan <n>` after a `/prepare`.";
        ui.horizontal(|ui| {
            if ui.small_button("−").on_hover_text(N_HOVER).clicked() {
                fan_n = fan_n.saturating_sub(1).max(1);
            }
            ui.weak(format!("×{fan_n} candidates"))
                .on_hover_text(N_HOVER);
            if ui.small_button("+").on_hover_text(N_HOVER).clicked() {
                fan_n += 1;
            }
        });
    }
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
            let edit = ui
                .add_sized(
                    [ui.available_width(), box_height],
                    egui::TextEdit::multiline(&mut pending.goal),
                )
                .on_hover_text(
                    "The goal this conversation starts with — the agent's first instruction. \
                 Edit it freely; it is only read when you press Send — or Enter, which \
                 is the same fire. Typed whole, it is `/prompt <goal…>`.",
                );
            super::focus::take(state, ui, &edit);
        });
    // Armed only by a goal that says something (bl-9acf) — the same predicate
    // [`send_pending`] refuses on, so the disabled button and the inert Enter
    // are one rule wearing two faces rather than a check the pointer can dodge.
    // …and by a wall that can reach a model at all (bl-1fd0). Two refusals, one
    // button: the gate's own sentence when the wall is the blocker, so Send
    // states the real reason instead of firing a start that dies on no-models.
    let refusal = gate.refusal();
    let armed = crate::actions::goal_present(&pending.goal) && refusal.is_none();
    ui.horizontal(|ui| {
        send = ui
            .add_enabled(armed, egui::Button::new("Send (detached prompt)"))
            .on_hover_text(
                "Launch the conversation: `litany prompt`, detached, in the workspace \
                 named above. It keeps running whatever yog does afterwards (Enter).",
            )
            .on_disabled_hover_text(refusal.unwrap_or_else(|| {
                "The goal above is empty — say what you want done before this can \
                 launch anything."
                    .to_owned()
            }))
            .clicked();
        cancel = ui
            .button("Cancel")
            .on_hover_text(
                "Drop this goal without launching anything. A ball claimed on the way \
                 here stays claimed. Escape does the same.",
            )
            .clicked();
    });
    state.start.fan_n = fan_n;
    state.start.pending = Some(pending);
    if send {
        send_pending(model, state);
    }
    if cancel {
        state.start.pending = None;
        state.start.fan_n = 1;
        super::focus::request(state);
    }
}

/// Fire the pending start goal as a detached `litany prompt` (§8.1) — the Send
/// button's body, shared with the §11 Enter binding. Nothing pending is a
/// no-op.
///
/// **It reads nothing back** (REMOTE §9.8, bl-1747): the act is posted and the
/// pane, the goal and the §3.3 seed all stand until the receipt says the engine
/// launched something. A failed launch therefore keeps the composer open with
/// the edited goal by doing nothing at all, and the §11 focus hand-back rides
/// the same receipt — neither a retry-able failure nor an empty press moves the
/// keyboard.
///
/// **A blank goal is not a goal** ([`goal_present`](crate::actions::goal_present),
/// bl-9acf): the pending start is *read* rather than taken here, so a blank
/// draft stays standing with the cursor in it instead of spawning `litany
/// prompt` with the identity preamble and nothing after it. The guard lives
/// here, not only on the button, because the §11 Enter binding is the other
/// hand on the same trigger.
pub(super) fn send_pending(model: &mut AppModel, state: &mut ShellState) {
    // **A wall that cannot reach a model is not a goal either** (bl-1fd0). The
    // guard lives here as well as on the button for `goal_present`'s own
    // reason: §11's Enter is the other hand on the same trigger, and a start
    // fired past the rung is a conversation born to die on no-models with the
    // operator's first typed goal inside it.
    if super::start_login::gate(model, state).refusal().is_some() {
        return;
    }
    let Some(p) = state
        .start
        .pending
        .clone()
        .filter(|p| crate::actions::goal_present(&p.goal))
    else {
        return;
    };
    // The boundary's Prompt action (§8.5), carrying the seat's own §3.3 seed;
    // the §3.4 start claim and the seed's retirement (bl-28ba) both ride its
    // receipt — one rule, and every hand that fires a start spends it there.
    // With N > 1 picked, the same Enter is the §3.8 fan (bl-77bc): one
    // `Fan(Spread)` whose receipt walks the rebound starts back through this
    // very door, so the ceiling gates every birth exactly as it gates one.
    // **The act is addressed by the name the prepare came back with, never by
    // a path** (REMOTE §8.2, bl-e349): the poster routes `Prompt` by
    // `prepared.workspace`, so a start on a workspace a §8.2 entry hosts goes
    // down that entry's channel and is founded, claimed and fired on its HOST.
    // The path beside it is this box's own, for the frame-side folds alone, and
    // for a remote workspace there simply is not one — `None`, and each fold
    // skips its local optimism.
    //
    // It used to be `ws_path(…).unwrap_or_default()`, which flattened *"this
    // box has no directory for that name"* into `PathBuf::new()` and fired
    // anyway. The empty path reached no spawn — the act carries the name, not
    // the path — but it reached every fold that keys on one: the §3.4 start
    // claim was taken at `""`, which matches no workspace, so the conversation
    // this fire started got no row and no focus and the operator watched a
    // fire land on nothing. The phantom `home` in the same report was the
    // *other* half, one rung up: `start_workspace` read the focused PATH,
    // which is `None` for a workspace an entry hosts exactly as for no focus
    // at all, so the act itself was addressed `home` — a local name, raised
    // locally, and the goal ran there.
    let ws = model.start_path(&p.workspace);
    if state.start.fan_n > 1 {
        super::acting::fan::fan(model, state, ws.as_deref(), &p, state.start.fan_n);
        return;
    }
    super::acting::start::prompt(model, state, ws.as_deref(), &p, &p.goal);
}
