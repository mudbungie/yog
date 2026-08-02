+++
title = "right-click a conversation → delete the agent"
created = 1785645885
updated = 1785645898
claimant = "delete-fixer"
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator request 2026-08-02, verbatim: 'I need to be able to right click -> delete an agent'. Add a context menu (egui context_menu) on conversation rows (chat list; descent-tree members if it generalizes) with Delete. Constraints: (1) deletion is yog's destructive-verb class — DESIGN says workspace deletion is v1's ONLY member, with typed-name confirm enumerating what dies and no keyboard path; deleting an agent joins that class: same confirm discipline (name what dies: the agent, its children, pending inbox), scaled sanely (a click-confirm dialog; decide whether typed-name is warranted for a single agent vs the whole workspace — record the ruling in DESIGN). (2) Find lernie's actual removal surface first: does lernie 0.0.3 expose deleting an agent (agents/* ref + files + steps), or is there only yog-step delete-workspace? If lernie has no lawful removal verb, DO NOT hand-delete lernie-owned state from yog — write findings + the lernie-side ask into this ball and stand down (cross-repo, cf. bl-50f3). (3) Running driver: a delete of a live agent must stop it first (Stop verb exists) or refuse while live.