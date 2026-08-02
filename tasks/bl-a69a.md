+++
title = "composer draft bleeds across verbs and targets — a start goal becomes a message to whatever is selected next"
created = 1785647005
updated = 1785647005
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Repro: in the new-conversation view type a goal (leave it unsent — e.g. after a failed start, where the draft survives by design). Now select an existing conversation: the bottom composer switches to '→ message <name>' mode with the SAME text still in the box. Enter would send a text written as a fresh start goal to an unrelated agent as a message. The draft is RAM until sent (§13.1) — but it must be RAM per context (per target+verb), not one global buffer that re-labels itself. Industry norm (every chat app): drafts are per-conversation. Fix: key the draft buffer by its target (new-conversation-in-<ws> / message-to-<agent> / ball-<id> draft); switching selection shows that target's own draft. Acceptance: type in new-conversation, select an agent → its message box is empty; select back → the goal draft is still there.