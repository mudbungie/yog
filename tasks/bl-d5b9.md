+++
title = "paint: the subagent field (▶/▼ + direct/total, words on hover) and the indented reply-elbow child rows in the conversation list"
created = 1786511695
updated = 1786511695
parent = "bl-fa82"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-27d9"
on = "claim"
+++
Subtask of the expander epic; the spec rules the seat, the nav ball supplies
visible_rows. Shell glue only — no derivation logic here. Verify cited paths
against HEAD first.

**src/shell/conv_list.rs** (217/250 — split first if this projects over the
shell cap; the spec named the seam):

- Replace the `({members})` weak label (:181-183) with the **subagent
  field** in the right-pinned group: ▶/▼ (reuse the jsonview disclosure
  vocabulary) + the two numbers, direct and total. It is an interactive
  control: hover MUST state what each number means, what clicking does, and
  the key combo the spec chose (the acceptance hover scan,
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
- **Keyboard:** the spec's binding on the selected row (expand/collapse);
  wire it in src/shell/keys.rs beside the roster walk. **Auto-reveal:** when
  roster_step lands focus on a member whose ancestor is collapsed, insert the
  ancestors into the expanded set (the spec's rule 4).

shell/* is tarpaulin-excluded but acceptance-tested — the hover scan, the
naming scan (no paint seat spells a raw agent id), and the geometry tests all
run in the close gate; make them pass, do not carve exceptions.