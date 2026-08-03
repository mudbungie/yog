+++
title = "alignment watcher sidecar: cheap model streams agent thinking/output, scores intent alignment, drives graduated interventions"
created = 1785652106
updated = 1785718975
claimant = "Evenly"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["safety"]
+++
Operator concept 2026-08-01; design ruling DELIVERED 2026-08-02 (this ball's close lands it): VISION amended — new §4.9 (the alignment monitor), new story rung V6 (Invigilator), lernie ledger item 6, sequencing item 1, and two refusal-list updates (judging-supervisor bullet annotated; new "No silent guardian" refusal).

## The ruling, compressed

A cheap model reads what an agent is doing and answers one question — does the recent work serve the stated goal? — with a graduated ladder on divergence. The suite's disciplines made the original design constraints dissolve:

- Intent needs no pinning mechanism: goal.md + principal messages are already committed per-branch (SSOT).
- No wire tap: lernie streams every event to disk; the committed transcript carries thinking blocks; disk is the bus.
- The check is one bounded tool-less brazen call (embedded adapter), level-triggered off the step spine by yog's clock; every check writes an ops.jsonl row (audit + level-trigger memory + tuning dataset). Policy/arming ride cadence.yaml — I2 holds at three artifacts.
- The ladder is a logic-board tie-point, every rung an existing verb: flag (attention), notice (message — off by default, obfuscation cost recorded), escalate (judge dispatch, §4.4), stop (non-destructive, branch persists).
- The revoke-auto-approval rung has no substrate until bl-0cea rules the capability boundary; division of labor recorded: capability boundary rules what an agent MAY do, monitor rules whether the doing SERVES the goal.
- I7 held the fleet-mode way: arming per workspace is the explicit user action, continuing. Unarmed = the mechanism does not exist.
- v1 reads committed transcript only (replayable verdicts); mid-step fast path is a permitted later increment, stop-rung-only.

## Implementation

Deliberately deferred by operator (2026-08-02). Follow-up implementation ball files at close, referencing VISION §4.9; DESIGN.md gets amended by that ball when the mechanics land (§4.2 ops row roster, cadence.yaml schema, §12 modules), per VISION §6's own pattern.