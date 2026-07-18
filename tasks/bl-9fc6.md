+++
title = "Y3: cli_outbound generalization — parametric binaries, current_dir, detached spawn"
created = 1784349554
updated = 1784349554
parent = "bl-4e66"
priority = 1
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-b4d9"
on = "claim"
+++
DESIGN.md §15 Y3. Binary resolution parametric over env var (LERNIE_BINARY, BL_BINARY, BZ_BINARY, PATH-name defaults), current_dir support on run, and a detached-spawn mode (setsid via libc, stdio->null, no retained Stream — the caller gets only a spawn result). Recorder-script tests for cwd/env propagation and a detach test proving the child survives the parent. Files: src/cli_outbound/mod.rs (-> ~240), tests in the Y1 split.