//! **The start flow's first rung** (§8.1, §8.3; bl-1fd0): on a wall that
//! carries no credential, the pane leads with provider sign-in instead of with
//! a text box.
//!
//! It is a rung, not a surface. Everything it paints already exists: the
//! sentence is [`StartGate`]'s (tested, in `crate::start::gate`), and the rows
//! and the streamed sign-in beneath it are [`super::login_pane::login_section`]
//! — the §8.3 machinery, aimed at the start's own target wall, in a third seat
//! beside the Login tab and the conversation's auth-failed banner. One
//! capability; §8.3 rule 4 (a keyless/api-keyed row gets the reason, never a
//! dead button) therefore holds here for free.
//!
//! **It is a band of the pane, not content inside the goal box's panel**
//! (§11 rule 5). It was written inside the composer first, and it does not fit
//! there by construction: the start box is 240 points
//! ([`Panel::StartGoal`](crate::ui_state::Panel)) and this is a sentence, ten
//! provider rows and a live command stream — the roster took the box's room and
//! the pane clipped the Send row off the bottom. So it docks directly above the
//! goal box and asks [`crate::layout::share`] for a share of its own, exactly as
//! the settings band does, and the box below it is untouched.
//!
//! **The wall is already lensed.** [`super::render`] folds every brazen-shaped
//! seam onto `model.start_workspace()` — §3.4's sphere, which is the focused
//! workspace whenever anything is focused and the wall the next Enter founds
//! when nothing is — so `state.wall.login` IS the target wall's holder and no
//! second read is taken here (the ruling's "no network round trip in the pane").
//!
//! Coverage-excluded shell glue like the rest of `src/shell/*`; the decision is
//! [`StartGate`]'s and the acceptance is `shell/acceptance/start_provider.rs`.

use crate::AppModel;
use crate::cli_outbound::Cli;
use crate::start::StartGate;
use crate::theme;

use super::ShellState;

/// The gate for the start this pane is composing — the §3.4 sphere's channel
/// paired with the wall credit the §8.3 `ask` already folded.
///
/// `pub(super)`: the §11 Enter binding refuses through the same read the button
/// does ([`super::start_pane::send_pending`]), so a pointer and a keypress
/// cannot disagree about whether the wall can run.
pub(super) fn gate(model: &mut AppModel, state: &ShellState) -> StartGate {
    let name = model.start_workspace_name();
    StartGate::read(model.hosting_entry(&name), state.wall.login.credit)
}

/// Paint the rung. Called only where [`gate`] has something to say
/// ([`StartGate::note`]), so the band is never an empty bar above the box.
///
/// Two paints: the honest note and **no roster** for a workspace a §8.2 entry
/// hosts (bl-61bf's seam — the rows this box holds are the wrong wall's), and
/// otherwise the reason plus the §8.3 roster that remedies it.
///
/// **A sign-in that lands hands the keyboard to the goal box.** The roster's
/// own frame duty folds a clean outcome back into the rows
/// ([`LoginHolder::poll_run`](super::LoginHolder::poll_run)), so the credit
/// flips inside this call; comparing it across the roster is what makes the
/// hand-back a one-shot rather than a per-frame grab, and the draft is
/// untouched throughout — the goal lives in `state.start.pending`, which
/// nothing here reaches into.
pub(super) fn band(ui: &mut egui::Ui, model: &mut AppModel, state: &mut ShellState, bz: &Cli) {
    let gate = gate(model, state);
    let Some(note) = gate.note() else {
        return;
    };
    if !gate.roster() {
        ui.weak(note);
        return;
    }
    // **Wrapped, not truncated** (§11 rule 1's own carve-out, the rule
    // `login_pane::provider_row` already keeps for its refusal): this sentence
    // is the whole answer to "why can I not send", so a panel-root `Truncate`
    // cutting it mid-clause would leave the rung stating a fragment.
    ui.scope(|ui| {
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
        ui.colored_label(theme::ICHOR, note);
    });
    let state_root = model.state_root().to_path_buf();
    let before = state.wall.login.credit.credentialed;
    // Scrolled inside the band's own share (§11 rule 6): ten rows and a live
    // command stream do not fit in every window, and what does not fit is
    // reached by scrolling — the one answer that stays true at the documented
    // 420x320 minimum.
    egui::ScrollArea::vertical()
        .id_salt("start-provider-rung")
        .show(ui, |ui| {
            super::login_pane::login_section(ui, &mut state.wall.login, bz, &state_root);
        });
    if !before && state.wall.login.credit.credentialed {
        super::focus::request(state);
    }
}
