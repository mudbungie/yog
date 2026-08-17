+++
title = "the transcript is a readdir of a directory compaction DELETES from, and yog reads no summary/**, so a squashed span is silent — derive the gap, paint what replaced it"
created = 1786936867
updated = 1786937185
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
**Premise restored and now PROVEN on disk. This ball was briefly parked on a
mistaken correction; the compaction mechanism is confirmed, not suspected.**

The Transcript pane is a bare `read_dir` of `<workspace>/agents/<id>/messages/`
(`src/transcript/mod.rs::build`, sorted by filename, no git behind it), reached
through the one chokepoint both seats use
(`src/boundary/answer/inspector.rs::transcript`). lernie's compactor **deletes
files from that directory** — `mark_for_deletion` is a literal `git rm -r -q --`
and the landing squashes the deletion into a new compaction base — so the pane
silently renders a rewritten record as the whole record.

## The evidence

A traced conversation's `messages/` begins at `002-*` with no `001-*`. Its
branch log shows a `compaction base` commit sitting on the dispatch step, and
`git show --stat` on that base re-adds `messages/002…020` **plus
`summary/001.md`, and nothing named `001-user.md`**. The compactor child's
transcript records the `mark_for_deletion` call and its `{"status":"marked"}`
result verbatim. Across every agent directory in two workspaces the correlation
is total and has no counterexample: **the directories missing `001-*` are
exactly the directories that have a `summary/`.**

With the default `AutoExpand { responses: true, others: false }`
(`src/transcript/rows.rs`), the surviving machinery rolls into one `⚙ N
inference calls · …` line, so the pane opens on an aggregate and a model answer
with **no user turn above them** — the reported shape exactly.

## What this ball builds

**yog has no reader for `<agent>/summary/**` anywhere.** A `grep -rn summary
src/` finds only unrelated `summary` fields in fan/opslog/help. The text that
*replaced* the deleted span has no seat in the pane. Build one:

1. **Derive the gap** from the monotonic `NNN` counter — a first entry whose
   number is not the lowest expected, or a discontinuity mid-sequence. A query
   over bytes already read; not a stored field, not an index.
2. **Paint what replaced it**, reading `summary/<NNN>.md`. Verified location:
   the summary lives at the **worktree root, a sibling of `messages/`** — NOT
   inside it — zero-padded to 3, with a **branch-global** sequence (`next_seq` =
   max + 1 across all passes).
3. The marker must read as *the record was rewritten here*, never as another
   turn — the summary is the compactor model's prose, not the operator's words.
   If the summary is missing or unreadable the gap marker alone is still the
   honest answer, so the marker must not depend on the summary existing.

**The one hard constraint, found by the parked attempt:** there is **no on-disk
link between a summary and the span it replaced.** One pass can delete several
disjoint runs (one summary, many gaps); two passes' deletions can abut into a
single hole (one gap, many summaries). **Any pairing is positional and must be
stated in the code as an assumption, never asserted as exact.** This was the
weakest part of the parked sketch — do better or say plainly that you cannot.

## Parked work to resume or discard on its merits

Commit `4e1b85fa` on the machine-local branch `work/bl-7bd2` (never pushed,
never touched `main`) holds a compiling sketch with **no tests, no DESIGN
amendment, and it never saw the gate**: a new `src/transcript/compaction.rs`, an
`EntryKind::Compacted { first, last, summary }`, a `parse_name` that also
returns the counter, one weak role-less row per gap, the turn-rollup boundary
classification, the wire encode/decode both ways, and a `u32_of` in
`src/boundary/codec/fields.rs`. Treat it as a sketch, not a baseline — take it
only where it is right.

## The invariant this falsifies — fix the doc

DESIGN §5.1 #31 and `src/rail/pin.rs` rest on append-only, and deletion is not
append:

> Exact, not convenient: `messages/` entries are append-only under a monotonic
> counter, so the pinned tree's entries and today's leading entries are the same
> bytes …

After a compaction the leading entries of today's listing are NOT the pinned
tree's bytes, so every pinned as-of transcript view is wrong on its own stated
terms. Amend §5.1 #31 and `pin.rs` to state what is actually exact and what is
not. `tests/design_citations.rs` and `tests/design_module_map.rs` gate the
correspondence.

## Scope

This ball makes a squash **visible**. Preventing the opening prompt's deletion
is upstream lernie bl-898f (the prompt is written twice — as `goal.md` and as
`messages/001-user.md` — so the compactor reads it as duplication). Both are
wanted and neither substitutes for the other. Secondary consumers that read the
same shrinking directory are already filed as bl-fde5.