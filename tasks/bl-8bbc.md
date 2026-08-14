+++
title = "REMOTE §9.6 — registration and scoping: the per-workspace client registry, reply filtering, auto-registration on create, per-seat ui.json split"
created = 1786684039
updated = 1786684039
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"

[[blockers]]
id = "bl-b6fa"
on = "claim"
+++
docs/REMOTE.md §9 step 6 of the client/server split (bl-b9a2). The workspace is the trust domain (§1.5, §4): a registration file in the world records that client C participates in workspace W; a connection sees and gestures into exactly its registered set, and everything else is ABSENT from replies, not forbidden (scope errors that confirm existence are a disclosure). Create-over-the-wire auto-registers the creating client; first registration on a fresh server is an operator-written file (§4). Revocation is deletion. ui.json splits per §7: world facts (seen/pins/acks) stay one shared document; pane facts (panel sizes, collapsed sets, knobs) become per-client documents keyed by client identity, server-held. Verify current ui_state/boundary shapes against the tree before editing; this body drifts.