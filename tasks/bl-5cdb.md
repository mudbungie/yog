+++
title = "chat transcript reads top-anchored: following the streaming tail requires constant manual scrolling — anchor the tail, make scrolling the review gesture"
created = 1785648846
updated = 1785648846
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
## Operator complaint (2026-08-01)

Text in the chat streams in at the top and grows downward; to follow the tail you must scroll continuously. It should be the other way: the newest text sits at the bottom and pushes older text up and out of view. Following the live tail is the default that costs nothing; scrolling (up) is the deliberate act of reviewing history.

## Verified code fact (verify again before editing — do not trust this ball blindly)

`src/transcript/render.rs` line ~39: `egui::ScrollArea::vertical().show(ui, ...)` — a bare scroll area, no tail anchoring. egui provides exactly this behavior: `ScrollArea::vertical().stick_to_bottom(true)` keeps the view pinned to the bottom as content grows, and releases the pin the moment the user scrolls up (scrolling back down to the bottom re-engages it). That is the whole desired semantic.

## Scope

- Primary: the Transcript tab chat view (`src/transcript/render.rs`).
- Then audit yog's other tail-growing surfaces (e.g. `src/shell/activity.rs`, `src/steps_view/render.rs`, `src/inboxview/render.rs`) — any view whose content is a live-appending stream gets the same anchor. A view that is a static list (conv_list, config) does NOT. Use judgment; one idiom, applied where the content is a tail.
- 'Text starts at the bottom' when content is shorter than the viewport (bottom-aligned underfull content, terminal-style) is OPTIONAL — implement only if it falls out cleanly; stick_to_bottom is the load-bearing fix.

## Discipline

Render behavior must be tested (the repo has simulated-pointer render tests in `src/transcript/tests/render.rs` as prior art). DESIGN.md §11 governs the transcript tab — check whether it pins scroll behavior; amend the doc if this changes stated behavior.