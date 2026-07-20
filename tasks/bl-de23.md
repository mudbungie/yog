+++
title = "Z3: start flow — generalize start::plan to §3.4's axes"
created = 1784523545
updated = 1784525446
claimant = "damson"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-173a"
on = "claim"
+++
DESIGN §3.4/§8.1 (as amended by bl-c195 AND bl-d7a1) / §15 M6 Z3 / STORIES S0-T1, S0-T3 (abort half), S1-T1, S2-T1, S3-T1..T4, INV-1. Two axes: target workspace x payload rung. Target resolution: the focused workspace; zero workspaces -> mint + lernie new <root>/<name> (bootstrap = the empty general case); explicit + New workspace verb runs the same pair deliberately. Layer YOG_NAME=<name> on workspace-scoped spawns (§8). Ball rung inserts bl create / bl claim <id> --as <name> with resume-on-already-claimed convergence. Composer: harness-stamped identity preamble with PRE-MINT PREVIEW (§3.3 as amended by bl-d7a1 — the greyed 'You are <name>.' line renders pre-submit from a pure read; re-derived at fire; phase-1 interim stamp instruction 'Stamp every bl verb you run with --as <name>.' until W9); + target preamble for path; + ball title/body/worktree preamble for ball. Per-rung driver cwd: ~ / the path / the work worktree. TWO STANDING DEFECTS DIE HERE (bl-d7a1 amendment): (1) prepare() re-implements the sequence while start::plan sits dead — §8.1 mandates planner as the one source, executor runs its output; (2) shipped step order ran bl create/claim BEFORE the seed — amended §8.1 order is seed -> lernie new -> bl mutations (the orphaned-claim wound; bl-e5fe in the nested world is the standing evidence). Story tests S0-T1/S0-T3-abort/S1-T1/S2-T1/S3-T1..T4/INV-1 land red-first in this worktree, green at close. Files: src/start/mod.rs, src/start/exec.rs, shell start-pane wiring.