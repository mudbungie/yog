+++
title = "sweep the drive beats for vacuous assertions: empty-variable and generic-string greps pass in runs where the gesture never happened"
created = 1786513798
updated = 1786513798
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["testing"]
+++
Filed by Alkaloid 2026-08-11 from concrete instances found by Dowel (bl-afa7, bl-2d45) and the parallel Rust-side finding in bl-bc06.

## The pattern

Two independent harnesses in this repo were asserting **vacuously** — passing green in runs where the thing under test provably never happened. Found in one session, by two agents who were not looking for it:

**Shell drive beats** (Dowel, while repairing bl-afa7 and bl-2d45):
- An unset `$minted` turned a beat's check into `grep -q '""'` — which matches anything, so two beats passed with no minted sphere at all.
- An ack watermark checked with `grep '"seen"'` passed **in the very runs where the stop never happened** — the generic key was present regardless of the gesture.

**Rust paint assertions** (Ferrule, bl-bc06): `egui::Galley::text()` returns the INPUT string, so every paint-layer assertion was blind to elision. All 1815 tests passed on the fix, proving none covered truncation. That half is tracked in **bl-36c3**; this ball is the shell-side half.

## Why these are one bug class, not two

Both are assertions whose predicate is satisfied by something other than the behaviour under test — an empty expansion, a key that is always present, a string that was never laid out. A beat like this **can never go red**, so it reads as coverage while providing none. It is worse than a missing test: a missing test is visible in a coverage report.

## Scope

Sweep `scripts/drive/` and `tests/integration/stories_*.rs` for assertions that cannot fail:

1. **Greps on interpolated variables** that are empty or unset on the failure path — quote-and-check, or assert the variable is non-empty first. `grep -q "$x"` with empty `$x` is an unconditional pass.
2. **Greps on generic keys** (`"seen"`, `"ok"`, a bare field name) present in the output regardless of outcome — assert on the identity that distinguishes this run (conversation id, agent id, the argv the gesture actually carried), which is what bl-2d45's repair does.
3. **Beats that have never failed** in the log history. Per the standing prior, a never-red beat is a suspect, not a success.

## The standard for each repair

**Prove the beat bites**: revert the fix (or run against a tree where the gesture does not happen) and show the beat goes red. Dowel's bl-afa7 repair met this — two real drives, three FAILs and two vacuous PASSes became five real PASSes. An assertion not shown to fail is not evidence.

## Relationship to other balls

- **bl-36c3** — same sweep, paint layer. Sibling, not duplicate.
- **bl-bb20** — S2 and S9-S18 have ZERO real-substrate beats. That is *absent* coverage; this ball is *fake* coverage. Do them separately; fake coverage is the more dangerous of the two because it reads as done.

Verify all cited paths against HEAD first; ball bodies drift.