+++
title = "Y2: xdg module — env-snapshot path folds with runtime-injected target_os"
created = 1784349553
updated = 1784349575
claimant = "filtered"
parent = "bl-4e66"
priority = 1
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
DESIGN.md §15 Y2. One module owning every path derivation from an injected env snapshot: balls state root, lernie config/data roots including the LERNIE_HOME collapse, brazen config path fold ($BRAZEN_CONFIG > XDG > ~/.config), brazen per-OS credentials/cache dirs with target_os passed as a runtime parameter (so the macOS branch is covered by Linux tarpaulin), yog data/state roots, and percent-decode (hand-rolled, ~25 lines). Table-driven tests for every fold and both OS branches. No env reads anywhere else in the crate. Files: src/xdg/mod.rs (~150).