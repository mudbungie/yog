+++
title = "design: batteries-included rearch spec + arch"
created = 1784784066
updated = 1784784067
claimant = "Cleansing"
parent = "bl-b5d1"
priority = 1
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Deliverable: a tracked design doc (docs/ in the yog repo, edited like code) covering: (1) SPEC — invariants: batteries-included install, version independence from local checkouts/installed binaries, single source of truth for each pinned version; (2) ARCH — how each seam moves from spawn-a-binary to call-a-library, including lernie's process model (detached prompts, self-exec advance hops), bz's data-plane spawn inside lernie, and yog's bl reads; what must change in the lernie/balls repos (lib targets, publishing) vs in yog; (3) PLAN — the implementation balls, ordered, with their cross-repo dependencies. Grounded in the seam maps, not assumption.