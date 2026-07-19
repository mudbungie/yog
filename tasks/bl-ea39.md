+++
title = "W2: world-env injection at every spawn"
created = 1784435199
updated = 1784437054
claimant = "filtered"
parent = "bl-1a3c"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-c68f"
on = "claim"

[[blockers]]
id = "bl-d3f6"
on = "claim"
+++
DESIGN §16.6 W2. Thread the composed world Env overrides into cli_outbound so every run/run_in/run_env/spawn_detached layers them — generalize the existing run_env seam into a standing world-env carried by the action layer, so the detached lernie prompt and every short verb run in the world, and an agent's own tool subprocesses inherit the nested $XDG_STATE_HOME (§16.4 phase-1 correctness: agents closing balls hit the world's clones/worktrees, not ambient ones). No scrub — overrides layer over the inherited environment. NOTE: gated on B5 (bl-d3f6) because both churn cli_outbound/actions; cli_outbound/mod.rs sits at 300/300 — split before adding lines. Files: src/cli_outbound/mod.rs (env-carrying), src/actions/* wiring. Gate as always.