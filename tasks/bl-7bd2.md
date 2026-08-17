+++
title = "the transcript is a readdir of a directory compaction DELETES from, so a squashed opening question just vanishes — derive the gap and paint what replaced it"
created = 1786936867
updated = 1786936877
claimant = "Gapwright7bd2"
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Operator report, 2026-08-16: a conversation's opening question was gone from the
Transcript pane while the reply that answered it was still there. That asymmetry
is the signature of compaction, and the pane says nothing about it.

## Why it happens — verified, not inferred

The transcript is a **readdir of the agent worktree as it stands right now**,
with no git history behind it. `src/transcript/mod.rs` `build()` joins
`<workspace>/agents/<agent-id>/messages/` and `read_messages()` runs
`std::fs::read_dir`, sorts by filename, reads each file. That directory is the
agent branch's git worktree checkout (`src/files_view/mod.rs`). The wire path is
`src/shell/inspector/reads.rs` → `src/boundary/answer.rs` →
`src/boundary/answer/inspector.rs`, which is the one home for the answer both
seats read.

lernie's compactor **deletes message files**. `src/prompt/compactor/tools.rs`
`mark_for_deletion` runs a literal `git rm -r -q -- <path>` — and its own test
fixture is `messages/001-user.md`. `land.rs` then squashes the span into a
compaction base and rebases the branch forward in the parent worktree, so the
files leave the checkout that yog is reading. Compaction is **on by default**:
the shipped `template/workflow.yaml` sets `every_n_commits, n: 20` with no
`keep_recent`, so the compaction point is the branch tip and entry 001 is inside
the span.

lernie's ARCHITECTURE already names this consequence outright:

> A branch whose whole record of being prompted was squashed by compaction
> (below) falls back to the id: the dispatch message is the transcript's first
> entry, so its absence means the record was rewritten, and the id still
> carries the one sender the branch's own existence records.

So the disappearance is upstream-lawful. **What is yog's defect is that the pane
renders a rewritten record as if it were the whole record** — no gap, no marker,
no summary, nothing. The operator cannot tell a conversation that started with
that reply from one whose opening was squashed away.

## The one invariant this falsifies — fix the doc, do not code around it

DESIGN §5.1 #31 and `src/rail/pin.rs` both rest on append-only, and it is not
true:

> **Transcript** is a *prefix* of #12, cut at the notch's own `Place::cut` (#29)
> — everything ahead of that call's own model output. Exact, not convenient:
> `messages/` entries are append-only under a monotonic counter, so the pinned
> tree's entries and today's leading entries are the same bytes …

Deletion is not append. After a compaction the leading entries of today's
listing are NOT the pinned tree's bytes, so the as-of view is silently wrong on
its own stated terms. Amend §5.1 #31 and `pin.rs`'s comment to state what is
actually exact and what is not. This is the global rule in force: "Nothing is
set in stone. Don't implement the wrong thing just because an arch doc said do,
but don't implement a deviation: fix the doc."

DESIGN is otherwise consistent with the code — §5.1 #12 and §11 both define the
Transcript as "`messages/NNN-*` in filename order", never as a durable record.
The spec is silent on compaction rather than wrong about it. That silence is the
hole to fill.

## What to build

**Derive the gap; do not store it.** The `NNN` counter is monotonic, so a
compacted span is visible in the listing itself — a first entry whose `NNN` is
not the lowest expected, or a discontinuity mid-sequence. That is a query over
bytes already read, not a new field and not an index.

**Render what replaced the span, not just a hole.** Compaction does not destroy
the meaning, it *substitutes* one: `land()` writes `summary/**` alongside the
deletions, and yog reads none of it today. A compacted span should paint as a
distinct row carrying the summary, seated at the position the gap occupies, so
the pane's answer to "where did my question go" is on screen rather than in this
ball. Confirm the on-disk shape of `summary/**` against the lernie source before
designing the row — do not trust this paragraph for it.

The empty case already has the only absence path there is
(`src/transcript/render.rs`, `"(no messages yet)"`); a gap is not that case and
must not borrow it.

**Attack it before committing.** A summary row is content lernie's compactor
model wrote — it is not the operator's own words and must not be painted as if
it were. Whatever tone/class it lands in, the row has to read as *the record was
rewritten here*, not as another turn in the conversation. If the summary is
absent or unreadable, the gap marker alone is still the honest answer, so the
marker cannot depend on the summary existing.

## Secondary consumers read the same directory and shrink with it

Out of scope to fix here, in scope to check and file: `src/science/observed.rs`,
`src/search/corpus.rs`, `src/monitor/window.rs`, and the message COUNT at
`src/git_tree/enumerate.rs`. A compacted conversation silently shrinks its
science verdicts, its search corpus and its counted messages. File what you find
as its own ball rather than widening this one.

## Wire

Any new field (a gap marker, a summary payload) crosses the boundary:
`src/transcript/wire.rs`, `src/boundary/reply/encode.rs`,
`src/boundary/reply/decode/inspector.rs`. `tests/design_citations.rs` and
`tests/design_module_map.rs` gate doc/code correspondence — the DESIGN amendment
above is not optional for a green gate.