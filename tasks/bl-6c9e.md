+++
title = "headless prepare races its own workspace birth: the immediate continuation says unknown workspace"
created = 1786843305
updated = 1786843305
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["bug", "headless", "drive"]
+++
The shipped real-substrate drive reproduces this from a fresh world:

1. Start `yog serve`.
2. `yog gesture --ws home --project proj /prepare` succeeds, returns a prepared start for `home`, and the workspace exists on disk.
3. Immediately send `yog gesture --ws home --project proj "/prepare dir /home/u/proj"`.

Expected: another prepared reply whose typed `binding` is `/home/u/proj`; the documented two-process `/prepare` to `/prompt` flow must compose immediately.

Actual: `{"error":"unknown workspace home","ok":false}`. A second immediate directory prepare fails identically. A later `/assign` resolves `home` after the derived snapshot catches up.

`make drive DRIVE_RUNS=run-headless` failed only S2-T1 and passed the engine, board, blocker, claim, work-diff, and refusal beats. The same clean-world run reproduced twice.

The boundary invariant is: when an action returns a newly addressable resource, its success reply must be a barrier for subsequent boundary calls. Today workspace birth returns before name resolution can see its own write.