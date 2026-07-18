+++
title = "Rebrand lernie-ui-egui -> yog: crate, binary, README, Makefile"
created = 1784348197
updated = 1784349044
claimant = "filtered"
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
The package is now named yog (Yog-Sothoth: knower/guardian of the gate, manifests as a congeries of iridescent globes — balls through gates). The name is reserved on crates.io (yog 0.0.0 placeholder, published 2026-07-17) and the repo already lives at ~/dev/yog. This ball is the in-repo rebrand:

- Cargo.toml: name -> yog; decide publish story (placeholder 0.0.0 is on crates.io; first real publish supersedes it — bump past 0.0.0)
- [[bin]]/binary name -> yog (crate = binary = entrypoint; no abbreviation)
- src/main.rs clap name/about; any lernie-ui-egui self-references in code/comments (e.g. tests/pluggability.rs)
- README: title + framing (keep the lernie/balls composition story), Running section binary name
- Makefile: install/uninstall/run target binary name
- .githooks header comment

Out of scope, filed separately in lernie: README pointer there still says "lernie-ui-egui".