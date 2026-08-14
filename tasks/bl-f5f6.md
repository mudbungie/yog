+++
title = "REMOTE §9.3 — name-based addressing: PathBuf leaves the boundary types; the wire spelling is the name, resolved at the dispatch chokepoint"
created = 1786683972
updated = 1786683972
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"

[[blockers]]
id = "bl-7067"
on = "claim"
+++
docs/REMOTE.md §9 step 3 and §8 of the client/server split (bl-b9a2): paths never cross the wire — absolute PathBufs in Action/Query are meaningless across machines and a disclosure besides. Wire identity is the workspace name (operator-chosen, validated shape) and the project display name; the engine resolves names to paths at dispatch/answer. Migrate the boundary types themselves, not just the codec spelling. Verify current type shapes against the tree before editing; this body drifts. Boundary-surface work: serialize (needs-edge enforces order).