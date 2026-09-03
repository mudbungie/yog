//! **What one tick does with its move** — the acts a decision is run through,
//! split off at §12's per-file budget on the seam [`super`]'s own doc draws:
//! `plan` is the pure function that says *which* act, this is *doing* it, and
//! the file above is the thread and the tick that hold the two together.
//!
//! Everything here goes through the boundary's own typed doors, so a loop spawn
//! runs the same bodies a click, a line and a deposit run — the §3.5 spend
//! ceiling and the §4.11 confinement refusal are seated inside them, and the
//! loop is gated by construction rather than by remembering to ask.

use std::sync::Arc;

use super::super::row;
use super::{Move, PilotCtx};
use crate::app::Snapshot;
use crate::board::BoardRow;
use crate::boundary::Action;
use crate::boundary::dispatch::{self, Deps};
use crate::opslog;
use crate::start::{BallSpec, Payload};
use crate::ui_state::UiState;

impl PilotCtx {
    /// Make the move and leave one row behind — only if it landed. A refused or
    /// failed move already has its own §4.2 row from the executor that refused
    /// it (the ceiling's, `bl`'s, the start flow's), and the level trigger is
    /// the whole retry: the next tick re-reads the world and decides again.
    pub(super) fn fire(
        &self,
        snapshot: &Arc<Snapshot>,
        ui: &mut UiState,
        ts: &str,
        workspace: &std::path::Path,
        one: &Move,
    ) -> bool {
        let deps = self.deps(snapshot);
        let entry = match one {
            Move::Reap {
                row,
                claimant,
                since,
            } => {
                // The verb must actually have released it. A non-zero `bl`
                // is not an `Err` here (§8.2: its stderr is the product), so
                // the outcome is what decides — a row saying the loop reaped a
                // ball it still holds would be the trail lying.
                if !release(&deps, ui, ts, row, claimant) {
                    return false;
                }
                row::reaped(ts.to_owned(), workspace, &row.id, claimant, since)
            }
            Move::Spawn { row } => {
                let Some(conversation) = Self::birth(&deps, ui, ts, workspace, row) else {
                    return false;
                };
                row::spawned(ts.to_owned(), workspace, &row.id, &conversation)
            }
        };
        let _ = opslog::append(&deps.state_root, &entry);
        true
    }

    /// The §8.1 start flow, through the boundary's own two typed doors — the
    /// same bodies a click, a line and a deposit run. The §3.5 spend ceiling
    /// and the §4.11 confinement refusal are seated inside
    /// [`dispatch::prompt`], so a loop spawn is gated by construction rather
    /// than by this module remembering to ask.
    ///
    /// **A birth is atomic against its own claim** (bl-ab13). [`dispatch::prepare`]
    /// runs the `bl claim`, and that is the flow's LAST mutating step — so a
    /// prepare that failed claimed nothing, while a prompt that failed has left
    /// the ball held by a workspace with no conversation on it. Nothing else
    /// would ever undo that: the §4.3 lease compares a *drone's* idleness and
    /// there is no drone to be idle, so the slot and the ball were consumed
    /// forever while the trail said the spawn had succeeded. The failing door
    /// therefore releases what the door before it took. No loop row either way —
    /// the birth did not land, and the `bl` claim/unclaim pair is the trail.
    fn birth(
        deps: &Deps,
        ui: &mut UiState,
        ts: &str,
        workspace: &std::path::Path,
        row: &BoardRow,
    ) -> Option<String> {
        // The row names its project (bl-b4b5); the live cache is keyed by the
        // clone's path and the `prepare` door takes one, so the name resolves
        // here through the snapshot's own round trip.
        let project = deps.snapshot.project_path(&row.project).ok()?;
        let ball = deps
            .snapshot
            .balls_by_project
            .get(&project)?
            .iter()
            .find(|b| b.id == row.id)?;
        let payload = Payload::Ball {
            project: row.project.clone(),
            ball: BallSpec::Existing {
                id: ball.id.clone(),
                title: ball.title.clone(),
                body: ball.body.clone(),
                join: row.state,
                // §8.7: the loop reads the whole ball off the snapshot, so its
                // tags reach the start plan exactly as a clicked ▶ Start's do —
                // a fleet birth and a hand birth select one lineage (bl-380f).
                tags: ball.tags.clone(),
            },
        };
        let prepared = dispatch::prepare(deps, ts, workspace, &project, &payload).ok()?;
        // The composed goal verbatim (§3.3, bl-6920): there is no operator at
        // the composer to edit it, and the loop must not become a second author.
        let goal = prepared.goal.clone();
        // No preview, so no seed (bl-1747): the mint draws off the stamp.
        let fired = dispatch::prompt(deps, ui, ts, workspace, &prepared, &goal, None);
        if fired.is_err() {
            // The claim above landed and the fire did not: give it back. The
            // claimant is the workspace's own leaf, which is what the start
            // flow stamped `--as` a moment ago.
            release(deps, ui, ts, row, &crate::naming::leaf(workspace));
        }
        fired.ok()
    }

    /// This pass's [`Deps`]: the template plus the snapshot it just read,
    /// exactly as the gestures consumer builds one.
    fn deps(&self, snapshot: &Arc<Snapshot>) -> Deps {
        Deps {
            snapshot: Arc::clone(snapshot),
            ..self.deps.clone()
        }
    }
}

/// Give `row`'s claim back from `name`, through the boundary's own door, and
/// say whether it actually came back. **The loop's one spelling of a release**
/// — the lease reap spends it and so does a birth undoing its own claim
/// (bl-ab13), which is what keeps the two from drifting into two acts.
fn release(deps: &Deps, ui: &mut UiState, ts: &str, row: &BoardRow, name: &str) -> bool {
    released(dispatch::dispatch(
        deps,
        ui,
        ts,
        &Action::Ball(crate::actions::verbs::Verb::Release {
            project: row.project.clone(),
            id: row.id.clone(),
            name: name.to_owned(),
        }),
    ))
}

/// Whether a released claim actually came back: the verb ran *and* exited
/// clean. Anything else leaves the ball where it was, and the next tick decides
/// again against the world as it then is.
fn released(reply: Result<crate::boundary::reply::Reply, String>) -> bool {
    matches!(reply, Ok(crate::boundary::reply::Reply::Outcome(o)) if o.ok())
}
