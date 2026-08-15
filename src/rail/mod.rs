//! The **step spine** — every operable commit a conversation has, and what
//! hangs off each one (VISION V1, the Historian rung; DESIGN §11).
//!
//! One **notch** per step, each notch that step's read-state commit — the
//! branch tip the model call was assembled against, already recorded in
//! `meta.json` and already read by the Steps view (§5.1 #29). Selecting a notch
//! **pins** the whole inspector to that commit (the [`pin`] submodule):
//! transcript as of, agent-context files as of, config frozen at, budget folded
//! to that point.
//!
//! **The spine is drawn through the chat, not beside it** (bl-1802). It used to
//! be a `SidePanel` gutter of notches; the operator's ruling retired that seat:
//! *"every operable commit should be a horizontal rule across the chat, instead
//! of a window on the side. the fork overlay should show up when you click on
//! one."* The transcript was already drawing one faint rule per commit boundary
//! (§5.1 #29) — the same commits, from the same `meta.json`, rendered twice —
//! so the rule **is** the notch, and the gutter is gone. Each notch's seat in
//! the chat is its [`Place`], derived by the [`place`] submodule; the painting
//! is `transcript::spine`.
//!
//! **Two edges, one rendering** (VISION V1.3). A child's *context* edge — what
//! it inherited — is git ancestry; its *provenance* edge — who dispatched it —
//! is the descent id plus the parent-transcript notch where the dispatch
//! landed. A clean child (`lernie dispatch --from config/<name>`) has
//! provenance only. The taxonomy is intact and still derived here; what it is
//! **drawn** as changed with the seat. A gutter had room for two strokes, solid
//! and dashed; a rule across a chat has no gutter to stroke in — and the fact
//! those strokes drew is already stated in words by the card's own fork label
//! (`from here` / `from <Name>@<oid>` carry ancestry, `from config/<name>`
//! carries provenance alone). Two renderings of one fact is one too many
//! (single source of truth), so the words survived the move and the strokes did
//! not. §11's descent tree stays the descent-**id** tree either way (§5.1 #8).
//!
//! **Both edges are derived, and neither costs a git call.** An agent's
//! `steps` list is `git log --first-parent <branch> --not --branches=config/*`
//! (§5.1 #8), so a *fork* child's list opens with the parent's own commits up
//! to the fork point and a *clean* child's shares nothing with it. The longest
//! common prefix of the two lists therefore **is** the fork point, and its
//! emptiness **is** cleanness — both read off facts the snapshot already
//! carries. The dispatch notch is located the same way for both kinds: the
//! last notch whose commit is no later than the child's own first commit.
//!
//! Everything here is a pure read of the lernie workspace repo — refs, trees,
//! commits — derived, never pushed (VISION V1.6). The card updates on the
//! ordinary fs_watcher + off-thread snapshot read like every other §5.1
//! derivation, and the frame renders snapshots only.
//!
//! Burden check, verbatim from VISION V1: *"no dispatches → no cards, no
//! edges; the rail collapses to today's transcript for anyone who never clicks
//! a notch — the S0 stranger sees today's transcript exactly."* It needs no
//! `navigable` gate any more, and that gate is deleted: the rules the chat
//! draws are the ones bl-929d already shipped, so a conversation nobody forked
//! from paints exactly what it painted before — one faint line per commit
//! boundary, now clickable — and no cards, because there are none.

use std::collections::BTreeMap;

use crate::git_tree::{AgentState, StepCommit};
use crate::steps_view::StepsView;
use crate::transcript::Transcript;

mod cards;
pub mod cohort;
mod pin;
mod place;
pub(crate) mod tree;
pub(crate) mod wire;

pub use cohort::{Cohort, cohorts};
pub use pin::{Pin, pin, transcript_as_of};
pub use place::Place;
pub use tree::{files_at, preview_at};

/// Short-oid width, git's own default — the width the Steps list's Commit
/// column and the transcript's boundary rules already wear.
pub(crate) const SHORT_OID: usize = 7;
/// What a notch shows when its step recorded no read-state commit: the call is
/// in flight, or died before `meta.json`. Absence is said, never guessed.
const NO_COMMIT: &str = "—";

/// One notch: a step, and the read-state commit its model call was assembled
/// against. `commit` is `None` for a step that landed no `meta.json` — such a
/// notch is a point on the spine but not a pinnable one, because there is
/// no tree to pin to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notch {
    /// The step's zero-padded sequence name (`001`).
    pub seq: String,
    /// `meta.json`'s `commit` — the branch tip at step-start (§5.1 #29).
    pub commit: Option<String>,
    /// **The budget as of this notch** (VISION V1.2): every notch's spend up to
    /// and including this one — a per-row **rollup**, not this step's own
    /// figure. REMOTE §9.7's altitude ruling (bl-44e9): the pin is a *selection*
    /// out of an answer, so what a pinned tab shows has to be on the notch the
    /// operator picked rather than summed by whoever picked it. A seat that
    /// folded the prefix itself would be deriving over a reply, which is the one
    /// thing the read path exists to stop.
    pub budget: u64,
    /// Where in the chat this notch's rule paints, and what its pin cuts to
    /// ([`place`]). `None` for a notch the chat has no seat for — a call that
    /// sealed no output and was superseded — which is therefore a notch no
    /// gesture can reach, because a rule is the only way to reach one.
    pub place: Option<Place>,
}

impl Notch {
    /// The notch's label: its read-state commit clipped to short-oid width, or
    /// [`NO_COMMIT`] for a step that recorded none. Derived at render — the
    /// commit IS this string's storage.
    pub fn short(&self) -> String {
        self.commit.as_ref().map_or_else(
            || NO_COMMIT.to_owned(),
            |oid| oid.get(..SHORT_OID).unwrap_or(oid).to_owned(),
        )
    }
}

/// The live inline card at a dispatch notch (VISION V1.4): who the child is,
/// where it forked from, what it is doing, what it has spent, and the last of
/// its in-flight inference text. `provenance_notch` is the notch the card hangs
/// from — the rule in the chat where this child was born. The *context* edge is
/// not a second field: it is what [`ChildCard::fork`] says in words, and a
/// stored index nothing reads would be that fact's second home.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChildCard {
    pub agent_id: String,
    /// The child's §3.3 display name — the same ladder every other seat reads.
    pub name: String,
    /// The fork-point label: `from here` / `from config/<name>` /
    /// `from <Name>@<oid>`.
    pub fork: String,
    pub state: AgentState,
    /// The per-agent fold of `steps/<id>` (VISION V1.5) — this child's own
    /// spend, never its descent's.
    pub tokens: u64,
    /// The streaming tail: moving text means active, still text means
    /// tool-wait or quiescent. `None` when the child has produced no
    /// inference text yet.
    ///
    /// **This is also V2.3's terminal-response preview, and it is one fold,
    /// not two** (bl-dc0c). `Agent::stream.text` is the accumulated text of
    /// the *latest* step's `response.json` (§5.1 #10), re-read every tick
    /// regardless of state — so while the child runs it is the live tail, and
    /// once the child settles the same bytes are the last thing it said. A
    /// separate "terminal response" derivation would be a second reader of one
    /// file, disagreeing with this one at every moment the file is still open.
    pub tail: Option<String>,
    pub provenance_notch: usize,
}

/// The spine: the parent's notches, and the cards hanging off them.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Rail {
    pub notches: Vec<Notch>,
    pub cards: Vec<ChildCard>,
}

impl Rail {
    /// Which notch each chat row's rule belongs to — the render's one lookup,
    /// derived per paint from the notches themselves rather than carried
    /// beside them. A notch with no [`Place`] is absent, which is exactly what
    /// makes it unreachable; no two notches share a row, because each place
    /// consumes its own run of entries.
    pub fn rules(&self) -> BTreeMap<String, usize> {
        self.notches
            .iter()
            .enumerate()
            .filter_map(|(index, notch)| notch.place.as_ref().map(|at| (at.row.clone(), index)))
            .collect()
    }
}

/// What [`build`] needs about one child, all of it already on the snapshot
/// (`Agent`) or already folded for another seat. Owned, so the rail holds the
/// whole card in hand and borrows nothing from the tree beneath it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChildInput {
    pub agent_id: String,
    pub name: String,
    pub state: AgentState,
    /// The child's `Agent::stream.text` (§5.1 #10) — the same fold the
    /// in-flight strip reads, pointed at this agent id.
    pub streaming_text: Option<String>,
    /// The child's `Agent::steps` (§5.1 #8), oldest first.
    pub commits: Vec<StepCommit>,
    /// The per-agent `steps/<id>` fold (`budgets::Scope::Agent`).
    pub tokens: u64,
    /// The child's governing config branch name, when its governing commit is
    /// the tip of one (§5.1 #17) — what makes a clean child's label name the
    /// branch it started from rather than an oid.
    pub config_label: Option<String>,
}

/// Derive the spine for one focused agent. `parent_name` labels a fork point
/// that is not a notch; `parent_commits` is the parent's `Agent::steps`, whose
/// order gives every notch commit a position; `steps` supplies the notches;
/// `transcript` gives each notch its seat in the chat ([`place`]); `children`
/// are the parent's direct descent-id children.
pub fn build(
    parent_name: &str,
    parent_commits: &[StepCommit],
    steps: &StepsView,
    transcript: &Transcript,
    children: &[ChildInput],
) -> Rail {
    let places = place::places(transcript, &steps.steps);
    let notches: Vec<Notch> = steps
        .steps
        .iter()
        .zip(places)
        .scan(0u64, |spent, (step, place)| {
            *spent += step.tokens.total_tokens();
            Some(Notch {
                seq: step.seq.clone(),
                commit: step.commit.clone(),
                budget: *spent,
                place,
            })
        })
        .collect();
    let cards = children
        .iter()
        .filter_map(|child| cards::card(parent_name, parent_commits, &notches, child))
        .collect();
    Rail { notches, cards }
}

#[cfg(test)]
mod tests;
