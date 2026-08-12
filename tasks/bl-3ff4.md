+++
title = "the world has no `yog` shim, so an agent cannot drive yog's own headless surface"
created = 1786515223
updated = 1786515247
claimant = "Hessian"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Split out of bl-02c2 (see that ball's premise-check comment). bl-02c2's ladder is gated on an operator calibration pass that has not happened — zero `monitor` ops rows in 19,508, no `cadence.yaml` under the world, so the monitor has never been armed. Its scope item 3 is NOT gated by that, because adding a shim fires nothing and decides nothing, so it lands here on its own.

VERIFIED AT HEAD, not guessed:
- `src/world/tools.rs` `ROSTER` is exactly six entries — `bl`, `lernie`, `bz`, `bl-delivery`, `bl-tracker`, `tool-control`.
- `src/cli_outbound/resolve.rs` `Binary` has no `Yog` variant.

So `<world>/tools/` carries no `yog`, and the world's `PATH` prepend therefore hands an agent every substrate tool EXCEPT the one that drives yog itself. Consequences:

1. **A responder cannot signal out.** VISION §4.9 rules the floor grant for a monitor responder to be `flag`, which is a boundary action — i.e. a `yog gesture` call. With no `yog` on PATH there is no floor grant to give, which is why bl-02c2 called this "the first rung of this ball, not an afterthought".
2. **V5's promise is unreachable from inside the world.** DESIGN §8.5 / VISION §4.8 make every operator gesture drivable headlessly, and §16.4's premise is that an agent drives tools *by bash*. An agent that types `yog gesture '/attention'` finds nothing.
3. **The clean room proves it.** `make drive-cleanroom` puts only `yog` and `git` on `PATH`; there is no host `yog` to fall through to. Everywhere else the fallthrough is worse than absence — it silently resolves the operator's INSTALLED yog, which drifts stale against the build under drive (the same class of defect as bl-d1af, where `command -v yog` in the drive logskel resolved the wrong binary).

SHAPE (small, and the same converge-on-the-way-in every other shim already uses): a `Binary::Yog` whose resolution is `current_exe()` with **no namespace prefix** — every other roster entry prefixes a verb word, and this one must not, because the shim's whole job is to be yog's own argv surface. Add it to `ROSTER` so `ensure_tools` seeds and converges it like the rest; it needs no `*_BINARY` override beyond the uniform one.

WATCH FOR — the recursion the other shims do not have: `world/tools/yog` re-execs yog with the caller's argv verbatim, and a bare `yog` with no args is the GUI. The shim must not become a way for an agent's bash to launch a window. Decide and record what a bare `yog` through the shim does; a namespaced arm cannot answer it, because there is no namespace word to dispatch on.

DONE WHEN: an agent's bash inside the world resolves `yog` to the running executable, `yog gesture '/attention'` answers from an agent seat, and the clean room proves it with no host `yog` installed.