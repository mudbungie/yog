//! The spawn gate (DESIGN §3.5, §8.5): the control boundary's **one** ceiling
//! seat.
//!
//! §3.5 named the missing piece of the ceiling as its *seat*, not its
//! arithmetic: "a gate that covers one spawn path and not the others is worse
//! than none". This is that seat, and there is deliberately no second one.
//! Every drone yog ever births is fired by
//! [`dispatch::prompt`](super::dispatch::prompt) — the typed door the §8.5
//! `Prompt` arm and the frame's `fire_prompt` both run — so gating there
//! covers the click, the line, the deposit and the argv seat at once.
//!
//! **What it refuses is a birth and only a birth.** Nothing already running is
//! stopped, messaged less, or hurried: killing mid-ball destroys uncommitted
//! work, and early termination is the expensive failure (the vision's
//! spend-attribution ruling). A
//! `Message` to a live conversation is therefore *not* gated — refusing to
//! answer a drone that is mid-ball strands exactly the work the ruling exists
//! to protect — and the bound on a drone already alive is lernie's own
//! `max_total_tokens`, one layer down. Nor is `Prepare`: a claim spawns no
//! drone and is releasable, so the refusal lands at the one irreversible step.
//!
//! **A refusal is never silent.** It writes the §4.2 `["yog-step","ceiling"]`
//! failure line before it rides back, so it renders exactly where every other
//! refused action renders — the ops trail, its §7.3 banner at the rung that
//! fired it (the row carries the start's own [`Origin`](crate::opslog::Origin)),
//! and the §6 attention count that follows.

use std::path::{Path, PathBuf};

use crate::opslog::{self, OpEntry};
use crate::ui_state::UiState;

/// The step name a refusal's `["yog-step", …]` ops row carries (§4.2).
const STEP: &str = "ceiling";

/// Let a spawn into `workspace` through, or refuse it (§3.5).
///
/// **`world` is the comparison's scope and `workspace` is only the row's
/// subject** (bl-a80a): the ceiling bounds what this whole world has spent, so
/// the roster of every workspace is what the figure is folded over, while the
/// ops row still names where the refused birth was headed. Both come from the
/// caller — the §3.1 roster is [`crate::binding::workspaces`]' answer, asked at
/// the door rather than re-derived here, so the gate stays pure over its inputs.
///
/// `Ok(())` is the ungated world's only answer — no `ceiling` key, no price
/// table, or a world still under the number. The `Err` is the operator's
/// refusal text, already durable on the trail when it is returned.
pub fn gate(
    ui: &UiState,
    state_root: &Path,
    ts: &str,
    workspace: &Path,
    world: &[PathBuf],
    origin: crate::opslog::Origin,
) -> Result<(), String> {
    let Some(refusal) = ui.ceiling().refusal(world, &ui.prices()) else {
        return Ok(());
    };
    let entry = OpEntry::step_failure(
        ts.to_owned(),
        STEP,
        workspace.display().to_string(),
        refusal.clone(),
        origin,
    );
    // Best-effort like every other trail write whose product is not the write
    // (§7.2's drift lines, §9.3's editor line): the refusal is what the caller
    // must get, and a trail yog cannot append to is that file's problem, not
    // this gesture's.
    let _ = opslog::append(state_root, &entry);
    Err(refusal)
}

#[cfg(test)]
mod tests {
    use super::gate;
    use crate::opslog::Origin;
    use crate::ui_state::UiState;
    use std::path::{Path, PathBuf};

    const CONV: &str = "20260803T120000Z-root";

    /// A `ui.json` holding `doc`, opened as the durable state.
    fn ui(dir: &Path, doc: &str) -> UiState {
        let path = dir.join("ui.json");
        std::fs::write(&path, doc).unwrap();
        UiState::open(path)
    }

    /// A workspace whose one step spent 3 Mtok of input on `opus`.
    fn spent(ws: &Path) {
        let step = ws.join("steps").join(CONV).join("001");
        std::fs::create_dir_all(&step).unwrap();
        std::fs::write(
            step.join("response.json"),
            r#"{"type":"usage","input_tokens":3000000}"#,
        )
        .unwrap();
        std::fs::write(step.join("request.json"), r#"{"model":"opus"}"#).unwrap();
    }

    /// `$1/Mtok` input, so the fixture workspace has spent exactly $3.
    const PRICED: &str = r#"{"v":1,"prices":{"opus":{"input":1}},"ceiling":2}"#;

    /// The world roster a one-workspace fixture presents (bl-a80a).
    fn world(dir: &Path) -> Vec<PathBuf> {
        vec![dir.to_path_buf()]
    }

    #[test]
    fn an_unconfigured_world_is_ungated() {
        let dir = tempfile::tempdir().unwrap();
        spent(dir.path());
        let ui = ui(dir.path(), r#"{"v":1,"prices":{"opus":{"input":1}}}"#);
        assert!(
            gate(
                &ui,
                dir.path(),
                "T1",
                dir.path(),
                &world(dir.path()),
                Origin::Balls
            )
            .is_ok()
        );
        assert!(!dir.path().join("ops.jsonl").exists(), "nothing to log");
    }

    #[test]
    fn under_the_ceiling_flies_and_logs_nothing() {
        let dir = tempfile::tempdir().unwrap();
        spent(dir.path());
        let ui = ui(
            dir.path(),
            r#"{"v":1,"prices":{"opus":{"input":1}},"ceiling":5}"#,
        );
        assert!(
            gate(
                &ui,
                dir.path(),
                "T1",
                dir.path(),
                &world(dir.path()),
                Origin::Balls
            )
            .is_ok()
        );
        assert!(!dir.path().join("ops.jsonl").exists());
    }

    #[test]
    fn over_the_ceiling_refuses_and_leaves_the_ops_row() {
        let dir = tempfile::tempdir().unwrap();
        spent(dir.path());
        let ui = ui(dir.path(), PRICED);
        let refusal = gate(
            &ui,
            dir.path(),
            "T1",
            dir.path(),
            &world(dir.path()),
            Origin::Balls,
        )
        .unwrap_err();
        assert!(refusal.contains("spend ceiling reached"), "{refusal}");
        let trail = std::fs::read_to_string(dir.path().join("ops.jsonl")).unwrap();
        assert!(trail.contains("yog-step"), "{trail}");
        assert!(trail.contains("ceiling"), "{trail}");
        assert!(trail.contains("\"exit\":-3"), "{trail}");
    }

    /// bl-a80a: the scope of the comparison and the subject of the row are two
    /// different things. A birth into a workspace that has spent nothing is
    /// refused because a *sibling* spent, and the row still names the workspace
    /// the birth was headed for — otherwise the trail would say the refusal
    /// happened somewhere nobody was going.
    #[test]
    fn a_sibling_s_spend_refuses_an_idle_workspace_and_the_row_names_the_target() {
        let dir = tempfile::tempdir().unwrap();
        let (idle, busy) = (dir.path().join("idle"), dir.path().join("busy"));
        std::fs::create_dir_all(&idle).unwrap();
        spent(&busy);
        let ui = ui(dir.path(), PRICED);
        let roster = vec![idle.clone(), busy];
        let refusal = gate(&ui, dir.path(), "T1", &idle, &roster, Origin::Balls).unwrap_err();
        assert!(refusal.contains("$3.00"), "{refusal}");
        let trail = std::fs::read_to_string(dir.path().join("ops.jsonl")).unwrap();
        assert!(trail.contains("idle"), "{trail}");
    }

    #[test]
    fn an_unwritable_trail_still_refuses() {
        let dir = tempfile::tempdir().unwrap();
        spent(dir.path());
        let ui = ui(dir.path(), PRICED);
        // A state root that is not a directory: the append cannot land, and the
        // refusal is still what rides back.
        let wall = dir.path().join("ui.json");
        assert!(
            gate(
                &ui,
                &wall,
                "T1",
                dir.path(),
                &world(dir.path()),
                Origin::Balls
            )
            .is_err()
        );
    }
}
