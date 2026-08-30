+++
title = "a parallel multi-tool envelope costs N sequential round trips once every tool routes"
created = 1788060204
updated = 1788060204
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Raised from the engine side while landing the seam inversion (engine ball bl-a00a). A consequence, not a defect: the engine ships green and nothing regresses until the pin is bumped.

**What changed.** The engine's host router runs on the executor's own calling thread, in list order. That was always true, and it used to cost little: only the *designated* subset was routed, and the spawned remainder still overlapped in a thread scope. With the router total, a `parallel` multi-tool envelope of N calls is N routed answers in sequence — under §5's pipeline, N adjudications, N mailbox hand-offs and N thrall round trips, none of them overlapping.

**Why the engine leaves it there.** Overlapping `route` would put a `Sync` bound on the injection object and, through it, on whatever the host holds behind it — a connection pool, a mailbox, a registry lock. Whether a transport is safe to drive concurrently is the host's fact and the engine holds none of it, so the engine will not assume it. Results still render in list order, so nothing observable changes; only wall-clock does.

**The shape if it is ever paid for**, recorded so it is not re-derived: a *defaulted* `route_all(&self, calls)` on the injection trait, whose default maps over `route`. Purely additive — no host is broken by its arrival, and a host that cannot fan out simply does not override it. The concurrency guarantee is then made by the host, at the one place that can honestly make it.

**What would decide it.** Whether real conversations emit `parallel` envelopes wide enough for the serialization to be felt against a loopback thrall (microseconds per handshake, per §5.4) versus a remote one. Nobody has measured that; this ball is where the measurement goes.

Engine-side record: `docs/DESIGN_TOOL_INJECTION.md` §7, third bullet.