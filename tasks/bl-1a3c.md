+++
title = "epic: the yog world, phase 1 — nested tool state, version gate, no-marks knob, escape hatches (DESIGN §16.6)"
created = 1784435198
updated = 1784435199
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-c68f"
on = "claim"

[[blockers]]
id = "bl-ea39"
on = "claim"
+++
Phase 1 of DESIGN §16 (the yog world): nested LERNIE_HOME/XDG_STATE_HOME/BRAZEN_CONFIG for all child tools, brazen creds/cache shared, lernie home seeded via lernie's own bootstrap verb (upstream lernie bl-6d83), shared balls store branch with a per-project no-marks knob, a phase-1-scoped version gate, and yog env/exec escape hatches. W7 (install-tools convenience) is recorded in §16.6 but deliberately unfiled — file only if fresh-machine friction is felt. Phase 2 (embedded crates, no shipped binaries, agent-tool shims) is §16.7, gated on upstream readiness (lernie bl-231c).