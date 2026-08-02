+++
title = "spend attribution (VISION §4.5): the yog-side join and price table"
created = 1785648911
updated = 1785649260
claimant = "Vellum"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
VERIFY docs/VISION.md §4.5 on main before editing; where this body and VISION disagree, VISION wins.

Cost per ball as a QUERY, single-source-of-truth: brazen counts tokens (never prices — its boundary); lernie commits usage per step (already folded by the budget inspector); balls tags every delivery [bl-id] and stays metric-free. yog owns the join and the price table.

Deliverables:
1. Price table as yog world config (severable: deleting it deletes a column, not code). No crate below yog ever learns a price.
2. The join: sum step usage over the agents tied to a ball, x prices. RULING (operator, 2026-08-02): the ball-to-conversation link is the goal stamp, which exists for board-started drones; a ball claimed mid-conversation records no conversation link (STORIES' honest limit) — ACCEPT workspace-granularity attribution for unstamped claims; do NOT invent a linkage fact.
3. Rendered as the V4 board's spend column and a per-conversation figure; rolls up the epic tree.
4. The ceiling, when armed: gates SPAWNS, never kills a running drone (early termination is the expensive failure).
5. DESIGN.md amendment in this delivery where the join/board surface (§3.5) is reframed.
Soft ordering: lands as a derivation first; becomes a §4.8 boundary query when the boundary exists.