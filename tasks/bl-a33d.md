+++
title = "composer send-and-interrupt: Ctrl+Enter (and a button) interrupts the agent and triggers on the new message"
created = 1785650733
updated = 1785824346
claimant = "interrupter"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator ruling (2026-08-01, codex-comparison follow-up), verbatim: "this is a good feature. obviously a knob. it seems pretty easy to implement, I think? just an interrupt into brazen and then a trigger. both ways have value. probably enter: send, ctrl+enter: send and interrupt? and a button to do the same. really, the key point is, 'interrupt and trigger', which just happens to also be a send control."

## What
Enter keeps today's semantics (deposit; a busy agent sees it at its next step boundary). Ctrl+Enter — and an equivalent button by the composer — is send-and-interrupt: stop the in-flight work, deposit the message, and the deposit's driver-start is the trigger (lernie's existing law: a deposit into a quiescent agent starts a driver — no new verb).

## BLOCKED on lernie bl-b98d — the mechanism does not exist yet

Verified 2026-08-03 against the pinned lernie 0.0.6 source
(`~/.cargo/registry/src/*/lernie-0.0.6/`). **`lernie stop` is too blunt to
build this on**, and the failure is worse than losing tool work:

- **Stop in the model-call window is clean.** `run_exchange` breaks at
  `src/prompt/dispatch/mod.rs:195`, *before* `transcript::commit_assistant`
  (line 215). No assistant entry lands, the transcript tail stays user-side, and
  a later deposit resumes at `Warrant::ModelCallDue`. This half works today.
- **Stop in the tool window wedges the agent.** The assistant entry — `tool_use`
  blocks included — is committed at line 215, *before* `run_tool_calls` at line
  249. The §2.9 group SIGTERM fells the in-flight tool; `tool_step` returns
  `ToolWindow::Stopped` (`tool_step.rs:222-225`) and the branch settles with an
  **unpaired `tool_use` tail**. Nothing settles the window. The next
  `lernie advance` — which is exactly what our deposit's driver-start runs —
  returns `Error::UnpairedToolUse` (`advance.rs:192`): *"tool side effects are
  not replayable, so this is declined (§6). Recover by fork-from-history."*

So the gesture built on `stop` today would, whenever the agent happened to be
inside a tool call, deposit the message and then **permanently brick the
agent** — the trigger it depends on is the very call that declines. Filed
upstream as lernie **bl-b98d** (the stopped exit should settle its own tool
window with in-band `is_error` `tool_result`s, the same shape `tool_step`'s
`refusal` already commits). Nothing was built yog-side; a workaround here would
be a yog-side guess at lernie's step state, which is a race, not a mechanism.

Resume when bl-b98d lands and the yog pin bumps.

## Decisions already taken (do not re-litigate)

**Keymap: no rule-3 exemption is needed, because Ctrl+Enter is not in the key
table at all.** §11's rule 3 ("a combo may repaint or create; it may never fire
a verb at the selection") governs `src/keymap`'s pure table. The composer's
Enter family never reaches it — §11 already records *"Enter belongs to the box,
not the table… the composer's Send is the same ownership on its multiline box,
whose return key is Shift+Enter (bl-4515)"*. Ctrl+Enter is the third member of
that box-owned family, beside Enter (send) and Shift+Enter (newline). Rule 3's
safety property holds a fortiori anyway: the hazard it guards is a combo firing
at a selection the typing operator is not looking at, and here the target *is*
the composer's own addressee — the thing the operator is looking at. Record the
row in §11 beside the bl-4515 line when the gesture lands; do not add an
exemption clause to rule 3.

**Ops trail: two rows, one variant.** §4.2 rows are "everything that mutates a
substrate", and this gesture mutates twice — the interrupt and the deposit —
with independently observable outcomes (the interrupt can fail while the message
lands). A composite row would hide that an interrupt fired, which the ruling
forbids. So: **one `boundary::Action` variant** (one gesture, per §8.5 — new
gestures land as variants first, bl-8aab), **two ops rows** written by its
executor.

**Shape:** Action variant first, widget second. The variant needs its
`boundary::codec` envelope arm and its `boundary::line::spell`/`parse` arm
(both are exhaustive — it will not compile without them) plus a
`boundary::help::TABLE` page; the line spelling takes the message as a verbatim
tail, like `/message` (§8.5 "a message's content… is the whole tail, taken
verbatim"). The button beside Send is the widget half — the operator's ruling
asks for it explicitly, so bl-8aab's "no new control" does not apply here.

Coordinate with bl-3f46 (`wave-two`), which is adding the config family to the
same Action enum, codec and DESIGN §8.5.