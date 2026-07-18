+++
title = "Y14: projects + balls view-models + join-state table"
created = 1784349559
updated = 1784352414
claimant = "filtered"
parent = "bl-4e66"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-9fc6"
on = "claim"

[[blockers]]
id = "bl-32d3"
on = "claim"

[[blockers]]
id = "bl-7f92"
on = "claim"
+++
DESIGN.md §15 Y14. Clone enumeration with nested-delivery detection (decoded-path prefix match, "internal" toggle); bl list --json / bl show <id> --json invocation (cwd = project) and parse; the derived status ladder; the §3.5 join-state classification joining balls to enumerated workspaces (detached, claimed-elsewhere, delivered, orphaned-project); ball detail (full bedrock frontmatter + body); closed listing on demand. Roster gains ball rows and join badges. Files: src/projects/mod.rs (~160), src/projects/balls.rs (~240).