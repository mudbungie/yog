+++
title = "paint: the subagent field (▶/▼ + direct/total, words on hover) and the indented reply-elbow child rows in the conversation list"
created = 1786511695
updated = 1786511786
parent = "bl-fa82"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-27d9"
on = "claim"
+++
Subtask of the expander epic; the spec rules the seat, the nav ball supplies
visible_rows. Shell glue only — no derivation logic here. Verify cited paths
against HEAD first; the parent body carries both operator rulings verbatim.

**src/shell/conv_list.rs** (217/250 — split first if this projects over the
shell cap; the spec named the seam):

- Replace the `({members})` weak label (:181-183) with the **subagent
  field** in the right-pinned group: ▶/▼ (reuse the jsonview disclosure
  vocabulary) + the two numbers, direct and total. It is an interactive
  control: hover MUST state what each number means, what clicking does, and
  the ←/→ combos per the spec (the acceptance hover scan,
  src/shell/acceptance/hover/mod.rs:69-101, and the spelling test both bite).
  Click toggles the row's id in the ShellState expanded set. Absent when the
  agent has no descent children.
- Iterate the nav ball's visible rows instead of root rows — both flat and
  by-ball paths (conversations(), group loop). Child rows go through the SAME
  conversation_row fn: indent per depth plus the spec's reply-elbow glyph
  ahead of the prefix group, so the title edge is consistent per depth and
  the prefix group grows no conditional (bl-b9e3's rule, bl-6b83's beat stays
  green).
- Clicking a child row focuses THAT member (the §6 acknowledgement gesture —
  same semantics the altitude-1 member rows have, src/shell/members.rs);
  right-click menu: the Stop/Flush seat targets that agent. Check
  focus::conversation vs member-focus semantics before wiring — do not aim
  the composer at a root the click did not name.
- **Expanded set:** new field on ShellState (src/shell/ram.rs), HashSet of
  agent ids, ephemera — never ui.json.
- **Keyboard (the operator's second ruling, spec has the amended doc):**
  - ↑/↓ walks the VISIBLE list rows in paint order — rewire the walk for
    this seat off the visible-rows derivation, not the attention-ranked
    roster (src/shell/keys.rs:141-142 → focus::roster_step →
    src/attention/roster.rs). A collapsed subtree is skipped (down from a
    collapsed parent = next same-level row). The walk never mutates the
    expanded set.
  - → expands the selected row; ← collapses, and ← on a child/leaf moves
    selection to its parent. Wire per the spec's doctrine amendment; update
    the row hover text that currently says "↑ / ↓ walks the roster onto it"
    (conv_list.rs:186-190) to the spec's wording.
  - Whatever the spec decided for jump-to-attention vs hidden targets, wire
    exactly that — nothing more.

shell/* is tarpaulin-excluded but acceptance-tested — the hover scan, the
naming scan (no paint seat spells a raw agent id), and the geometry tests all
run in the close gate; make them pass, do not carve exceptions.