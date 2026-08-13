//! Notch-pinning: one selection, four tabs (VISION V1.2).
//!
//! *"Selecting a notch pins the agent-history inspector to that commit:
//! transcript as of, **agent-context** files as of, config-frozen-at, budget
//! folded to that point."* One mechanism serves all four, which is the shape
//! STORIES §S7 point 3 named when it declined a per-tab checkbox: the pin is a
//! commit, and each tab reads that commit the way it already reads its own
//! source. Transcript and budget fold here (both are prefixes of what the tab
//! already holds); files go through [`super::files_at`]; config-frozen-at is
//! the existing governing-config derivation asked at the pinned commit instead
//! of the tip, with no new code at all.
//!
//! Nothing selected leaves every tab on today's read — the burden check.
//!
//! **Where the pin is released** (bl-1802). The gesture that raises a pin is
//! now a rule in the chat, which lives in the Transcript tab; the pin still
//! reaches all four pinnable tabs, so the release has to be reachable from each
//! of them. It is the **pin banner itself** — the line that already paints
//! above every pinnable tab naming the commit. That adds no verb and no second
//! control: it is one existing gesture given the seat it always needed, and it
//! makes the banner's own sentence true again wherever it paints (it used to
//! say *"Pick the same mark again to come back"*, which stopped being true the
//! moment the mark lived in another tab). Clicking the pinned rule still
//! releases too — one gesture, both directions, unchanged where you made it.

use super::Rail;
use crate::transcript::Transcript;

/// The inspector, folded to one notch. Held only while the operator keeps a
/// notch selected; the selection itself is viewport ephemera (§5.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pin {
    /// The read-state commit every pinned tab reads from.
    pub commit: String,
    /// That commit at short-oid width — the label the pinned banner wears.
    pub short: String,
    /// Budget as of this notch: every notch's spend up to and including it.
    pub tokens: u64,
    /// How many transcript entries this call had read — the notch's
    /// [`Place::cut`](super::Place::cut), carried so the fold below is a slice
    /// and not a second walk of the entries.
    pub cut: usize,
}

/// Resolve a notch selection. `None` — nothing selected, an index the spine no
/// longer has, a notch whose step recorded no read-state commit, or one the
/// chat gave no seat — leaves the inspector on today's transcript. A notch
/// with no commit names no tree, so it cannot pin: absence declines rather
/// than guessing a neighbour's.
pub fn pin(rail: &Rail, selected: Option<usize>) -> Option<Pin> {
    let index = selected?;
    let notch = rail.notches.get(index)?;
    let commit = notch.commit.clone()?;
    let cut = notch.place.as_ref()?.cut;
    let tokens = rail
        .notches
        .get(..=index)
        .unwrap_or(&rail.notches)
        .iter()
        .map(|n| n.tokens)
        .sum();
    Some(Pin {
        commit,
        short: notch.short(),
        tokens,
        cut,
    })
}

/// The transcript as of a pinned notch: every entry that had been committed
/// when that model call was assembled — everything ahead of its own model
/// output, and none of what the call went on to produce.
///
/// This is a **prefix**, not a second read, and that is exact rather than
/// convenient: lernie's `messages/` entries are append-only files under a
/// monotonic `NNN` counter (§5.1 #12), so the entries in the pinned tree and
/// the leading entries of today's are the same bytes. The Raw toggle therefore
/// keeps showing verbatim bytes of the pinned tree, with no `git show` per
/// message.
///
/// The cut arrives already derived on the notch ([`super::Place`]) — the same
/// walk that decided where the notch's rule paints, so the line in the chat and
/// the fold behind it can never disagree. A cut the transcript cannot honour
/// reads whole, which is the general path with an out-of-range input.
pub fn transcript_as_of(transcript: &Transcript, cut: usize) -> Transcript {
    Transcript {
        entries: transcript
            .entries
            .get(..cut)
            .unwrap_or(&transcript.entries)
            .to_vec(),
    }
}
