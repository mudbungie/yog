//! **What the agent actually changed** — the project work-diff (DESIGN §5.1
//! #32, §11 Altitude-2 Work tab; VISION §4.10, bl-2b8c).
//!
//! The ruling this rung is built on, verbatim (VISION §4.10 item 4): *"the
//! project diff is a pure git read (`target..source`); a missing project repo
//! renders as a named absence, never a guess"*. Both ends are already derived
//! facts, so nothing new is stored and no verb is spent:
//!
//! - the **source** is the ball's claim — `work/<id>`, spelled by balls' own
//!   [`work_branch`](balls::delivery_path::work_branch), never by a literal
//!   here;
//! - the **target** is what `bl close` already derives — the parent's work
//!   branch for a close-gating subtask, else the project's integration branch
//!   (`git symbolic-ref --short HEAD`, balls' own spelling). The graph
//!   arithmetic is [`plan`]'s; the repo names the integration branch.
//!
//! The binding from the seat is the §3.2 claimant equality: the focused
//! conversation's workspace holds the balls whose claimant is its name. A
//! workspace holding two is two attempts, both shown — the layer that binds a
//! *conversation* to a project directory is litany's working-directory mark,
//! which yog passes typed only once bl-6654 lands, and until then a
//! per-conversation answer would be a guess.
//!
//! **Everything unreadable is said, not swallowed** (the §4.10 mandate): a
//! project repo that cannot name a branch reads as [`Change::Unreadable`], a
//! ref that does not resolve as [`Change::Absent`] naming exactly which end is
//! missing, and a resolved pair with no changes as an empty [`Change::Diff`] —
//! three distinct states, never one silent empty listing.
//!
//! The listing is churn counts (`--numstat`), bounded at the Files tab's own
//! [`MAX_ENTRIES`]; one file's patch is read only when asked for
//! ([`patch`]) and comes back through the same
//! [`Preview`] vocabulary the Files tab already uses. Both reads are memoized
//! per snapshot by the seat that asks (§7.2), never per frame.

mod candidates;
mod plan;
/// **The read itself**, split off at §12's budget: this file is the vocabulary
/// an answer is said in, `read` is the git read that says it.
mod read;
mod render;

pub use read::{patch, read};
pub(crate) mod wire;

pub use render::render;

/// How much one file changed. `Binary` is git's own `-`/`-` numstat row said
/// as itself: a file that changed by an amount lines cannot express.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Churn {
    Text { added: u64, removed: u64 },
    Binary,
}

/// One changed file in an attempt's diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChurn {
    /// The path as git names it — a rename's `{old => new}` composite included.
    pub path: String,
    pub churn: Churn,
}

/// What the project repo could say about one attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    /// The project path is not a readable git repo, or its `HEAD` names no
    /// branch — either way it can state no target, so there is no comparison
    /// to make and the surface says so.
    Unreadable,
    /// One or both ends of `target..source` do not resolve here; `missing`
    /// names them.
    Absent {
        target: String,
        source: String,
        missing: Vec<String>,
    },
    /// The read: the two refs, the two commits they resolved to, and the
    /// per-file churn between them. `files` empty means the attempt has
    /// changed nothing yet — a fact, and a different one from every arm above.
    Diff {
        target: String,
        source: String,
        target_oid: String,
        source_oid: String,
        files: Vec<FileChurn>,
        truncated: bool,
    },
}

/// One delivery attempt as this seat can read it (VISION §4.10 item 1): the
/// project it lands in, the ball whose claim materialized it, and what git
/// says about the two ends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attempt {
    /// The project's **wire name** (REMOTE §8, bl-ccf7) — the §5.1 #1 identity
    /// [`Snapshot::project_name`](crate::app::Snapshot::project_name) derives,
    /// which is the same word the §11 roster labels it with and the same word
    /// `--project` takes. It carried the absolute repository path until this
    /// ball: the last row of §8's residual list that identifies rather than
    /// locates, and the one a client on another machine could neither resolve
    /// nor unsee.
    pub project: String,
    pub ball_id: String,
    /// balls' opaque attempt handle when this row is a §3.8 fan candidate
    /// (bl-c2bd); `None` is the ordinary claim attempt, whose source is
    /// `work/<id>` rather than `attempt/<handle>`.
    pub handle: Option<String>,
    /// The delivery commit the target's history records for this attempt — the
    /// **derived acceptance mark** (VISION V3.2), never a stored winner. `None`
    /// is one fact covering pending and rejected alike, because rejection is
    /// the absence of a delivery (§4.10 item 6).
    pub delivered: Option<String>,
    pub change: Change,
}

/// One file of one attempt — what a patch read is asked for. The ball id
/// rides along because a workspace may hold more than one attempt and a bare
/// path would not say which diff it belongs to; the handle rides for the same
/// reason one row deeper — a fan's candidates all carry the obligation's ball,
/// and only the handle says which candidate's diff the path belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkFile {
    pub ball: String,
    pub handle: Option<String>,
    pub path: String,
}

impl Attempt {
    /// The exact range this attempt is read at, `target..source` — the literal
    /// spelling of the ruling, and what the seat shows so the operator can run
    /// the same read in a shell. `None` for an unreadable project.
    pub fn range(&self) -> Option<String> {
        match &self.change {
            Change::Unreadable => None,
            Change::Absent { target, source, .. } | Change::Diff { target, source, .. } => {
                Some(format!("{target}..{source}"))
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod tests;
