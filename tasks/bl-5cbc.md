+++
title = "/ops help says every gesture is recorded, but boundary queries leave no audit row"
created = 1787206354
updated = 1787275400
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