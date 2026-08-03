+++
title = "implement the alignment monitor: tier-0 check + ladder per VISION §4.9"
created = 1785718992
updated = 1785718992
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["safety"]
+++
The design ruling landed with bl-af1a (VISION §4.9 + story rung V6 'Invigilator'). This ball is the deferred implementation — read §4.9 and V6 first; they are the authority. Verify every premise against the then-current tree before editing (boundary surface, cadence.yaml schema, embedded brazen adapter).

Scope, per the ruling:

1. **Arming**: a boundary Action variant per workspace (headless spelling included — §4.8 compile gate), recorded as the cadence.yaml monitor entry (model, thresholds, verdict→rung map). Unarmed = mechanism absent; severability is deleting the entry.
2. **The check**: bounded tool-less cheap-model call via the embedded brazen adapter; request = goal.md verbatim + transcript delta since last-checked sha + standing verdict; response = aligned|drifting|diverged + one-sentence reason. Level-triggered off the step spine by the clock: check only when the branch tip moved. Off-thread, never the frame.
3. **The ops row**: every check appends one ops.jsonl row (sha, verdict, reason, usage) — audit trail, level-trigger memory, and §4.5 tuning dataset in one. ≤4096 bytes.
4. **Rungs** (tie-point config, each an existing verb): flag = attention item; notice = message deposit (OFF by default); escalate = judge dispatch; stop = existing stop verb. NO revoke-auto-approval rung — that waits on bl-0cea's capability ruling.
5. **Rendering** (V6): standing verdict per armed conversation derived from the ops tail; rungs render where their verbs already render. No silent guardian: everything auditable.
6. **DESIGN.md amendments** with the mechanics: §4.2 ops row roster, §7.2/cadence schema, §12 module rows, and the attention-model hook — per VISION §6's pattern.

v1 reads the committed transcript only (replayable verdicts). The mid-step fast path is explicitly out of scope.