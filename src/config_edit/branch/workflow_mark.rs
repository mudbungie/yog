//! **The workflow mark, read** (§9.4 as amended by bl-b680; litany ARCH §6
//! *The workflow mark*, upstream bl-5c02) — the one control fact with a
//! per-agent override, and the second per-agent fact this tree ports rather
//! than scrapes.
//!
//! litany's `workflow.yaml` follows the governing lineage's tip like every
//! other control fact ([`super::follow`]), and `litany workflow <ws> <agent>
//! --config <lineage>` writes the **standing** ref
//! `refs/litany/workflow/<agent>` at that lineage's head: from the agent's
//! next step boundary on, *that* commit's `workflow.yaml` governs instead of
//! the followed tip's, until re-marked or cleared. Resolution is **nearest
//! mark on the agent's descent** — the agent's own id, then each ancestor by
//! the §2.3 token arithmetic — so marking a root switches its whole tree and
//! a child's own mark overrides its ancestors'. This file is a faithful port
//! of litany's `prompt/resolve/workflow_source.rs::nearest_mark` and
//! `cmd/workflow.rs::lineage_at`, on [`super::follow`]'s precedent exactly:
//! litany keeps its `workspace` module crate-private, the verb's read mode
//! answers one prose line on stdout, and bl-b95e ruled a sentence litany is
//! free to reword a diagnosis and never a trigger — so both sides run the
//! same git query against the same refs, which is the one place the fact
//! lives.
//!
//! **What it says, and what it does not.** A mark moves `workflow.yaml`
//! alone: `providers.yaml`, the souls and the manifest keep following the
//! tip. So the answer rides *beside* [`super::GoverningConfig`] on the wire
//! rather than replacing its `oid` — the commit-side facts still describe
//! the followed tip, and this names the one file that comes from elsewhere.
//! The **lineage** is recovered from the marked commit rather than stored
//! with it, because the mark names a commit and not a name: a lineage that
//! advances after the mark changes nothing the agent reads, and a mark left
//! standing on an older commit is rendered as the absence it is (`None`)
//! rather than declined.

use super::config_branches;
use crate::git_tree::{GitTreeError, REPO_DIR, parent_id, rev_parse};
use std::path::Path;

/// litany's mark namespace for the workflow override (ARCH §6).
const WORKFLOW_REF: &str = "refs/litany/workflow/";

/// A standing workflow mark on a conversation's descent: **who holds it**,
/// the config commit it names, and the `config/*` lineage standing exactly on
/// that commit when one does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowMark {
    /// The descent id carrying the mark — the agent's own, or the ancestor
    /// whose mark it inherits (marking a root switches its tree, §6).
    pub holder: String,
    /// The marked config commit, whose `workflow.yaml` governs.
    pub oid: String,
    pub short_oid: String,
    /// The lineage whose head is that commit, or `None` once the lineage has
    /// advanced past a mark that deliberately pins an older one.
    pub lineage: Option<String>,
}

/// The nearest workflow mark on `agent`'s descent, or `None` — the ordinary
/// state of every agent, not an error. **An unreadable mark reads the same
/// way**, litany's own rule (*a step never fails for want of a mark it does
/// not have*): a ref that is not there and a repo that cannot answer are one
/// `None`, and the workspace that is defective is refused by the governing
/// derivation this rides beside, never by the mark. Only a git that cannot be
/// spawned at all refuses here, in its own words.
pub fn read(workspace: &Path, agent: &str) -> Result<Option<WorkflowMark>, GitTreeError> {
    let repo = workspace.join(REPO_DIR);
    let mut id = agent.to_owned();
    loop {
        if let Some(oid) = rev_parse(&repo, &format!("{WORKFLOW_REF}{id}"))? {
            let lineage = config_branches(workspace)?
                .into_iter()
                .find(|b| b.tip_oid == oid)
                .map(|b| b.name);
            return Ok(Some(WorkflowMark {
                holder: id,
                short_oid: super::short(&oid),
                oid,
                lineage,
            }));
        }
        let Some(parent) = parent_id(&id) else {
            return Ok(None);
        };
        id = parent;
    }
}

#[cfg(test)]
mod tests;
