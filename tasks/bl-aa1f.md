+++
title = "attention ack is a one-shot gesture where it should be a state: evidence landing while you are looking still raises the flag"
created = 1785374069
updated = 1785459749
claimant = "pedantic-attn"
priority = 4
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["bug"]
+++
Split (b) from bl-2194's design investigation.

## The defect

`focus_agent → record_seen` (`src/app/focus.rs:134-141, 174-178`) stamps the present evidence oids **once, at the focus transition**. It is a gesture, not a state.

So evidence that lands *while an agent is already focused* was not covered by the stamp that predates it: the conversation you are actively reading raises ⚑ at you, and stays raised until you click away and back. You are being told to look at the thing you are looking at.

## The fix, and why it is nearly free

**Ack-as-state:** while an agent is focused, stamp its present evidence oids continuously (per repaint) rather than once on entry. DESIGN §4.1's write discipline already makes this cost nothing — verbatim (`docs/DESIGN.md:748-750`): *"re-acknowledging a seen agent … costs no writes at all"*.

The contract it buys is crisp and worth stating in §6: **attention = evidence that arrived while you weren't looking.**

## Status against bl-2194

**Correct under either ruling**, so this does not wait on the contract question. But note the asymmetry:
- If bl-2194 rules **A** (exception channel, current rules kept), this is a real but marginal annoyance — wounds landing on a focused conversation are uncommon.
- If bl-2194 rules **B** (work queue — a conversation at rest at an unseen tip stirs the strip), this becomes **mandatory**: it would otherwise fire on every turn-end of the conversation you are actively driving. B is unlivable without it.

## Explicitly out of scope

- **The jump control acking its own destination is NOT a bug** and must not be "fixed" here. Arrival puts the conversation on screen, which is the definition of seen; the self-ack is what makes the `⏭` walk a triage that empties the queue (STORIES S6-T4).
- **Keyboard transit acking pass-through rows** (`src/shell/keys.rs:110-111 → roster_step → focus_agent`) is accepted, not fixed. Each row did render focused for a frame, and the only knob-free alternative is a dwell timer — a clock threshold with no §5.3 home. Mail clients with preview panes make the same trade.

## Cross-refs
- **bl-2194** — the contract ruling. Read its journal before starting; if B is chosen, this ball is a prerequisite for it.
- Sibling split (c): **bl-efa2**, the notify fact surface.