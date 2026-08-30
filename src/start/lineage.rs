//! **Tag → config lineage** (VISION §4.2 / §4.6, DESIGN §8.7): which
//! `config/<name>` a start forks its drone off, derived from the ball's own
//! tags and from nothing else.
//!
//! VISION §4.2 promises *"Skills are seeded at spawn, keyed on ball tags … yog
//! selects skills and model from the ball's tags at fire"*, and §4.6 fixes the
//! shape it must take: *"Policy table, yog config, severable. No crate below
//! yog ever names a model."* A litany **config lineage is already that pair**
//! — its `providers.yaml` names the worker role's provider and model, its
//! `descriptions/skills/**` is the skill set the tools composer offers, and
//! `litany prompt --config <name>` forks the new agent off its head (litany
//! ARCH §2.3) — so the policy needs no table of its own. **The lineage's
//! existence IS the policy**: a ball tagged `deep` is born on `config/deep`
//! where the workspace has one, and on the ordinary default where it has not.
//!
//! That is the whole mechanism, and it answers the four questions the design
//! had to answer without adding a surface to yog:
//!
//! - **Conflict — the ball's own tag order.** The first tag naming a lineage
//!   wins. Tags are an ordered, operator-authored list, so the precedence is
//!   already written where the tags are; a priority column or a longest-match
//!   rule would be a *second* home for one fact, and the two would drift.
//! - **Default — no match, no flag.** A ball whose tags name no lineage, an
//!   untagged ball, and the bare/path rungs are one case, not three: an empty
//!   or unmatched tag list selects nothing, [`select`] answers `None`, and the
//!   fire omits `--config` exactly as it always did. The default is not a
//!   value yog holds; it is litany's own `config/default`.
//! - **Severability — `git branch -d config/<tag>`.** Creating the policy is
//!   creating the ref (`litany config <ws> <tag> --from default`, or §9.3's
//!   editor drive); removing it is deleting the ref. Neither edits a line of
//!   yog, which is the severability test stated the right way round.
//! - **Authority — the ball, once.** Nothing is mirrored into yog state. The
//!   tags ride the §3.4 payload from the board row that carried them and are
//!   read exactly here.
//!
//! **The name is taken from the branch, never from the tag.** `select` returns
//! the [`ConfigBranch::name`] it matched, so what reaches an argv is always a
//! ref git already enumerated — a tag is only ever compared, never spelled
//! into a command.
//!
//! **Resolved once, at [`prepare`](super::prepare), not twice.** The answer
//! governs two acts: §8.6's policy convergence, which runs during the prepare
//! and must land the control block on the lineage the drone will actually fork
//! off, and the fire's own `--config`. Deriving it separately in each would be
//! two homes for one fact — and a lineage created between the two reads would
//! converge one branch and fork another.

use crate::config_edit::branch::config_branches;
use crate::start::{BallSpec, Payload};
use std::path::Path;

#[cfg(test)]
mod tests;

/// The `config/<name>` this payload's ball selects in `workspace`, or `None`
/// for the ordinary default (see the module doc for all four cases).
///
/// A workspace with no repository yet — the bootstrap start, whose
/// [`Step::EnsureWorkspace`](super::Step::EnsureWorkspace) has not run — has no
/// lineage to name, and `config_branches` failing is that answer rather than an
/// error: a world with one `config/default` in it is where every fresh
/// workspace begins, and that is the `None` case.
pub(crate) fn select(workspace: &Path, payload: &Payload) -> Option<String> {
    let tags = tags(payload);
    if tags.is_empty() {
        return None;
    }
    let branches = config_branches(workspace).ok()?;
    tags.iter()
        .find_map(|tag| branches.iter().find(|b| &b.name == tag))
        .map(|b| b.name.clone())
}

/// The ball's tags, in the order `bl` records them. Empty for every rung that
/// names no existing ball — the bare and path rungs, and a ball `bl create` has
/// not minted yet (its tags are the operator's to add afterwards, and the
/// re-plan that follows the mint reads the ball back with whatever it has).
fn tags(payload: &Payload) -> &[String] {
    match payload {
        Payload::Ball {
            ball: BallSpec::Existing { tags, .. },
            ..
        } => tags,
        _ => &[],
    }
}
