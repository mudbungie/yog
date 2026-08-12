+++
title = "gesture deposit ids are not unique across process namespaces, so one caller can receive another reply"
created = 1786510236
updated = 1786510236
priority = 1
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["boundary", "concurrency"]
+++
Runtime reproduction on `96d5f4e` in an isolated headless world:

1. Start `XDG_DATA_HOME=/tmp/yog-sparrow-smoke/data target/release/yog headless`.
2. From separate concurrent process namespaces, issue `yog gesture /attention` and `yog gesture /ops 1` against that world.
3. One exits 0; the other exits 1 with `deposit failed: gesture "1786491765-2" already deposited`.
4. A three-way run of `/balls`, `/board`, and `/workspaces` was worse: `/balls` refused the duplicate, while `/workspaces` returned the `/board` reply.

`src/multiplex.rs` mints `<SystemClock seconds>-<process id>`. Separate containers or PID namespaces commonly share the same pid, and pid reuse can collide within a second. The create-only inbox detects the collision but the reply path is keyed by that same id, so callers can consume the wrong response.

The id must be unique across every process writing one shared world. Add a regression that holds timestamp and pid equal across concurrent depositors and proves each query receives only its own reply.