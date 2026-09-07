+++
title = "context maintenance is queued against the inevitable cache miss: prefix edits like tool unload merge into the operating branch when the miss is already paid"
created = 1788066933
updated = 1788754168
priority = 4
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Operator direction, jotted for eventual design (not for dispatch now). A prefix edit — unloading a finished host's tools (bl-3455), pruning a stale loaded set, any subtraction from the declared surface — costs a full prompt-cache rebuild the moment it lands, which is why the loaded document today only grows. The mechanism wanted: maintenance acts on an agent's context QUEUE rather than land, and the queue merges into the agent's operating branch at the moment a cache miss is known to be inevitable anyway (a compaction, a config retarget, a re-declaration some other act already forced) — so subtraction rides a rebuild that was already being paid, and the cache is never broken on maintenance's own account. First customer is bl-3455's unload; the queue generalizes to any deferred prefix edit. Open questions for the eventual design: where the queue lives (the loaded document beside its subject, or a sibling), who detects miss-inevitability (litany's assembly is the only place that knows), and whether a queued act can expire or be superseded before it merges.

---

Designed and landed on the litany side as litany bl-b902 (the assembly is the only place that knows when a miss is inevitable). The queue's home is the previous step's own request.json; the fold is a byte comparison against it — model, system slot, first wire message — so no boundary flag and no list of boundary kinds. First customer is exactly this ball's: a host retiring an injected tool (bl-3455 unload) is held, byte for byte, until a paid miss. Invariant and classification table in litany ARCH §5.5; follow-ups litany bl-70d2 (load_skill anchoring) and brazen bl-8b47 (prompt_cache_key on the OpenAI dialects).

---

litany 0.0.12 (upstream bl-b902, ARCH $5.5) landed the tools-array half of this ball upstream: a subtraction from the declared surface is now held until the next PAID cache miss rather than landing at the next assembly, with the queue's home the previous step's own request.json. yog consumed it as a wording amendment to REMOTE $5.2 and no code (bl-9ced). What is left here is the rest: a mid-run load_skill still inserts into the body and re-bills from its position until a landing, and a config edit that moves the soul, the model or the descriptor cut is a paid miss by ruling. Re-read the ball's premise before working it — its 'unload lands the edit immediately and immediately costs the rebuild' sentence is no longer true.
