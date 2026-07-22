+++
title = "conversation-ball association: derived join + badges + organizing views"
created = 1784696431
updated = 1784699030
claimant = "Cleansing"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-abd9"
on = "claim"
+++
## Intent
Conversations are unique to themselves but may have one or more balls associated. Association is DERIVED, never stored (single source of truth):
1. start-flow balls: the root's goal.md carries the composed 'Ball <id>:' stamp (§3.3) — parse it back (yog composed it; the parse is the inverse of the compose, one module owns both).
2. agent-picked balls: claimant == workspace name (§3.2 join) gives workspace-level bound balls; a conversation-level attribution beyond the goal stamp does not exist yet — render workspace-level balls in the workspace header, per-conversation badges only from (1). Record this scoping honestly in DESIGN §3.
## Render
- Conversation rows: ball-id badges (from 1), status-colored via the §3.5 join.
- Conversation header: its balls with title/status, link to ball detail.
- Views to organize: the conversation list gains a grouping toggle — flat by recency (default) | grouped by ball (balls with their conversations under them, unassociated conversations last). Pure VM + tests; shell glue thin.
DESIGN §3.2/§3.5 amendment for the derived conversation-level join. make check green, 100%, caps hold.