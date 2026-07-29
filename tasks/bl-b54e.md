+++
title = "SIGTERM loses an in-flight ui.json flush — yog has no non-graceful shutdown path"
created = 1785202005
updated = 1785287134
claimant = "scorched-b54e"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["reliability"]
+++
Flagged by the bl-b416 implementer, 2026-07-27, and deliberately left unfixed
there. Pre-existing; `make ux` only makes it easy to hit.

## The defect

yog's `ui.json` flush is an eframe `on_exit` hook (`src/main.rs:197`) that fires
only on a **graceful window close**. `pkill` / SIGTERM does not reach it. A §4.1
durable change — a pin, a collapse override, a seen-acknowledgement — still
inside its debounce window at the moment of the signal is lost.

`make ux` (landed `5433e93`) runs `pkill -x yog` on every iteration, so the
UX-testing loop now hits this path many times a session.

## Why it is small but real

The window is genuinely small: gesture dispatch also forces a flush
(`AppModel::flush_ui`), so only a change in the last debounce interval before
the signal is at risk. But §4.1 state is the converging, cross-instance kind
(§13.1) — the whole point is that it survives — and "durable except when the
process is signalled" is not durable.

## Fix direction

A SIGTERM handler that flushes and exits. Note the constraint: `rules/`
confines every `unsafe` to `src/cli_outbound/sys.rs` (AGENTS.md rule 3), which
already holds one SIGTERM syscall — signal handling has an established home in
this crate, so use it rather than opening a second one. Check whether a safe
crate-level facility suffices before reaching for the syscall at all.

Alternative worth weighing first, since it may dissolve the problem instead of
handling it: if the debounce is the only thing standing between a gesture and
disk, ask whether §4.1 writes need debouncing at all, or whether the coalescing
window can be short enough that no handler is needed. AGENTS.md prefers the
reframe. `src/ui_state` owns the debounce; §7.2 sets the policy.

## Acceptance

A test that signals a yog holding an unflushed `ui.json` change and asserts the
change is on disk afterwards.