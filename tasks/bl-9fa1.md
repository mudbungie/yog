+++
title = "compactor recursion: compaction triggers on compactor branches — 229-branch geometric dispatch storm in <workspace>"
created = 1785287786
updated = 1785287786
priority = 1
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Live incident 2026-07-28 ~08:07Z, operator report verbatim: 'whoa, something just happened. a chat went crazy, some loop, and I got a non-responsiveness/kill flag.'

## What happened

Conversation <agent-id> (workspace <workspace>) triggered a recursive compactor storm: every spawned branch's goal.md reads 'You are the compactor for branch <parent>'. Compactors were spawned FOR compactor branches, recursively. Depth census of refs/heads/agents/<agent-id>*: 15 branches at descent depth 2, 32 at 4, 62 at 6, 112 at 8 — 229 total in ~80 seconds (08:07:02Z–08:08:24Z), geometric until halted (operator kill flag / non-responsiveness likely ended it; machine was also under load ~15 from six concurrent tarpaulin gates). No drivers remained after; count stable.

## The defect

Whatever fires compactor dispatches (find it: yog sweep/trigger vs the embedded lernie 0.0.2 harness — read the compactor goal template's source to locate the owner) treats every branch as compaction-eligible, including branches that ARE compactors. The invariant that dissolves the whole class: a compactor run is not a member of the compaction-eligible set (and/or eligibility requires a transcript actually past threshold — a seconds-old branch can never be eligible). One invariant, not special cases per level.

## Ask

1. Attribute the trigger precisely (who decided to spawn each compactor, what condition it read).
2. State the guard invariant and where it belongs (yog if yog owns the trigger; if lernie-side, record the venue question on this task like bl-bd9d).
3. Cleanup path for the 229 debris branches in <workspace> (do NOT delete without operator sign-off; document the command).
4. Also explain the non-responsiveness/kill flag the operator saw: which yog surface raised it and was it a correct read of the storm.