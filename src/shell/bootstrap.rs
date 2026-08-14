//! The empty-world placeholder + bootstrap composer (§3.4, STORIES S0) — the
//! center panel's other half, painted when no workspace is focused at all.
//! A separate surface from [`super::workspace`]'s conversation view: it has no
//! conversation, no inspector and no bottom composer to defer to, so it carries
//! its own box and its own Enter, both riding the one planner (`super::fire`).
//! Coverage-excluded glue like the rest of `src/shell/*`.

use crate::AppModel;
use crate::actions::DraftKey;
use crate::cli_outbound::Cli;

use super::ShellState;

/// The gap that keeps the §3.3 name prediction from reading as the tail of the
/// tagline (bl-fb1c) — a line's worth of air between two lines that are not
/// one sentence.
pub(crate) const SAID_APART: f32 = 8.0;

/// The empty world's one box and its Start (§3.4): both say the whole gesture,
/// because founding the first workspace is invisible until it has happened.
const BOOTSTRAP_HINT: &str = "Say what you want done. This founds your first workspace (`home`) and starts \
     the conversation in it — there is nothing else to set up first. Enter fires it; \
     the keyboard is already here.";

/// The empty-world placeholder + bootstrap composer (§3.4, STORIES S0): the
/// wordmark, the greyed name prediction (the conversation the Enter will mint
/// and pass via `--name`, §3.3), and one box whose Enter runs the bare-rung bootstrap —
/// `lernie new <names-root>/home` (§3.1's default name) and the detached prompt —
/// through the same planner. No wizard, no dead end, and no name picker.
pub(super) fn render(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    bl: &Cli,
) {
    // The preview's own read of the held seed (§3.3); the fire below reads —
    // and retires — it through [`ShellState`] itself (bl-28ba), so the
    // prediction and the stamp still come off one fact.
    let seed = state.start.mint_seed;
    ui.vertical_centered(|ui| {
        crate::theme::wordmark(ui);
        ui.weak(crate::theme::TAGLINE);
        // The prediction is not the rest of the tagline (bl-fb1c). Two `ui.weak`
        // runs in the same size and colour on adjacent lines read as one
        // wrapped sentence — "the key and the gate will be named growing" —
        // so the line that says what the *Enter* would mint is set apart from
        // the line that says what yog is: a gap, and italics.
        ui.add_space(SAID_APART);
        let inputs = model.start_bare_inputs();
        ui.weak(
            egui::RichText::new(
                crate::start::preview(&inputs, &mut crate::names::SplitMix64::from_seed(seed))
                    .preview,
            )
            .italics(),
        );
    });
    // The empty world's own draft key (bl-a69a): a new conversation with no
    // workspace yet — the general [`DraftKey::composer`] case with the workspace
    // absent, so this box does not share a buffer with any workspace's composer.
    let key = DraftKey::composer(None, None);
    let text = state.actions.drafts.text(&key);
    let mut buffer = text.clone();
    // Stack the composer vertically (STORIES S0 step 1): invitation, full-width
    // box, then a visible Start — so it fits altitude-1 at the 900px default
    // window instead of running off-screen in one horizontal row.
    ui.label("start a conversation:");
    let edit = ui
        .add(
            egui::TextEdit::singleline(&mut buffer)
                .desired_width(f32::INFINITY)
                .hint_text("say what you want done"),
        )
        .on_hover_text(BOOTSTRAP_HINT);
    // Open focused, through the one §11 mechanism (`super::focus`): the launch
    // request stands from `ShellState::new`, so the empty world's first frame
    // takes the keyboard with no bootstrap-only memory flag of its own.
    state.actions.drafts.set(key.clone(), buffer);
    super::focus::take(state, ui, &edit);
    // Enter (lost_focus + Enter) or the Start button both ride the same bare-rung
    // fire (§3.4); the draft clears only on a clean send.
    // No work-directory field on this surface — the bootstrap is the bare rung
    // only (§3.4), so the empty string is the honest answer to "where", not a
    // missing check: `work_dir_refusal` reads it as the bare rung and passes.
    let submit_enabled = crate::actions::new_prompt_enabled(&text, "");
    let entered = edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
    let clicked = ui
        .add_enabled(submit_enabled, egui::Button::new("Start"))
        .on_hover_text(BOOTSTRAP_HINT)
        .on_disabled_hover_text("type what you want done first — there is nothing to start yet")
        .clicked();
    if submit_enabled && (clicked || entered) {
        // A send keeps the keyboard (§11 focus discipline), asked on the
        // attempt: a bootstrap that fails banners below, and the operator's
        // retry is a keystroke away rather than a click.
        super::focus::request(state);
        if super::fire::fire_bare(model, state, lernie, bl, &text) {
            state.actions.drafts.set(key, String::new());
        }
    }
    // Per-frame, from the model (§7.3, bl-4895) — the bootstrap's first prompt is
    // exactly the one whose driver most often dies on a stale config.
    //
    // The same `Origin::Conversation` the docked composer reads (bl-48f8), and
    // that is not a duplicate: this box **is** the composer before a workspace
    // exists — the general `DraftKey::composer(None, None)` case, on the one
    // surface, and the two never paint in the same frame (this renders only when
    // nothing is focused, which is exactly when the composer is withheld).
    if let Some(failure) = model.last_failure(crate::opslog::Origin::Conversation) {
        super::banner::failure_banner(ui, model, &failure);
    }
}
