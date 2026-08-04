+++
title = "composer send-and-interrupt: Ctrl+Enter (and a button) interrupts the agent and triggers on the new message"
created = 1785650733
updated = 1785824290
claimant = "interrupter"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator ruling (2026-08-01, codex-comparison follow-up), verbatim: "this is a good feature. obviously a knob. it seems pretty easy to implement, I think? just an interrupt into brazen and then a trigger. both ways have value. probably enter: send, ctrl+enter: send and interrupt? and a button to do the same. really, the key point is, 'interrupt and trigger', which just happens to also be a send control."

## What
Enter keeps today's semantics (deposit; a busy agent sees it at its next step boundary). Ctrl+Enter — and an equivalent button by the composer — is send-and-interrupt: stop the in-flight work, deposit the message, and the deposit's driver-start is the trigger (lernie's existing law: a deposit into a quiescent agent starts a driver — no new verb).

## Mechanism, verify before building
The spellable-today composition is `lernie stop <ws> <agent>` then `lernie message ...`. Verify what stop's SIGTERM (process-group, 5s flush deadline) does to an in-flight step — a mid-tool-call stop may discard uncommitted tool work; the operator's instinct is finer: "just an interrupt into brazen and then a trigger", i.e. interrupt the model call, keep the step machinery clean. If stop is too blunt, the refinement is an upstream lernie ask (interrupt-current-attempt), filed separately when proven needed — do not build a yog-side workaround.

## Discipline
- The gesture is a control-boundary action variant first, widget second (bl-8aab landed; new gestures land as variants).
- Keybinding rides the keymap table; combo rules (§ keymap rule 3) reviewed — this combo DOES fire verbs at the selection, so it needs its own ruling or an exemption recorded in DESIGN.
- Ops trail: two rows (stop, message) or one composite — decide; the trail must not hide that an interrupt fired.