+++
title = "the conversation list unfolds: a right-pinned subagent field (direct/total + arrow) expands the descent as indented rows"
created = 1786511693
updated = 1786511693
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-b9e3"
on = "claim"
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
- **Out of scope, untouched:** altitude 1's compact descent tree
  (`src/shell/members.rs`) — after this lands it is a second rendering of the
  same membership; if the spec finds it indicted, SAY so in the doc and file a
  follow-up ball, do not widen this one. The boundary `conversations` answer
  (`src/boundary/reply/rows.rs`) stays root-rows; expansion state is viewport
  ephemera (§13.1), never a boundary fact and never a ui.json field.

Subtasks: spec (doc first, rules the rest), nav derivation, shell paint,
acceptance beats. Each verifies its cited paths against HEAD before editing
(ball bodies drift).