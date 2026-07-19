+++
title = "epic: Rust Bootstrap v3 adoption — pinned toolchain, deny.toml, ast-grep rules, panic-free prod, owned signatures, pedantic gate"
created = 1784433623
updated = 1784437110
claimant = "filtered"
priority = 1
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-da37"
on = "claim"

[[blockers]]
id = "bl-cbf9"
on = "claim"

[[blockers]]
id = "bl-c5d8"
on = "claim"

[[blockers]]
id = "bl-4bb4"
on = "claim"

[[blockers]]
id = "bl-d3f6"
on = "claim"

[[blockers]]
id = "bl-a47a"
on = "claim"

[[blockers]]
id = "bl-eafd"
on = "claim"
+++
COMPLETE. B1-B7 all delivered: pinned toolchain 1.95.0 + deny.toml (B1); unsafe reduced to one confined SIGTERM site + first ast-grep rule (B2); locks chokepointed in state.rs, zero Rc/RefCell repo-wide (B3); rules 2/9 owned signatures + pub(crate) boundary (B4); rule 1 zero named lifetimes (B4b/bl-ca57); panic-free prod + restriction lints + assert/suppression rules (B5); pedantic=deny with 13-entry sanctioned manifest allow-list (B6); AGENTS.md + full gate wiring, CI green both platforms (B7). Eight ast-grep rules live with two-direction fixtures audit. Surfaced skips recorded in DESIGN §12.1 and AGENTS.md: workspace split, musl, forbid-vs-confine unsafe, anyhow, pre-commit-framework/nextest/bacon.