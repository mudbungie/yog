//! **The learning loop's operator half, at the boundary** (§9.6; REMOTE §9.22,
//! bl-dd88): the staged proposals a reviewer left, one of them whole, and the
//! settle that accepts or rejects one.
//!
//! litany's reviewer stages a real config patch on `proposal/<reviewer-id>`
//! (litany `docs/DESIGN_LEARNING_LOOP.md` §3) and the loop's whole design turns
//! on a person being able to see what was learned and veto it. Adopting the
//! loop was already two gestures a seat could make; reading, accepting and
//! rejecting were `litany proposal` and nothing else, so from any seat — the
//! window, the phone, `yog gesture` — a staged patch was invisible. On a server
//! install (DESIGN §10.1) that made the veto an `ssh` and a container `exec`,
//! which is exactly what the §8.5 boundary exists to make unnecessary.
//!
//! **The read is yog's and the act is litany's, and the split is not
//! arbitrary.** yog already reads this workspace's other two ref namespaces
//! itself — `agents/*` for the §7.1 tree, `config/*` for the §9.3 browse — so a
//! listing over the third is that same derivation through the same scrubbed
//! doorway ([`git_tree`](crate::git_tree)'s named vocabulary), and it answers
//! typed rows rather than a rendered table a seat would have to parse back. The
//! **settle** is not read arithmetic: accepting is a compare-and-swap
//! fast-forward whose expected old value is the freshness the listing showed,
//! and re-implementing that here would be a second home for a rule whose
//! failure mode is a lost race. So it stays `litany proposal`, spawned through
//! the §8.2 workspace seam like every other litany verb and leaving that
//! family's ops row.
//!
//! **Freshness is derived at read time and never stored.** `fresh` is two
//! commits compared at the moment it is asked, and the accept re-derives the
//! same test as git's own `update-ref <head> <new> <old>` — so a proposal
//! cannot be listed fresh and then accepted on a stale field, and a race is
//! refused by git rather than by a check that could lose to it.

use std::path::Path;

use crate::git_tree::{
    GitTreeError, REPO_DIR, for_each_ref_config, for_each_ref_proposals, rev_commit, shortstat,
    show_commit, subject,
};

/// The `proposal/` ref-namespace prefix (litany ARCH §2.3's third namespace).
/// Mirrored here for the reason `REPO_DIR` and the `config/` prefix already
/// are: yog reads these refs and does not link the module that names them.
const PREFIX: &str = "proposal/";

/// One staged proposal as a seat reads it. Every field is derived at the moment
/// it is asked; nothing here is stored anywhere.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProposalRow {
    /// The reviewer that staged it — the branch is `proposal/<id>`, and the id
    /// is the vocabulary the settle takes.
    pub id: String,
    /// The config lineages whose head is this proposal's parent. Empty is the
    /// stale reading said as the pool it is, not as a second flag.
    pub lineages: Vec<String>,
    /// The parent commit, abbreviated.
    pub parent: String,
    /// Is that parent still a lineage head? **Derived, never stored** — and
    /// carried on the row rather than left to the seat, because a seat holding
    /// only the two commits could not ask the question without a second read.
    pub fresh: bool,
    /// `git diff --shortstat` against the parent — how big the patch is, which
    /// is what decides whether an operator reads it whole before settling.
    pub diffstat: String,
    /// The commit subject: the reviewer's own first line.
    pub subject: String,
}

/// What a proposals read answers: the listing, and — when the read named one —
/// that proposal **whole**, message and diff.
///
/// **A listing and one entry's bytes are one question asked at two depths**,
/// which is [`Query::Files`](crate::boundary::Query::Files)' shape and taken
/// for its reason: the seat that names an id has already been answered the row
/// it names, so a second query would be a second derivation of one subject.
/// `whole` is `None` for the bare read, which is the general path with no input.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProposalView {
    pub rows: Vec<ProposalRow>,
    pub whole: Option<String>,
}

/// **The operator's verdict on one staged proposal** (litany's §3 *operator
/// verb*): take it onto its lineage, or delete it.
///
/// Two words and no third: there is no *defer*, because a proposal nobody
/// settles is exactly the state the listing already shows, and no *merge*,
/// because a stale proposal's reviewer read a config that no longer governs —
/// litany refuses to merge one forward, and the remedy is a reject plus the
/// next checkpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Fast-forward the proposal's lineage onto it, then delete the branch.
    Accept,
    /// Delete the branch. The reviewer's own branch survives as the record of
    /// its reasoning.
    Reject,
}

impl Verdict {
    /// The word the gesture spells and the wire carries — one table read both
    /// ways, so a spelling cannot drift from its reading.
    pub(crate) fn word(self) -> &'static str {
        match self {
            Self::Accept => "accept",
            Self::Reject => "reject",
        }
    }

    /// The `litany proposal` flag this verdict is: the act's one home is that
    /// verb, so the flag is named here and the settle spends it.
    pub(crate) fn flag(self) -> &'static str {
        match self {
            Self::Accept => "--accept",
            Self::Reject => "--reject",
        }
    }

    /// The verdict a word names, or `None` for anything else.
    pub(crate) fn parse(word: &str) -> Option<Self> {
        [Self::Accept, Self::Reject]
            .into_iter()
            .find(|v| v.word() == word)
    }
}

/// One settle: which workspace, which proposal, and which way.
///
/// **The id is required and the verdict is explicit.** A settle that took "the
/// only one" would do something different the day a second proposal was staged,
/// which is the class of surprise a destructive act must not have; litany's own
/// verb refuses it for that reason and this does not soften it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settle {
    pub workspace: String,
    pub id: String,
    pub verdict: Verdict,
}

impl Settle {
    /// The workspace slot REMOTE §8.2's name→path rewrite borrows — the shape
    /// every other addressed carrier has.
    pub(crate) fn workspace_slot(&mut self) -> &mut String {
        &mut self.workspace
    }
}

/// Every staged proposal of one workspace, and — when `id` names one — that
/// proposal whole. A workspace with none answers an empty listing, which is the
/// general path with no input rather than a refusal.
///
/// An `id` naming no staged proposal refuses in git's own words. That is the
/// same split [`config::read`](crate::boundary::config) already draws: a
/// listing is total, and a read that names one thing must not answer emptiness
/// for a thing that is not there.
pub(crate) fn read(workspace: &Path, id: Option<&str>) -> Result<ProposalView, String> {
    let repo = workspace.join(REPO_DIR);
    view(&repo, id).map_err(|e| e.to_string())
}

/// The read itself, in git's own error type — [`read`]'s body, so the one
/// `map_err` sits at the boundary rather than at every call.
fn view(repo: &Path, id: Option<&str>) -> Result<ProposalView, GitTreeError> {
    let rows = rows(repo)?;
    let whole = match id {
        Some(id) => Some(show_commit(repo, &format!("{PREFIX}{id}"))?),
        None => None,
    };
    Ok(ProposalView { rows, whole })
}

/// The listing, in `for-each-ref` order — which is the ids' own, so two engines
/// render one workspace identically without sharing ordering state (I9).
fn rows(repo: &Path) -> Result<Vec<ProposalRow>, GitTreeError> {
    let listing = for_each_ref_proposals(repo)?;
    let heads = config_heads(repo)?;
    String::from_utf8_lossy(&listing)
        .lines()
        .filter_map(|line| line.split_once(' ').map(|(name, _)| name.to_owned()))
        .map(|name| row(repo, &name, &heads))
        .collect()
}

/// One proposal's row. Its parent is the commit the reviewer read and staged
/// on, and `fresh` is whether any config lineage still stands there — which is
/// exactly the set an accept's fast-forward would move.
fn row(repo: &Path, name: &str, heads: &[(String, String)]) -> Result<ProposalRow, GitTreeError> {
    let parent = rev_commit(repo, &format!("{name}^"))?;
    let lineages: Vec<String> = heads
        .iter()
        .filter(|(_, oid)| *oid == parent)
        .map(|(lineage, _)| lineage.clone())
        .collect();
    Ok(ProposalRow {
        id: name.strip_prefix(PREFIX).unwrap_or(name).to_owned(),
        fresh: !lineages.is_empty(),
        lineages,
        diffstat: shortstat(repo, &parent, name)?,
        subject: subject(repo, name)?,
        parent: parent.get(..8).unwrap_or(&parent).to_owned(),
    })
}

/// Every config lineage and the oid its head stands at — the set `fresh` is
/// derived against, read once per listing rather than once per row.
fn config_heads(repo: &Path) -> Result<Vec<(String, String)>, GitTreeError> {
    let listing = for_each_ref_config(repo)?;
    Ok(String::from_utf8_lossy(&listing)
        .lines()
        .filter_map(|line| {
            let mut it = line.split(' ');
            // `%(refname:short)` yields `config/<name>`; the lineage NAME is
            // what an operator and litany's own accept both speak, so the
            // prefix is stripped here exactly as the §9.3 browse strips it.
            let name = it.next()?.strip_prefix("config/")?.to_owned();
            Some((name, it.next()?.to_owned()))
        })
        .collect())
}

/// The answer's JSON spelling, both directions — beside the type that owns it.
pub mod wire;

#[cfg(test)]
mod tests;
