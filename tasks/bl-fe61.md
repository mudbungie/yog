+++
title = "litany seam inversion: every tool routes through injection"
created = 1787977101
updated = 1787977846
claimant = "OrderInverter"
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"

[[blockers]]
id = "bl-37fd"
on = "claim"
+++
The driver's local executor is removed: every tool — not only designated ones — flows through the injection seam to yog's adjudication and onward to a thrall's mailbox. The engine's side is configuration, per the bl-37fd ruling: litany does not care where execution happens, it emits the invocation and waits on the capture. The tool-injection seam generalizes from "route designated tools to a remote executor" to "route all tools"; refusal-in-band semantics are unchanged.

Consequence stated plainly: with the front-door invariant, there is exactly one invocation pipeline (adjudicate → mailbox → execute → capture), and a server with no enrolled thrall refuses every tool call in band — which is the ship-inert posture working as designed, not an error state.

Upstream engine work lands in the engine's repo; this ball tracks the REMOTE §5 invocation-path amendment and yog's consumption of the generalized seam.