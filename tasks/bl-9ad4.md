+++
title = "panel boundaries are not draggable — make them resizable"
created = 1785645033
updated = 1785645348
claimant = "Tamsin"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator report 2026-08-01: the boundaries between UI panels can't be dragged; they should be resizable. egui SidePanel/TopBottomPanel support .resizable(true) — audit every panel in the layout, enable resizing where it makes sense (workspace list, conversation list, any side/bottom panels vs the central message area), and persist the chosen sizes with the same UI-persistence mechanism bl-42e7 lands for text size (single source of truth — coordinate, don't invent a second store; if bl-42e7 is still open when claimed, check its state first).

## Note to panel-fixer (2026-08-01, from the coordinator) — read this when close refuses

A second implementer (Tamsin) was dispatched against a claim that looked dead (clean worktree at main) and claim-stole before discovering your live work. It stood down; **none of your files were touched**. Residue:

- The claim record now says `Tamsin`. Ignore it — `bl close` has no identity check; do NOT unclaim (that would tear down your worktree). Close normally when done.
- Your `bl close` will refuse ONCE with an unseen task-file diff (these body edits). That is the documented refusal — **re-run `bl close bl-9ad4` bare** and it seals.

## Findings from the stood-down implementer (headless-probed on egui 0.29 — check these at review)

- **`.resizable(true)` is silently defeated unless the panel body claims the panel's size.** egui stores the rect the contents painted as next frame's panel size: a `TopBottomPanel` with a short body settles at ~20 px, drag or no drag; with `ui.set_min_height(ui.max_rect().height())` first in the body it holds its dragged height. `SidePanel` behaves identically (why the conversation list already drags today — its rows fill the width).
- Any inner `ScrollArea` with a fixed `max_height` (start_pane goal editor at 160, activity ops list at 160) must derive from `ui.available_height()` instead, or dragged space is dead.
- The activity pane conflicts with `CollapsingHeader`: stored `PanelState` wins over `default_height`, so a height dragged while collapsed persists and a re-open comes back at chip height.
- `src/shell/mod.rs` owns `DEFAULT_SIDE_PANEL_WIDTH` (260) / `MIN_SIDE_PANEL_WIDTH` (24); `src/shell/acceptance/geometry.rs` imports the former and `docs/DESIGN.md` cites the latter by name — both must move with the constants if relocated.
- The bl-42e7 persistence store to extend: `ui.json` via `UiState` (`src/ui_state/knobs.rs`, forgiving reads off one `serde_json::Map` root, write-through `save()`); round-trip test pattern `src/app/tests/knobs.rs::text_size_survives_a_relaunch` (two `Harness::model()` builds over one XDG root).