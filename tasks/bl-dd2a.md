+++
title = "Y6: Watch registry + repaint bridge + clock-injected sweeps; single-workspace view goes live (M1)"
created = 1784349555
updated = 1784350603
claimant = "filtered"
parent = "bl-4e66"
priority = 1
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-f144"
on = "claim"

[[blockers]]
id = "bl-9512"
on = "claim"
+++
DESIGN.md §15 Y6. WatchSet owning one Watcher per root with reconcile (desired vs live watchers); a bridge thread draining notify channels into a Mutex<DirtySet> and calling request_repaint(); frame-side drain -> re-derive dirty roots with a 100ms coalescing debounce; the 2s cheap sweep (enumerations + WatchSet reconcile + targeted liveness re-probe of only Live/InFlight agents) and 15s full sweep — all timing through an injected Clock trait. Wire the existing single-workspace view to re-render on disk change. Milestone M1: the current UI finally re-renders live. Files: src/watch/mod.rs (~210), src/app/dirty.rs (~150), shell wiring.