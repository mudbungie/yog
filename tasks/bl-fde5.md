+++
title = "compaction shrinks the science verdicts, the search corpus, the monitor window and the message count, and none of them says so"
created = 1786936978
updated = 1786937893
claimant = "Compact-fde5"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Split out of bl-7bd2, which fixed the Transcript pane only. lernie's compactor
`git rm`s files out of `<workspace>/agents/<id>/messages/` and squashes the span
(lernie ARCH §2.6, §2.7); compaction is on by default. bl-7bd2 made
`transcript::build` derive the hole in the monotonic `NNN` counter and seat a
virtual `EntryKind::Compacted` marker in it, carrying the `summary/<NNN>.md`
lernie wrote in its place. Four other readers of the same directory shrink with
it and say nothing. Verified against the tree at bl-7bd2's delivery:

- `src/science/observed.rs` — `verdicts()` filters `EntryKind::Delivered` over
  `transcript::build`, so a compacted conversation's early verdicts silently
  leave the projection. A science run over a compacted arm reports fewer
  verdicts than the arm produced, with no marker anywhere in the answer. It
  already skips the new `Compacted` arm via its `_ => None`, so this is a
  behaviour question, not a compile one.
- `src/search/corpus.rs` — `read_conversation()` maps every entry's `raw` into
  a `Field::Text`. Since bl-7bd2 the marker's `raw` IS the summary's bytes, so
  a compacted conversation is at least searchable through the summary; the
  deleted messages' own text is unrecoverable from disk and no rule can fix
  that. What is missing is that a hit count over a compacted conversation is
  not the count over the conversation, and nothing states it.
- `src/monitor/window.rs` — `say()` puts the marker in the same arm as
  `Streaming`/`Raw` (the empty string), deliberately: the marker is yog's own
  statement, not the agent's, so quoting it would fold yog's words into the
  evidence. But the alignment monitor's window over a compacted conversation
  is a window with a hole in it, and VISION §4.9's premise is that the monitor
  reads what the agent read. The summary is what lernie handed the agent in
  place of the span, so it is arguably exactly the right substitute — that is
  the ruling this ball needs.
- `src/git_tree/enumerate.rs` — `messages_from_disk()` is its own readdir
  (§5.1 #12's one-walk discipline), so `Agent::messages` is a count of files
  present, not of messages that ever landed. The §7.2 pending echo reconciles
  against it: a compaction landing mid-flight makes the count go DOWN, which
  the echo has no reading for.

Not one of these is a compile break — bl-7bd2's new variant lands in existing
wildcard arms. Each is a separate judgement about what the surface should say,
which is why it was declined rather than widened.