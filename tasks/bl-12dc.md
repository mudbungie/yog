+++
title = "the 300-line cap is only enforced on staged files, so pre-existing violations ride forever: src/app/balls.rs sat at 308 undetected"
created = 1785374145
updated = 1785374620
claimant = "entrance-12dc"
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["bug"]
+++
Found by bl-7e32's agent while landing `51939f8`.

## The defect

`.githooks/pre-commit` enforces the 300-line source cap **only over the files in the staged diff**. A file that crosses 300 lines without ever appearing in a subsequent commit is never checked again, so the violation rides indefinitely.

Concrete instance: **`src/app/balls.rs` was already 308 lines on main** and had never been caught. It only surfaced because bl-7e32 happened to edit it, which put it in the diff. That agent fixed the instance in passing — it split the §8.2 workspace-name derivations into `src/app/balls/targets.rs` (`focused_ws_name`, `workspace_names`, `move_targets`), bringing balls.rs to 290 — but the **hole that let it happen is still open**.

## Why this matters beyond one file

The cap is one of the repo's two machine-enforced structural rules (with 100% coverage). A gate that only sees what you happen to touch is not an invariant, it is a sampling. Nobody knows today how many other files are over — the only reason we know about balls.rs is coincidence.

## Scope

1. **Audit the tree now.** Enumerate every source file over the cap on current main and report the list. That is the immediate fact nobody has.
2. **Close the hole.** The cap check should cover the tree, not the diff. Note the tension before writing it: a whole-tree scan on every commit is the obvious fix but adds latency to a hook that already runs tarpaulin, and a pre-existing violation in an untouched file would then block an unrelated commit — which is correct in principle but may be a rude first encounter. Decide deliberately and record the reasoning; if the answer is "whole-tree scan, and fix whatever it finds first", say so and do the fixing as part of this ball.
3. Whatever the mechanism, the rule must end up genuinely enforced rather than sampled.

## Relationship to bl-0dff

**Adjacent, not duplicate.** bl-0dff fixes two different pre-commit defects: the inherited `GIT_DIR`/`GIT_INDEX_FILE` breaking three `fs_watcher` drift tests, and the hook discarding tarpaulin stdout so failures are unnamed. This ball is about the cap check's *scope*. They touch the same file and should not be worked concurrently — check whether bl-0dff has landed before claiming, and fold main first if it has.