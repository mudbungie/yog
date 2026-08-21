+++
title = "/ops help says every gesture is recorded, but boundary queries leave no audit row"
created = 1787206354
updated = 1787275444
claimant = "Zircons-Boundary"
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["bug", "cli", "docs"]
+++
The `/ops` help says it returns:

> “The tail of ops.jsonl — every gesture anything has fired, with its outcome.”

The architecture instead defines one row per attempted yog-initiated action. In a live engine, `/workspaces`, `/attention`, `/board` and `/search` added no rows. The implementation follows the architecture; the help overclaims the audit.

This also matters for handoff: an end-of-day reader can see actions but cannot reconstruct which snapshot a coordinator read before choosing them. That deeper provenance question needs a ruling; it must not be implied by inaccurate help.

## Required result

Make the help state the actual action-only contract, or deliberately amend the architecture and logging policy. Add a parity test that fires one query and one action, then pins exactly which appears in `/ops`.

---

Resolved toward the help text, not toward logging queries, because DESIGN §4.2 is unambiguous and is the authority: "One JSON line per **attempted** yog-initiated action." README already agreed ("Actions … log the same ops.jsonl rows; queries return the same typed data the GUI renders"); the /ops help detail was the only place that overclaimed, so the smaller honest change closes it.

The help now says actions only, states the reason (a query reads the world and changes nothing), and says outright that what a coordinator READ before choosing an action is not recoverable from the trail — so the provenance question the ball raises stays open and unimplied rather than being half-answered by a sentence.

Parity test: boundary::consume::tests::an_action_leaves_an_ops_row_and_a_query_leaves_none — one query deposit (/attention, a pure snapshot read) and one action deposit (/ack, the operator's own §4.2 line, which spawns nothing and writes exactly one row), asserting the trail is empty after the first and exactly one row after the second. Both halves, because the claim is only worth anything as a pair.

Not done, deliberately: read-provenance (which snapshot a coordinator saw). That is an architecture amendment, not a help fix, and it wants its own ball and a ruling.
