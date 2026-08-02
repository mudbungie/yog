+++
title = "text selection: double-click-drag doesn't extend selection by word boundaries"
created = 1785645008
updated = 1785645008
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator report 2026-08-01: standard text-selection idiom — double-click to select a word, keep the button down and drag — should extend the selection word-by-word (adjoin to word boundaries). In yog it doesn't; the drag reverts to character-level selection. Small UX thing, high irritation.

Likely an egui behavior in whatever widget renders conversation text (Label/TextEdit/custom galley handling). Determine whether yog owns the selection code path (fixable here) or it's stock egui (then: can yog intercept double-click-drag and do word-granularity selection itself? yog pins eframe 0.29 — check what that version does before writing custom code). Triple-click-drag → line granularity is the same family; handle it if it falls out cheaply, note it if not.