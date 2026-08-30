//! The start-flow's hand-off from [`AppModel`] (DESIGN §3.4, §3.5, §8.1): the
//! instance's start axes resolved into the [`StartInputs`] the composer's ▶ Start,
//! ▶ Continue, new-ball, and bare Enter entry points compose an
//! [`Action::Prepare`](crate::boundary::Action::Prepare) from.
//! Split from [`super`] per §12's 300-line budget — the ball fetch + join live
//! there, the start-input construction here.
//!
//! **Everything here is pure over the pre-fetched caches** (bl-1747). It had one
//! effectful entry, `prepare_start`, which ran the flow in process and adopted
//! the workspace it resolved; the run crossed the wire with the rest of the acts
//! and the adoption went with it, onto the receipt — the seat that posts the
//! gesture is the seat that knows what its landing means (REMOTE §9.8).

use super::AppModel;
use crate::binding::workspace_path;
use crate::names::{self, NameError};
use crate::projects::balls::Ball;
use crate::projects::join::JoinRow;
use crate::start::{self, BallSpec, Payload, StartInputs};
use std::path::{Path, PathBuf};

impl AppModel {
    /// The live ball with `id` in `project`, from the cached projection (§5.1 #2).
    fn ball_of(&self, project: &Path, id: &str) -> Option<&Ball> {
        self.snap
            .balls_by_project
            .get(project)?
            .iter()
            .find(|b| b.id == id)
    }

    /// **The where axis for a start, as a §3.1 NAME** (§3.4): the focused
    /// workspace's — named, foreign, **or held at a §8.2 entry** — else `home`,
    /// the §3.1 default the empty world takes and only it (a focus is derived
    /// whenever the roster holds anything, §4.1).
    ///
    /// A name and not a path since bl-e349, because a name is what a start is
    /// **addressed** by: `Action::Prepare` carries one, the poster routes the
    /// act by it (REMOTE §8.2), and the chokepoint resolves it at the far end —
    /// founding an absent one, which is what "raising a workspace" is
    /// (`dispatch::resolve_workspace`). Reading the focused *path* instead made
    /// two different states answer `None` as one — nothing focused, and a
    /// workspace whose directory is on another box — and the `home` substituted
    /// for the second founded a phantom local workspace and ran the operator's
    /// goal in it.
    pub fn start_workspace_name(&self) -> String {
        self.focused_ws_name()
            .unwrap_or_else(|| names::DEFAULT_NAME.to_owned())
    }

    /// That target **spelled as a path** — the enumerated workspace's own, or
    /// the §3.1 names-root spelling of a name this box does not enumerate (the
    /// §11 raise's shape, and the bootstrap's).
    ///
    /// It is a spelling and never an address: the only things read off it are
    /// its leaf — which is [`start_workspace_name`](Self::start_workspace_name)
    /// again, the name the act carries — and the §16.2 wall the leaf names.
    /// What this box may actually *do* with the directory is
    /// [`start_path`](AppModel::start_path)'s answer, which withholds one
    /// outright for a workspace an entry hosts.
    ///
    /// **It is the sphere the §16.2 wall lens rides** (bl-3b62). The wall is
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
        let name = self.start_workspace_name();
        self.workspace_path(&name)
            .unwrap_or_else(|| workspace_path(&self.roots.yog_data, &name))
    }

    /// The common [`StartInputs`] shape at an **explicit** workspace (§3.4): the
    /// roots, `~`, and that workspace's occupied conversation names around
    /// `workspace` + `payload`. The bare / ball / new-ball entries resolve the
    /// instance's workspace ([`start_inputs`]); the resume entry pins the ball's
    /// own claimant workspace ([`resumable`]); the raise pins the operator's
    /// validated name ([`new_workspace_inputs`]). A workspace that does not exist
    /// yet simply has no stamped names — no branch (§3.3).
    fn start_inputs_at(&self, workspace: PathBuf, payload: Payload) -> StartInputs {
        let repo = payload
            .project()
            .and_then(|name| self.snap.project_path(&name).ok());
        StartInputs {
            // The §3.3 occupied set the mint may not re-use — the boundary's own
            // fold, read here because a `Prepare` is composed in process and its
            // refusal is re-derived at fire (bl-b4b5 retired the accessor over
            // it; the §11 preview reads the same fact off the answered forest).
            conversation_names: crate::boundary::answer::names_in(&self.snap, &workspace),
            workspace,
            repo,
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
            &crate::binding::roots(&self.roots.yog_data, &self.roots.litany_data),
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
                let workspace = self.snap.ws_path(row.workspace.as_deref()?).ok()?;
                Some(self.start_inputs_at(workspace, self.ball_payload(row)?))
            })
            .collect()
    }

    /// The existing-ball [`Payload`] for a live join row (§3.5): its id/title/body/
    /// join state. `None` when the row's ball is not in the live projection (a
    /// broken row drops rather than panics — both callers filter it out).
    fn ball_payload(&self, row: &JoinRow) -> Option<Payload> {
        Some(Payload::Ball {
            // The row already says the name (bl-b4b5) — the payload's `project`
            // and the join's are one word now, not a path spelled back down.
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
        let project = self.snap.project_path(&row.project).ok()?;
        let ball = self.ball_of(&project, &row.ball_id)?;
        Some(BallSpec::Existing {
            id: row.ball_id.clone(),
            title: ball.title.clone(),
            body: ball.body.clone(),
            join: row.state,
            // The §8.7 birth policy's input, carried off the live ball rather
            // than the join row: the row is the projection, the ball is the fact.
            tags: ball.tags.clone(),
        })
    }

    /// A new-ball start input (§8.1): a [`BallSpec::New`] in `project` with the
    /// operator's RAM-drafted title/body — the new-ball entry's hand-off.
    pub fn new_ball_inputs(&self, project: &Path, title: &str, body: &str) -> StartInputs {
        self.start_inputs(Payload::Ball {
            project: self.snap.project_name(project),
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
