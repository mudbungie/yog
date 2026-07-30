+++
title = "notify attention clears the signal and leaves no fact behind: nothing renders notify_oid anywhere in the UI"
created = 1785374065
updated = 1785374065
priority = 4
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["bug"]
+++
Split (c) from bl-2194's design investigation. **Actionable now — it does not depend on how bl-2194's contract question is ruled.**

## The invariant, verbatim

DESIGN §6 (`docs/DESIGN.md:948-950`): *"Acknowledging it clears the **signal**, not the fact."*

That holds for four of the five §6 attention rules:
- **stopped** → the state badge carries the fact after the ack.
- **conflicted** → the "declined-transfer" label, `src/shell/workspace.rs:168-173`.
- **budget-exhausted** → the "budget-exhausted" label, same region.
- **mail** → the `✉n` accessory.

It is **false for notify.** Grep-verified across `src/shell` and `src/nav`: **nothing renders `notify_oid` at all.**

## The consequence

Jump to a notify-flagged conversation → arrival focuses it → `record_seen` stamps the notify oid → the flag clears → **and no surface anywhere says why you were sent there.** Notify is the one signal whose only carrier is the signal itself. The operator is delivered to a conversation that looks exactly like a conversation with nothing to say.

This is also a §11 glyph-doctrine failure of the same family the four sibling balls just fixed (bl-ae05 `7df46d7`, bl-b88e `14e723b`, bl-51cb, bl-9a01): the state has no carrier once its glyph is gone. Here it is worse — the glyph is not merely the *only* carrier, it is a *self-deleting* one.

## Scope

Give the notify fact a surface that survives its acknowledgement, consistent with how the other four rules already do it. The §11 badge-seat pattern (DESIGN §11, "The badge-seat pattern — how a failing seat is fixed (bl-ae05)") is the established shape: the words live with the glyph in the mapping function, the seat chooses inline-vs-hover.

Do NOT reach for a new ref, a new config knob, or a second notification store — the fact is already on disk as `refs/lernie/notify/<id>` and already read into the model. This is a rendering gap, not a data gap.

## Cross-refs
- **bl-2194** — the contract question (what does the strip answer). Independent of this; do not wait on it.
- Sibling split (b): ack-as-state, `src/app/focus.rs` — separate ball.