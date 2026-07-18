+++
title = "Y24: (optional, only if measured) incremental streaming refresh"
created = 1784349564
updated = 1784349564
parent = "bl-4e66"
priority = 4
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-dd2a"
on = "claim"
+++
DESIGN.md §15 Y24. Replace whole-root rebuild with a streaming-text-only re-read on steps/** events for the focused workspace, IF profiling shows the 100ms debounced rebuild costing frames. Correctness is already owned by Y6; this is purely an optimization and lands only with a measurement in the task. Do not claim without a measurement. Files: src/app/dirty.rs, src/git_tree/streaming.rs.