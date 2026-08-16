+++
title = "a second yog window opens inert when the fixed loopback wire port is occupied"
created = 1786843450
updated = 1786843450
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["bug", "wire", "gui", "drive"]
+++
Reproduction:

1. Run one yog engine with its self-provisioned loopback material.
2. Launch a second bare `yog`, either on the same world or under a separate XDG data root.
3. Type a goal and press Enter or click Start.

The second process writes `yog: wire: bind 127.0.0.1:7737: Address already in use` to stderr, then opens a normal-looking window anyway. The box accepts text, but Enter and Start produce no op and no visible refusal. The window is inert because `listen` returned `None`, so `window_wire` also returned `None`.

This also makes the isolated real-substrate command `make drive DRIVE_RUNS=run` fail whenever an ordinary yog is already up: the window opens, then all start and reply beats fail. Two isolated reproductions showed the same bind error.

The governing promise says: `two yog instances running side-by-side faithfully replicate the same data`. The implementation instead hard-codes self-provisioning to port 7737 and treats bind failure as non-fatal even though the window now has no other read or act path.

Required outcome: a second window either joins the live engine for that world, owns a distinct bound port such as the already-supported port zero path, or refuses visibly before presenting an operable-looking window. The drive must choose an isolated port so its isolation includes the wire.