//! The **birth-config block** (DESIGN §11, §3.4, bl-824e): the parameters a new
//! conversation is born with, painted in the conversation surface's **settings
//! seat** — the bottom stack beside the composer, where every config-shaped row
//! sits since the settings-seat ruling ([`super::settings`], bl-2e18).
//!
//! The block occupies the very rows a selected conversation's settings would:
//! those rows answer "what is this conversation running on", and with nothing
//! selected the same question is "what would one started now run on". It is the
//! general path with an empty selection, not a second surface — and the composer
//! beside it stays one box and one Enter (§11).
//!
//! **The pick is not per-conversation, and the block says so.** lernie 0.0.3's
//! `litany prompt <repo> <message>` takes no config argument: it resolves
//! `ConfigSource::ConfigBranch("config/default")` itself, so a conversation is
//! always born on that branch's head. There is therefore no start-time scope to
//! write into — a start-time pick IS the §9.4 write, made one gesture before
//! the start instead of after it, and
//! [`birth_sentence`](crate::model_pick::birth_sentence) admits that in one
//! plain sentence above the dropdowns.
//!
//! The picker itself is the §9.4 one, reused inline and **collapsed by
//! default**: the current pair on one weak line with a `change…` affordance
//! beside it, exactly the conversation's own model-line idiom. Coverage-excluded
//! shell glue; the two sentences and the line it paints are composed and tested
//! in `crate::model_pick`.
//!
//! **The work directory is the block's other row** (bl-7927): an editable text
//! box with the default pre-chosen, at the top in the config block rather than
//! at the bottom beside the message. It is an ordinary editable box holding *where the next start
//! runs*, seeded at boot with the bare rung's own resolution — the operator's
//! home dir (§3.4 `~`), spelled out as the absolute path it actually is rather
//! than a tilde nothing in yog expands. Because the default is pre-filled, the
//! path rung is no longer a mode to opt into: leaving the box alone runs exactly
//! where the bare rung would, and editing it moves the next start. The box is
//! the **only** carrier — the composer's copy is gone, not duplicated — and it
//! survives a send, because it is a parameter the block states, not a draft
//! (§5.3 applies to the message, not to this).
//!
//! **bl-7927's "at the top" and the settings-seat ruling do not collide.** What
//! that ruling refused was the box riding the composer as `dir (optional)` — a
//! birth parameter loose in the drafting seat — and its remedy was *"in the
//! config block"*. The block has since moved as a unit, so the row is still in
//! the block and still not in the message box; only the block's own seat
//! changed (§3.4, §11).

use super::ShellState;
use crate::AppModel;
use crate::cli_outbound::Cli;
use std::path::Path;

/// Paint the block for the focused workspace. `bz` drives the §9.4 roster
/// query; the §9.3 write the picker's selection fires is a posted gesture and
/// carries no binary of its own (REMOTE §9.8).
pub(super) fn block(
    ui: &mut egui::Ui,
    model: &mut AppModel,
    state: &mut ShellState,
    ws: &Path,
    bz: &Cli,
) {
    ui.strong("new conversation");
    ui.horizontal(|ui| {
        ui.label("work directory:");
        ui.add(
            egui::TextEdit::singleline(&mut state.actions.path_dir).desired_width(f32::INFINITY),
        )
        .on_hover_text(
            "Run the new conversation in this directory instead of a fresh \
             worktree. Leave it empty for the default. Typed, it is `/prepare dir \
             <path>`.",
        );
    });
    // §3.1's refusal idiom, borrowed from the `new` workspace form
    // (`super::new_ws`): a sentence beside the field, never an ops wound —
    // nothing has spawned. The verdict is the spawn boundary's own question
    // ([`crate::actions::work_dir_refusal`], bl-6191), so a directory that is
    // not there is red *before* Enter fires, and the same predicate disarms the
    // composer's send rather than letting a fork misname the fault.
    if let Some(refusal) = crate::actions::work_dir_refusal(&state.actions.path_dir) {
        ui.colored_label(crate::theme::ICHOR, refusal);
    }
    // The §2.2 config-lineage tip — the very commit `litany prompt` will fork.
    // Off the landed enumeration (bl-b4b5): it is a fact about a *workspace*,
    // so it rides `Query::Workspaces`' row beside the §6 rollups rather than
    // being folded out of the window's own tree. A workspace with no lineage
    // derived yet says nothing rather than a line about nothing.
    let config_tip =
        crate::nav::tabs::config_tip(&super::chrome::ws_rows(model), &model.snap.ws_name(ws));
    super::model_pick::birth_seat(ui, model, state, ws, config_tip.as_ref(), bz);
}
