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
use litany::mint::SplitMix64;

use super::super::{answer, ceiling, control};
use super::Deps;

/// The §8.1 step 2 refusal a blank goal rides back (bl-6191, seated by
/// bl-54c1). One sentence, spelled once, so every spelling of the fire is
/// refused in the same words.
const BLANK_GOAL: &str = "the goal is blank: say what the conversation is for";

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
    repo: &std::path::Path,
    payload: &crate::start::Payload,
) -> Result<crate::start::Prepared, String> {
    let inputs = StartInputs {
        conversation_names: answer::names_in(&deps.snapshot, workspace),
        workspace: workspace.to_path_buf(),
        // The payload's project name, already located by the chokepoint
        // (REMOTE §8) — `None` for the two rungs that name none.
        repo: payload.project().map(|_| repo.to_path_buf()),
        payload: payload.clone(),
        home: deps.home.clone(),
        yog_data_root: deps.yog_data_root.clone(),
        balls_state_root: deps.balls_state_root.clone(),
    };
    let start_deps = start::Deps {
        bl: deps.bl.clone(),
        litany: deps.litany.clone(),
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
///
/// **A blank goal never sends** (§8.1 step 2, bl-6191): the refusal is
/// [`BLANK_GOAL`] and it stands **first**, ahead of both gates above — the
/// confinement gate reads the workspace's live policy and the ceiling's
/// refusal writes a §4.2 row, and a fire with nothing to say must cost
/// neither. Trimmed, because whitespace is not a payload. The invariant is
/// seated here for the reason the ceiling is: this is the one door every
/// spelling passes — the table's `Prompt` arm behind a line and a deposit, and
/// the §4.3 loop's own re-prompt — so one test gates every one of them, and a
/// blank goal is spend for nothing and a conversation whose first entry is
/// empty. A seat may grey its own send button, but that is a view (bl-54c1).
///
/// **`seed` is the firing seat's own §3.3 prediction** (bl-1747), and `None` is
/// a caller that made none — a deposited line, the §4.3 loop — for which this
/// moment's stamp is the draw. One default, at the one door that mints, rather
/// than a `Deps` field every intake filled the same way and one seat filled
/// differently.
pub fn prompt(
    deps: &Deps,
    ui: &UiState,
    ts: &str,
    workspace: &std::path::Path,
    prepared: &crate::start::Prepared,
    goal: &str,
    seed: Option<u64>,
) -> Result<String, String> {
    if goal.trim().is_empty() {
        return Err(BLANK_GOAL.to_owned());
    }
    control::confinement_gate(workspace)?;
    // The §3.5 ceiling is the **world's** since bl-a80a, so its comparison is
    // folded over the §3.1 roster rather than over the one workspace this birth
    // names — enumerated here, at the door, because "every workspace" has one
    // home ([`crate::binding::workspaces`]) and the gate stays pure over what it
    // is handed. It is read at the instant of the refusal, not off the snapshot:
    // a gate compares against the world as it is, not as a debounce window ago.
    let world: Vec<std::path::PathBuf> =
        crate::binding::workspaces(&deps.yog_data_root, &deps.world.litany_data_root())
            .into_iter()
            .map(|w| w.path)
            .collect();
    ceiling::gate(ui, &deps.state_root, ts, workspace, &world, prepared.origin)?;
    // The fired loop carries the target workspace's wall (§16.2 as amended):
    // litany hands its own environment to every tool subprocess, and a bare
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
    let litany = deps
        .litany
        .and_env(crate::world::wall::pairs(&deps.world, workspace))
        .and_env(crate::world::marks::pairs(
            &deps.world,
            workspace,
            own_space,
        ))
        // The §8.6 confinement wrapper, when the workspace's live policy
        // requires one: unconditional under the policy — the gate above proved
        // availability, and a backend that vanished since fails this spawn
        // loudly rather than running bare. Empty otherwise.
        .and_wrapper(crate::control::confine::wrapper(&deps.world, workspace));
    start::execute_prompt(
        &litany,
        &deps.state_root,
        ts,
        &start::Fire {
            workspace: workspace.to_path_buf(),
            prepared: prepared.clone(),
            goal: goal.to_owned(),
        },
        &answer::names_in(&deps.snapshot, workspace),
        &SplitMix64::from_seed(
            seed.unwrap_or_else(|| crate::ui_state::content_hash(ts.as_bytes())),
        ),
    )
    .map_err(|e| e.to_string())
}
