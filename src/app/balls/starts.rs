//! The start-flow's hand-off from [`AppModel`] (DESIGN §3.4, §3.5, §8.1): the
//! instance's start axes resolved into the [`StartInputs`] the composer's ▶ Start,
//! ▶ Continue, new-ball, and bare Enter entry points hand [`crate::start::prepare`].
//! Split from [`super`] per §12's 300-line budget — the ball fetch + join live
//! there, the start-input construction here. The input constructions are pure over
//! the pre-fetched caches; [`AppModel::prepare_start`] is the one effectful entry —
//! it runs the flow and **adopts the prepared workspace as the focus** (§3.4), so
//! the tab bar, the conversation list and both composers derive from one fact.

use super::AppModel;
use crate::binding::workspace_path;
use crate::cli_outbound::Cli;
use crate::names::{self, NameError};
use crate::projects::balls::Ball;
use crate::projects::join::JoinRow;
use crate::start::{self, BallSpec, Payload, Prepared, StartInputs};
use std::path::{Path, PathBuf};

impl AppModel {
    /// Run the start flow (§8.1) and **focus the workspace it resolved** (§3.4) —
    /// the single entry every start rung fires through, so a start never leaves the
    /// surfaces naming two different workspaces.
    ///
    /// Focus is unconditional because it is never a second decision: `prepare`
    /// resolves exactly one target workspace, and every rung's is the one the
    /// operator is now working in. ▶ Start / Create-&-Start target the focused
    /// workspace ([`start_workspace`](Self::start_workspace)), so the adoption is
    /// a no-op — except in the bootstrap empty case, where the founded `home` *is*
    /// the new focus.
    /// ▶ Continue and the New workspace raise name a workspace that is deliberately
    /// **not** the focused one ([`resumable`](Self::resumable),
    /// [`new_workspace_inputs`](Self::new_workspace_inputs)), and there the adoption
    /// is the correction: without it the raise left the tab bar, the conversation
    /// list and the bottom composer's bare rung ([`start_bare_inputs`](
    /// Self::start_bare_inputs)) pointed at the workspace the operator just walked
    /// away from, and Enter silently prompted it (bl-2826). A failed prepare
    /// resolved nothing, so it moves nothing.
    ///
    /// This is only the *workspace* half of §3.4's "a start focuses what it
    /// started". The **conversation** half cannot be decided here — the mint
    /// happens at the fire, one step later — so it rides
    /// [`AppModel::await_conversation`](Self::await_conversation), which the fire
    /// leaves for the first roster carrying the started root (bl-49cb).
    ///
    /// **Birth gates on nothing brazen knows** (bl-00ee): bl-c3a9 judged the
    /// world's birth template against brazen's table here, and §16.2's wall made
    /// that table the workspace's own — born with it, filled after it — so the
    /// gate is retired rather than moved. A dead provider row is faulted in the
    /// §9.5 pane and surfaced at the first dispatch (§8.3), both against a wall
    /// that exists.
    pub(crate) fn prepare_start(
        &mut self,
        lernie: &Cli,
        bl: &Cli,
        workspace: &Path,
        payload: &Payload,
        ts: &str,
    ) -> Result<Prepared, String> {
        let deps = self.boundary_deps(lernie, bl);
        // It takes the two §3.4 axes and nothing else — the same pair the
        // boundary's Prepare variant carries — because the rest (roots,
        // occupied names) re-derives inside the chokepoint from the sources
        // every caller filled it from: one derivation. This is the
        // chokepoint's typed door, the same body the dispatch match's Prepare
        // arm runs (§8.5), so a click, a line and a deposit share it.
        let prepared = crate::boundary::dispatch::prepare(&deps, ts, workspace, payload)?;
        self.focus_workspace(&prepared.workspace);
        Ok(prepared)
    }

    /// The live ball with `id` in `project`, from the cached projection (§5.1 #2).
    fn ball_of(&self, project: &Path, id: &str) -> Option<&Ball> {
        self.snap
            .balls_by_project
            .get(project)?
            .iter()
            .find(|b| b.id == id)
    }

    /// The **where** axis for a start from this instance (§3.4): the focused
    /// workspace's path verbatim whenever one is focused (named **or foreign**: a
    /// foreign workspace is a real lernie workspace, so §3.4's "prompt into the
    /// focused workspace" is unconditional), else `<names-root>/home` — the §3.1
    /// default name, taken by the empty world and only by it (a focus is derived
    /// whenever the roster holds anything, §4.1). The bootstrap is that path not
    /// existing yet, which the planner's `EnsureWorkspace` founds: the empty case
    /// of the general path, never a wizard and never a name picker.
    ///
    /// **It is also the sphere the §16.2 wall lens rides** (bl-3b62). The wall is
    /// pure path algebra over a workspace's leaf — no IO, nothing stored — so
    /// "which sphere's settings is this window showing" and "which sphere would
    /// the next Enter land in" are the same question, and keying the lens on the
    /// *focused* workspace answered it with `None` in exactly the state a
    /// stranger is in. That left the §8.3 Login roster empty on a fresh install:
    /// sign-in was reachable only as derived state after an auth-failed step, so
    /// discovering it cost a conversation and a dead first turn. Read through
    /// this one method the empty world's roster is `home`'s — the very wall the
    /// first Enter's workspace will use — and the ruling at bl-9b52 Q3 is one
    /// call site, not a new verb.
    pub fn start_workspace(&self) -> PathBuf {
        match self.focus.ws.as_deref() {
            Some(ws) => ws.to_path_buf(),
            None => workspace_path(&self.roots.yog_data, names::DEFAULT_NAME),
        }
    }

    /// The conversation mint's occupied set for a workspace (§3.3): the names its
    /// live roots stamped on their goals, already parsed into the tree the §11
    /// list renders — the mint re-reads nothing. A workspace with no derived tree
    /// (never swept, or one that does not exist yet) is simply empty: the general
    /// path with no inputs.
    pub fn conversation_names(&self, workspace: &Path) -> Vec<String> {
        crate::boundary::answer::names_in(&self.snap, workspace)
    }

    /// The common [`StartInputs`] shape at an **explicit** workspace (§3.4): the
    /// roots, `~`, and that workspace's occupied conversation names around
    /// `workspace` + `payload`. The bare / ball / new-ball entries resolve the
    /// instance's workspace ([`start_inputs`]); the resume entry pins the ball's
    /// own claimant workspace ([`resumable`]); the raise pins the operator's
    /// validated name ([`new_workspace_inputs`]). A workspace that does not exist
    /// yet simply has no stamped names — no branch (§3.3).
    fn start_inputs_at(&self, workspace: PathBuf, payload: Payload) -> StartInputs {
        StartInputs {
            conversation_names: self.conversation_names(&workspace),
            workspace,
            payload,
            home: self.roots.home.clone(),
            yog_data_root: self.roots.yog_data.clone(),
            balls_state_root: self.balls_state_root(),
        }
    }

    /// A [`StartInputs`] carrying `payload` on this instance's resolved workspace
    /// (§3.4) — the focused one, or the bootstrap's `home`.
    fn start_inputs(&self, payload: Payload) -> StartInputs {
        self.start_inputs_at(self.start_workspace(), payload)
    }

    /// The bare rung (§3.4): a new root in the focused workspace, or the bootstrap
    /// `home` when none is focused — the composer's everyday Enter.
    pub fn start_bare_inputs(&self) -> StartInputs {
        self.start_inputs(Payload::Bare)
    }

    /// The §3.1 validation of an operator-typed workspace name, against **this**
    /// world's three roots — what the §11 `new` form renders inline and arms its
    /// Create on. A pure read; nothing spawns, because nothing is committed to
    /// until the name is lawful.
    pub fn validate_workspace_name(&self, typed: &str) -> Result<String, NameError> {
        names::validate(
            typed,
            &crate::binding::roots(&self.roots.yog_data, &self.roots.lernie_data),
        )
    }

    /// The **New workspace** verb (§3.4/§11): a bare start into
    /// `<names-root>/<name>` — the operator's own name, already through
    /// [`validate_workspace_name`](Self::validate_workspace_name), so the
    /// directory cannot exist and `EnsureWorkspace` raises it. Deliberately
    /// **not** the focused workspace (contrast [`start_bare_inputs`], which
    /// prompts into it); the raise then adopts what it raised (§3.4).
    pub fn new_workspace_inputs(&self, name: &str) -> StartInputs {
        self.start_inputs_at(workspace_path(&self.roots.yog_data, name), Payload::Bare)
    }

    /// The path rung (§3.4, STORIES S2): a [`Payload::Path`] at `dir` on the
    /// instance's resolved target — the composer's optional work-directory field.
    /// The dispatch already composes the target preamble + directory cwd (Z3); this
    /// is the hand-off.
    pub fn start_path_inputs(&self, dir: &Path) -> StartInputs {
        self.start_inputs(Payload::Path {
            dir: dir.to_path_buf(),
        })
    }

    /// The ▶ Start entry points (§3.5, §8.1): one [`StartInputs`] per start-eligible
    /// join row ([`start::is_start_eligible`] — a ready ball) targeting the
    /// instance's workspace. The claim lands `--as` that workspace's name.
    pub fn startable(&self) -> Vec<StartInputs> {
        self.snap
            .join_rows
            .iter()
            .filter(|r| start::is_start_eligible(r.state))
            .filter_map(|row| Some(self.start_inputs(self.ball_payload(row)?)))
            .collect()
    }

    /// The ▶ Continue entry points (§8.1 resume, addendum): one [`StartInputs`] per
    /// **bound** join row ([`start::is_resume_eligible`]) targeting the ball's *own*
    /// claimant workspace — not the focused one — so the re-plan is prompt-only (no
    /// second claim). Reaches a ball stranded between claim and prompt by a
    /// crash/Cancel, which ▶ Start (ready-only) can never re-enter.
    pub fn resumable(&self) -> Vec<StartInputs> {
        self.snap
            .join_rows
            .iter()
            .filter(|r| start::is_resume_eligible(r.state))
            .filter_map(|row| {
                let workspace = row.workspace.clone()?;
                Some(self.start_inputs_at(workspace, self.ball_payload(row)?))
            })
            .collect()
    }

    /// The existing-ball [`Payload`] for a live join row (§3.5): its id/title/body/
    /// join state. `None` when the row's ball is not in the live projection (a
    /// broken row drops rather than panics — both callers filter it out).
    fn ball_payload(&self, row: &JoinRow) -> Option<Payload> {
        Some(Payload::Ball {
            project: row.project.clone(),
            ball: self.ball_spec(row)?,
        })
    }

    /// One join row as an **existing-ball** spec (§3.4): its id/title/body and
    /// §3.5 join state. The one home of that build — the start payload above
    /// and the §8.5 line context ([`AppModel::line_context`]) are its two
    /// readers, and a ball the live projection does not carry is `None` for
    /// both (a broken row drops rather than panics).
    pub(crate) fn ball_spec(&self, row: &JoinRow) -> Option<BallSpec> {
        let ball = self.ball_of(&row.project, &row.ball_id)?;
        Some(BallSpec::Existing {
            id: row.ball_id.clone(),
            title: ball.title.clone(),
            body: ball.body.clone(),
            join: row.state,
        })
    }

    /// A new-ball start input (§8.1): a [`BallSpec::New`] in `project` with the
    /// operator's RAM-drafted title/body — the new-ball entry's hand-off.
    pub fn new_ball_inputs(&self, project: &Path, title: &str, body: &str) -> StartInputs {
        self.start_inputs(Payload::Ball {
            project: project.to_path_buf(),
            ball: BallSpec::New {
                title: title.to_owned(),
                body: body.to_owned(),
            },
        })
    }

    /// The enumerated project paths (§5.1 #1), sorted — the new-ball entry offers a
    /// form per project, including those with no live balls yet.
    pub fn project_paths(&self) -> Vec<PathBuf> {
        let mut ps: Vec<PathBuf> = self.snap.balls_by_project.keys().cloned().collect();
        ps.sort();
        ps
    }
}
