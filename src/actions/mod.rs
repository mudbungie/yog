//! User actions issued through `cli_outbound` (ARCH §3.4 / §3.5).
//!
//! This root holds every **enablement predicate** (pure, no egui): whether an
//! action is offered for the current selection. The **short, piped, logged**
//! verbs — message/stop/scan and `bl` close/unclaim/create/update — live in
//! [`verbs`], which appends each outcome to `ops.jsonl` (§8.2, §15 Y16); the
//! **detached** `litany prompt` (the new-root launch, §8.1) is now the one
//! [`start::execute_prompt`](crate::start::execute_prompt) path (Y17 moved it
//! onto `spawn_detached`, unifying the composer's new-prompt with the start
//! flow's final prompt — one detached, logged launch, no piped-and-drained
//! variant whose `Stream` drop would SIGTERM the loop on yog's exit).
//!
//! Predicate discipline (§8.2): **Stop** needs a Live/InFlight executor to
//! signal ([`stop_enabled`]); **Nudge** is its complement — the states with no
//! driver holding the lease, less the one shape litany reads as nothing-due
//! ([`nudge_enabled`]); **Message** is the resume
//! gesture and works on *any* selected agent (ARCH §2.9 — no resume verb; its
//! gate's text half is a composer's, so the conjunction is the seat's, bl-7cc8);
//! **Close** needs the ball bound to a local workspace ([`close_enabled`] =
//! `JoinState::Bound`); **Unclaim**/release needs the same
//! ([`unclaim_enabled`]); **Assign** needs a ready, unclaimed
//! ball ([`assign_enabled`] = `JoinState::ReadyStartable`) — each predicate
//! refuses exactly what the underlying `bl` verb would (§8.2/§3.5); **Scan** is
//! unconditional (offered for any focused workspace — no predicate). All are
//! functions of their inputs and carry no egui dependency, reusable by any future
//! frontend. One reads the filesystem — [`work_dir_refusal`], which asks whether a
//! typed work directory is there (bl-6191); it is a *question about an input*, not
//! state, and it is asked here rather than in the coverage-excluded form for the
//! same reason every other refusal is.
//!
//! **A seat's own RAM is not held here** (bl-7cc8): the composer drafts keyed by
//! target (§5.3), the selection they were typed against, and the predicate that
//! decided when a send cleared one were a running frontend's state kept in a
//! server no §8.5 act ever asked for it. They left with the face they served
//! (bl-7942); if a seat ever needs one of these facts stated, it files against
//! yog and the reply is designed then.

/// **Whether a verb is offered for the current selection** (§8.2), split off at
/// §12's budget: this file asks whether there is anything to fire, `enabled`
/// asks whether this selection permits it.
pub mod enabled;
pub mod verbs;

pub use enabled::{
    assign_enabled, close_enabled, nudge_enabled, stop_children_offered, stop_enabled,
    unclaim_enabled,
};

/// The §11 birth-config work-directory field's verdict (bl-6191): the refusal
/// sentence when `typed` names something that is not an existing directory, or
/// `None` when the next start may run there.
///
/// **Empty is lawful, not a refusal** — an emptied box is the bare rung, which
/// resolves to the same home the bare rung resolves to, so it is the general
/// path with no input rather than a case.
///
/// The question itself is the **spawn boundary's**
/// ([`crate::cli_outbound::work_dir_fault`]), asked here before Enter fires
/// anything instead of after a fork has already misattributed it: `std::process`
/// reports a bad `current_dir` as ENOENT against the *program*, so the operator
/// who typed a bad directory was told their binary was missing. One reading, one
/// sentence, both layers.
pub fn work_dir_refusal(typed: &str) -> Option<String> {
    let dir = typed.trim();
    if dir.is_empty() {
        return None;
    }
    crate::cli_outbound::work_dir_fault(std::path::Path::new(dir)).map(|e| e.to_string())
}

/// True iff `goal` is a goal at all (§8.1): at least one non-whitespace
/// character. **The one definition of "blank", shared by every site a goal can
/// fire from** — the composer's Enter and the bootstrap box (through
/// [`new_prompt_enabled`], which adds the work-directory half), and the start
/// pane's Send / its §11 Enter binding, which have no directory to ask about
/// (the planner resolved their cwd).
///
/// It also decides whether a start **opens** a goal draft at all: a rung whose
/// prefill is blank composed nothing to edit, so there is nothing to draft
/// (§3.4's table — the bare rung's prefill is "none"). Before bl-9acf the raise
/// opened one anyway, and its Send fired the identity preamble followed by
/// nothing onto the wire — spend with no instruction behind it.
pub fn goal_present(goal: &str) -> bool {
    !goal.trim().is_empty()
}

/// True iff the composer's Enter may fire a new conversation (§11): there is
/// something to say ([`goal_present`]) **and** somewhere lawful to say it
/// ([`work_dir_refusal`] on the birth block's work directory). One predicate
/// rather than two because they arm one gesture; the field's own red flag is
/// what tells the operator *which* half refused.
pub fn new_prompt_enabled(input: &str, work_dir: &str) -> bool {
    goal_present(input) && work_dir_refusal(work_dir).is_none()
}

/// A new ball's Create & Start is offered iff its title is non-blank (§8.1): a
/// ball with no title has nothing to name the work. A distinct §3.5 rule from
/// [`new_prompt_enabled`] though the current bodies coincide (as [`close_enabled`]
/// and [`unclaim_enabled`] share `Bound`) — each names the rule
/// its `bl`/start path enforces. Covered here, not inlined in coverage-excluded
/// shell glue.
pub fn create_ball_enabled(title: &str) -> bool {
    !title.trim().is_empty()
}

/// The new-ball form's two placeholder hints (§8.1, bl-b2ed). The form is two
/// bare `TextEdit`s and a button; empty, nothing distinguished the title box
/// from the body box, nor said the second was multiline. The hints are the
/// affordance, so they live here beside the form's other rule
/// ([`create_ball_enabled`]) rather than as literals in coverage-excluded shell
/// glue. The body's hint states the goal the way the composer's does ("say what
/// you want done"), not a bare field name — a body that is not a definition of
/// done is what the start flow hands the agent as its preamble (§8.1).
pub fn new_ball_hints() -> NewBallHints {
    NewBallHints {
        title: "title".to_owned(),
        body: "body — what done looks like".to_owned(),
    }
}

/// The [`new_ball_hints`] pair, named so the form cannot swap them.
#[derive(Debug, PartialEq, Eq)]
pub struct NewBallHints {
    /// Placeholder for the single-line title box.
    pub title: String,
    /// Placeholder for the multiline body box.
    pub body: String,
}

#[cfg(test)]
mod tests;
