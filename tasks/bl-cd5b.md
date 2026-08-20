+++
title = "fresh HOME passes drive preflight, then S8 falsely reports absent ambient balls as damaged"
created = 1787206335
updated = 1787206335
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["bug", "drive", "testing"]
+++
## Reproduction

Run `run-s5s8` with a fresh isolated `HOME` and all tools installed. Preflight reports every required prerequisite present. The product nesting assertions pass. The final S8 severability beat fails with “ambient damaged” because `$HOME/.local/state/balls` did not exist before the run.

This reproduced in two fresh worlds. `preflight.sh` declares no ambient balls prerequisite; `beats_s8.sh` later treats its absence as product damage.

## Required invariant

A clean host must either be a supported fixture or be rejected by preflight with the missing prerequisite named. Prefer seeding a sentinel ambient tree in the fixture, then prove it is byte-for-byte intact after deleting the nested world. Do not blame the product for a baseline the drive never established.