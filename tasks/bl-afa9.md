+++
title = "ops row for the detached lernie prompt records exit -2 on a successful start — investigate and render truthfully"
created = 1785646887
updated = 1785647334
claimant = "Culver"
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
A healthy S0 start (reply landed and painted) records its ['lernie','prompt',…] ops row with exit: -2. Follow-up finding: a FAILED spawn (nonexistent cwd) also records exit: -2 — so -2 is a 'no exit code' sentinel doing double duty for 'detached, running fine' and 'never started'. Two opposite facts, one encoding; the trail cannot distinguish them, and a negative number reads as a signal death besides. Fix: give the detached-success and spawn-failure cases distinct, honest renderings (e.g. 'detached' vs 'failed to spawn', never a fake numeric exit), and assert both in the S0 fake-substrate tests. Find the recorder (ops row writer) and the sentinel's definition first; the -2 may be load-bearing elsewhere — migrate readers with it.