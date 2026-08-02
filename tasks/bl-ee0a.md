+++
title = "yog derives on the egui frame thread, so a 227-branch dispatch storm stalls the window — the 'non-responsiveness/kill flag' the operator saw was the desktop's, not yog's"
created = 1785460448
updated = 1785644989
claimant = "Riffle"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["investigation"]
+++
## Origin

Ask (a) folded into bl-ebbd from bl-9fa1 (closed as duplicate), operator report
verbatim:

> "whoa, something just happened. a chat went crazy, some loop, and I got a
> non-responsiveness/kill flag."

bl-9fa1's ask #4: *"Also explain the non-responsiveness/kill flag the operator saw:
which yog surface raised it and was it a correct read of the storm."*

bl-ebbd closed against the lernie 0.0.3 fixes for the storm itself. This ask is
yog's own code and survives that close.

## Finding: no yog surface raised it

yog ships no non-responsiveness affordance and no kill affordance. Verified by
reading the tree, not by inference:

- `grep -rn 'not responding\|Not Responding\|force quit\|ANR' src/ docs/` — zero hits.
- yog's only kill-adjacent surfaces are (i) `src/actions/verbs.rs::stop`, an
  operator-initiated `lernie stop <ws> <agent> [--stop-children]`, which nothing
  raises by itself; (ii) `src/steps_view/render.rs::framing_badge`, which paints
  `Framing::Killed` as the ash `("■", theme::ASH, "no clean end")` badge; (iii) the
  §7.3 wound `src/steps_view/wound.rs::NO_RESPONSE = "driver produced no response"`.
  None of the three is a dialog, a flag, or a prompt to kill anything.

So the flag the operator saw was **not yog's** — it was the desktop's standard
unresponsive-window affordance, raised against yog's own window. (Not directly
observed: no screenshot or log of the dialog exists. What is verified is that yog
contains no such surface to have raised it.)

## The real defect: derivation is synchronous on the egui frame thread

`AppModel::tick(&mut self, ts: &str) -> bool` (`src/app/derive.rs`) does one frame's
work inline: drain dirty roots, `dispatch_dirty`, then `self.full_sweep()` /
`self.cheap_sweep()`, then `self.rederive(&root)` for every due root. There is no
`thread::spawn` and no async anywhere in `src/app/`. The cadences
(`src/app/dirty.rs`) are `DEBOUNCE = 100ms`, `CHEAP_SWEEP = 2s`, `FULL_SWEEP = 15s`.

Derivation cost scales with the workspace's branch count — DESIGN §5.1 #8 "Agent
set, descent, tips" derives from `git for-each-ref agents/*`, plus per-agent
liveness probes. During the 2026-07-28 storm `<workspace>` went from 2 branches
to 227 in ~90 seconds while every one of those branches was also streaming step
files that mark roots dirty. Every 100 ms debounce release and every 15 s full sweep
re-walked the whole set on the paint thread, with the machine already at load ~15
from six concurrent tarpaulin gates. A frame that does not return does not pump
events, and a window that does not pump events is what the desktop calls
unresponsive.

## Was it a correct read?

Correct in the literal sense — the window genuinely was not servicing events — and
useless in the sense that mattered: it named yog as the problem, not the storm. yog
had no surface that said "227 branches just appeared under one conversation," so the
one signal the operator got pointed at the wrong layer, which is exactly the
misdiagnosis pattern bl-ebbd was filed about.

## What to decide (not yet decided here)

Two candidate directions, both real changes, neither obviously right:

1. **Move derivation off the paint thread** — derive into a snapshot on a worker and
   have the frame read the latest completed one. Bounds the frame cost by
   construction, at the cost of yog's current "the frame IS the derivation" simplicity
   (§7.2) and a new staleness surface.
2. **Bound the per-frame work** — cap how many roots one tick re-derives and carry the
   rest to the next frame. Keeps the single-thread model; makes the 15 s staleness
   bound (§7.2) soft under load, which it currently is not.

Direction 1 is the honest fix for "cost scales with the workspace"; direction 2 is
the smaller diff. Attack both before committing.

## Also worth having, cheaply

A storm is a fact yog can render. Nothing in the tree names "this conversation's
descent grew by N branches in the last sweep" — the drift instrumentation
(`src/app/drift.rs`, §7.2) reports *that* a sweep found unannounced change, not the
shape of it. A branch-count delta on the ops surface would have pointed the operator
at lernie in one glance.

## DECIDED (2026-08-01, operator): direction 1, strengthened to a standing principle

Operator verbatim: *"ui and operations in the backend should be totally isolated. The UI should never freeze, which means that it should do as little as possible."*

So not merely "move derivation off the paint thread" — **total isolation**: the frame thread does read-only rendering of the latest completed snapshot and input capture, nothing else. All derivation (dirty dispatch, sweeps, rederive, liveness probes) moves to a worker. The frame never blocks on the worker; staleness is surfaced honestly (the §7.2 drift instrumentation adapts to snapshot age). Direction 2 (per-frame work caps) is rejected — a roots-per-tick cap cannot bound the cost of re-deriving ONE root, and the observed storm was one root growing 227 branches.

Also in scope per the body's "Also worth having, cheaply": a branch-count delta on the ops surface, so a storm names lernie, not yog.

DESIGN.md §7.2's "the frame IS the derivation" simplicity claim must be rewritten as part of this ball — fix the doc, not around it.

## Sequencing

Ready for implementation, but HOLD dispatch until bl-52f8 (tree-wide decomposition sweep, claimed @Umber) lands — both touch src/app/* broadly and would collide at fold.