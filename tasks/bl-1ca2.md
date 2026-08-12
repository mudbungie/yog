+++
title = "no full-cover overlays: Config and its kin become tab focus, not a toggle painted over everything"
created = 1786508785
updated = 1786509509
claimant = "Mullion"
priority = 1
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-9dde"
on = "claim"

[[blockers]]
id = "bl-c21f"
on = "claim"

[[blockers]]
id = "bl-2e18"
on = "claim"
+++
Operator ruling 2026-08-11: "Several places (config, eg) are interface overlays
instead of tabs (toggle on), but cover everything so really should just be a
tab focus."

Per the amended DESIGN §11: enumerate every surface that takes the whole
center as a toggled overlay (Config is the named case; check Login, world
search, and any other full-cover toggle) and reseat each as a tab focus with
ordinary tab semantics — reachable, dismissable, keyboard-addressable, and
never painting over the conversation. Small frame-owning modals (new-workspace
name form) are out of scope. Surfaces: src/shell/{config_edit*,login_pane,
search_pane,top_bar,mod}.rs — verify seating against the tree first.