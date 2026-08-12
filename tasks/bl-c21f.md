+++
title = "focus lands in the chat composer: on app open and on any agent selection, by pointer or keyboard"
created = 1786508777
updated = 1786509107
claimant = "Fenwick"
priority = 1
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-9dde"
on = "claim"
+++
Operator ruling 2026-08-11: "When you open the app, focus to the chat prompt.
When you select an agent, focus to the chat prompt."

DESIGN §11 focus discipline (src/shell/focus.rs) already sends launch and
POINTER selection to the composer; its rule 2 exempts keyboard gestures
(roster ↑/↓, digits, bare letters). The amended DESIGN makes agent selection
land the composer unconditionally. Implement per the amended section, keep the
one-mechanism rule (single deferred request bit), and extend the acceptance
focus drive to cover keyboard selection. Verify the current behavior against
src/shell/focus.rs and acceptance/focus before editing.