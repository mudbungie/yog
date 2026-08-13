//! The inbox-composer's derivations (DESIGN §11 inbox-composer, bl-929d): the
//! pending-queue row projection, the derived fold-line height, and the
//! snap-down ephemera. Pure over injected inputs — the excluded shell glue
//! (`shell::inbox_queue`) only paints what these decide.
//!
//! **The queue** is the target agent's pending inbox listing (§5.1 #11, the
//! same [`crate::inboxview::InboxEntry`] derivation the `✉n` badge and the
//! Inbox tab read — a third seat, never a second representation), oldest-first,
//! one line each, with the typed draft as the queue's LAST item. Every pending
//! row carries the jsonview fold arrow; only the input has no arrow. Fold
//! overrides are RAM keyed by the deposit's inbox path (§5.3) and die with the
//! pending row — the delivered transcript entry it becomes is a different fact
//! under a different key (`tx/…`).
//!
//! **The fold line is derived, never stored** (§11 rule 3): its position *is*
//! the queue's content height — pending rows plus the draft's wrapped height,
//! measured by the one tail measurement ([`crate::tail::scroll`]) — with a
//! floor of the bare input row (the content itself: zero items plus an empty
//! draft, the general path, no flag) and a cap of half the pane, past which the
//! queue scrolls tail-anchored. [`SnapState`] holds the measurement across
//! frames (a scroll body only learns its extent while painting, the same
//! one-frame settle as `tail`).
//!
//! **The recall is the same shape one door over** ([`recall`], bl-f908): ↑ at
//! the box's top row pages back through what the operator already said here,
//! and that history is not stored either — it is the pending listing above
//! plus the delivered transcript, read through the one role derivation.
//!
//! **The snap-down is render-layer viewport ephemera** (§13.1) whose trigger is
//! **structural, not gestural**: the pending count dropping, because delivery
//! commits landed. One path therefore covers every drain — the operator's
//! Enter, a live driver's step-boundary drain, a `lernie scan` flush, another
//! instance's send — and the snap can never show a crossing the substrate
//! didn't make. The animation eases the region from its pre-drain height down
//! to its content, nothing stored, nothing claimed durably.

use crate::actions::DraftKey;
use crate::inboxview::{InboxEntry, header_line};
use crate::transcript::Tone;
use std::collections::HashSet;

mod recall;
pub use recall::{Caret, Recall, Step, prompts};

/// How long the snap-down takes to reach its floor, in seconds — render
/// ephemera pacing, nothing durable.
pub const SNAP_SECS: f64 = 0.3;

/// One pending-queue line above the draft (§11 inbox-composer): the deposit's
/// `✉ from · at` header, its first line as the folded preview, and its whole
/// body for the unfolded state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueRow {
    /// The fold-override key: the deposit's workspace-relative inbox path
    /// (`inbox/<agent-id>/<file>`, §5.3) — the row's identity, so the override
    /// dies with the pending row and never collides with a `tx/…` key.
    pub key: String,
    /// `✉ from · at` — the shared §5.1 #11 wording ([`header_line`]).
    pub header: String,
    /// Who the deposit speaks for (§11 role stripe, bl-3acb) — the same
    /// [`crate::theme::message_role`] derivation the transcript's delivered
    /// rows wear, over the deposit's own asserted sender and epitaph, so a
    /// message looks the same pending as it does delivered.
    pub role: crate::theme::Role,
    /// The body's first line, shown while folded (the transcript's density
    /// idiom: one line per row, truncated at the pane's edge).
    pub preview: String,
    /// The body verbatim, shown below the header while unfolded.
    pub body: String,
    /// Folded is the auto-state; membership in the override set flips it.
    pub expanded: bool,
    /// How solid the row paints (§11, bl-915e): [`Tone::Weak`] while the
    /// deposit is only §7.2's pending echo — yog's own word for a send the
    /// driver has not written — and [`Tone::Plain`] the moment the derivation
    /// makes it a statement. The one input is
    /// [`InboxEntry::in_memory`](crate::inboxview::InboxEntry::in_memory), so
    /// nothing here decides; it reads.
    pub tone: Tone,
}

/// Project an agent's pending listing into queue rows, oldest-first (the
/// listing's own order). `folds` is the caller's RAM override set (§5.3):
/// membership flips a row open — the jsonview discipline, an empty set means
/// "everything as configured" (folded).
pub fn rows(agent_id: &str, pending: &[InboxEntry], folds: &HashSet<String>) -> Vec<QueueRow> {
    pending
        .iter()
        .map(|entry| {
            let key = format!("inbox/{agent_id}/{}", entry.name);
            QueueRow {
                header: header_line(&entry.deposit),
                role: crate::theme::message_role(
                    entry.deposit.sender.as_deref().unwrap_or_default(),
                    entry.deposit.epitaph.is_some(),
                ),
                preview: entry.deposit.body.lines().next().unwrap_or("").to_string(),
                body: entry.deposit.body.clone(),
                expanded: folds.contains(&key),
                tone: tone_of(entry),
                key,
            }
        })
        .collect()
}

/// A pending row's tone (§11, the faded-send ruling): faded while the
/// deposit is only in memory, solid once it is on disk. The whole predicate is
/// "does this deposit have a file", which is what
/// [`InboxEntry::in_memory`](crate::inboxview::InboxEntry::in_memory) answers —
/// the §7.2 echo has none, and a listed deposit always does.
fn tone_of(entry: &InboxEntry) -> Tone {
    if entry.in_memory() {
        Tone::Weak
    } else {
        Tone::Plain
    }
}

/// The queue region's cross-frame RAM (§5.3 carve-out, discarded on exit):
/// the pending-row fold overrides, the snap machinery, and the prompt recall
/// with the caret fact it gates on.
#[derive(Debug, Default)]
pub struct ComposerRam {
    /// Explicit per-row fold overrides, keyed by the deposit's inbox path.
    pub folds: HashSet<String>,
    pub snap: SnapState,
    /// How far back ↑ has paged, and the draft it displaced (bl-f908).
    pub recall: Recall,
    /// Where the caret sat when the box last painted — the recall's gate.
    pub caret: Caret,
}

/// The derived fold-line position and the snap-down over it. Holds the one
/// measurement (last frame's painted content height) and the pending count
/// whose structural drop triggers the ease — per composer *target*, so a
/// selection switch resets rather than reading as a drain.
#[derive(Debug, Default)]
pub struct SnapState {
    target: Option<DraftKey>,
    last_count: usize,
    settled: f32,
    /// A running snap: when it started, and the height it eases down from.
    anim: Option<(f64, f32)>,
}

impl SnapState {
    /// Fold this frame's structural facts in, before layout. A target switch
    /// resets everything (a different queue, not a drain); a pending count
    /// **drop** on the same target starts the snap from the settled height —
    /// delivery commits are the only thing that shrinks the count, so the
    /// trigger is the substrate's, whatever gesture caused the drain.
    pub fn observe(&mut self, target: &DraftKey, count: usize, now: f64) {
        if self.target.as_ref() != Some(target) {
            *self = Self {
                target: Some(target.clone()),
                last_count: count,
                ..Self::default()
            };
            return;
        }
        if count < self.last_count {
            self.anim = Some((now, self.settled));
        } else if self.anim.is_some_and(|(start, _)| now - start >= SNAP_SECS) {
            self.anim = None;
        }
        self.last_count = count;
    }

    /// The region height to lay out at: the settled content height (the fold
    /// line's derived position), lifted while a snap is easing down from its
    /// pre-drain height, and never past `cap` (half the pane — past it the
    /// queue scrolls instead of growing).
    pub fn desired(&self, cap: f32, now: f64) -> f32 {
        let base = self.settled;
        let value = match self.anim {
            Some((start, from)) => {
                let t = ((now - start) / SNAP_SECS).clamp(0.0, 1.0);
                let eased = 1.0 - (1.0 - t).powi(3);
                let height = f64::from(from) + (f64::from(base) - f64::from(from)) * eased;
                base.max(height as f32)
            }
            None => base,
        };
        value.min(cap)
    }

    /// Record what the body actually painted — the one measurement, taken from
    /// [`crate::tail::scroll`]'s own read, landing a frame after the content
    /// changes (the same settle the pad has).
    pub fn settle(&mut self, painted: f32) {
        self.settled = painted;
    }

    /// The settled content height — the fold line's steady-state position,
    /// before the cap and the snap lift. The render reads it to seat the
    /// queue: content past [`desired`](Self::desired) scrolls (the cap), and
    /// content short of it is padded down onto the floor (the snap's
    /// descending headroom, zero in the steady state).
    pub fn settled(&self) -> f32 {
        self.settled
    }

    /// Whether a snap is still easing — the caller keeps repainting while so.
    pub fn active(&self, now: f64) -> bool {
        self.anim.is_some_and(|(start, _)| now - start < SNAP_SECS)
    }
}

#[cfg(test)]
mod tests;
