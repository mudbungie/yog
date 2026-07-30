+++
title = "DESIGN.md is 3653 lines and is now a merge-conflict chokepoint: decompose the major detail sections"
created = 1785392865
updated = 1785392865
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["design"]
+++
Operator-raised, 2026-07-29: *"DESIGN.md should probably be decomposed; it's too big, and major detail sections can be broken out."*

## The evidence

`docs/DESIGN.md` is **3653 lines** (STORIES.md is 1007). It is the architecture authority for every ball, so nearly every implementation ball edits it.

In a single 18-ball session it produced **three genuine merge conflicts**, all in or beside §11's glyph-doctrine audit table:
- bl-4305 vs bl-b88e — both appended an audit-table row.
- bl-9a01 vs bl-4305 — both appended a paragraph after §11 step 2.
- bl-51cb vs bl-7e32/bl-9a01 — adjacent audit rows.

Each was resolved correctly by keeping both sides, but every one cost a hand-merge plus a full re-gate (~10 min). The file is a serialization point on parallel work: **any two agents touching the same doctrine section collide by construction**, even when their code touches nothing in common.

Note the 300-line cap does not apply (AGENTS.md exempts md/yaml), so this is not a rule violation — it is a working-practice problem, which is why it needs a deliberate design ruling rather than a mechanical split.

## What to decide

The deliverable is a **ruling plus the decomposition itself**, not a proposal. Attack it before committing:

1. **What is the seam?** Candidates, none endorsed: by concern (the §5.1 fact table, §11 the UI doctrine, §16 the nested world, §12 the module map); by altitude (invariants vs detail); by volatility (the append-heavy tables that actually collide vs the stable prose). The audit table is the demonstrated hot spot — if one section causes most conflicts, that is evidence about where the seam is.
2. **What must NOT split.** yog's AGENTS.md says DESIGN.md is *the* architecture authority and *"when code and DESIGN disagree, one of them is a bug; never invent a third answer."* A decomposition that leaves an agent unsure which file rules has made things worse, not better. There must remain exactly one obvious entry point, and every cross-reference (`§7.1`, `§16.2`, and the ~dozens of `bl-xxxx` amendment citations) must still resolve. Section numbering is load-bearing across the whole backlog — balls cite `§3.3`, `§8.1`, `§11` by number, so renumbering breaks filed work.
3. **Does splitting actually reduce conflicts,** or just relocate them? If two agents both append to the audit table, they collide whether it lives in DESIGN.md or `docs/design/glyph-doctrine.md`. Say honestly whether the win is conflict reduction, navigability, or both — and if the append-collision is the real cost, consider whether the fix is structural (a table that appends without adjacency) rather than a file split.
4. **Precedent exists**: `docs/design/bl-cdec-atomicity.md` in the balls repo is a broken-out design doc. Check whether yog already has a `docs/design/` convention to follow before inventing one.

Per AGENTS.md: minimalism and subtraction — if part of the bulk is stale or duplicated prose rather than needed detail, deleting beats moving. A split that preserves 3653 lines across five files has not reduced anything.

## Cross-refs
- **bl-2194** also edits §5.1/§6/§11 and STORIES S6; sequence deliberately against it.
- STORIES.md:764 currently contradicts DESIGN §11 on the attention strip (per-kind vs aggregate totals) — an example of the two-doc drift this ball should not multiply.