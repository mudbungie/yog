+++
title = "coverage debt: S2 and S9-S18 are graduated rungs with in-crate tests and ZERO real-substrate drive beats"
created = 1786162702
updated = 1786162702
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["drive"]
+++
Filed by the 2026-08-07 re-baseline drive (bl-c63e), docs/drive-logs/2026-08-07-ladder-rebaseline.md.

STORIES.md s done-bar has two halves: a story is done when its tests pass against the FAKE substrate AND the flow works against the REAL one. For S2 and S9 through S18, only the first half exists. **The gap is total, not partial.**

## The measurement

    $ grep -o "S[0-9]\+" scripts/drive/*
    S0 S1 S3 S4 S5 S6 S7 S8

and `scripts/drive/stories.sh` s verbs are only `run`, `run-s3s4s6`, `run-s5s8`, `run-s7`. No beat file mentions any S2, S9, S10-S18 test id. Beat files present: `beats_s3res.sh`, `beats_s3s4s6.sh`, `beats_s5.sh`, `beats_s6.sh`, `beats_s7.sh`, `beats_s8.sh`.

STORIES.md s test-map table also stops at S9 — S10+ have no declared test-map row at all, which matches the harness gap exactly.

## The rungs, and that they are declared LANDED

docs/STORIES.md "Priority (this epic)", verbatim:

> S10+ are the VISION rungs, graduating one at a time as their enabling verbs land — S10 (Historian), S11 (Auditor), S12 (Counterfactualist), S13 (Admiral), S14 (Teleoperator), S15 (Warden), S16 (Releaser), S17 (Warden, again) and S18 (Admiral, armed) are here, and none of them needed a verb below yog

So these are not future work; they are shipped surface with a half-empty done-bar.

| Rung | Title | Story test ids | In-crate carriers (examples) |
|---|---|---|---|
| S2 | Director: point the conversation at a directory | S2-T1 | `tests/integration/stories_s2_t1.rs` (literal id) |
| S9 | Settler: the only thing you installed is yog | S9-T1..T4 | `src/world/tools/tests.rs`, `src/world/hatch.rs` (no literal id) |
| S10 | Historian: the step spine is a commit spine | S10-T1..T6 + T3b | `src/rail/tests/build.rs`, `src/rail/tests/pin.rs`, `src/transcript/tests/spine.rs`, `src/rail/tests/tree.rs`, `src/inspector/tests/pinned.rs` |
| S11 | Auditor: what the agent actually changed | S11-T1..T4 | `src/workdiff/tests/{plan,read,paint}.rs`, `src/boundary/reply/tests/workdiff.rs` |
| S12 | Counterfactualist: try it again from here | S12-T1..T6 | `src/fork/tests/{argv,choices,composer,paint}.rs`, `src/rail/tests/cohort.rs` |
| S13 | Admiral: the balls section is the board | S13-T1..T8 | `src/board/tests.rs`, `src/board/tests/{fixture,rollup}.rs` (no literal ids) |
| S14 | Teleoperator: yog without the window | S14-T1..T8 | `src/boundary/answer/queue/tests.rs`, `src/boundary/consume/tests.rs:155`, `src/engine/tests.rs` |
| S15 | Warden: nothing runs unadjudicated | S15-T1..T5 | `tests/tool_control_shim.rs` (literal S15-T1), `src/control/{bash,root,judge,author}/tests.rs` |
| S16 | Releaser: a parked drone is seen, and answered | S16-T1..T6 | `src/boundary/control/tests/mod.rs`, `.../confinement.rs`, `src/control/policy/tests.rs` |
| S17 | Warden, again: a drone that drifted stops acting | S17-T1..T5 | `src/boundary/control/tests/floor.rs`, `src/boundary/codec/tests/control.rs` |
| S18 | Admiral, armed: the loop is facts, every move a row | S18-T1..T6 | `src/fleet/facts/tests.rs`, `src/fleet/pilot/tests.rs`, `src/fleet/row/tests.rs` |

## Why this is worth a ball rather than a shrug

The 2026-08-07 drive found FOUR stale beats across the rungs that ARE driven (bl-afa7, bl-2d45, bl-00ee, bl-52c7) — every one of them a surface that moved while only `scripts/drive/` stayed behind. Rungs with no beats at all cannot even produce that signal: their surfaces can move arbitrarily far without anything going red. docs/QUALITY.md §4 s currency rule only expires verdicts that exist.

## Scoping note, not a plan

Many S13-S18 rungs are headless/boundary rungs (`S14-T8 a-windowless-engine-answers`, the `three-spellings` ids that recur at S12/S13/S14/S16/S17/S18). Those need NO seat and NO window — `stories.sh` already owns that transport in its `gesture` helper ("It needs NO SEAT: a boundary gesture is named, not aimed"). A boundary-only drive verb is therefore much cheaper than the windowed rungs and is the obvious first slice. Also relevant: bl-56d5 (`make drive`: one verb for the whole ladder, a structured per-beat verdict, and a host preflight) is live and this ball should be sequenced with it rather than against it.

Attack the scope before committing: the ask is coverage of the graduated rungs, not one beat per test id.