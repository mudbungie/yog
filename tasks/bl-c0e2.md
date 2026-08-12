+++
title = "workspace blast radius: providers and every setting live inside the wall — per-workspace brazen config, nothing ambient but the roster"
created = 1786508776
updated = 1786509105
claimant = "Walling"
priority = 1
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-9dde"
on = "claim"
+++
Operator ruling 2026-08-11: "workspaces are an entirely separate space;
essentially an app-wide blast radius. Different sets of conversations,
settings, providers, all of it."

Today conversations and claimed balls are per-workspace, but brazen
(providers/credentials) is deliberately ambient-shared (DESIGN §16.2 before
amendment) and app settings are global. After the design ball lands, implement
the extended wall: switching workspace switches EVERYTHING — provider config,
login state surfaces, settings, conversations, balls. The only cross-workspace
fact is the workspace roster itself.

Start by enumerating every fact that is currently ambient (brazen config path
fold in src/xdg / world env composition, ui.json scoping, cadence.yaml, ops
scoping) and move each inside the wall per the amended DESIGN. Verify premises
against the tree before editing — this body was written from docs, not code.