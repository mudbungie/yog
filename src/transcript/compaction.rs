//! **Compaction gaps** — the entries lernie's compactor deleted, derived from
//! the counter that survived them (DESIGN §5.1 #12, §11 Transcript).
//!
//! `messages/` is not append-only. lernie's compactor `git rm`s message files
//! out of the agent worktree and squashes the span they lived in (lernie ARCH
//! §2.6 the compaction landing), so a readdir of a compacted conversation is a
//! **rewritten** record, not a shorter one — the operator's question is simply
//! absent while the reply that answered it is still there, and the pane
//! rendered that as the whole record until bl-7bd2.
//!
//! **The gap is a query, never a field.** The `NNN` counter is monotonic and
//! lernie starts it at `001`, so a deleted span leaves a hole in the listing
//! that nothing has to record: a first entry above `001`, or a discontinuity
//! mid-sequence. [`splice`] reads that hole out of the names already read and
//! seats a virtual [`Compacted`](super::EntryKind::Compacted) entry in it —
//! the same move [`Transcript::with_live`](super::Transcript::with_live)
//! makes for the streaming tail, which is the other entry no file backs.
//!
//! Two things the counter cannot say, and neither is answered by guessing.
//! Entries deleted off the **end** leave no hole, because nothing above them
//! survives to bound one; and a conversation compacted **whole** leaves an
//! empty directory, where there is no counter left to read at all. Both read
//! as an uncompacted transcript, which is the honest floor of this derivation
//! rather than a case it handles.
//!
//! ## What replaced the span, and why it is not paired to one
//!
//! Compaction substitutes a meaning rather than destroying one: the compactor
//! writes `summary/<NNN>.md` at the agent worktree root — a sibling of
//! `messages/`, not a child — zero-padded to three and numbered across every
//! pass from one **branch-global** sequence, and the landing commits it into
//! the base beside the deletions.
//!
//! Those files are the whole of the on-disk link, and **there is none between
//! a summary and the span it replaced.** One pass may delete several disjoint
//! runs, which is one summary against several gaps; two passes' deletions may
//! abut into one hole, which is one gap against several summaries — and that
//! second shape is the ordinary one, since the shipped compaction template
//! retains no tail, so each pass deletes from where the last one stopped.
//! Nothing on disk distinguishes the two, so **no positional pairing is
//! made**: an i-th-gap-takes-the-i-th-summary rule is a claim the bytes do not
//! support, and it is wrong in exactly the case that occurs most.
//!
//! So the summaries are treated as what they demonstrably are — **the
//! conversation's compaction record, collectively replacing everything the
//! counter says is gone** — and the whole record is seated on the **earliest**
//! gap, which is the first point at which the record is known to diverge and
//! the one placement that asserts nothing about which summary replaced which
//! span. A later gap carries the marker alone. What each marker *does* assert
//! is only what the counter proves: these `NNN` values are not there.
//!
//! The marker never depends on a summary existing. A gap with no readable
//! summary still says the entries are gone, which is the honest answer on its
//! own.

use std::path::{Path, PathBuf};

use super::{Entry, EntryKind};

/// The compactor's summary directory, a sibling of `messages/` at the agent
/// worktree root.
const SUMMARY_DIR: &str = "summary";
/// Extension of a summary file; anything else in the directory is not one.
const SUMMARY_EXT: &str = "md";
/// Between two summaries of one record (module docs: they are read in pass
/// order, as one account of what was cut).
const SUMMARY_SEP: &str = "\n\n";

/// One hole in the counter: where it sits in the listing, and the inclusive
/// range of `NNN` values that are missing.
#[derive(Debug, Clone, Copy)]
struct Gap {
    /// Index in the read listing the marker is seated *before*.
    at: usize,
    first: usize,
    last: usize,
}

/// The listing with a marker entry spliced into every hole in its counter.
/// `agent` is the agent worktree — `messages/`'s own parent, because the
/// summaries are its sibling and not its contents. A contiguous listing is
/// returned untouched, which is the general path with no hole in it and not a
/// special case.
pub(super) fn splice(agent: &Path, entries: Vec<Entry>) -> Vec<Entry> {
    let gaps = gaps(&entries);
    if gaps.is_empty() {
        return entries;
    }
    // Read only once a hole is known to exist, and seat the whole record on
    // the earliest of them (module docs) — every other gap gets the marker
    // alone.
    let mut record = summaries(&agent.join(SUMMARY_DIR)).join(SUMMARY_SEP);
    let mut out = Vec::with_capacity(entries.len() + gaps.len());
    let mut pending = gaps.into_iter().peekable();
    for (index, entry) in entries.into_iter().enumerate() {
        // At most one gap per index: each is revealed by the entry it sits
        // before, and no two entries share an index.
        if let Some(gap) = pending.peek().copied().filter(|g| g.at == index) {
            out.push(marker(gap, std::mem::take(&mut record)));
            pending.next();
        }
        out.push(entry);
    }
    out
}

/// Every hole in the listing's `NNN` counter, in listing order. An entry whose
/// name is not `NNN-<origin>.<ext>` (the Raw bucket) carries no counter and so
/// neither opens nor closes a hole — it is skipped, not counted as one.
fn gaps(entries: &[Entry]) -> Vec<Gap> {
    let mut out = Vec::new();
    let mut expected = 1;
    for (at, entry) in entries.iter().enumerate() {
        let Some(seq) = seq_of(&entry.name) else {
            continue;
        };
        if seq > expected {
            out.push(Gap {
                at,
                first: expected,
                last: seq - 1,
            });
        }
        expected = seq.saturating_add(1);
    }
    out
}

/// An entry's `NNN`, off the one filename parse this module tree has
/// ([`super::parse_name`]) — a second reading of the shape would be a second
/// truth about it. `None` for a name that is not a message file's, and for a
/// counter no `usize` can hold. `pub(crate)` since bl-fde5: the §5.1 #12
/// message count is this counter's high-water mark, and the enumerate walk
/// ([`crate::git_tree`]) reads it through this one definition.
pub(crate) fn seq_of(name: &str) -> Option<usize> {
    super::parse_name(name).and_then(|(num, _, _)| num.parse().ok())
}

/// Every `summary/<NNN>.md` the compactor has written, in counter order (the
/// names are zero-padded, so the string sort *is* the numeric one). Bytes are
/// decoded lossily rather than dropped: mangled prose still says the record
/// was rewritten, which is §15 Y12's rule applied to these bytes too. An
/// absent directory — every conversation never compacted, and every one
/// compacted by a lernie that wrote no summary — yields none.
fn summaries(dir: &Path) -> Vec<String> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = read
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|e| e == SUMMARY_EXT))
        .collect();
    files.sort();
    files
        .iter()
        .filter_map(|p| std::fs::read(p).ok())
        .map(|b| String::from_utf8_lossy(&b).into_owned())
        .collect()
}

/// The virtual entry seated in a gap. Its `raw` is the record's own bytes, so
/// the §11 Raw toggle shows what actually replaced the span and not a
/// rendering of the marker; a marker carrying no record has nothing to show
/// and says so with empty bytes.
fn marker(gap: Gap, record: String) -> Entry {
    Entry {
        name: name_of(gap),
        raw: record.clone().into_bytes(),
        kind: EntryKind::Compacted {
            first: gap.first,
            last: gap.last,
            summary: record,
        },
    }
}

/// The marker's synthetic filename — the guillemet idiom the streaming tail
/// already spells (`«live»`), naming the span it stands in for. Gaps are
/// disjoint, so it is unique within one listing and the §11 fold key built
/// from it is stable across the stateless re-read.
fn name_of(gap: Gap) -> String {
    if gap.first == gap.last {
        format!("«{:03}»", gap.first)
    } else {
        format!("«{:03}–{:03}»", gap.first, gap.last)
    }
}
