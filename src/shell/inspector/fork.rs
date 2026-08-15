//! The fork composer's glue (VISION V2): derive what a pinned notch may fork
//! into, seat the composer beside the pin, and fire one boundary gesture per
//! candidate.
//!
//! Coverage-excluded like the rest of `shell/*`: every decision lives in a
//! tested module — [`crate::fork::choices`] (what the workspace offers),
//! [`crate::fork::composer::Composer`] (the ×N and the readiness rule),
//! [`crate::fork::render`] (the widget) and the boundary's own dispatch. This
//! file only wires them, and holds the one rule that is nobody else's: **the
//! composer lives and dies with the pin**, which is VISION V2's burden check
//! ("the composer is reachable only from a pinned notch").
//!
//! **The fan is N gestures, and that is the point.** Fire builds one
//! [`Action::Fork`] per attempt and crosses the boundary with each, so the
//! §4.2 trail carries N committed rows and the cohort that appears on the rail
//! is derived from where those N children were born — never from anything
//! this file wrote down.

use std::path::Path;

use crate::AppModel;
use crate::boundary::Action;
use crate::fork::{self, Choices};
use crate::rail::Pin;

use super::super::InspectorState;

/// Where the composer is seated: the focused agent, and the notch (if any) the
/// operator has pinned. One value, because the three are one fact — *this
/// conversation, as of this mark* — and no caller has two of them without the
/// third.
pub struct Seat {
    pub ws: std::path::PathBuf,
    pub agent_id: String,
    pub pin: Option<Pin>,
}

/// Seat the composer under the pin banner and fire what it composes. Nothing
/// pinned, or a workspace whose config declares no role anywhere, paints
/// nothing at all — no button that cannot work.
pub fn seat(ui: &mut egui::Ui, model: &mut AppModel, inspector: &mut InspectorState, at: &Seat) {
    let (ws, agent_id, pin) = (at.ws.as_path(), at.agent_id.as_str(), at.pin.as_ref());
    let Some(pin) = pin else {
        // The pin is the seat: releasing it discards the draft, exactly as
        // releasing it discards every other as-of read.
        inspector.fork = None;
        return;
    };
    let choices = choices(model, inspector, ws, pin);
    if !choices.fireable() {
        inspector.fork = None;
        return;
    }
    let composer = inspector
        .fork
        .get_or_insert_with(|| fork::composer::Composer::seeded(&choices));
    if !fork::render::render(ui, composer, &choices) {
        return;
    }
    let attempts = composer.attempts.clone();
    let goal = composer.goal.clone();
    inspector.fork = None;
    fire(model, ws, agent_id, &goal, &attempts);
}

/// What this pinned notch may fork into, memoized per snapshot: the derivation
/// asks the workspace's config branches and reads a `providers.yaml` per fork
/// point, so it is disk work and must never run per frame. The pinned commit
/// rides the key, so re-pinning re-reads and scrolling does not.
fn choices(model: &AppModel, inspector: &mut InspectorState, ws: &Path, pin: &Pin) -> Choices {
    let snap = std::sync::Arc::clone(model.derivation());
    let key = (ws.to_path_buf(), pin.commit.clone());
    let root = fork::skills_root(model.yog_data_root());
    inspector
        .fork_memo
        .read(&snap, key, &mut || fork::choices(ws, &pin.commit, &root))
        .clone()
}

/// One boundary gesture per candidate (§8.5), **posted** (REMOTE §9.8). Each is
/// the ordinary [`Action::Fork`]; a refusal rides back as that attempt's own
/// `ops.jsonl` row, which the per-frame banner reads (§7.3) — so a cohort with
/// one bad candidate says which one, and the others still fly.
///
/// The cohort keeps its order over the wire: posted acts are sent one at a time
/// in the order they were fired, so the fork that was listed first is the fork
/// the engine sees first.
fn fire(model: &mut AppModel, ws: &Path, agent_id: &str, goal: &str, attempts: &[fork::Attempt]) {
    for attempt in attempts {
        let action = Action::Fork {
            workspace: model.snap.ws_name(ws),
            parent: agent_id.to_owned(),
            attempt: attempt.clone(),
            goal: goal.to_owned(),
        };
        super::super::act::fire(model, &action);
    }
}
