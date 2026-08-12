+++
title = "the conversation list unfolds: a right-pinned subagent field (direct/total + arrow) expands the descent as indented rows"
created = 1786511693
updated = 1786515529
claimant = "Fretwork"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-b9e3"
on = "claim"

[[blockers]]
id = "bl-720a"
on = "close"

[[blockers]]
id = "bl-27d9"
on = "close"

[[blockers]]
id = "bl-d5b9"
on = "close"

[[blockers]]
id = "bl-89de"
on = "close"
+++
**Operator ruling, 2026-08-11, verbatim:**

> "The subagent system needs a ux overhaul. I _think_ this is actually all in
> ux. Right now, subagents are hidden. The relationship exists, and we want to
> represent it, but it's always in tension. What I want to do is make each
> agent have a field on the right of it that indicates the number of
> subagents, which if clicked, expands the list; arrow pointing right
> normally, click on it to expand down. Two numbers; one for direct, one for
> total. Mousover should indicate what the numbers mean. Once expanded, you
> see them like any other agent. Subagents, recursively, indent and get the
> little chat-reply line. Handle this alongside bl-b9e3, the removal of the
> notification flag."

**Second operator ruling, 2026-08-11, verbatim (keyboard):**

> "up down walks the list, left right expands/collapses the list (including
> paging back up to the last level, if you hit left while on a child). don't
> automatically expand just from going down; skip to the next thing at the
> same level."

**The reframe that makes this small:** today's conversation list (§11 altitude
0) is "one row per root agent", each row aggregating its whole subtree
(`src/nav/convs/row.rs` — state badge, attention, flight, members are already
subtree folds). The general form: **every row is the subtree rooted at its
agent**, and the root-only list is just the all-collapsed case. Expansion
reveals a member's direct children as rows of the SAME anatomy, recursively —
no second row kind, no special child rendering path. Dissolves the special
case instead of adding one.

**The field (replaces the bare `({members})` count at
`src/shell/conv_list.rs:181-183`):** right-pinned, on every row whose agent
has subagents — an arrow (▶ collapsed / ▼ expanded, the crate's disclosure
vocabulary, `src/jsonview/mod.rs:39-41`) plus TWO numbers: **direct** children
and **total** descendants. Hover states what the numbers mean (operator
requirement, verbatim above). Zero subagents ⇒ no field, exactly like today's
`(N)`.

**The keyboard (second ruling):** ↑/↓ moves the selection through the
**visible** rows in list/paint order — a collapsed subtree is skipped, so
down from a collapsed parent lands on the next row at the same level; the
walk NEVER expands anything. → expands the selected row; ← collapses it, and
← on a child (or a leaf) pages up to its parent. This is a semantics change
to the current ↑/↓ roster walk (attention-ranked, every generation —
`src/attention/roster.rs:89-109`); the spec subtask amends §6/§11
accordingly and decides what a deliberate jump-to-attention does when its
target is hidden.

**Membership is §5.1 #8's strict descent-id rule** (`docs/DESIGN.md:1441`,
`git_tree::descent_order` / `children_of`) — NOT the loose prefix test
`actions::stop_children_offered` uses for the Stop menu seat. Total =
subtree members − 1; direct = `children_of` count.

**Interplay, in order:**
- **bl-b9e3 lands first** (this ball's claim is gated on it): same right-pinned
  group, same §11 anatomy paragraphs. After it, the row's attention paint is a
  bare right-pinned ⚑ — the expander must not reintroduce a left-prefix
  conditional or move the title's left edge (bl-6b83's beat must keep passing;
  child rows establish a per-depth left edge).
- **bl-5cf8** (where is the descent graph drawn): this ball seats the
  *provenance* tree (descent-id) at altitude 0. It does NOT draw the *context*
  edge (VISION V1.3) — bl-5cf8 stays open for that question; the spec subtask
  cross-references, nothing more.
- **Out of scope, untouched (operator: "I'm not really sure what to do about
  this" — so the fences hold):** altitude 1's compact descent tree
  (`src/shell/members.rs`) — after this lands it is a second rendering of the
  same membership; the spec SAYS so in the doc and files a follow-up ball for
  the decision, it does not widen this one. The boundary `conversations`
  answer (`src/boundary/reply/rows.rs`) stays root-rows; expansion state is
  viewport ephemera (§13.1), never a boundary fact and never a ui.json field.

Subtasks: spec (doc first, rules the rest), nav derivation, shell paint,
acceptance beats. Each verifies its cited paths against HEAD before editing
(ball bodies drift).

---

Correction to a conclusion reached under child bl-89de (recorded here because a closed ball has no task file to comment on): **DESIGN §11 layout rules 6, 7 and 8 do exist on main.** Rule 6 'What the remainder cannot show, it scrolls (bl-9551)'; rule 7 'A control's width is a share of its row, never a constant (bl-76f8)'; rule 8 'A strip of peers wraps; none of them is dropped (bl-b531)'; plus the rule 5 amendment ('The ceiling is a budget over the stack, not a cap on each member') and rule 1b ('the control wins the row', src/shell/row.rs::control_last). The finding that they were absent — 'ball-body drift, nothing owed' — was read against the unmerged epic ref work/bl-fa82, which branched 17 commits before that wave landed. Nothing was owed by that ball's own work, but the premise was false; a 'verified against the tree' note verified against a stale ref is worse than no note.

They did bite on merge. The conversation column truncates a title that does not fit (rule 1), and paint_probe::collect now returns laid-out glyphs rather than Galley::text (bl-bc06), so the epic's acceptance needles — the derivation's full display_name — stopped equalling the painted galley and six beats reddened. Judged for the landed rule: acceptance::unfold::drive::reads_as now matches a title by the head egui left on screen, guarded by one_title_each, which fails loudly if a head names more than one row. No assertion was loosened; both directions still bite (verified by mutation).
