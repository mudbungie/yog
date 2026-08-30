//! Where a notch lands **in the chat** (bl-1802) — the pairing that makes the
//! boundary rule and the notch one thing rather than two drawings of one fact.
//!
//! **The bug this replaces.** Both the old crossings derivation (bl-929d) and
//! the old pin cut (bl-98da) paired *the i-th maximal run of delivered `.md`
//! entries* with *the i-th step*, on the stated ground that "drains and steps
//! are serialized under litany's executor lock". Serialization gives order, not
//! a bijection, and the two sets are nowhere near the same size: litany's step
//! is **one model call** (`StepOutcome::ToolsRan` re-enters `advance` for the
//! next one — litany ARCH §2.3 step 3), while a boundary drain lands delivery
//! commits only when the inbox holds something. A turn that calls five tools is
//! five steps behind one delivered run, so from the second tool-using turn
//! onward every rule carried the wrong commit and every pin cut the transcript
//! in the wrong place. **The sets were never the same set.**
//!
//! **What pairs exactly.** A model call that reaches `Finish` seals its output
//! and the executor commits it as `messages/NNN-<model-id>.json` (litany ARCH
//! §2.3 *The transcript writer*); a call that errors, is killed, or is still
//! open commits nothing. So the transcript's model entries are exactly the
//! steps whose framing is [`Framing::Complete`], **in step order, one for
//! one** — the pairing below, needing no new read on either side.
//!
//! Each step therefore owns the run of entries between its predecessor's model
//! output and its own: the tool results its predecessor's calls resolved and
//! whatever the boundary drain delivered (ARCH §2.11 orders the drain after the
//! tool entries). That run's first row is where the step's rule paints, and the
//! index of its own model output is the cut its pin reads to.
//!
//! **A step that sealed nothing has a place only when it is the last one.**
//! The running call's read state is everything committed so far, so it marks
//! the tail of the chat — which is where a child dispatched right now hangs its
//! card. A *superseded* call that sealed nothing (a crash, then a revival
//! deposit re-driving the branch) produced no output to sit above and left no
//! place: the notch stays on the spine, and the revival's own rule is the next
//! line down.

use crate::git_tree::Framing;
use crate::steps_view::StepSummary;
use crate::transcript::{EntryKind, Transcript};

/// One notch's place in the chat: the row its rule paints above, and how much
/// of the transcript that call had read. Both are positions in one sequence —
/// derived per snapshot from the entries and the step spine, stored nowhere.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Place {
    /// The row key ([`crate::transcript::key`]) of the first entry this call
    /// read that its predecessor had not — the rule's seat.
    pub row: String,
    /// Entry count of the read state: everything ahead of this call's own
    /// model output. [`super::transcript_as_of`] is that prefix.
    pub cut: usize,
}

/// Each step's place in `transcript`, parallel to `steps`. `None` is a notch
/// the chat has no seat for — see the module note; it is a value, not an arm.
pub(super) fn places(transcript: &Transcript, steps: &[StepSummary]) -> Vec<Option<Place>> {
    let end = transcript.entries.len();
    let mut from = 0usize;
    let mut out = Vec::with_capacity(steps.len());
    for (index, step) in steps.iter().enumerate() {
        let sealed = step.framing == Framing::Complete;
        let spoke = spoken(transcript, from);
        // The running call has sealed nothing yet, so its read state runs to
        // the tail; a superseded one that sealed nothing has no seat at all.
        let cut = if sealed {
            spoke
        } else {
            (index + 1 == steps.len()).then_some(end)
        };
        out.push(
            cut.zip(transcript.entries.get(from))
                .map(|(cut, entry)| Place {
                    row: crate::transcript::key(&entry.name, 0),
                    cut,
                }),
        );
        if sealed && let Some(next) = spoke {
            from = next + 1;
        }
    }
    out
}

/// Index of the first model-output entry at or after `from` — the entry the
/// ensuing call sealed, and therefore the end of what it read.
fn spoken(transcript: &Transcript, from: usize) -> Option<usize> {
    transcript
        .entries
        .iter()
        .enumerate()
        .skip(from)
        .find(|(_, entry)| matches!(entry.kind, EntryKind::Model { .. }))
        .map(|(index, _)| index)
}
