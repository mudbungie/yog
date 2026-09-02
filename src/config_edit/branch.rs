//! Per-workspace config-branch surface (DESIGN §9.3, §5.1 #17–#18).
//!
//! This is the **read-only browse half**. A litany workspace's policy lives on
//! `refs/heads/config/<name>` branches in the bare `<workspace>/repo.git`
//! (ARCH §2.2 — there is no `main`). This module enumerates those branches,
//! lists and reads any file from a config commit's tree, and answers **which
//! config governs a conversation**.
//!
//! That last answer is two derivations on one seam (bl-e654). Here: the pure
//! ancestry walk to the agent's **fork commit**, the nearest `config/*`
//! ancestor of its branch, which never moves. In [`follow`]: that commit
//! resolved against the lineages reaching it, which is what control actually
//! reads at every step boundary since litany's follow-the-tip ruling. The seam
//! is where litany's own is — its `workspace.rs::governing_config` beside its
//! `workspace/current_config.rs` — and yog ports both rather than inventing a
//! third answer.
//!
//! Every git call routes through the env-scrubbed `git_tree::cmd` wrapper
//! (extended with the config plumbing this needs); no git is spawned here.
//!
//! The `$EDITOR`-driven **edit half** (Y21, §9.3) appends to this file — the
//! module map (§12) budgets `branch.rs` at 240 lines for browse + edit + the
//! edit plan; tests live under `branch/` and do not count against it.

use crate::git_tree::{
    GitTreeError, REPO_DIR, for_each_ref_config, is_ancestor, ls_tree, merge_base, show_file,
};
use std::path::Path;

pub mod edit;
pub mod follow;

pub use follow::{Governance, GoverningConfig};

/// One config branch: `refs/heads/config/<name>` (§5.1 #18). `name` is the
/// user-facing bare name (the `config/` prefix stripped) — exactly what a user
/// passes to `litany config <ws> <name>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigBranch {
    pub name: String,
    pub tip_oid: String,
    pub tip_short_oid: String,
    pub tip_timestamp_unix: i64,
}

/// One lineage as the §9.3 pane browses it: the branch and the files its tip
/// commit holds. The pane reads the two in one pass so "the listing and the
/// tree can never be of different commits"; this pairs them in the datum, so a
/// seat with no pane gets the same guarantee (bl-dff8).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lineage {
    pub branch: ConfigBranch,
    /// Every path in the tip's tree — `git ls-tree -r` at [`ConfigBranch::tip_oid`].
    pub files: Vec<String>,
}

/// First 8 hex of an oid (the repo-wide short-oid convention). `unwrap_or`
/// evaluates its argument eagerly, so a shorter oid is a value, not a branch.
fn short(oid: &str) -> String {
    oid.get(..8).unwrap_or(oid).to_string()
}

/// Enumerate the workspace's config branches (§5.1 #18), in git
/// `for-each-ref` order (ref name ascending) so two instances render
/// identically without sharing ordering state (I9).
pub fn config_branches(workspace: &Path) -> Result<Vec<ConfigBranch>, GitTreeError> {
    parse_branches(&for_each_ref_config(&workspace.join(REPO_DIR))?)
}

fn parse_branches(stdout: &[u8]) -> Result<Vec<ConfigBranch>, GitTreeError> {
    let text = String::from_utf8_lossy(stdout);
    let mut out = Vec::new();
    for line in text.lines() {
        let mut it = line.splitn(3, ' ');
        match (it.next(), it.next(), it.next()) {
            (Some(refname), Some(oid), Some(ts)) => out.push(ConfigBranch {
                // `for-each-ref refs/heads/config/` guarantees the prefix; the
                // fallback is unreachable, so `unwrap_or` (eager) adds no arm.
                name: refname
                    .strip_prefix("config/")
                    .unwrap_or(refname)
                    .to_string(),
                tip_short_oid: short(oid),
                tip_oid: oid.to_string(),
                tip_timestamp_unix: ts
                    .parse()
                    .map_err(|_| GitTreeError::LogFormat(line.to_string()))?,
            }),
            _ => return Err(GitTreeError::LogFormat(line.to_string())),
        }
    }
    Ok(out)
}

/// The full file listing of a config commit's tree (`git ls-tree -r`, §5.1
/// #18). `refspec` is any committish — a `config/<name>` ref or a commit oid.
pub fn config_tree(workspace: &Path, refspec: &str) -> Result<Vec<String>, GitTreeError> {
    tree_paths(&workspace.join(REPO_DIR), refspec)
}

fn tree_paths(repo: &Path, refspec: &str) -> Result<Vec<String>, GitTreeError> {
    let out = ls_tree(repo, refspec)?;
    Ok(String::from_utf8_lossy(&out)
        .lines()
        .map(str::to_string)
        .collect())
}

/// The whole browse in one answer (§9.3, bl-dff8): every config branch with the
/// files its tip holds — [`config_branches`] and [`config_tree`] composed, which
/// is what the pane does across two gestures and what a headless seat needs in
/// one. Each tree is read **at the tip oid**, not at the ref, so a lineage
/// advanced mid-read still answers files of the commit this listing names.
pub fn lineages(workspace: &Path) -> Result<Vec<Lineage>, GitTreeError> {
    let repo = workspace.join(REPO_DIR);
    let branches = parse_branches(&for_each_ref_config(&repo)?)?;
    let mut out = Vec::new();
    for branch in branches {
        let files = tree_paths(&repo, &branch.tip_oid)?;
        out.push(Lineage { branch, files });
    }
    Ok(out)
}

/// Raw bytes of one file in a config commit's tree (`git show <ref>:<path>`,
/// §9.3). Rendered as raw text by the inspector — YAML included, no YAML dep.
pub fn config_file(workspace: &Path, refspec: &str, path: &str) -> Result<Vec<u8>, GitTreeError> {
    show_file(&workspace.join(REPO_DIR), refspec, path)
}

/// Answer **which config governs** the agent at branch tip `agent_tip`, an oid
/// (§5.1 #17, ARCH §2.2). Two derivations in sequence, both faithful ports:
/// the fork commit — the nearest ancestor reachable from any `config/*` ref,
/// `litany/src/workspace.rs::{governing_config, nearest}` — and then
/// [`follow::resolve`] over it, which is what control reads.
///
/// The name is the old one because the *question* is the old one; only its
/// answer moved, from the fork commit to the tip the fork commit's lineage now
/// stands at (bl-e654).
///
/// For each config branch, `merge-base(agent_tip, config_tip)` is the shared
/// ancestor on that lineage — or nothing, when an unrelated orphan config
/// shares no history (it contributes no candidate and is skipped). The
/// candidates are folded keeping, of any two, the **descendant** — the one
/// nearer the agent tip ([`nearest`]).
///
/// The fold's tie-break is **order-independent in its result**: `nearest`
/// returns the descendant whichever order its two arguments arrive in, and two
/// equal candidates short-circuit. So `for-each-ref` order changes only which
/// internal arm fires, never the governing commit. Declined **loudly** (never
/// guessed) when no config lineage reaches the branch (`Governing`), and when
/// two candidates are incomparable ancestors — both are defective workspaces.
pub fn governing_config(
    workspace: &Path,
    agent_tip: &str,
) -> Result<GoverningConfig, GitTreeError> {
    let repo = workspace.join(REPO_DIR);
    let branches = parse_branches(&for_each_ref_config(&repo)?)?;
    let mut best: Option<String> = None;
    for b in &branches {
        // An unrelated config lineage yields no merge-base — skip it.
        let Some(base) = merge_base(&repo, agent_tip, &b.tip_oid)? else {
            continue;
        };
        best = Some(match best {
            None => base,
            Some(prev) if prev == base => prev,
            Some(prev) => nearest(&repo, prev, base)?,
        });
    }
    let fork = best.ok_or_else(|| {
        GitTreeError::Governing(format!(
            "no config/* ancestor for {agent_tip} — every agent forks off a config commit"
        ))
    })?;
    let (oid, governance) = follow::resolve(&repo, &branches, &fork)?;
    let files = tree_paths(&repo, &oid)?;
    Ok(GoverningConfig {
        short_oid: short(&oid),
        oid,
        governance,
        files,
    })
}

/// Of two ancestor candidates of one branch tip, keep the descendant — the
/// one nearer the tip (mirrors litany `workspace.rs::nearest`). Incomparable
/// candidates are declined loudly: a defective workspace (§2.2), not a guess.
fn nearest(repo: &Path, a: String, b: String) -> Result<String, GitTreeError> {
    if is_ancestor(repo, &a, &b)? {
        return Ok(b);
    }
    if is_ancestor(repo, &b, &a)? {
        return Ok(a);
    }
    Err(GitTreeError::Governing(format!(
        "ambiguous governing config: {a} and {b} are incomparable ancestors — declined"
    )))
}

#[cfg(test)]
mod tests;
