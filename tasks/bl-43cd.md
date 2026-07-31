+++
title = "DESIGN.md rewrite: 3653 lines, two hot tables serialize parallel work, ~700 lines of completed plans"
created = 1785392865
updated = 1785460429
claimant = "Waddled"
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["design"]

[[blockers]]
id = "bl-2194"
on = "claim"
+++
Operator-raised, 2026-07-29: *"DESIGN.md should probably be decomposed; it's too big, and major detail sections can be broken out."*

Operator ruling, 2026-07-30: **this is a full rewrite, and a fable will take it.** The investigation below is evidence for that rewrite, not a binding plan — the earlier "decompose it" framing survives only as one option among several, and the analysis argues against it. Do not treat §"Recommendation" as settled; attack it.

## Investigation (pedantic-43cd, 2026-07-30) — read-only, no edits made

### Corrections to this ball's original premises

- DESIGN.md is 3653 lines; STORIES.md is 1007. **True.**
- No `docs/design/` convention exists in yog. The balls-repo precedent (`docs/design/bl-cdec-atomicity.md`) would be an import, not a convention to follow. **True.**
- **The three cited merge conflicts cannot be confirmed from git** — history has zero merge commits touching DESIGN.md (bl's gate lands linear). The five implicated commits (993ad02/bl-4305, 14e723b/bl-b88e, 6699c37/bl-9a01, 2759bdc/bl-51cb, 51939f8/bl-7e32) all touched §11 and three also touched §12, which is consistent with the claim but does not prove it. Not false — unverifiable beyond consistency.
- **This ball under-names the hotspot.** It frames §11's glyph audit table as the chokepoint. Empirically **§12's module table is hotter**: 37 of 68 all-time commits touch §12 vs 32 for §11, and 46 diff hunks land inside the module table itself. In the last 20 commits: §12 in 13, §11 in 12, every other section ≤3.

**The serialization point is not "the 3653-line file" — it is two tables totaling ~170 lines.**

### Line count per section

| § | title | lines | character |
|---|---|---|---|
| 0–2 | what / taxonomy / invariants | 134 | stable prose |
| 3 | named workspace | 532 | prose, 14 commits |
| 4 | durable state | 139 | stable |
| 5 | state inventory | 90 | numbered fact table, 20 commits |
| 6 | attention model | 68 | stable |
| 7 | watch / re-render | 194 | stable, 11 |
| 8 | action surface / argv | 273 | prose+lists, 20 |
| 9 | config write paths | 239 | stable, 5 |
| 10 | portability | 45 | stable |
| **11** | UI, glyph doctrine + audit table | **564** | **hot: 32/68 commits** |
| **12** | module map table | **109** | **hottest: 37/68 commits, 46 hunks in-table** |
| 13–14 | interpretations / rejections | 153 | stable |
| **15** | implementation epic M1–M6 | **376** | **completed plan — historical record** |
| **16** | yog world (16.6 "complete", 16.7 "landed") | **727** | ~480 lines record finished adoption work |

### Cross-reference load — renumbering is off the table

- **2405** `§N` citations in `src/*.rs`
- 626 internal to DESIGN.md
- 145 in STORIES.md
- 128 `bl-xxxx` amendment citations

Any rewrite must keep every one of these resolving. **Precedent for retiring a section without renumbering:** §13 already runs 13.0, 13.1, 13.3 — there is no 13.2, and nothing broke.

### Does a file split reduce conflicts? The evidence says no

Every observed collision is two agents inserting **adjacent lines in one table** — audit rows (bl-4305 vs bl-b88e), adjacent audit rows (bl-51cb vs bl-7e32), a paragraph after §11 step 2 (bl-9a01 vs bl-4305). That collides identically in `docs/design/glyph-doctrine.md`. A by-concern or by-altitude split puts both colliding agents in the *same new file* every time.

A split also costs the single-authority rule, AGENTS.md:7–9 verbatim: *"**`docs/DESIGN.md` is the architecture authority** — what yog *is*, its invariants, module map (§12), and the world substrate it composes (§16). When code and DESIGN disagree, one of them is a bug; never invent a third answer."* A decomposition that leaves an agent unsure which file rules has made things worse.

So the win a split buys is **navigability only**. Whether that is worth having is the operator's call, not the analysis's — see "Open question" below.

### Stale / completed bulk: ~600–850 lines

- **§15 (376 lines)** — completed epic. M2 marked "complete" in-text; M6 landed (ae80e99, 9125f3b, 67903fa are its wave). Git history is the archive. **50 src citations + 4 STORIES citations** ⇒ needs a tombstone stub, not silent deletion.
- **§16.6 (125 lines)** — header itself says "(complete; W5 and W7 retired at W13)". **31 src citations** ⇒ same treatment.
- **§16.7 (355 lines)** — §12 says "All three landed… bl-89a4 retired the last git pin". The work-plan portion (~200+ lines) is done. But at **84 src citations** this is the most-cited subsection in the file: keep the ruling, trim the schedule cautiously.
- **§12's responsibility cells** — several rows are paragraph-length restatements of doctrine living in the §-sections the cells themselves cite (the `src/shell/*` row is a ~200-word single line). This is both bulk and conflict surface: markdown tables are one line per row, so *any* two edits to one subsystem's row are a guaranteed conflict.

### Recommendation from the investigation — evidence, not a ruling

Do **not** split. Instead, pure subtraction:

1. **§12** — shrink landed modules' rows to `path | budget | one clause + § citation`; doctrine prose moves to (or already exists in) the owning §. Keep rows sorted by module path so inserts distribute rather than stack at subsystem boundaries.
2. **§11 audit table** — freeze it. Every row now reads "pass"; the 2026-07-28 audit campaign is over. The durable artifact is the doctrine + the badge-mapping pattern + `tests/glyph_coverage.rs` + the exhaustive-match badge tests. A new glyph seat is governed by the pattern and its tests, not by appending an audit row. (If the audit is to stay live: sort rows by seat key — same adjacency dilution as §12.)
3. **Retire §15 and §16.6 to git history**, each leaving a one-line tombstone ("§15 — implementation epic, complete; retired at bl-43cd, see git history") so all 50+31 code citations still resolve.

Net ~-450 lines immediately, ~-700 with §16.7 trimming — landing near ~2900.

**Case against this recommendation, stated honestly:** if the operator's real complaint is *reading* cost rather than *merge* cost, it under-delivers — 2900 lines is still a large file. And freezing the audit table trades a written per-seat record for test-enforced doctrine; a future audit campaign would have to reconstruct the seat list.

## Open question the rewrite must settle

**Is the complaint merge cost or reading cost?** The evidence answers only the first. If reading cost is the real driver, the volatility data above now identifies the only defensible seam — **hot §11+§12 vs the cold ~2980 lines**, not the by-concern seam this ball originally proposed — and a split becomes a deliberate second ruling made on evidence rather than on the file's line count.

## Constraints on any answer

- **Exactly one obvious entry point must remain** (AGENTS.md:7–9, quoted above).
- **No renumbering.** 2405 src citations. Retire-with-tombstone is the sanctioned move; §13.2's hole is the precedent.
- **Subtraction beats relocation** — per AGENTS.md, if bulk is stale or duplicated prose rather than needed detail, deleting beats moving.
- **Same-row / same-paragraph semantic conflicts are unsolvable by structure** and should not be chased: two agents ruling on one fact *should* serialize. The fill is bl sequencing (as this ball itself does with bl-2194).
- **Sequencing: must land after or rebase over bl-2194**, which edits §6/§11/§4.1. Colliding with it would re-enact this ball's own complaint.

## Adjacent findings to fold in or file separately

1. **STORIES.md:764 contradicts DESIGN §11.** STORIES says the strip shows *"totals per signal kind"*; §11's bl-e266 amendment rules the opposite — an aggregate total, the strip never itemizes kinds (*"the per-kind detail lives on the row badges, not the strip"*, DESIGN.md:1744-1747). A genuine doc-vs-doc drift; DESIGN is the authority, so STORIES is the bug.
2. **A durable principle worth adding** (AGENTS.md or §12's preamble), to stop the next append-heavy table recurring: *keyed tables stay sorted by key; completed plans retire to git history.*