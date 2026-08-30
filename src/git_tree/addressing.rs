//! **The live conversation-name enumeration** the boundary addresses over
//! (REMOTE §8 as amended by bl-49bc, DESIGN §8.5): every `agents/*` ref that
//! wears a litany-stored name, asked of disk at the moment a gesture names one.
//!
//! Two facts per agent and nothing else — the id and the name — which is why it
//! is a module of its own rather than a row on the §7.1 derivation. The whole
//! tree walk (steps, inboxes, worktrees, liveness probes) is not affordable per
//! gesture; this is: one `for-each-ref` plus one `git show` per ref, and only
//! when a gesture spells a **name** at all (an id-shaped needle is an id by
//! construction and reads nothing — [`crate::boundary::address`]).
//!
//! **Asked, never remembered**, for bl-6c9e's reason one noun down. A
//! conversation's branch is written by the *detached* driver a `/prompt` fired,
//! so the set the last derivation cached is always the set as of the last
//! sweep — and the name a `Started` receipt just handed back would refuse until
//! that sweep landed. Existence is a query here exactly as workspace existence
//! is at the intake.

use super::REPO_DIR;
use super::cmd::{for_each_ref_agents, ref_name};
use std::path::Path;

/// The `refs/heads/` prefix every agent branch carries (ARCH §2.3) — the branch
/// name minus this prefix is the agent id, the same strip
/// [`enumerate`](super::enumerate) makes.
const AGENT_REF_PREFIX: &str = "agents/";

/// Every living agent in `workspace`, as `(id, stored name)` — the `agents/*`
/// enumeration with each ref's own [`ref_name`] blob read out of it. The
/// **stored** name and only it: the §3.3 ladder's legacy `You are <x>.`
/// goal-stamp rung is a title with no ref behind it (bl-8068), so it is not a
/// reading this set may offer.
///
/// The same shape the published derivation answers in, so one rule reads both
/// (`crate::boundary::address`) — `None` for an unnamed agent, absence and an
/// empty blob being one fact.
///
/// A workspace whose repository cannot be read answers with **none**: nothing
/// there is addressable, and the caller's refusal names the token exactly as an
/// unenumerated workspace's does. It never surfaces a raw git failure, because
/// the question asked was "what does this name mean", and the answer is
/// "nothing here".
pub(crate) fn living_agents(workspace: &Path) -> Vec<(String, Option<String>)> {
    let git_dir = workspace.join(REPO_DIR);
    let Ok(out) = for_each_ref_agents(&git_dir) else {
        return Vec::new();
    };
    String::from_utf8_lossy(&out)
        .lines()
        // `%(refname:short) %(objectname) %(committerdate:unix)` — only the
        // first field is asked for here; a line carrying no space at all is its
        // whole branch name.
        .map(|line| line.split_once(' ').map_or(line, |(branch, _)| branch))
        .map(|branch| {
            let id = branch.strip_prefix(AGENT_REF_PREFIX).unwrap_or(branch);
            (id.to_owned(), ref_name(&git_dir, branch).ok().flatten())
        })
        .collect()
}

#[cfg(test)]
mod tests;
