+++
title = "design: money everywhere the wire says tokens — the price table keyed by (provider, model) and set from the boundary, a cost beside every token count, a per-workspace ledger, and a ceiling that parks a running conversation instead of only refusing a birth"
created = 1788745494
updated = 1788745495
claimant = "Cantaloups-D2"
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["usability-r2"]
+++
Round 2, lane D2 — the design ball for the spend/currency gap (bl-06f3 in yog, bl-9bae in lernie, comparator G3). Deliverable: amendments to docs/VISION.md §4.5, docs/DESIGN.md §3.5/§4.1/§8.6, docs/REMOTE.md §9.20 (the wire fields and the PROTOCOL rule). The implementation children are filed against the landed text and carry the contract in their own bodies.

The premise both round-1 balls state — 'the suite never says a dollar' — is half wrong on today's tree and the half that is wrong is what shapes the fix: yog has carried a price table (ui.json prices), a USD ceiling (ui.json ceiling), a micro-USD Cost with an unpriced remainder, and a usd string on the /agent, /workspace-balls and /board answers since bl-afc4/bl-56d5/bl-b4b5, and lernie paints it. The comparator saw no dollar because (1) the table is empty by default and lives in the engine's ui.json, which since the four-component split is on a box the seat may not be able to edit, so the 'hand edit is live within a tick' premise the read-only ruling rested on died with the window; (2) it is keyed by model id alone, and the row this box actually runs on (a subscription row) prices the same model id at zero marginal cost while an API-key row prices it at list — one key cannot say both; (3) the per-step, per-attempt, per-notch and per-workspace answers carry tokens with no cost beside them; (4) the ceiling refuses births only, so a running fleet past the number keeps spending until each drone ends.

The ruling is in the landed sections; the children carry it.