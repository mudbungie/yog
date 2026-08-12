+++
title = "beat: drive the unfold — arrow toggles children into the list, per-depth title edge holds, hidden-member focus auto-reveals, hover names the numbers"
created = 1786511695
updated = 1786511787
parent = "bl-fa82"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-d5b9"
on = "claim"
+++
Subtask of the expander epic — the acceptance surface for the paint ball, in
the bl-6b83 mold: assert through the real paint pipeline
(src/shell/acceptance/, Screen/press/painted; fixture World::add_child at
src/shell/acceptance/fixture.rs:95-99 already forks descent children). The
parent body carries both operator rulings verbatim.

Beats:
1. A collapsed parent row shows the subagent field (both numbers) and none of
   its children; clicking the arrow (or → on the selected row) paints each
   direct child as a row — the nameless chained child shows its terminal
   segment (the existing naming.rs beat's ladder), indented, elbow present.
2. Recursion: expanding a child reveals a grandchild; the collapsed child's
   badge speaks for its hidden subtree.
3. Alignment: within a depth, title left edges are equal regardless of
   attention/flight/verdict — extend bl-6b83's assertion to depth > 0.
4. The walk skips the hidden (the operator's ruling): with a collapsed
   parent selected, ↓ lands on the next SAME-LEVEL row and the expanded set
   is unchanged — the walk never expands. After →, ↓ enters the first child.
5. Paging up: ← on a child moves selection to its parent; ← again collapses
   the parent (or per the spec's exact wording); ← on an expanded parent
   collapses it.
6. Hover: the subagent field's hover states both numbers' meanings and the
   ←/→ combos (the spelling scan should already enforce; add the explicit
   beat if it does not reach this control).

Also: if bl-2d45/bl-52c7-class staleness bites (fixtures aiming at moved
pixels), fix the beat, not the paint. The close gate runs the full suite —
ALL tests pass before close, no skips.