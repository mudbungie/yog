+++
title = "V4.2 armed-loop facts: cap, count, last/next tick, and every spawn/reap as an ops row"
created = 1785824553
updated = 1785893224
claimant = "loopwright"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-0cea"
on = "claim"

[[blockers]]
id = "bl-765d"
on = "claim"
+++
VISION §5 V4 item 2, split out of bl-9dd4 (which landed the board itself: the four derived columns, gates, drone rows, spend column and epic rollup — DESIGN §11's board paragraph, STORIES S13).

The board renders no loop facts today and the reason is recorded, not deferred silently: THERE IS NO ARMED LOOP. bl-3381/a94ce81 shipped the WATCHER clock only (src/app/cadence.rs, cadence.yaml, DESIGN §7.2) — verify that before building on any claim to the contrary. V4's own precondition holds fleet mode shut until a drone has a mechanical isolated project target (bl-2b8c, closed 2026-08-04) AND its tools have an explicit noninteractive capability policy (bl-0cea, still open — this ball's blocker). V4's burden check rules the interim, verbatim: 'unarmed, the board is today's balls section'.

What this rung owes when bl-0cea lands:
- Arming is a gesture: a boundary Action variant per workspace with its headless spelling (DESIGN §8.5), recorded in cadence.yaml beside the watcher entry — the same arm-is-the-explicit-action pattern V6 uses. I7 is preserved by making the arm the explicit user action.
- The loop renders as FACTS on the board: cap, current count, last tick, next tick — every one derived, none stored.
- Every spawn and reap is an ops row (DESIGN §4.2), and a reap reason is the COMPARISON itself ('lease expired 14m ago'), never a diagnosis. The loop spawns and reaps; it never diagnoses.
- The spend ceiling renders where it will bind — on the next spawn (bl-56d5 owns the gate itself).
- Unarmed must stay exactly today's board: no chip, no rows, no calls.