+++
title = "ops row for the detached lernie prompt records exit -2 on a successful start — investigate and render truthfully"
created = 1785646887
updated = 1785646887
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
A healthy S0 start (reply landed on disk and painted) records its ['lernie','prompt',…] ops row with exit: -2. Negative exits elsewhere in the trail read as signal deaths; -2 would be SIGINT. Either the detach path mislabels the spawn outcome (the setsid double-fork parent's exit being recorded as the op's?), or the driver really receives SIGINT and survives — find which. The trail is the transparency surface; a green flow must not log what reads as a kill. Acceptance: after a successful detached prompt the ops row's exit reflects the actual spawn outcome (0, or an explicit 'detached' marker), asserted in the S0 fake-substrate test.