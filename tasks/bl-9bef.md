+++
title = "an explicit prompt gesture: fire inference from the current state — the lernie nudge; a failed first turn re-dispatches in place"
created = 1786683728
updated = 1786684419
claimant = "Newel"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Ruled at bl-9b52 (operator, 2026-08-13), question 1: a failed first turn must be re-dispatchable IN PLACE — no new conversation, no goal retype. The operator's words: 'we should have an explicit "prompt" button that just triggers inference from the current state. yes, re-dispatchable in place. really just the lernie nudge.'

Scope: a prompt/nudge gesture on the conversation surface that triggers inference from the current conversation state. The substrate mechanism is lernie's nudge — verify what the pinned lernie (=0.0.8) actually exports for this before designing; if the verb is unpublished, record the gate in this body the way bl-cd38 did (release-ordering prose gate) rather than claiming.

Context from bl-9b52: on a fresh install the first Enter mints workspace+conversation, dispatches, and the step dies on auth (credential missing). S0 step 4's draft-survives clause covers a start that aborted BEFORE spawning; here the start succeeded and the driver died after, leaving a conversation, a dead first step, and a goal the operator must retype after signing in. This gesture is the fix: sign in, press prompt, same conversation continues. Keyboard-operable per QUALITY F1; new gestures land as boundary variants first (bl-8aab lineage).