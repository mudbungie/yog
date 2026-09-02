//! **The followed config commit** (§9.4 as amended by bl-e654) — what control
//! actually resolves from, and the answer every surface that says *which config
//! governs this conversation* renders.
//!
//! litany's operator ruling of 2026-09-01 (upstream bl-403b, its
//! `docs/DESIGN_CONFIG_FOLLOW.md`; landed here at the `=0.0.5` pin) inverted the
//! default this file's sibling was written under. Fork settles only which
//! **lineage** governs; control resolves from that lineage's **current tip at
//! every step boundary**, so a `litany config` edit reaches every running
//! conversation on the lineage at its next step with no per-conversation act.
//! The governing commit ([`super::governing_config`]'s ancestry walk) survives
//! as this derivation's **input**, never again as its answer.
//!
//! The rule, with no special case: take the config heads whose history contains
//! the governing commit and collect their **distinct tips**. Exactly one is
//! [`Governance::Follows`] — the single-lineage case, and equally the
//! freshly-forked case where several refs still stand on one commit. Two or
//! more is real divergence this derivation must not guess between: the fork
//! commit itself resolves, [`Governance::Held`], until `litany retarget`
//! settles the lineage. This is a faithful port of
//! `litany/src/workspace/current_config.rs`, the way the ancestry walk beside
//! it is of `litany/src/workspace.rs::governing_config` — litany keeps its
//! `workspace` module crate-private, so the port is the only way to share the
//! answer, and porting it is what keeps the two sides from disagreeing about a
//! conversation in front of the operator.
//!
//! **yog derives the held state; it does not read litany's notice for it.**
//! litany prints `litany: notice: N diverged config lineages reach [<agent>] …`
//! on the driver's stderr at every step, and that line is its *operator*
//! channel, not a wire. Scraping it back out of `driver.log` is the shape
//! bl-b95e already refused and deleted (`opslog::notice`): a phrase table over
//! sentences litany is free to reword is not a classifier, and content is
//! diagnosis, never a trigger. Held-ness is one fact with one home, and the
//! home is this git query — which both sides run, against the same refs.

use super::ConfigBranch;
use crate::git_tree::{GitTreeError, is_ancestor};
use std::path::Path;

/// Which lineage governs a conversation, and whether it is being followed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Governance {
    /// Exactly one config lineage reaches this conversation, and control
    /// resolves its tip at every step boundary. The name is that lineage's,
    /// bare (the `config/` prefix stripped, as [`ConfigBranch::name`] is).
    Follows(String),
    /// Two or more distinct config tips reach it. Nothing may be guessed
    /// between them, so control stays on the fork commit until `retarget`
    /// settles the lineage. The count is the one litany's own per-step notice
    /// names — derived here rather than read from there.
    Held { diverged_lineages: usize },
}

/// The config commit a conversation's control resolves from (§5.1 #17): the
/// followed lineage's tip, or the fork commit a divergence holds it on. A pure
/// view-model — `oid`/`short_oid`/`files` are all of the **resolved** commit,
/// so a reader never has to know which arm answered to know what governs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoverningConfig {
    pub oid: String,
    pub short_oid: String,
    pub governance: Governance,
    /// Every path in the resolved commit's tree (`souls/**`, `workflow.yaml`,
    /// `manifest.yaml`, `providers.yaml`, `version`, `descriptions/**`).
    pub files: Vec<String>,
}

impl GoverningConfig {
    /// The inspector Config-tab label (§9.3) — the one authoritative home for
    /// the wording, and the sentence that replaced *policy frozen at
    /// `<short-oid>`* when the freeze it named stopped being true.
    pub fn label(&self) -> String {
        match &self.governance {
            Governance::Follows(branch) => {
                format!("policy follows config/{branch}, now at {}", self.short_oid)
            }
            Governance::Held { diverged_lineages } => format!(
                "policy held at {} — {diverged_lineages} diverged config lineages",
                self.short_oid
            ),
        }
    }

    /// The lineage this conversation follows, by name. `None` while a
    /// divergence holds it — the one state with no single lineage to name,
    /// which is why a card that labels a conversation with its lineage falls
    /// silent there rather than picking a claimant.
    pub fn followed_lineage(&self) -> Option<String> {
        match &self.governance {
            Governance::Follows(branch) => Some(branch.clone()),
            Governance::Held { .. } => None,
        }
    }

    /// How many distinct config lineages reach this conversation while none
    /// can be followed — `0` whenever one is. The figure litany's own per-step
    /// notice names, derived here so the wire carries one number rather than a
    /// sentence somebody has to parse.
    pub fn diverged_lineages(&self) -> usize {
        match &self.governance {
            Governance::Follows(_) => 0,
            Governance::Held { diverged_lineages } => *diverged_lineages,
        }
    }
}

/// Resolve `fork_oid` — the governing commit — against the workspace's config
/// branches: the distinct tips of every lineage whose history contains it.
///
/// A lineage that does not contain the fork commit contributes nothing (it is
/// somebody else's), and tips are deduplicated by oid before they are counted,
/// which is what makes several refs standing on one commit a *followed* case
/// rather than a divergence. Zero cannot occur once the ancestry walk has
/// succeeded — the head that contributed the fork commit contains it — so the
/// count arm is written over `n` rather than special-cased, exactly as
/// upstream's is.
pub(super) fn resolve(
    repo: &Path,
    branches: &[ConfigBranch],
    fork_oid: &str,
) -> Result<(String, Governance), GitTreeError> {
    let mut reaching: Vec<&ConfigBranch> = Vec::new();
    for branch in branches {
        if is_ancestor(repo, fork_oid, &branch.tip_oid)?
            && !reaching.iter().any(|b| b.tip_oid == branch.tip_oid)
        {
            reaching.push(branch);
        }
    }
    match reaching.len() {
        1 => {
            let followed = reaching.remove(0);
            Ok((
                followed.tip_oid.clone(),
                Governance::Follows(followed.name.clone()),
            ))
        }
        n => Ok((
            fork_oid.to_owned(),
            Governance::Held {
                diverged_lineages: n,
            },
        )),
    }
}
