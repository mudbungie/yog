+++
title = "REMOTE §9.4 — the shell paints only boundary payloads: retire the raw GitTree/Agent imports from paint code"
created = 1786683979
updated = 1786687639
claimant = "Gudgeon"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"

[[blockers]]
id = "bl-f5f6"
on = "claim"
+++
docs/REMOTE.md §9 step 4 of the client/server split (bl-b9a2). Paint code still imports raw domain types (git_tree::{Agent, AgentState, GitTree} and friends) in several shell files, and AppModel hands out domain references directly; a thin client can only paint Reply payloads. Promote the remaining raw-domain paint inputs to boundary view-models so the shell consumes only what the wire can carry. Verify the current import sites against the tree before editing; this body drifts. Boundary-surface work: serialize (needs-edge enforces order).