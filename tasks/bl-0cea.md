+++
title = "design the capability boundary before unattended fleet execution"
created = 1785649816
updated = 1785823886
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["design"]

[[blockers]]
id = "bl-2b8c"
on = "claim"
+++
Source: bl-e249's Claude Code comparison, 2026-08-01.

## Verified gap

Pinned lernie grants tool names, then `bash` executes unrestricted `sh -c`. Yog's nested world isolates balls/lernie state; it does not confine host files, processes, network, environment, or ambient brazen credentials. Claude Code has allow/deny/ask permission mediation; its OS sandbox is separately optional and defaults off. The useful comparison is the permission boundary, not a false claim that Claude Code is sandboxed by default.

This must be settled before unattended drones are treated as safe. A modal prompt copied from an interactive REPL would instead deadlock the fleet.

## Deliverable: design first

Amend the cross-suite authorities and file the implementation balls only after the ruling answers:

1. Effect vocabulary for built-in and external tools: read, target write, destructive, process, network/open-world, secret/environment access.
2. Enforcement point in lernie before execution; yog owns per-role/ball policy, never the lower crate.
3. Writable-root definition from bl-2b8c's authoritative project target.
4. Durable request/allow/deny facts that survive driver exit and revival; GUI and headless parity through bl-8aab's control boundary.
5. Safe noninteractive defaults for an armed fleet, including what "ask" does without a human at the window.
6. Audit rendering, revocation, once-vs-persistent grants, and race behavior.
7. Optional OS confinement as a later, platform-explicit layer; never promise it where unavailable or silently fall back when required.

Threat-model the actual ambient PATH/network/credentials and external cwd. Do not add permission modals or a second workflow engine.