//! The start flow's **inert shapes** (DESIGN §3.4, §8.1): the request
//! [`plan`](super::plan) reads and the step sequence it returns.
//!
//! §3.4's two axes are [`StartInputs::workspace`] (*where* — a path, and only a
//! path) and [`Payload`] (*what* — bare / path / ball). [`Step`] is the ordered
//! projection the executor runs; every variant is idempotent-or-convergent, so
//! a re-plan after a partial failure yields the shorter remainder.
//!
//! Split off [`super`] at §12's pre-split band: the planner is one function
//! over these, and a type carrying its own §3.4 reasoning is not the derivation
//! that consumes it.

use crate::projects::join::JoinState;
use std::path::PathBuf;

/// The ball a start targets (§3.4 ball rung): an **existing** ball (id + join
/// state known, from the roster) or a **new** ball whose id `bl create` mints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BallSpec {
    Existing {
        id: String,
        title: String,
        body: String,
        join: JoinState,
        /// The ball's own tags, in `bl`'s order — the §8.7 birth policy's one
        /// input (VISION §4.2). Carried, never stored: [`super::lineage::select`]
        /// reads them at [`super::prepare`] and nothing else in yog ever does.
        tags: Vec<String>,
    },
    New {
        title: String,
        body: String,
    },
}

/// The **what** axis (§3.4): the payload rung, each the one below plus inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Payload {
    /// bare — the empty composer; driver cwd `~`.
    Bare,
    /// path — a work directory; target preamble; driver cwd the directory.
    Path { dir: PathBuf },
    /// ball — a ball in `project`; `bl claim`/`create`; driver cwd the worktree.
    Ball { project: String, ball: BallSpec },
}

impl Payload {
    /// The §7.3 banner surface every ops row this rung writes is attributed to
    /// (bl-48f8) — **the rung is the origin**, and it is the one thing the argv
    /// cannot say: a ball-rung start and a bare-rung one write byte-identical
    /// `litany new` / `litany prompt` / `["yog-step","mkdir"]` lines, so a
    /// derivation at read time would hand the balls fold and the composer the
    /// same row and be wrong for one of them every time.
    ///
    /// A ball rung was offered on the roster's balls section — the ▶ Start /
    /// ▶ Continue / Create-&-Start rows — so that is where its whole flow
    /// banners, substrate steps included (§11, bl-6ad8: "banners where the start
    /// was offered … the surface the ▶ Start row itself is on"). The bare and
    /// path rungs are the composer's own Enter, the empty world's bootstrap box
    /// being the same box before a workspace exists.
    /// The project this payload names (REMOTE §8), or `None` for the two rungs
    /// that name none. The one home of that question — the boundary's
    /// after-verb refresh table and the ball rung's own re-plan both ask here.
    pub fn project(&self) -> Option<String> {
        match self {
            Self::Ball { project, .. } => Some(project.clone()),
            Self::Bare | Self::Path { .. } => None,
        }
    }

    pub fn origin(&self) -> crate::opslog::Origin {
        match self {
            Self::Ball { .. } => crate::opslog::Origin::Balls,
            Self::Bare | Self::Path { .. } => crate::opslog::Origin::Conversation,
        }
    }
}

/// The start request the shell hands [`plan`] / [`prepare`] / [`preview`]: the
/// two §3.4 axes plus the roots the worktree and seed paths derive from, and the
/// conversation mint's occupied set. `home` is the bare rung's driver cwd (`~`,
/// resolved from the env at the shell boundary).
///
/// **One input type, because there is nothing left to resolve** (bl-d942). It
/// carried a `Target` and an occupied set of workspace claimants, and
/// [`prepare`] resolved those into a second, name-bearing struct; with workspace
/// names chosen by the operator (§3.1) the target *is* the workspace path and its
/// name *is* the leaf — a computed fact, so it is a query ([`crate::naming::leaf`])
/// and not a field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartInputs {
    /// The target workspace's absolute path (§3.4): the focused workspace (named
    /// **or foreign**), a resume ball's claimant workspace, or `<names-root>/
    /// <name>` for a raise — the operator's typed name, or `home` at bootstrap
    /// (§3.1). Its leaf is the `--as`/`YOG_NAME` stamp. Carrying the path, not a
    /// names-root-relative leaf, is what lets a foreign focus (which lives outside
    /// yog's flat names root) resolve to the right `litany prompt <ws>`. An absent
    /// directory is founded by [`Step::EnsureWorkspace`] — that, and nothing else,
    /// is what "raising a workspace" is.
    pub workspace: PathBuf,
    pub payload: Payload,
    /// The ball rung's project **located** (REMOTE §8, bl-f5f6): where `bl`
    /// runs. [`Payload::Ball`] addresses its project by *name*, because a
    /// payload is a boundary datum and the wire carries no paths; this is that
    /// name resolved once at the dispatch chokepoint, exactly as
    /// [`workspace`](StartInputs::workspace) above is the resolved address of
    /// [`Action::Prepare`](crate::boundary::Action::Prepare)'s own. `None` for
    /// the bare and path rungs, which name no project at all.
    pub repo: Option<PathBuf>,
    pub home: PathBuf,
    pub yog_data_root: PathBuf,
    pub balls_state_root: PathBuf,
    /// The occupied set for the **conversation** mint (§3.3): the stamped names of
    /// the target workspace's live roots, read back from the goals the §11
    /// conversation list already parses. Per-workspace and nothing wider —
    /// workspaces are isolation walls, so two spheres never need distinct names.
    /// Empty for a workspace that does not exist yet: the general path with no
    /// inputs, not a bootstrap case.
    pub conversation_names: Vec<String>,
}

/// One step of the start flow (§8.1). The sequence is a projection; [`prepare`]
/// runs the mutating steps in order and **defers** [`Prompt`](Step::Prompt) to
/// the composer (fired later, edited, by [`execute_prompt`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// **The world is founded** — always planned; both halves converge rather
    /// than branch. The executor seeds `<world>/tools/bl`, the agent-tool shim
    /// (§16.7 W9, rewritten only on drift), then runs `LITANY_HOME=… litany
    /// prime`, skipping a seeded home (§16.6 W3, the general path with the seed
    /// present). A marker step: the executor derives the world layout from
    /// `yog_data_root` (the single source), so the step carries no path of its own.
    EnsureSeeded,
    /// `mkdir -p` + `litany new <workspace>` — always planned; the executor skips
    /// an existing dir (§8.1 convergence; bootstrap is this with an absent dir).
    EnsureWorkspace { workspace: PathBuf },
    /// `bl create <title> [--body B]` — the ball New rung; mints the id, after
    /// which the plan re-derives as an existing ball (§8.1).
    Create {
        project: PathBuf,
        title: String,
        body: String,
    },
    /// `bl claim <id> --as <name>` — the ball rung, unclaimed only; stamped with
    /// the target workspace name (§3.2). Dropped for a bound ball (resume).
    Claim {
        project: PathBuf,
        id: String,
        name: String,
    },
    /// `litany prompt <workspace> <goal>` fired detached, `YOG_NAME=<workspace>`
    /// (the §3.2 stamp *is* the name, §3.1), cwd per the §3.4 rung. `goal` is the
    /// editable payload prefill; the conversation's identity line is minted and
    /// stamped at fire, never carried here (§3.3).
    Prompt {
        /// The workspace's name (REMOTE §8) — the preview of what the fire
        /// re-resolves.
        workspace: String,
        /// The §3.3 typed work target the fire will pass as litany's `--cwd`
        /// (bl-6654) — the plan's preview of it, off the ball's *canonical*
        /// worktree formula; the executor re-derives it from the claim.
        binding: Option<PathBuf>,
        goal: String,
    },
}
