+++
title = "V2 Counterfactualist: N>=1 history attempts from any notch"
created = 1785719120
updated = 1785730548
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-98da"
on = "claim"

[[blockers]]
id = "bl-5134"
on = "claim"
+++
VERIFY docs/VISION.md and docs/DESIGN.md on main before editing; VISION wins over this body.

Implement VISION §5 V2 over agent history. Needs V1's pinned-notch rail and lernie's ref/config verbs after their releases and yog pin bumps.

One attempt and a parallel cohort use one path:

- Fork from here dispatches from the pinned agent ref.
- Fire-time controls expose model, config branch, and skills.
- N >= 1 repeats that dispatch with per-attempt overrides; N > 1 is an alternative candidate cohort.
- Rows render state, terminal response, usage, and common ancestry side by side.
- Cohort membership and provenance derive from refs/ancestry and committed execution facts; add no fan registry or winner field.

This task delivers only the read-only/context surface. Do not expose project-mutating attempts: they require bl-2b8c's project contract and its resulting implementation tasks. No capability theater and no new fan verb.

Graduates as S11.