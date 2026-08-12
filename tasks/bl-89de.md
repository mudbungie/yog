+++
title = "beat: drive the unfold — arrow toggles children into the list, per-depth title edge holds, hidden-member focus auto-reveals, hover names the numbers"
created = 1786511695
updated = 1786511695
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
src/shell/acceptance/fixture.rs:95-99 already forks descent children).

Beats:
1. A collapsed parent row shows the subagent field (both numbers) and none of
   its children; clicking the arrow (or the spec's key on the selected row)
   paints each direct child as a row — the nameless chained child shows its
   terminal segment (the existing naming.rs beat's ladder), indented, elbow
   present.
2. Recursion: expanding a child reveals a grandchild; the collapsed child's
   badge speaks for its hidden subtree.
3. Alignment: within a depth, title left edges are equal regardless of
   attention/flight/verdict — extend bl-6b83's assertion to depth > 0.
4. Auto-reveal: walk ↑/↓ (walk.rs idiom) onto a hidden descendant — the list
   expands its ancestors and the focused row is painted.
5. Hover: the subagent field's hover states both numbers' meanings and the
   combo (the spelling scan should already enforce; add the explicit beat if
   it does not reach this control).

Also: if bl-2d45/bl-52c7-class staleness bites (fixtures aiming at moved
pixels), fix the beat, not the paint. The close gate runs the full suite —
ALL tests pass before close, no skips.