+++
title = "implement the alignment monitor: tier-0 check + ladder per VISION §4.9"
created = 1785718992
updated = 1785731024
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["safety"]
+++
The design ruling landed with bl-af1a (VISION §4.9 + story rung V6 'Invigilator'), amended by bl-156b (the responder hook + anti-reinvention guardrail). This ball is the deferred implementation — read §4.9 and V6 first; they are the authority. Verify every premise against the then-current tree before editing (boundary surface, cadence.yaml schema, embedded brazen adapter, world tool shims).

Scope, per the ruling:

1. **Arming**: a boundary Action variant per workspace (headless spelling included — §4.8 compile gate), recorded as the cadence.yaml monitor entry (model, thresholds, verdict→response wiring). Unarmed = mechanism absent; severability is deleting the entry.
2. **The check**: bounded tool-less cheap-model call via the embedded brazen adapter; request = goal.md verbatim + transcript delta since last-checked sha + standing verdict; response = aligned|drifting|diverged + one-sentence reason. Level-triggered off the step spine by the clock: check only when the branch tip moved. Off-thread, never the frame. ANTI-REINVENTION LAW (§4.9): the check never grows tools, retry machinery, or a transcript — retry is the level-trigger (failed check leaves last-checked behind the tip; next tick re-fires), audit is the ops row, any response needing a decision is a dispatch.
3. **The ops row**: every check appends one ops.jsonl row (sha, verdict, reason, usage) — audit trail, level-trigger memory, and §4.5 tuning dataset in one. ≤4096 bytes.
4. **Responses** (tie-point config, two shapes — bl-156b): DIRECT = a verdict mechanically fires one verb (flag = attention item; notice = message deposit, OFF by default; stop = existing stop verb). HOOKED = a verdict dispatches a responder: an ordinary lernie worker on an operator-named workflow, goal carrying the flagged window, granted tools = yog boundary verbs via the world tool shims — tool selection is ladder selection; no rung executor in yog. Verify the shim mechanics at implementation time (world/tools seeding, `yog gesture` as the argv spelling agents call). NO revoke-auto-approval — that waits on bl-0cea's capability ruling.
5. **Rendering** (V6): standing verdict per armed conversation derived from the ops tail; responses render where their verbs already render (a responder is a conversation row). No silent guardian: everything auditable.
6. **DESIGN.md amendments** with the mechanics: §4.2 ops row roster, §7.2/cadence schema, §12 module rows, and the attention-model hook — per VISION §6's pattern.

v1 reads the committed transcript only (replayable verdicts). The mid-step fast path is explicitly out of scope.

## Concerns to verify at implementation (recorded at design close-out, 2026-08-02)

- **Calibrate before arming rungs.** A false 'diverged' that fires visibly teaches the operator to distrust the monitor (the same alarm-fatigue rule DESIGN applies to banners). Ship flag-only as the default wiring; measure verdict quality against operator judgment via the ops-row dataset before recommending any direct stop wiring or responder hooks.
- **The responder reopens the injection blast radius the tool-less check closed.** It reads the flagged window (attacker-influenced content) while holding real verbs. Floor the default grant at nothing (pure judge); treat every grant beyond that as the operator's deliberate wiring, and revisit once bl-0cea's capability policy can bound responder tools mechanically. Also decide explicitly whether armed monitoring covers responders themselves (they are ordinary agents in the workspace — the uniform answer is yes, and it should stay uniform).
- **Thinking visibility varies by provider.** Some providers summarize or redact thinking blocks; the transcript may carry less early-warning signal than §4.9 implies. The check prompt must degrade gracefully to actions-and-output-only — verify per pinned provider, don't assume.
- **The check prompt is policy, not code.** Severability: the verdict prompt text belongs with the cadence.yaml monitor entry (or a file it names), not a Rust string constant — it is a logic-board tie-point the operator tunes.