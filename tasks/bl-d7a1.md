+++
title = "Paved-path spec: user-story ladder (docs/STORIES.md) + DESIGN amendments"
created = 1784524205
updated = 1784524205
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Session goal: zero-to-running as good as Codex — login + Enter in a text box. Deliverables: (1) docs/STORIES.md — escalating user-story ladder S0 stranger → S5 operator, each rung with integ-test enumeration and a no-burden-below check; (2) DESIGN.md amendments the stories force: W5 capability gate (probe verbs, not presence; gate the start path), error-surfacing invariant (no swallowed start errors; ops carries stderr), §8.3 login (render bz --login device flow), start::plan as the one planner (prepare() SSOT violation). Evidence: exploration wf_e81bb868 2026-07-19 — stale lernie kills every start at 'lernie prime' exit 2, swallowed to stderr; bare/path rungs + bootstrap mint unimplemented (Z2-Z4 open).