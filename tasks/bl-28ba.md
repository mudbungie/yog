+++
title = "consecutive fires mint sibling names (recite-a, recite-b, recite-c): mint_seed never re-rolls after fire and the collision walk clusters on the first word"
created = 1785649444
updated = 1785649444
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
## Operator complaint (2026-08-01, verbatim)

'I just opened three new agents, and they're all called recite- something. What gives?'

## Root cause (verified; re-verify against the tree before editing)

Three facts compose into the observed clustering:

1. `StartState::mint_seed` (src/shell/ram.rs:~36,~179) is seeded from entropy ONCE at shell construction — 'held stable across frames so the composer's preview predicts the name each frame and at fire'. Nothing re-rolls it after a fire.
2. Every mint (preview and fire alike, src/shell/fire.rs:~84, start_pane.rs, input_bar.rs) builds a fresh `SplitMix64::from_seed(mint_seed)` and takes ONE draw (src/names/mod.rs `mint_from`) → the SAME start index into the pair pool on every fire of the session.
3. Collision retry is a linear forward walk from that index, and `pair` is first-word-major (`idx / (n-1)` picks word one) — so consecutive pool slots share the first word for n−1 slots.

Fire 1 mints pool[start] = recite-a. Fire 2: same start, occupied → pool[start+1] = recite-b. Fire 3 → recite-c. The occupied-set uniqueness guarantee works; the NAMES are unique — but they're siblings, and the operator reads the fleet by name diversity.

## Fix (minimal — the mint stays a pure function; the seed lifecycle is the caller's bug)

Re-roll `mint_seed = entropy_seed()` after every successful fire (wherever the fire consumes `pending` / resets the composer — put it at the one point the old prediction dies, so the invariant becomes 'a seed lives exactly as long as the prediction it backs'). Preview semantics stay correct: after fire, the composer predicts the NEXT name from the fresh seed, and preview/fire still agree frame-to-frame between fires.

Do NOT change the mint's walk-retry design without cause — 'no retry budget, no probabilistic termination' is deliberate (src/names/mod.rs header). With fresh seeds per fire, a collision (random start landing exactly on an occupied slot with pool = n·(n−1)) is vanishingly rare, so the walk's first-word clustering stops mattering. If you find a second consumer of mint_seed whose stability matters across fires, stop and record it in the ball rather than guessing.

## Discipline

Test both directions: two consecutive fires in one workspace yield different seeds (inject/observe the re-roll — entropy itself isn't assertable, the re-roll is); the preview-fire agreement within one pending draft still holds. Check DESIGN §3.3's mint wording; amend if it pins the seed's lifetime. Note: agents ack-clear (src/shell/mod.rs, activity.rs, opslog) and turn-rollup (src/transcript) are working concurrently — your area is shell/ram.rs + fire.rs; fold conflicts honestly if a close lands under you.