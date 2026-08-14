+++
title = "the shared drive 'gesture' helper has no deadline: a yog that dies mid-run hangs every windowed verb forever instead of failing"
created = 1786684957
updated = 1786684957
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Found by Ingot while building bl-bb20's headless beats (2026-08-13): 'yog gesture' blocks forever when no engine consumes the deposit. Ingot bounded its OWN boot probe, but the shared gesture helper in scripts/drive/ still has no deadline, so a yog that dies mid-run hangs every windowed verb instead of failing fast.

Fix direction per the operator's waiting rules (AGENTS.md 'Waiting for Things'): every wait gets a hard deadline and exits early on the terminal failure state — the helper should watch the engine's own liveness (its PID) alongside the deposit, and a dead engine is an immediate red, not a hang. Prove the repair bites per bl-f16e's standard: kill the engine mid-gesture and show the beat goes red in bounded time. Verify premises against the tree before editing.