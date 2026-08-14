//! The §8.1 start family's two **typed doors** into the §8.5 chokepoint
//! ([`super::dispatch`]): prepare a start, then fire the prompt it made ready.
//!
//! Split from the action table at §12's cap on the seam [`super`]'s own doc
//! draws — "the match and the two `pub` typed doors the frame's start glue
//! enters through". They are `pub` because the window enters here directly (it
//! owes a landed fire the §3.4 workspace adoption and the §3.3 mint seed, which
//! a headless consumer must not do), and the table's `Prepare`/`Prompt` arms
//! delegate to these same bodies — so a click, a line and a deposit spend one
//! implementation, which is the whole claim the boundary makes.

use crate::start::{self, StartInputs};
use crate::ui_state::UiState;
use lernie::mint::SplitMix64;

use super::super::{answer, ceiling, control};
use super::Deps;

/// The §8.1 mutating half: seed → ensure-workspace → the ball rung's `bl`
/// steps, the composer's [`Prepared`](crate::start::Prepared) back. The
/// occupied names and roots re-derive here from [`Deps`] — the same sources
/// every frontend fills them from, one derivation. `pub` as the chokepoint's
/// typed door — the frame's start glue enters here, and the [`dispatch`]
/// Prepare arm delegates here, so both spellings share this one body.
pub fn prepare(
    deps: &Deps,
    ts: &str,
    workspace: &std::path::Path,
    payload: &crate::start::Payload,
) -> Result<crate::start::Prepared, String> {
    let inputs = StartInputs {
        conversation_names: answer::names_in(&deps.snapshot, workspace),
        workspace: workspace.to_path_buf(),
        payload: payload.clone(),
        home: deps.home.clone(),
        yog_data_root: deps.yog_data_root.clone(),
        balls_state_root: deps.balls_state_root.clone(),
    };
    let start_deps = start::Deps {
        bl: deps.bl.clone(),
        lernie: deps.lernie.clone(),
        state_root: deps.state_root.clone(),
        yog_binary: deps.yog_binary.clone(),
    };
    start::prepare(&start_deps, &inputs, ts).map_err(|e| e.to_string())
}

/// The deferred detached fire (§8.1): mint against the occupied set, spawn
/// with `--name` and the goal verbatim (bl-6920) — the minted conversation
/// name back. `pub`
/// as [`prepare`]'s sibling typed door; the [`dispatch`] Prompt arm delegates.
///
/// **The §3.5 spend ceiling gates here and nowhere else** ([`super::ceiling`]):
/// this is the one door every drone yog births passes through, so one gate
/// covers every spawn path, and a birth is the only thing it can refuse — the
/// ruling forbids touching a drone that is already running. `ui` is the durable
/// `ui.json` the ceiling and the price table are read from.
///
/// The §4.11 item-8 **confinement refusal** rides the same door, and before the
/// ceiling: a workspace that requires a confinement layer this platform does
/// not have fires nothing at all, so there is no spend to judge.
pub fn prompt(
    deps: &Deps,
    ui: &UiState,
    ts: &str,
    prepared: &crate::start::Prepared,
    goal: &str,
) -> Result<String, String> {
    control::confinement_gate(&prepared.workspace)?;
    ceiling::gate(ui, &deps.state_root, ts, prepared)?;
    // The fired loop carries the target workspace's wall (§16.2 as amended):
    // lernie hands its own environment to every tool subprocess, and a bare
    // `bz` in an agent's bash is the world's shim re-entering yog — so this one
    // layer is what puts the whole descendant tree inside the sphere's
    // providers, sign-ins and model cache.
    // …and, for a launch that was NOT raised onto a project, its own balls
    // space (§16.3's launch clause): the ball rung is by construction pointed
    // at a project's board — it was offered on that project's balls section and
    // its `bl claim` already landed there — so it carries no `YOG_MARKS` and
    // its `bl` is the board's own, instantly consistent with what yog renders.
    // Every other rung tracks on a space of its own, which is the ruling's
    // default. Nothing new decides this: `Payload::origin` is the rung, already
    // carried on `Prepared` for the §7.3 banner.
    let own_space = prepared.origin != crate::opslog::Origin::Balls;
    let lernie = deps
        .lernie
        .and_env(crate::world::wall::pairs(&deps.world, &prepared.workspace))
        .and_env(crate::world::marks::pairs(
            &deps.world,
            &prepared.workspace,
            own_space,
        ));
    start::execute_prompt(
        &lernie,
        &deps.state_root,
        ts,
        prepared,
        goal,
        &answer::names_in(&deps.snapshot, &prepared.workspace),
        &SplitMix64::from_seed(deps.mint_seed),
    )
    .map_err(|e| e.to_string())
}
