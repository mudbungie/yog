//! Firing the composer's start (DESIGN §3.4, §8.1): resolve the bare / path rung
//! [`StartInputs`] and post the §8.1 prepare that carries the typed text through
//! into its own prompt. Split from [`super::input_bar`] per §12's budget.
//! Coverage-excluded shell glue — every decision lives in the tested
//! `start`/`AppModel` modules and the hold is [`super::acting`]'s; this only
//! wires the composer's Enter to them.
//!
//! **The whole flow is two posted acts now** (REMOTE §9.8, bl-1747), not two
//! synchronous calls: `Prepare`, then the `Prompt` its receipt chains. Nothing
//! is read back here, so nothing is returned — the draft this Enter composed
//! rides the hold and empties when the *prompt* lands, which is the same edge
//! the old `bool` reported and the same one the operator sees.

use super::ShellState;
use crate::AppModel;
use crate::actions::DraftKey;
use std::path::Path;

/// Fire the composer's start (§3.4): the **path** rung when the §11 birth-config
/// block's work-directory box holds one (`actions.path_dir`), else the **bare**
/// rung. Since bl-7927 that box is pre-filled with the bare rung's own
/// resolution, so the path arm is the ordinary one and the bare arm is what an
/// operator who empties the box by hand gets — the same driver cwd either way,
/// which is why an empty box is a value here and not an error.
pub(super) fn fire_start(model: &mut AppModel, state: &mut ShellState, key: &DraftKey, text: &str) {
    let dir = state.actions.path_dir.trim().to_owned();
    let inputs = if dir.is_empty() {
        model.start_bare_inputs()
    } else {
        model.start_path_inputs(Path::new(&dir))
    };
    super::acting::start::fire(model, state, &inputs, key, text);
}

/// Fire the bare rung (§3.4) — the empty-world bootstrap composer's Enter.
/// `pub(super)`: workspace.rs's placeholder reuses it (bare only, no path field).
pub(super) fn fire_bare(model: &mut AppModel, state: &mut ShellState, key: &DraftKey, text: &str) {
    // The §3.4 workspace adoption rides the prepare's receipt, so the
    // bootstrap's `home` workspace is the focused one the instant the engine
    // says it exists; the §3.3 seed the greyed preview drew off is fired
    // **with** the prompt (bl-28ba) and retired when that lands, so a launch
    // that failed keeps the prediction it never spent. A failure rides its own
    // ops line (§8.1), which the per-frame banner reads back either way (§7.3)
    // — including a detached driver that only dies later.
    let inputs = model.start_bare_inputs();
    super::acting::start::fire(model, state, &inputs, key, text);
}
