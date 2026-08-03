+++
title = "the eye must tell user input, model response, and other inbox-item kinds apart at a glance — one visual role language across transcript and pending queue"
created = 1785733748
updated = 1785734580
claimant = "role-tinter"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator request 2026-08-03, verbatim: 'It needs to be easier to visually discern user input and llm response, and other forms of inbox item.'

The transcript renders user entries, model turns, and the other inbox-borne kinds (inter-agent messages, scan-flushed deposits, epitaphs — enumerate what the parse actually distinguishes, from the bytes) with too little visual separation. Give each ROLE a consistent visual identity readable at a glance without reading content: e.g. distinct accent treatment (edge stripe, background tint, or glyph — pick ONE mechanism and apply it uniformly, not a different trick per kind), derived from the existing theme hues (no new RGB constants if the theme's named hues cover it; both light and dark must read). User input should be instantly separable from model output; third-party inbox items (messages from other agents, etc.) form a third visual family, differentiated within it only if the parse genuinely distinguishes kinds.

Scope discipline:
- One language, two seats: the transcript rows AND the composer's pending-inbox queue (bl-a119, landing now) must speak the same role identity — a message from another agent looks the same pending as it does delivered.
- Derivation: the role comes from what the committed bytes already say (entry kind/author), never inferred from content.
- Honesty: styling only — no reordering, no relabeling, no synthesized headers beyond what rows already state.
- The crossing lines (bl-95a9, 417c191) and turn rollups keep their own seats — don't restyle them, just ensure the new role treatment doesn't fight them.
- Amend DESIGN §11 where the row visual contract lives; hover strings for any new visual element per the discoverability invariant.
Verify all paths against the tree at claim time — bl-a119 will have just moved the composer surface.