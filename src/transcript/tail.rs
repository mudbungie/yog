//! The transcript's **virtual trailing entries** (§5.1 #12, §8.5): the two
//! rows no `messages/` file backs, folded onto a committed transcript by the
//! caller that already holds what they are made of.
//!
//! There are three virtual entries in all and they are one move made three
//! times. [`EntryKind::Compacted`](super::EntryKind::Compacted) stands in a
//! hole the compactor left and is seated by [`super::compaction`] during the
//! read; the two here are *trailing* and are folded after it, because their
//! inputs are not the directory:
//!
//! - **[`Streaming`](super::EntryKind::Streaming)** — the open
//!   `response.json`, folded from the rendered snapshot's own
//!   [`Stream`](crate::git_tree::Stream). Its clock is the model's write
//!   cadence, not the derivation's (§7.2), which is why it is folded rather
//!   than built.
//! - **[`Wounded`](super::EntryKind::Wounded)** — the **settled-failure
//!   notice** (bl-015b): the conversation stopped, and the last thing that
//!   happened to it was a §7.3 [`Wound`]. Its input is the steps tree, which
//!   the transcript read never touches.
//!
//! The two are exclusive by derivation and not by a rule here: a wound is
//! claimed only of a step nobody is driving (`steps_view::wound::driven`), and
//! a stream exists only while somebody is. Each fold **replaces** its own kind
//! of tail rather than appending beside one, so folding twice is folding once
//! — the reconciliation the follow lane needs, stated once per row.
//!
//! **Why the notice exists at all.** A conversation refused at its first model
//! call painted its user message and nothing else: one committed entry, no
//! tail, and a pane that was honest and useless. The fact was derived, stored
//! and answered correctly — it reached the roster's hue, the §6 attention word
//! and the per-step `auth_row` — and reached no surface the operator reading
//! the conversation was looking at.

use crate::steps_view::Wound;

use super::{Entry, EntryKind, Transcript};

/// Synthetic filename for the virtual live-streaming entry.
pub(super) const STREAMING_NAME: &str = "«live»";
/// Synthetic filename for the virtual settled-failure notice. Bracketed like
/// the other two, so a seat listing names never mistakes one for a file.
pub(super) const WOUND_NAME: &str = "«wound»";

impl Transcript {
    /// This transcript with the live tail as a virtual trailing entry (§7.2).
    /// `stream` is a fold somebody else already made — never a disk read from
    /// here, which is what lets the two halves keep different clocks.
    ///
    /// A stream that has said nothing yet adds no entry: an empty live row is
    /// not the same claim as a model that has begun, and "waiting for the API"
    /// is the §11 live mark's to say, not a blank line's.
    ///
    /// **It replaces a live entry rather than appending beside one** (bl-73e7),
    /// so `a.with_live(x).with_live(y) == a.with_live(y)`. That is not a
    /// convenience: the tail now reaches a seat by two routes at two cadences —
    /// the pull `Query::Transcript` folds one on at ask cadence, and the follow
    /// lane delivers a newer one at write cadence — and *the newest fold wins*
    /// is the only reconciliation either needs. Appending would paint the
    /// answer twice, and a caller stripping the older one by hand would be a
    /// second party deciding what a live row is.
    #[must_use]
    pub fn with_live(&self, stream: &crate::git_tree::Stream) -> Transcript {
        let (thinking, text) = (
            stream.thinking.clone().unwrap_or_default(),
            stream.text.clone().unwrap_or_default(),
        );
        let mut entries = stripped(self, |kind| matches!(kind, EntryKind::Streaming { .. }));
        if !thinking.is_empty() || !text.is_empty() {
            entries.push(Entry {
                name: STREAMING_NAME.to_string(),
                raw: format!("{thinking}{text}").into_bytes(),
                kind: EntryKind::Streaming { thinking, text },
            });
        }
        Transcript { entries }
    }

    /// This transcript with the **settled-failure notice** as a virtual
    /// trailing entry (bl-015b). `wound` is a derivation somebody else already
    /// made — `steps_view::latest_wound` off a built steps view — for
    /// [`with_live`](Self::with_live)'s reason: the transcript read owns
    /// `messages/` and nothing else, and a second reader of the steps tree
    /// here would be a second answer to one question.
    ///
    /// [`Wound::None`] adds no entry. A healthy conversation says nothing
    /// about its health, exactly as a stream that has said nothing adds no
    /// row: a notice that appears on every conversation is a notice nobody
    /// reads.
    ///
    /// It replaces its own kind of tail for the same reason `with_live` does,
    /// so a caller may fold a fresher derivation on without stripping the
    /// older one by hand.
    #[must_use]
    pub fn with_wound(&self, wound: &Wound) -> Transcript {
        let mut entries = stripped(self, |kind| matches!(kind, EntryKind::Wounded { .. }));
        if wound.wounded() {
            entries.push(Entry {
                name: WOUND_NAME.to_string(),
                raw: wound.banner().into_bytes(),
                kind: EntryKind::Wounded {
                    wound: wound.clone(),
                },
            });
        }
        Transcript { entries }
    }
}

/// The entries with a trailing row of the caller's own kind removed — the
/// *replace, never append* half both folds share, in one place so neither can
/// forget it. Only the tail is examined: a virtual row is never seated
/// anywhere else.
fn stripped(transcript: &Transcript, mine: fn(&EntryKind) -> bool) -> Vec<Entry> {
    let mut entries = transcript.entries.clone();
    if entries.last().is_some_and(|e| mine(&e.kind)) {
        entries.pop();
    }
    entries
}
