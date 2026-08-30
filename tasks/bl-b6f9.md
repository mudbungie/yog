+++
title = "context maintenance is queued against the inevitable cache miss: prefix edits like tool unload merge into the operating branch when the miss is already paid"
created = 1788066933
updated = 1788066933
priority = 4
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Operator direction, jotted for eventual design (not for dispatch now). A prefix edit — unloading a finished host's tools (bl-3455), pruning a stale loaded set, any subtraction from the declared surface — costs a full prompt-cache rebuild the moment it lands, which is why the loaded document today only grows. The mechanism wanted: maintenance acts on an agent's context QUEUE rather than land, and the queue merges into the agent's operating branch at the moment a cache miss is known to be inevitable anyway (a compaction, a config retarget, a re-declaration some other act already forced) — so subtraction rides a rebuild that was already being paid, and the cache is never broken on maintenance's own account. First customer is bl-3455's unload; the queue generalizes to any deferred prefix edit. Open questions for the eventual design: where the queue lives (the loaded document beside its subject, or a sibling), who detects miss-inevitability (litany's assembly is the only place that knows), and whether a queued act can expire or be superseded before it merges.