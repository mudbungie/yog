+++
title = "yog fixture's world has no roles: block in its config lineage, so /effort and /priority are refused in band on every fixture drive: seed the fixture with an assigned worker role"
created = 1789002937
updated = 1790058387
claimant = "Urinalyses-C"
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["usability-r3"]
+++
From yog-android bl-0691's drive: on `yog fixture busy` the tuning gestures are refused because the fixture's config lineage carries no roles: block, so no seat lane can drive effort/priority end to end against a fixture. Seed the fixture worlds with a worker role assigned to a provider row (a keyless one is fine for the refusal-free path) so the tuning reads answer and the writes take.

---

Premise verified at HEAD: src/fixture/lay.rs::found committed the config/default root with 'version' alone, so no laid workspace carried providers.yaml. Reproduced the exact refusal by driving the boundary against a laid world: providers.yaml: no `  worker:` block entry; yog edits only the block form litany writes. Fixed by committing the litany-template role block onto the trunk. No scripts/drive beat is unlocked — nothing in the drive harness drives a `yog fixture` world at all; the only door is 'make fixture STATE=<name>' and its consumers are the seat and phone harnesses in other repositories. The wall is untouched: the tuning pair reads no provider table, so a keyless row is all the refusal-free path needs, and Recipe::brazen stays the one state that lays one.
