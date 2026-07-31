+++
title = "decompose git_tree: 7 files over 250 — prior analysis says 2 SPLIT / 5 LEAVE, re-verify before acting"
created = 1785461153
updated = 1785461154
claimant = "pedantic-sweep2"
parent = "bl-52f8"
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["refactor"]
+++
Wave 5 of the bl-52f8 decomposition sweep.

Targets (line counts on work/bl-52f8):
- src/git_tree/fd_probe.rs 296
- src/git_tree/lsof.rs 292
- src/git_tree/terminal.rs 283
- src/git_tree/marks.rs 274
- src/git_tree/tests/fixture.rs 272
- src/git_tree/mod.rs 270
- src/git_tree/tests/repo.rs 267

Inherited (unverified) verdicts from the bl-52f8 seam analysis:
- SPLIT tests/fixture.rs — git-shape builders vs four methods writing plain files git never sees (steps/, inbox/) -> disk_fixture.rs
- SPLIT mod.rs — module wiring + platform cfg vs seven inert view-model types -> model.rs, re-exported so the public surface stays byte-identical
- LEAVE fd_probe.rs, lsof.rs, terminal.rs, marks.rs, tests/repo.rs — a /proc scanner with nine failure exits, an lsof -F codec, the §4.4 settled-tail classifier, a closed mark enum, 16 tests of one function

Re-verify each before acting. A justified LEAVE is a success. No behavior change. docs/DESIGN.md is owned by bl-43cd — do not touch.