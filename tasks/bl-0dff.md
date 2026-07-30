+++
title = "the pre-commit hook can never pass on a plain git commit: 3 fs_watcher drift tests inherit the hook's GIT_DIR"
created = 1785373550
updated = 1785373860
claimant = "entrance-0dff"
priority = 4
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["bug"]
+++
Discovered while working bl-df65 — a docs-only commit, which still could not pass.

## The defect

`git` sets `GIT_DIR` and `GIT_INDEX_FILE` in the environment of every hook it
runs. `.githooks/pre-commit` runs `cargo tarpaulin --fail-under 100
--skip-clean --engine llvm --out Stdout`, and the test binaries inherit that
env. Three tests shell out to `git` against temp repos they build themselves:

    fs_watcher::drift_tests::scenario_deleting_a_packed_ref_touches_nothing_under_refs
    fs_watcher::drift_tests::scenario_a_git_ref_update_coalesces_to_one_change_at_the_destination
    fs_watcher::drift_tests::a_packed_ref_deletion_now_reaches_the_watcher

The inherited `GIT_DIR` retargets those subprocesses at the REAL repo instead
of the temp fixture, so they fail. The hook then aborts the commit with
tarpaulin's opaque `Error: "Test failed during run"` — no test name, because
the hook sends tarpaulin stdout to /dev/null (hook line 57).

## Repro — exact, from any work worktree

    cargo test                                   # 851 passed, 0 failed
    GIT_DIR=$(git rev-parse --git-dir) \
      GIT_INDEX_FILE=$(git rev-parse --git-dir)/index cargo test
    # test result: FAILED. 848 passed; 3 failed

## Why it has stayed invisible

`bl close` is the normal delivery path and runs the hook itself, without git's
hook env — so closes pass and nobody meets this. It only bites an agent making
an ordinary `git commit` inside a `work/<id>` worktree, where the hook is
unpassable and the error names nothing. Bypass is "intentionally not
supported" per the hook header, so the only exit is `--no-verify` — exactly
what that header forbids.

## The fix — pick one; the first is the real one

1. **Test isolation.** The drift tests must clear `GIT_*` from the environment
   of every `git` they spawn (`Command::env_remove`, or `env_clear` plus an
   explicit allowlist). A test that builds its own repo must not be steerable
   by ambient git env. This is the single-source fix: it holds under any
   caller, not just the hook.
2. Hook-side palliative: `unset GIT_DIR GIT_INDEX_FILE GIT_WORK_TREE` before
   the tarpaulin line. Fixes the symptom, leaves the tests steerable.

Worth doing regardless: stop discarding tarpaulin stdout in the hook, or tee
it, so a failing gate names the failing test instead of reporting only
`Error: "Test failed during run"`.