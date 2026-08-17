+++
title = "messages/ is not append-only: compaction deletes entries and the transcript renders the rewritten record as whole — a real latent defect, NOT the vanished-first-prompt sighting"
created = 1786936867
updated = 1786937014
claimant = "Gapwright7bd2"
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
## The premise this ball was filed on is WITHDRAWN

This ball was opened from an operator sighting on 2026-08-16 — a conversation's
opening question gone from the Transcript pane while the reply that answered it
was still there — and attributed it to lernie's compactor. **The operator has
since corrected that**: "compaction" was used loosely, every OTHER message in
the conversation rendered correctly, and only the FIRST user prompt
disappeared. Compaction squashes a *span*, so it cannot produce that shape.

**Do not read this ball as the explanation for a vanished first prompt.** The
real defect is yog's own, is being scoped separately, and is not this. What is
below is what the investigation found on its way past — a genuine latent
defect and a genuinely false invariant, both worth keeping, neither of them
what the operator saw.

## What IS real here — verified against the tree, not inferred

`src/transcript/mod.rs` `build()` joins `<workspace>/agents/<agent-id>/messages/`
and `read_messages()` runs `std::fs::read_dir`, sorts by filename, reads each
file. There is no git history behind it. That directory is the agent branch's
worktree checkout (`src/files_view/mod.rs`). The answer both seats read is
`src/shell/inspector/reads.rs` -> `src/boundary/answer.rs` ->
`src/boundary/answer/inspector.rs`.

lernie's compactor deletes message files. Read at lernie 0.0.10:

- `src/prompt/compactor/tools.rs` `mark_for_deletion` runs
  `git rm -r -q -- <path>`; its own routing fixture nominates
  `messages/001-user.md`.
- `src/prompt/compactor/land.rs` mints a compaction base whose tree is the tree
  at the compaction point with the deletions applied and the new summary added,
  parented on the span's lower bound, then rebases the branch forward onto it.
  The files leave the checkout yog is reading.

So the pane can render a REWRITTEN record as if it were the whole record: no
gap, no marker, no summary. That is a real defect and it is still open. It just
is not the one that was reported.

## Corrections to the original body

- **The `summary/**` shape, which the original body told the reader not to
  trust, is now verified.** `tools.rs` `write_summary` writes
  `summary/<NNN>.md` at the WORKTREE ROOT — a sibling of `messages/`, not
  inside it — zero-padded to three (`SUMMARY_SEQ_WIDTH = 3`), and the seq is
  branch-global over that directory's contents (`next_seq` = max existing + 1),
  so several compaction passes on one branch share one numbering. The landing
  commits it into the base beside the deletions, and
  `src/prompt/dispatch/step_commit/inherited.rs` lists `summary` beside
  `messages` in `DIALOG_PATHS`, so it is inherited dialog like the transcript.
- **There is NO on-disk link between a summary and the span it replaced.** A
  pass may delete several disjoint runs (one summary, several gaps) and two
  passes' deletions may abut into one hole (one gap, several summaries). Any
  pairing a reader derives from the filesystem alone is positional and must be
  stated as such, never claimed exact.
- **The secondary consumers were checked and split out as bl-fde5** —
  `src/science/observed.rs`, `src/search/corpus.rs`, `src/monitor/window.rs`
  and the message COUNT at `src/git_tree/enumerate.rs`. None of them is a
  compile dependency on any new variant; each is its own judgement about what
  the surface should say.

## The invariant that is false regardless of the sighting

DESIGN §5.1 #31 and `src/rail/pin.rs` both rest on append-only:

> **Transcript** is a *prefix* of #12, cut at the notch's own `Place::cut` (#29)
> — everything ahead of that call's own model output. Exact, not convenient:
> `messages/` entries are append-only under a monotonic counter, so the pinned
> tree's entries and today's leading entries are the same bytes ...

Deletion is not append. After a compaction the leading entries of today's
listing are NOT the pinned tree's bytes, so the as-of view is wrong on its own
stated terms. `src/rail/pin.rs` `transcript_as_of` states the same premise in
its doc comment. Whoever takes this ball amends both rather than coding around
them. DESIGN §5.1 #12 and §11 are silent on compaction rather than wrong about
it; that silence is the hole.

## Work already done and parked

A branch `work/bl-7bd2` exists on this machine with one commit (`4e1b85fa`,
"WIP parked, NOT the operator's defect"), left there by the unclaim. It
derives the gap from the monotonic counter, seats a virtual
`EntryKind::Compacted { first, last, summary }` marker in it (the same
virtual-entry move `Transcript::with_live` already makes for the streaming
tail), pairs summaries to gaps positionally, projects one weak row with the
empty role seat and a turn-BOUNDARY step so no rollup can swallow it, and
spells the variant both ways across the boundary. It compiles (`cargo build
--all-targets` clean). It has NO tests, NO DESIGN amendment and has not been
through the gate. Treat it as a sketch, not a deliverable — in particular the
positional summary pairing deserves the attack the original body asked for.