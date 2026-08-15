+++
title = "REMOTE 9.7 residual: migrate the window's remaining reads onto the wire, surface by surface"
created = 1786763797
updated = 1786764088
claimant = "Taffrail"
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
bl-ae05 built the read path (REMOTE 1.2, 9.7) and pointed exactly one surface at it: the clients section, whose rows are a Reply::Clients that crossed loopback mTLS, was scoped against the window's own registrations and was decoded by reply::decode. Every other read still paints the published snapshot in process.

This is scope, not architecture. The transport is under the whole frame already: AppModel::wire_ask declares a standing question and returns whatever landed, the asker dials off-frame at human cadence, and nothing here needs an engine, codec or wire change. Migrating one surface is three moves:

1. Declare the Query the surface's content already has a boundary spelling for (REMOTE 9 steps 1-4 boundary-completed the reads, so it exists).
2. Paint the decoded Reply, and paint the refusal rather than swallowing it.
3. Delete the in-process accessor and its per-derivation memo — the fold is the engine's now. bl-ae05's own precedent: AppModel::clients, live_clients, the clients SnapMemo and the model's copy of the presence map all went with the one surface it migrated.

What to watch, learned from the first one:

- The memo goes with the derivation. An answer IS the cached fold, refreshed at human cadence rather than per derivation, so a SnapMemo keyed on the derivation is a second cache of a thing the link already holds.
- A question is keyed by its own encoded envelope, so a surface that stops painting stops asking and its answer is dropped. A surface behind a collapsed section therefore costs nothing, and needs no explicit unsubscribe.
- The frame order is settle-then-render, so a question reaches the asker on the frame after the one that first painted it and its answer lands one cadence period later. A surface that must be correct WITHIN a frame of an action is the acts ball's problem, not this one's.
- Scoping is real: a read naming a workspace the window is not registered in earns the resolver's own unknown-workspace refusal. The asker seats the window in every enumerated workspace each pass, so this only bites for one pass after a create.

Not in scope: the acts (its own ball), and Prepared::binding (REMOTE 8.1), which needs the composer migrated before Prepared can become opaque to the seat.

Honest increments are allowed and expected — one surface or a family per pass, each landing green, with what is left recorded in REMOTE 9.7.