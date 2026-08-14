//! The completed derivation the frame renders (DESIGN §7.2).
//!
//! A [`Snapshot`] is everything a frame may read that came off disk: the
//! classified workspace set, the per-workspace [`GitTree`]s, the live ball
//! projection and its §3.5 join, and the `ops.jsonl` tail. It is built **only**
//! by the worker ([`super::derive::Deriver`]) and handed to the frame behind an
//! `Arc` — immutable once published, so a frame never reads a half-derived one
//! and never blocks on the derivation that is building the next.
//!
//! Two fields exist because the derivation moved off the frame thread:
//!
//! - [`derived_at`](Snapshot::derived_at) is when this snapshot **completed**.
//!   Age against the injected clock is the honest staleness number (§7.2) —
//!   there is no longer a structural "the frame IS the derivation" guarantee to
//!   assert instead.
//! - [`growth`](Snapshot::growth) names what got bigger since the previous
//!   snapshot. A dispatch storm is a fact about the *workspace*, and yog had no
//!   way to say it: 227 branches appearing under one conversation rendered as
//!   yog being slow (bl-ee0a).
//!
//! `ui_bytes` is the one thing here that is not derived-and-rendered: the
//! worker's read of an externally-changed `ui.json` (§4.1, I5), carried to the
//! frame because the *document* is frame-owned (write-through at the gesture)
//! while the *read* is disk I/O and belongs to the worker like every other.

use crate::binding::Workspace;
use crate::budgets::StepBill;
use crate::git_tree::GitTree;
use crate::nav::convs;
use crate::opslog::OpRow;
use crate::projects::balls::Ball;
use crate::projects::join::JoinRow;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// One conversation's descent growing between two derivations (§7.2) — the
/// storm signal. `added` is branches, not agents-in-total: a conversation that
/// shrank or held still produces nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Growth {
    /// The workspace the conversation lives in.
    pub workspace: PathBuf,
    /// The conversation's §3.3 display name — what the operator calls it.
    pub conversation: String,
    /// How many branches its descent gained since the previous snapshot.
    pub added: usize,
}

/// A completed derivation (§7.2). Cheap to hand out (`Arc`), never mutated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    /// Every enumerated workspace, classified (§3.1).
    pub workspaces: Vec<Workspace>,
    /// Every enumerated project's decoded invocation path (§5.1 #1), internal
    /// clones included — the set the §11 roster labels and the boundary's
    /// project **names** are both derived over ([`names`], REMOTE §8). Ridden
    /// out on the snapshot for the same reason the workspace set is: a name is
    /// a query over the live enumeration, and neither frontend may readdir to
    /// resolve one.
    pub projects: Vec<PathBuf>,
    /// Per-workspace derived trees; a workspace absent here failed to derive.
    pub trees: HashMap<PathBuf, GitTree>,
    /// Per-workspace `steps/` walk (§3.5, bl-9dd4): every step's Usage fold and
    /// the model that billed it, folded **once per pass on the worker**. Every
    /// spend figure — a conversation's, a ball's, the V4 board's column, an epic
    /// rollup across workspaces — is a filter over this, so a board of N balls
    /// costs one walk rather than N on the frame thread (§7.2). Derived, never
    /// stored: it is re-walked from disk like every other §5.1 fact.
    pub bills: HashMap<PathBuf, Vec<StepBill>>,
    /// The context windows `models.yaml` declares, wire-model id → tokens
    /// (§5.1 #35): the denominator of every context-fullness figure, read by
    /// the worker on the same 15 s full sweep the balls fetch rides. World-wide
    /// rather than per-workspace because the file is — §9.2's global lernie
    /// config, one home for the declaration. Empty is the ordinary quiet case:
    /// nothing declared, so nothing is rendered.
    pub windows: std::collections::BTreeMap<String, u64>,
    /// Cached live balls per **visible** project (§5.1 #2).
    pub balls_by_project: HashMap<PathBuf, Vec<Ball>>,
    /// Cached **closed** balls per project (§5.1 #4) — fetched on demand, never
    /// on the cadence, so this is sparse by design. Published beside the live
    /// map because the §3.5 Delivered rows and the §8.5 search corpus are the
    /// same fact asked twice: whatever closed set this world has fetched.
    pub closed_by_project: HashMap<PathBuf, Vec<Ball>>,
    /// The derived §3.5 join (roster badges + verb enablement).
    pub join_rows: Vec<JoinRow>,
    /// The `ops.jsonl` tail (§4.2), the ops-pane render source.
    pub ops: Vec<OpRow>,
    /// Conversations whose descent grew since the previous snapshot (§7.2).
    pub growth: Vec<Growth>,
    /// Externally-changed `ui.json` bytes for the frame to adopt (§4.1).
    pub ui_bytes: Option<Vec<u8>>,
    /// When this derivation completed — the staleness datum (§7.2).
    pub derived_at: Instant,
    /// The clock's live periods (bl-3381): the worker's read of `cadence.yaml`
    /// (or the defaults), ridden out so every frame-side derived period — the
    /// wound grace, the staleness bound, the I4 poll floor — follows the
    /// operator's tuning without the frame reading disk (§7.2).
    pub cadence: super::Cadence,
    /// The armed §4.3 fleet loops, keyed by workspace (bl-66fb): the worker's
    /// read of the **same** `cadence.yaml`, ridden out for the same reason the
    /// periods above are — the V4 board renders the cap and the count, and a
    /// frame must not read disk to say them. Empty is the default and the
    /// ordinary world: no entry, no loop, no fact.
    pub fleet: BTreeMap<String, crate::fleet::Policy>,
}

/// The boundary's addressing, read off this snapshot in both directions
/// (REMOTE §8, bl-f5f6) — its own file at §12's budget.
mod names;

impl Snapshot {
    /// The empty snapshot a model starts from, before the worker's first pass.
    /// Not a special case: it is the general shape with no inputs, so every
    /// read surface answers its own empty state without a bootstrap branch.
    pub(crate) fn empty(now: Instant) -> Self {
        Self {
            workspaces: Vec::new(),
            projects: Vec::new(),
            trees: HashMap::new(),
            bills: HashMap::new(),
            windows: BTreeMap::new(),
            balls_by_project: HashMap::new(),
            closed_by_project: HashMap::new(),
            join_rows: Vec::new(),
            ops: Vec::new(),
            growth: Vec::new(),
            ui_bytes: None,
            derived_at: now,
            cadence: super::Cadence::default(),
            fleet: BTreeMap::new(),
        }
    }
}

/// The §11 ops-surface growth line, or `None` when nothing grew (the normal
/// case, which must render nothing at all). Names the biggest grower — a storm
/// has one — and counts the rest, so one glance says which conversation is
/// spawning and, by implication, that the spawning is lernie's, not yog's.
pub(crate) fn growth_label(growth: &[Growth]) -> Option<String> {
    let worst = growth.first()?;
    let rest = growth.len() - 1;
    let more = match rest {
        0 => String::new(),
        n => format!(" (and {n} more)"),
    };
    Some(format!(
        "{} +{} branches{more}",
        worst.conversation, worst.added
    ))
}

/// Per-conversation branch counts in one workspace's tree: `(root agent id,
/// branches in its descent)`. The §2.3 descent is the unit because that is what
/// a dispatch storm inflates — one root, N children.
pub(crate) fn branch_counts(tree: &GitTree) -> HashMap<String, usize> {
    let mut out: HashMap<String, usize> = HashMap::new();
    for agent in &tree.agents {
        if let Some(root) = convs::root_of(&tree.agents, &agent.agent_id) {
            *out.entry(root).or_default() += 1;
        }
    }
    out
}

/// What grew between `old` and `new` for one workspace (§7.2). Only roots
/// present in **both** count: a conversation that did not exist before did not
/// *grow*, it appeared, and the roster already says so.
pub(crate) fn growth_between(ws: &Path, old: Option<&GitTree>, new: &GitTree) -> Vec<Growth> {
    let Some(old) = old else {
        return Vec::new();
    };
    let before = branch_counts(old);
    let mut out: Vec<Growth> = Vec::new();
    for (root, after) in branch_counts(new) {
        let Some(prior) = before.get(&root) else {
            continue;
        };
        if let Some(added) = after.checked_sub(*prior).filter(|n| *n > 0) {
            out.push(Growth {
                workspace: ws.to_path_buf(),
                conversation: convs::display_name_of(&new.agents, &root),
                added,
            });
        }
    }
    out.sort_by(|a, b| {
        b.added
            .cmp(&a.added)
            .then(a.conversation.cmp(&b.conversation))
    });
    out
}

#[cfg(test)]
mod tests;
