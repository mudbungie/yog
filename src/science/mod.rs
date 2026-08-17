//! The **attempt science projection** — one derived row per delivery attempt
//! (VISION §4.10 item 7, bl-40ab; DESIGN §3.9).
//!
//! §4.10 item 7, verbatim: *"The science projection is a query: frozen inputs
//! (goal, pins, governing config commit — model and skills ride it),
//! base/source/target/delivered OIDs, terminal response, usage, wall time,
//! project diff, verdicts (messages), and the accepted/rejected/reworked
//! outcome — all derived at read time from lernie step records, balls delivery
//! identities, and git ancestry. Nothing stored."*
//!
//! So this module owns **no fact**. Every column names an authority that
//! already had it, and the projection's whole content is the join:
//!
//! | column | authority |
//! |---|---|
//! | the diff, its two refs and their OIDs, the acceptance mark | [`crate::workdiff`] (§5.1 #32) — composed, not restated |
//! | the conversation bound to the attempt, and its frozen pins | the §4.2 trail's own fire row ([`crate::fan::fires`]) |
//! | the goal | the agent worktree's `goal.md`, frozen at the dispatch commit |
//! | the governing config commit | §5.1 #17's walk ([`crate::config_edit::branch`]) |
//! | usage, wall time, step count | lernie's step records, off the one walk already published as `Snapshot::bills` |
//! | the terminal response and the verdicts | the committed `messages/` tree ([`crate::transcript`]) |
//! | the base the two ends departed from | git: `merge-base target source`, balls' own base formula ([`outcome`]) |
//! | accepted / rejected / reworked | git: the target's history and its ancestry ([`outcome`]) |
//!
//! **The projection composes the work-diff row rather than repeating it.** The
//! diff, the two refs, both OIDs and the derived acceptance mark are already one
//! derivation with one home and one wire spelling; a science row that restated
//! them would be VISION §4.5's two-representations-of-one-fact, and the two
//! would drift the first time a `Change` arm changed. So [`Attempt::diff`] *is* a
//! [`workdiff::Attempt`](crate::workdiff::Attempt), spelled by that module's own
//! codec in both directions.
//!
//! **It is a read, and reads nothing twice.** The step-record columns come off
//! `Snapshot::bills` — the walk the derivation worker already made (bl-9dd4) —
//! filtered in memory by the attempt's own conversation, so a projection over a
//! workspace of ten attempts makes no extra pass over `steps/` at all. The only
//! disk reads it adds are the frozen inputs of each row and, when a sibling has
//! landed, one ancestry probe per superseded attempt.
//!
//! **The §11 fan-group seat renders this projection** (bl-77bc): [`render`]
//! is the group card, [`compose`] turns its affordance clicks into composer
//! text, and [`respdiff`] is V3.3's response comparison — all consumers of the
//! rows above, owning no fact of their own.

use std::path::Path;

use balls::layout::Xdg;

use crate::app::Snapshot;
use crate::budgets::BudgetSpend;
use crate::opslog::OpEntry;

mod bound;
pub mod compose;
mod observed;
mod outcome;
pub mod render;
pub mod respdiff;
pub(crate) mod wire;

pub use outcome::Outcome;

#[cfg(test)]
mod tests;

/// One delivery attempt, as science reads it (VISION §4.10 item 7). Held
/// nowhere: every field is re-derived per ask, like every other §5.1
/// projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attempt {
    /// The attempt's identity and its project diff — the §5.1 #32 row,
    /// composed. It carries the project, the ball, the handle (`None` for the
    /// ordinary claim attempt), the derived acceptance mark and the
    /// `target..source` read with both OIDs.
    pub diff: crate::workdiff::Attempt,
    /// The commit both ends departed from — the third OID §4.10 item 7 names,
    /// and half of item 6's cohort key. It rides the science row rather than the
    /// diff row because it is a fact about the *cohort* and not about the churn:
    /// `Query::WorkDiff` would pay a git read it never renders. `None` when
    /// there is no resolved pair to compare ([`outcome::base`]).
    pub base: Option<String>,
    /// The conversation bound to this attempt — its **agent id**, which is the
    /// address every conversation gesture takes. `None` when no fire in this
    /// workspace bound it, and when the fire's driver has not written a branch
    /// yet: in both the attempt exists and nobody has spoken in it, which is a
    /// reading and not an error.
    pub conversation: Option<String>,
    /// The goal that fire carried, verbatim — the frozen input, read from the
    /// dispatch commit's own `goal.md`. `None` for an attempt with no
    /// conversation, and for one whose worktree will not read.
    pub goal: Option<String>,
    /// The `<dest>=<src>` instruction documents that fire froze (§3.7).
    pub pins: Vec<String>,
    /// The config commit this conversation is frozen on (§5.1 #17). **The
    /// model and the skills ride it** and earn no columns of their own — that
    /// is VISION §4.10 item 7's own parenthesis, and it is the single-source
    /// rule: the commit is where those facts live, so a `model` field here
    /// would be a second copy of one that could disagree with the config the
    /// step actually ran under.
    pub governing: Option<String>,
    /// What this attempt's conversation and its whole descent burned — the four
    /// ARCH §6 counters. A judge or synthesis child dispatched *from* the
    /// attempt is part of its cost, which is why the fold is tree-wide.
    pub usage: BudgetSpend,
    /// Seconds of model-call wall time over the same tree (`meta.json` spans,
    /// summed per step). Zero for an attempt whose steps are still unsettled —
    /// an honest unknown, never an elapsed-since-start guess.
    pub wall_secs: u64,
    /// How many steps that tree has taken.
    pub steps: usize,
    /// The attempt's **terminal response**: the text of the last committed
    /// model turn. Committed only — an in-flight tail is not a terminal
    /// anything, and folding one would make this column say something a re-read
    /// a second later contradicts. A live tail is [`Query::Transcript`]'s
    /// answer, where §8.5 rules that it *is* folded.
    ///
    /// [`Query::Transcript`]: crate::boundary::Query::Transcript
    pub response: Option<String>,
    /// The messages delivered into this attempt's conversation, oldest first —
    /// **the verdicts** (§4.10 item 7). A judge's verdict is an ordinary
    /// committed message (VISION V3.1) and yog classifies no prose: what a
    /// message *means* is the reader's, so every delivered message rides and
    /// none is filtered on its wording. Empty for an attempt nobody has
    /// written to.
    pub verdicts: Vec<Verdict>,
    /// How many `messages/` entries the conversation's counter proves
    /// **compacted away** (§5.1 #12, bl-fde5). Zero is an intact record — the
    /// general path. Nonzero says [`verdicts`](Self::verdicts) and
    /// [`response`](Self::response) were read over a **rewritten** record:
    /// verdicts delivered in a squashed span are gone from disk and are not
    /// guessed at, so this column is the projection stating its own bound
    /// rather than the reader inferring one from a short list.
    pub compacted: usize,
    /// Accepted, rejected, reworked — or pending, which is all three's absence
    /// ([`Outcome`]).
    pub outcome: Outcome,
}

/// One message delivered into an attempt's conversation: who sent it and what
/// it said, verbatim. The envelope-stripped body [`crate::transcript`] already
/// parses — a second parse of a `messages/` entry would be a second reading of
/// one file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    /// The filename's origin token — the sender, as lernie recorded delivery.
    pub sender: String,
    pub body: String,
}

/// Project every attempt the workspace at `workspace` holds (§3.9). The row set
/// is exactly [`crate::workdiff::read`]'s — the ordinary claim attempt and each
/// §3.8 fan candidate — because that is what "one row per attempt" already
/// means on this world, and a second enumeration would be a second answer to
/// "which attempts are there".
///
/// A workspace holding no delivery obligation projects nothing: a bare or path
/// start has no attempt (VISION §4.10 item 8), which is the general path with
/// no inputs.
pub fn project(
    snap: &Snapshot,
    workspace: &Path,
    entries: &[OpEntry],
    xdg: &Xdg,
    balls_state_root: &Path,
) -> Vec<Attempt> {
    let diffs = crate::workdiff::read(snap, workspace, entries, xdg);
    let fires = crate::fan::fires(entries, workspace);
    let claimant = crate::binding::named_of(&snap.workspaces, workspace).unwrap_or_default();
    let layout = bound::Layout::of(xdg, balls_state_root, &claimant);
    diffs
        .iter()
        .map(|diff| {
            let repo = snap.project_path(&diff.project).ok();
            let fire = bound::fire_for(&fires, diff, &layout, repo.as_deref());
            row(snap, workspace, diff, &diffs, repo.as_deref(), fire)
        })
        .collect()
}

/// One row: the composed diff, whatever the bound conversation can be asked,
/// and the derived outcome. The three come from three different authorities and
/// are assembled here — this function *is* the join, and there is nothing else
/// in it.
fn row(
    snap: &Snapshot,
    workspace: &Path,
    diff: &crate::workdiff::Attempt,
    siblings: &[crate::workdiff::Attempt],
    repo: Option<&Path>,
    fire: Option<crate::fan::Fire>,
) -> Attempt {
    let agent = fire
        .as_ref()
        .and_then(|f| bound::agent_of(snap, workspace, &f.conversation));
    let seen = agent
        .as_deref()
        .map_or_else(observed::Observed::default, |id| {
            observed::observed(snap, workspace, id)
        });
    Attempt {
        diff: diff.clone(),
        base: outcome::base(diff, repo),
        conversation: agent,
        goal: seen.goal,
        pins: fire.map(|f| f.pins).unwrap_or_default(),
        governing: seen.governing,
        usage: seen.usage,
        wall_secs: seen.wall_secs,
        steps: seen.steps,
        response: seen.response,
        verdicts: seen.verdicts,
        compacted: seen.compacted,
        outcome: outcome::of(diff, siblings, repo),
    }
}
