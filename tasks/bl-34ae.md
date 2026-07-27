+++
title = "CI red on macOS: tests/multiplex_bl.rs worktree-materialization assert fails (linux green)"
created = 1785132694
updated = 1785132886
claimant = "waxier-34ae"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Found while landing bl-89a4 (registry adoption); NOT caused by it — reproduced on unmodified main (0d914fc) and present in CI runs for a3bce10 and 0d914fc.

The macos CI job fails `make test`:

    thread 'the_bl_arm_runs_the_whole_rung_on_the_embedded_balls' panicked at tests/multiplex_bl.rs:174:5:
    worktree materialized at /var/folders/g3/.../T/.tmpg6hPFw/state/balls/plugins/bl-delivery/var/folders/g3/.../T/.tmpg6hPFw/proj/bl-a448

i.e. `bl claim` reported exit 0 but the assert on the derived worktree path found no README.md there. The path is a doubled tmpdir (territory root + the mirrored project path), which is expected shape — the suspicion is macOS `/var` vs `/private/var` symlink normalization: bl-delivery resolves the project path one way (realpath -> /private/var/...) and the test derives it the other (/var/...), so the test looks in a directory that is not where the worktree actually landed. Linux has no such symlink and passes.

Fix candidates: canonicalize the tmpdir in the test before deriving the expected path, or assert via the path bl-delivery itself prints. Verify on macOS.