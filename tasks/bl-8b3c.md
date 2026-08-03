+++
title = "turn-rollup aggregate states real token counts once the lernie pin carries committed usage"
created = 1785729853
updated = 1785731282
claimant = "rollup-teller"
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
## Operator ruling (2026-08-02)

'file the turn rollup ball. it's good telemetry.' — the bl-1f21 aggregate line (⚙ N inference calls · M tool calls · K thinking blocks) gains real token counts (e.g. '3150 thinking tokens') as soon as the committed transcript carries usage.

## Gated on (cross-repo — no needs edge expressible)

lernie ball filed 2026-08-02 ('commit usage telemetry into the transcript') must land AND yog's lernie pin must carry it. The operator is handling lernie release/integration; DO NOT claim this until the pin in Cargo.toml resolves to a lernie version whose committed messages/NNN-<model>.json carries usage — verify by inspecting a fresh world's transcript files, not by version arithmetic.

## What this ball is

- `src/transcript/parse.rs` reads the usage record when present (absence = today's behavior, the general path).
- The turn aggregate (src/transcript/rows/turns.rs) sums and states it: thinking/output tokens per the fields actually committed. Honesty rule stands: state only what the bytes carry, never estimate; a turn mixing usage-bearing and legacy entries states counts for what it has (decide and document the mixed-turn wording).
- DESIGN §11's 'the aggregate says only what the committed bytes carry' paragraph gains the token sentence.

## Discipline

Verify all paths against the tree at claim time (bl-1f21 landed 8cc8382; more may follow). Render tests both directions (usage present → tokens stated; absent → counts only; mixed turn).