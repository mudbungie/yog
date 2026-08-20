+++
title = "scoped snapshots drop real armed-fleet facts, so /board shows drones but not the policy running them"
created = 1787206327
updated = 1787206327
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["bug", "wire", "agentic"]
+++
## Reproduction

Over a registered wire seat:

1. Arm a workspace with `/fleet 2`.
2. Wait for two successful `yog-fleet spawn` rows.
3. Ask `/board`.

The response contains the two claimed balls and their drones, but no `fleet` property. The loop is autonomously acting while every wire seat, including the loopback GUI, is blind to its cap, fullness, tick, lease and last action.

## Cause

The cadence file keys fleet entries by absolute workspace path. `app/derive/route.rs` preserves those keys. `app/snapshot/scope.rs` filters them as though each key were a workspace name, so every real armed entry is dropped. Its test fabricates name-keyed fleet data rather than the representation written by `/fleet`.

`boundary/reply/encode.rs` already defines the promised wire shape for an armed board.

## Required invariant

An authorized scoped snapshot must carry the selected workspace's actual fleet facts; unauthorized workspaces must remain absent. Cover the real arm → cadence → scoped snapshot → board path rather than a hand-built name-keyed fixture.