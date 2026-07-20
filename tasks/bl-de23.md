+++
title = "Z3: start ladder — generalize start::plan to three rungs"
created = 1784523545
updated = 1784523545
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-173a"
on = "claim"
+++
DESIGN §3.4/§8.1 / §15 M6 Z3. Trunk: mint + lernie new <root>/<name> + composer + detached prompt. Ball rung inserts bl create / bl claim <id> --as <name>, with resume-on-already-claimed convergence (claimed by a local name -> prompt into that workspace, no second mint). Per-rung composer prefills: identity preamble always ('You are <name>. Stamp every bl verb with --as <name>.'); + target preamble for path rung; + ball title/body/worktree preamble for ball rung. Per-rung driver cwd: ~ / the path / the work worktree. Files: src/start/mod.rs, shell start-pane wiring.