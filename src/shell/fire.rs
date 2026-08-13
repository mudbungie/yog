//! Firing the composer's start (DESIGN §3.4, §8.1): resolve the bare / path rung
//! [`StartInputs`], run [`start::prepare`], then the detached prompt. Split from
//! [`super::input_bar`] per §12's 300-line budget. Coverage-excluded shell glue —
//! every decision lives in the tested `start`/`AppModel` modules; this only wires
//! the composer's Enter to them and refreshes the ops tail the banner derives
//! from every frame (§7.3).

use super::ShellState;
use crate::AppModel;
use crate::cli_outbound::Cli;
use crate::start::StartInputs;
use std::path::Path;

/// Fire the composer's start (§3.4): the **path** rung when the §11 birth-config
/// block's work-directory box holds one (`actions.path_dir`), else the **bare**
/// rung. Since bl-7927 that box is pre-filled with the bare rung's own
/// resolution, so the path arm is the ordinary one and the bare arm is what an
/// operator who empties the box by hand gets — the same driver cwd either way,
/// which is why an empty box is a value here and not an error. Returns whether
/// the prompt launched, so the *message* draft clears only on a clean send (RAM
/// until sent, §5.3); the directory is a birth parameter and survives.
pub(super) fn fire_start(
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    bl: &Cli,
    text: &str,
) -> bool {
    let dir = state.actions.path_dir.trim().to_owned();
    let inputs = if dir.is_empty() {
        model.start_bare_inputs()
    } else {
        model.start_path_inputs(Path::new(&dir))
    };
    fire_inputs(model, state, lernie, bl, inputs, text)
}

/// Fire the bare rung (§3.4) — the empty-world bootstrap composer's Enter.
/// `pub(super)`: workspace.rs's placeholder reuses it (bare only, no path field).
pub(super) fn fire_bare(
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    bl: &Cli,
    text: &str,
) -> bool {
    let inputs = model.start_bare_inputs();
    fire_inputs(model, state, lernie, bl, inputs, text)
}

/// Run `prepare` (`lernie new` when the target workspace does not exist yet, else
/// idempotent skips) then the detached prompt with the typed text, for an already
/// resolved [`StartInputs`]. The whole flow rides the one planner; a failure rode
/// its own ops line (§8.1), which the per-frame banner reads back either way
/// (§7.3) — including a detached driver that only dies later. It runs through
/// [`AppModel::prepare_start`], so the bootstrap's `home` workspace is the focused
/// one the instant it exists (§3.4) — the same adoption every start makes.
fn fire_inputs(
    model: &mut AppModel,
    state: &mut ShellState,
    lernie: &Cli,
    bl: &Cli,
    inputs: StartInputs,
    text: &str,
) -> bool {
    let launched = match model.prepare_start(
        lernie,
        bl,
        &inputs.workspace,
        &inputs.payload,
        &super::now_ts(),
    ) {
        // The conversation mint draws from a generator off the same held seed
        // the preview used (§3.3), so the greyed prediction and the stamp
        // agree; the fire is the boundary's own Prompt action (§8.5), and the
        // §3.4 start claim rides `fire_prompt`'s success. The seed is spent
        // with the mint (bl-28ba): the prediction it backed is a stamp now, so
        // the next preview must predict off a fresh one — held past here,
        // every later fire drew the same start index and the walk handed out
        // siblings (`recite-a`, `recite-b`, …).
        Ok(p) => {
            let fired = model
                .fire_prompt(
                    lernie,
                    bl,
                    &p,
                    text,
                    state.start.mint_seed,
                    &super::now_ts(),
                )
                .is_ok();
            if fired {
                state.start.spend_mint();
            }
            fired
        }
        Err(_) => false,
    };
    model.after_lernie_verb();
    launched
}
