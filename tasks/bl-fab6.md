+++
title = "a parallel multi-tool envelope costs N sequential round trips once every tool routes"
created = 1788060204
updated = 1790058298
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Raised from the engine side while landing the seam inversion (engine ball bl-a00a). A consequence, not a defect: the engine ships green and nothing regresses until the pin is bumped.

**What changed.** The engine's host router runs on the executor's own calling thread, in list order. That was always true, and it used to cost little: only the *designated* subset was routed, and the spawned remainder still overlapped in a thread scope. With the router total, a `parallel` multi-tool envelope of N calls is N routed answers in sequence — under §5's pipeline, N adjudications, N mailbox hand-offs and N thrall round trips, none of them overlapping.

**Why the engine leaves it there.** Overlapping `route` would put a `Sync` bound on the injection object and, through it, on whatever the host holds behind it — a connection pool, a mailbox, a registry lock. Whether a transport is safe to drive concurrently is the host's fact and the engine holds none of it, so the engine will not assume it. Results still render in list order, so nothing observable changes; only wall-clock does.

**The shape if it is ever paid for**, recorded so it is not re-derived: a *defaulted* `route_all(&self, calls)` on the injection trait, whose default maps over `route`. Purely additive — no host is broken by its arrival, and a host that cannot fan out simply does not override it. The concurrency guarantee is then made by the host, at the one place that can honestly make it.

**What would decide it.** Whether real conversations emit `parallel` envelopes wide enough for the serialization to be felt against a loopback thrall (microseconds per handshake, per §5.4) versus a remote one. Nobody has measured that; this ball is where the measurement goes.

Engine-side record: `docs/DESIGN_TOOL_INJECTION.md` §7, third bullet.

---

Measurement gathered per the ball's ask.

1. Envelope width (live world, $XDG_DATA_HOME/yog/workspaces, workspaces dev/lab/ops). 37 total steps found (lab has no step history). Every step dates to one day, 2026-08-30 — none fall in the last 14 days, so the "recent" cut is empty; all numbers below are the full population. Counted by entries under each step's tools/ dir (one dir per routed call, per src/steps_view/detail.rs's tool_ios): 0 tools: 12 steps (32%); 1 tool: 23 steps (62%); 2 tools: 1 step (3%); 4 tools: 1 step (3%). Max width seen = 4. Both multi-tool steps are a single provider-side multi_tool envelope split into N invocations of one call_id (call_id-1..call_id-N) — exactly the shape this ball describes. So width>1 is rare (2/37 steps) but not hypothetical, and the widest observed is 4.

2. Cost of one routed round trip on loopback. No existing test or drive-log beat times this — every test that touches these ticks overrides them to 1ms specifically to avoid a real sleep (src/tool_host/remote/tests.rs, src/registry/mailbox/slots/tests/mod.rs). So this is a code-derived estimate, not a measured run. REMOTE §5.4's "microseconds" figure is the connection handshake alone; the full invoke+capture round trip is polling-bound at several points, each a named production constant: engine gesture-inbox poll 50ms (src/multiplex.rs POLL); driver's ask-for-reply poll 125ms tick (src/tool_host/ask.rs Budget::default — its own doc comment: "three local file reads behind a 250 ms consumer poll"); engine mailbox take() notices a posted invocation within HOLD_TICK 125ms (src/registry/mailbox/slots/read.rs); driver's capture poll at 500ms tick (src/tool_host/remote.rs patience()). Summed, one non-parallel routed call's in-band overhead (excluding the tool's own run time) is on the order of several hundred ms, plausibly close to 1s average — two to three orders of magnitude above "microseconds," because the path is poll-based end to end, not a blocking handshake.

Recommendation: envelope width is overwhelmingly 1 (96% of steps that call a tool at all), and even the widest sampled case (4) at ~0.5-1s/call overhead costs on the order of a few extra seconds of wall clock serialized versus overlapped — felt, but rare in this sample. Population is thin (37 steps, one day, nothing in the last two weeks), so treat this as a first read, not a verdict: worth a second pass once there is a fresher, larger sample before closing outright.
