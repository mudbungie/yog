//! The §11 conversation row's **inert shapes**: one list row and the §3.3 ball
//! beside it, plus the two facts a row answers about itself (has it descent at
//! all, and the subagent field's total) — derived from what it already carries
//! rather than stored twice.
//!
//! [`super`] is the projection that fills these in. Split from it at §12's
//! pre-split band on the same seam `git_tree::model` draws one subsystem over:
//! the type a seat holds is not the fold that built it, and only one of the two
//! has anything to do with agents on disk.

use super::super::display_name;
use super::super::flight::Flight;
use crate::git_tree::AgentState;
use crate::nav::convs::Tone;
use crate::projects::join::JoinState;

/// The conversation's associated start-flow ball (DESIGN §3.2, §3.3, §3.5): the
/// `id` is the goal stamp — source 1, the *only* per-conversation attribution
/// that exists (§3.2: agent-picked balls bind at the workspace level, not the
/// conversation). The join facts (`state`/`title`/`badge`) come from the §3.5
/// claimant join and are `None` when no live/closed ball matches the stamped id
/// here (project unfetched, or a stray id) — the badge still renders from source 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvBall {
    pub id: String,
    pub state: Option<JoinState>,
    pub title: Option<String>,
    pub badge: Option<String>,
}

/// One list row (§11): the id of the agent this row is rooted at, the state
/// badge aggregated over **its** subtree (+ §10 uncertainty), the first-line
/// preview, seconds since that subtree's last activity, the live-activity
/// class, its attention count, its member count (this agent + its descendants),
/// how deep it hangs and how many children it dispatched itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvRow {
    /// The agent this row is the subtree of — the conversation root at depth 0,
    /// and the member itself at every depth below (bl-fa82).
    pub root_id: String,
    pub state: AgentState,
    pub uncertain: bool,
    pub preview: String,
    pub age_secs: i64,
    /// What kind of work is in flight here (§5.1 #28), by the operator's
    /// priority `inference > tools > subagents`; `None` when the conversation
    /// is at rest. The one carrier of "this row pulses" — there is no separate
    /// streaming flag, because `Some(Inference)` *is* that fact.
    pub flight: Option<Flight>,
    pub attention: usize,
    pub members: usize,
    /// How far this row hangs under its conversation root (§11's per-depth
    /// indent and its `↳` elbow): 0 for a root, +1 per descent generation.
    pub depth: usize,
    /// How many agents this one dispatched **itself** — the strict §5.1 #8
    /// [`children_of`](crate::git_tree::children_of) count, never the Stop menu's looser prefix test. The
    /// subagent field's first number; the second is [`total`](Self::total),
    /// derived from `members` rather than stored beside it.
    pub direct: usize,
    /// The conversation's start-flow ball (§3.3), derived from the root's goal
    /// stamp and coloured by the §3.5 join. `None` for a bare/path conversation.
    pub ball: Option<ConvBall>,
    /// The conversation's own name (§3.3): the root's
    /// [`name_fact`](crate::git_tree::Agent::name_fact) — the litany-stored
    /// blob, else the legacy goal-stamp parse — the display ladder's first
    /// rung. `None` for a foreign or hand-typed root, which lands on the
    /// payload line or the id.
    pub name: Option<String>,
    /// Whether [`name`](ConvRow::name) is the **legacy display-only rung**
    /// (bl-8068, [`Agent::name_display_only`]): a goal-stamp parse no
    /// litany-stored fact backs, so peers cannot message the conversation by
    /// this name. The row hovers [`crate::theme::NAME_DISPLAY_ONLY`] on it,
    /// and the boundary withholds the `name` key entirely.
    pub name_display_only: bool,
    /// The conversation's **standing alignment verdict** (VISION §4.9, rung
    /// V6): the worst of its members' latest monitor checks, with the reason
    /// and the sha it read. Derived from the ops tail on every build, never a
    /// stored flag — so an unarmed workspace, or one whose verdicts have aged
    /// out of the rendered tail, simply carries `None` and renders nothing.
    pub verdict: Option<crate::monitor::Check>,
    /// Whether §8.2's `Stop` is offered on this row — its agent holds a driver
    /// right now ([`stop_enabled`](crate::actions::stop_enabled)). **Not
    /// [`state`](ConvRow::state)**, which is the badge aggregated over the
    /// row's whole subtree: a quiet root with a working child paints Live and
    /// has nothing to kill. On the row since bl-1eb0, because a seat that
    /// cannot answer it cannot paint the row's own menu.
    pub stoppable: bool,
    /// Whether the `+children` cascade is offered beside it — some other agent
    /// id here descends from this one by the Stop menu's **looser prefix test**
    /// ([`stop_children_offered`](crate::actions::stop_children_offered)),
    /// which is `litany stop --children`'s own rule and deliberately not the
    /// strict §5.1 #8 descent [`members`](ConvRow::members) counts.
    pub stop_children: bool,
    /// How solid the row paints (§11, bl-915e): [`Tone::Weak`] while this is
    /// §7.2's **pending conversation** — a start yog has fired whose driver has
    /// not written a branch, so the row is only yog's own word for it — and
    /// [`Tone::Plain`] once the derivation carries it. The one input is
    /// [`Agent::in_memory`], so nothing here decides; it reads.
    pub tone: Tone,
}

impl ConvRow {
    /// Whether this row's agent has any descent below it — the §11 gate for
    /// painting the subagent field at all, so a lone root's row carries no
    /// arrow and nothing to unfold.
    pub fn has_children(&self) -> bool {
        self.members > 1
    }

    /// The subagent field's **total**: every agent under this one, at any
    /// depth. Derived from `members` (this agent plus its descendants) rather
    /// than stored beside it — two integers deriving from each other are two
    /// chances to disagree.
    pub fn total(&self) -> usize {
        self.members.saturating_sub(1)
    }

    /// The row's display name — [`display_name`]'s ladder over this row's own
    /// rungs, the one naming rule every seat shares.
    pub fn display_name(&self) -> String {
        display_name(self.name.as_deref(), &self.preview, &self.root_id)
    }

    /// The weak subtitle beside the title (§11: "the name with its preview weak
    /// beside it"): the first payload line — empty when the ladder already spent
    /// it as the title, so no row ever says the same thing twice.
    pub fn subtitle(&self) -> String {
        match self.name {
            Some(_) => self.preview.clone(),
            None => String::new(),
        }
    }
}
