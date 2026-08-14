+++
title = "README publication, reset, state, naming and installation claims contradict the code and the workflows"
created = 1786677244
updated = 1786677244
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["publication"]

[[blockers]]
id = "bl-2368"
on = "claim"
+++
Source: publication audit follow-up 2026-08-13 (item 6), snapshot yog `e758814`.

Every one of these is README (or DESIGN) text that the code or workflows
contradict. Verify each against the tree, then fix the DOC or the CODE —
whichever is wrong — and say which you chose.

- `rm -rf $XDG_DATA_HOME/yog` expands to `/yog` when the optional variable is
  unset, and it is unquoted. The code's fallback is `$HOME/.local/share/yog`.
  Prefer a product-owned reset command with a preview; failing that, document
  the resolved, quoted path and the exact deletion scope.
- "The only durable, yog-owned state is `ui.json` ... and `ops.jsonl`" omits
  `cadence.yaml`, monitor policy, and detached stderr sinks.
- The README claims a workspace uses a "minted name"; workspace names are
  chosen, or the fixed `home`. Conversation names are the minted ones.
- "No substrate needs installing" contradicts the install receipt:
  `note: the window drives the 'lernie' binary; install it from the lernie
  repo.`
- "Publishing ... [is] never a side effect of CI or delivery" contradicts the
  automatic successful-CI release workflow.
- "DESIGN ... is kept in sync with the code" is false while DESIGN's module map
  and its local-versus-lernie mint statements disagree with source.

Gated behind the personal-material scrub so the two do not conflict on
`docs/DESIGN.md`.