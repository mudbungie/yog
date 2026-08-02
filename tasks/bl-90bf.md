+++
title = "transient red on send: 'driver produced no response' then recovery — find the cause, fix or quiet it"
created = 1785645675
updated = 1785645896
claimant = "flash-fixer"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator report 2026-08-02: sent a message (ops workspace, conversation shudder-storeroom, ~22:54Z), UI flashed red with something about 'driver produced no response' for a second, then the send worked. Disk shows: step 001 completed clean in ~1s, no stderr, no detached err content — so the driver DID produce a response almost immediately. Investigate where the 'driver produced no response' classification comes from (framing classifier? attention rule 2?) and why it fired transiently on a healthy send — likely a race between send-gesture and the driver's first output frame, classified as dead-driver for a frame or two. Decide: if it's a premature classification window, widen/gate it (don't declare no-response before the driver could plausibly respond); if the red is honest but self-healing, it shouldn't flash for sub-second states. Fix the real fact.