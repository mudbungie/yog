+++
title = "chat scroll is sluggish and sticky — find the frame-time cost and make scrolling free"
created = 1785733752
updated = 1785733752
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator report 2026-08-03, verbatim: 'the scroll in the chat window is weirdly sluggish, sticky.'

The shell's rule is that the frame renders snapshots and never does work (UI/backend isolation; DESIGN records it). Sluggish+sticky scroll in the chat pane says something in the scroll path is paying per-frame: candidates to MEASURE, not guess — full-transcript row painting with no virtualization (every row laid out every frame regardless of visibility), galley/text layout recomputed per frame for unchanged content (no galley caching), texture/asset work in the row path, the new crossing-rule or rollup computations running per frame instead of per snapshot (bl-95a9 landed 417c191 — its crossings() is meant to be pure-per-snapshot; verify it isn't being recomputed per frame), egui ScrollArea sizing pathology (auto_shrink/content-size churn), or stick_to_bottom fighting user scroll (that reads exactly as 'sticky').

The work: profile first (puffin/frame timings or coarse instrumentation — whatever the repo already has; add nothing permanent), identify the actual cost or the sticky-scroll logic bug, fix it: virtualize rows via the ScrollArea show_rows/show_viewport idiom if painting is the cost, cache what a snapshot derives once, and if stick-to-bottom is the culprit make it yield to user scroll cleanly (bottom-stick re-engages only when the user returns to the tail — record the rule in DESIGN). A perf assertion per repo idiom if the testing surface supports it; otherwise the acceptance idiom for the stick behavior. Verify paths against the tree — bl-a119 (composer/inbox surface) and bl-3acb (row styling, held) are adjacent; coordinate via fold, not shared edits.