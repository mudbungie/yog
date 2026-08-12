+++
title = "the conversation row's ⚑N: delete the count, move the flag to the right edge"
created = 1786511500
updated = 1786511517
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-5c64"
on = "close"

[[blockers]]
id = "bl-85d0"
on = "close"
+++
**Operator ruling, 2026-08-11, verbatim:**

> "take it out. from the spec and the implementation, one master ball with subs.
> I'm okay with just the flag, at the right side, not the left (it makes the
> list not align)."

The conversation list's rows carry `⚑N` — an attention count over the
conversation's subtree, painted in the row's **left** prefix group
(`src/shell/conv_list.rs:149`):

```rust
if row.attention > 0 {
    ui.colored_label(theme::BRAZEN, format!("⚑{}", row.attention));
}
```

Two faults, one ruled and one structural.

**1. It breaks the column (the operator's complaint).** The row is laid out
prefix-first: state badge, flight chip, `?`, `⚑N`, verdict badge — and only
then the right-pinned metadata and the title. Every conditional prefix shifts
the title's left edge, so a list where some rows are flagged and some are not
has no readable name column. The flag belongs in the trailing metadata group
that already exists (`egui::Layout::right_to_left` at `conv_list.rs:175`,
holding the ball badge, the `(N)` member count and the age).

**2. The count is a second, lossier encoding of a fact the row already
carries.** DESIGN §6:1570 (bl-efa2) requires it:

> **Every signal's fact has a carrier the ack cannot reach (bl-efa2).**
> "Acknowledging clears the signal, not the fact" only holds where the fact is
> *rendered* somewhere the watermark does not touch. Rule 2's carrier is the
> state badge; rule 5's is the `✉n` accessory and the Inbox tab; rules 1, 3, 4
> and 6 are the agent's `refs/lernie/*` **marks**.

So the row already wears the state badge, the `✉n`, and the marks. `⚑N` restates
them as a number that says less. Single source of truth: one fact, one seat.

**What the count is FOR, and where it keeps its seat.** DESIGN §6:1617 defends
the number — *"the number is a queue depth, and a depth that stays high is
itself the information — you are over-subscribed"* — but that argument is about
the **strip**, the one global "how many conversations are waiting on you". §6
grants the row's count in half a sentence (§6:1591) with no argument of its own:

> **The conversation list is no longer one of these groups** — since bl-cad5 it
> sorts by recency alone (§11); attention there is a badge and a count, not a
> rank.

**Out of scope — these keep their counts, untouched:**
- the attention strip's total (`⚑ N need attention`, `top_bar.rs:49`, §11
  altitude 0)
- the workspace tab badges (`top_bar.rs:114/169/179`)
- `ConvRow::attention: usize` itself (`src/nav/convs/row.rs:49`) and the
  headless `conversations` answer's `"attention"` field
  (`src/boundary/reply/rows.rs:34,54`) — a machine surface has no width bind
  and no alignment to lose, so the derivation stays one `usize` and only the
  *paint* drops to a boolean read. Deleting the field would cost the boundary a
  fact for nothing.

**The end state:** the conversation row paints a bare `⚑` — no number — in the
right-hand metadata group, with its words on hover (the §11 glyph doctrine: a
dense repeating seat hovers rather than states; the current `⚑N` has **no hover
text at all**), and the title's left edge is at the same x on every row that
differs only by attention.