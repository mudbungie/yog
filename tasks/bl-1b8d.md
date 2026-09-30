+++
title = "Delegate the per-commit gate to the noodlezoo builder (bl-remote-gate); bump balls to 0.5.13"
created = 1790733827
updated = 1790733828
claimant = "Junketing-yog"
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Ops tracking: ~/ops bl-3e3f. scripts/pre-commit becomes leak-scan -> bl-speculate check -> bl-remote-gate; no local build path. make check stays the whole gate; speculate.yml (GH merge-queue builder) runs it and records verdicts with BALLS_TOOLCHAIN. balls pin 0.5.12 -> 0.5.13.