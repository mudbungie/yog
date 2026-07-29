+++
title = "DESIGN: context menus as an interaction surface; workspace deletion as the first consumer"
created = 1785287538
updated = 1785287538
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator, 2026-07-28, verbatim:

> we could probably introduce right click/context menus. I was thinking about deleting a workspace, and there isn't currently a way to do that (a gap)

A DESIGN task; deliverable is edits to docs/DESIGN.md. Settle: (1) the context-menu doctrine — right-click as a surface for object-scoped verbs; which objects (workspace tab, conversation row, ball row…), and the rule for what belongs in a context menu vs a visible affordance (glyph doctrine analogy: a context menu must never be the ONLY carrier of a critical verb? decide). Note egui has native context-menu support and yog already uses one (workspace_tab middle-click unpin mentions a context menu). (2) Workspace deletion semantics — what deletion means for the world dir, the workspace's conversations/agents branches, ops.jsonl history; destructive-action confirmation doctrine; interplay with bl-df65 (explicit workspace names, in flight — read its outcome first if landed). File implementation follow-up tasks; do not implement.