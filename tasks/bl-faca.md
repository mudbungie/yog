+++
title = "REGRESSION of bl-bb20: the real-substrate drive still excludes live S10 and S18 surfaces"
created = 1787206332
updated = 1787275657
claimant = "Zircons-Drive"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["regression", "drive", "testing"]
+++
`scripts/drive/beats_headless.sh` still records two scope decisions:

> “S10 ... OUT — transcript/steps/files/rail have no headless spelling.”

> “S18 ... OUT — the armed loop is not live.”

Both premises are stale. Closed `bl-6233` landed the transcript, steps, files, rail, governing and inbox boundary queries. Closed `bl-66fb` landed the armed fleet loop. Closed `bl-bb20` intentionally deferred these rungs only until those capabilities existed.

The gap is observable: a multi-tick failed-drone trajectory can wedge or repeat the top ball, while unit tests cover isolated planner decisions and no real-substrate run exercises the trajectory.

## Required coverage

Drive S10's real on-disk historian reads through their headless spellings. Drive S18 from `/fleet` through real claim, spawn, board facts, reap/disband and at least one failed trajectory. Remove the stale comments; every new beat must be mutation-proved and no-wire where a deterministic local adapter suffices.