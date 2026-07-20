+++
title = "Z3: start flow — generalize start::plan to §3.4's axes"
created = 1784523545
updated = 1784523881
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-173a"
on = "claim"
+++
DESIGN §3.4/§8.1 (as amended by bl-c195) / §15 M6 Z3. Two axes: target workspace x payload rung. Target resolution: the focused workspace; zero workspaces in the world -> mint + lernie new <root>/<name> (bootstrap = the empty general case); explicit + New workspace verb runs the same pair deliberately. Layer YOG_NAME=<name> on workspace-scoped spawns (§8). Ball rung inserts bl create / bl claim <id> --as <name> with resume-on-already-claimed convergence (claimed by a local name -> prompt into that workspace, no second mint). Per-rung composer prefills: identity preamble with the phase-1 interim stamp instruction ('You are <name>. Stamp every bl verb you run with --as <name>.' — load-bearing until W9's shim injects --as $YOG_NAME, then deleted); + target preamble for path; + ball title/body/worktree preamble for ball. Per-rung driver cwd: ~ / the path / the work worktree. Files: src/start/mod.rs, shell start-pane wiring.