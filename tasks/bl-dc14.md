+++
title = "wire bootstrap failure opens an inert window; a second instance reliably triggers it"
created = 1786843450
updated = 1786844986
claimant = "Wire-dc14"
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["bug", "wire", "gui", "drive", "design"]
+++
Reproduction:

1. Run one yog engine with its self-provisioned loopback material.
2. Launch a second bare `yog`, either on the same world or under a separate XDG data root.
3. Type a goal and press Enter or click Start.

The second process writes `yog: wire: bind 127.0.0.1:7737: Address already in use` to stderr, then opens a normal-looking window anyway. The box accepts text, but Enter and Start produce no operation and no visible refusal. The listener returned `None`, so the window has no read or action path.

The isolated real-substrate `make drive DRIVE_RUNS=run` fails the same way whenever an ordinary yog is already up: the window opens, then every start and reply beat fails. Two isolated runs reproduced it.

The governing promise says: `two yog instances running side-by-side faithfully replicate the same data`. A fixed process-global port contradicts that promise even for separate worlds. More generally, listener mint, certificate-read, and bind failures all collapse to the same optional `None` while the GUI still presents operable controls.

Related bl-4c50 covers three tests failing when a running yog owns the fixed port. It proposes preserving the constant for a real box and isolating tests. That is not sufficient here: the same constant breaks the user-facing two-instance invariant. The platform and test fixes should share one truthful address-ownership design.

Required outcome: a second window joins the appropriate live engine, owns a distinct published endpoint such as the supported port-zero path, or refuses visibly before presenting an operable-looking window. No composer or control may appear actionable without a wire. The drive must select an isolated endpoint. Acceptance covers two worlds, two windows on one world, certificate or bind failure, visible recovery, and the real-substrate drive while another instance runs.