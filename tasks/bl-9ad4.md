+++
title = "panel boundaries are not draggable — make them resizable"
created = 1785645033
updated = 1785645194
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator report 2026-08-01: the boundaries between UI panels can't be dragged; they should be resizable. egui SidePanel/TopBottomPanel support .resizable(true) — audit every panel in the layout, enable resizing where it makes sense (workspace list, conversation list, any side/bottom panels vs the central message area), and persist the chosen sizes with the same UI-persistence mechanism bl-42e7 lands for text size (single source of truth — coordinate, don't invent a second store; if bl-42e7 is still open when claimed, check its state first).