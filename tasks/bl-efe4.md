+++
title = "decompose shell: 7 files over 250 including mod.rs at 296 — seams must stay under src/shell/ (tarpaulin-excluded)"
created = 1785460973
updated = 1785460974
claimant = "pedantic-sweep2"
parent = "bl-52f8"
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["refactor"]
+++
Wave 4 of the bl-52f8 decomposition sweep.

Targets (line counts on work/bl-52f8):
- src/shell/mod.rs 296
- src/shell/config_edit/mod.rs 284
- src/shell/input_bar.rs 277
- src/shell/navigator.rs 268
- src/shell/workspace.rs 263
- src/shell/start_pane.rs 263
- src/shell/model_pick/mod.rs 254

Binding constraints:
- src/shell/* is tarpaulin-excluded; EVERY new module must stay under src/shell/ or headless-unreachable orchestration becomes 100%-coverage debt.
- No module may be named state.rs (rules/locks-outside-state.yml:28 ignores **/state.rs).
- Ask 'why is this file long?': two responsibilities -> split; one thing done thoroughly -> LEAVE with justification; duplication -> delete, don't move.
- No behavior change.
- docs/DESIGN.md is owned by bl-43cd — do not touch; record §12 debt paths in a journal note on bl-52f8.