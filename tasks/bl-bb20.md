+++
title = "coverage debt: S2 and S9-S18 are graduated rungs with in-crate tests and ZERO real-substrate drive beats"
created = 1786162702
updated = 1786684053
claimant = "Ingot"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
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

---

## Scope attacked first, then built and RUN (Ingot)

The ball says the ask is coverage of the graduated rungs, not one beat per test id. So: four rungs in, five out, every decision recorded in the code that carries it.

### The criterion I judged against

A drive beat earns its keep only where the REAL substrate can falsify something a fake cannot — real balls state and its blocker resolution, real git, real lernie's on-disk shapes, the real window, the real wire. Where a rung's claim is a derivation over yog's own structures, the in-crate test proves it exactly and a drive beat re-runs the same logic through a slower door, then rots.

### IN — scripts/drive/beats_headless.sh, the SEATLESS verb

The ball's own scoping note called it: a boundary-only verb is much cheaper than a windowed one and is the obvious first slice. It is cheaper than the note knew, because 'yog headless' exists — 'the same engine with no window: worker, watcher, gesture consumer' — so 'run-headless' claims NO X display, opens NO window and spends NOTHING on the wire. It is the only run verb drivable on a box with no X server, and it now runs FIRST in the ladder (drive.sh): a world broken in its own shape reddens in seconds instead of after four windowed runs.

Ten beats, four rungs:

    S14-T8 windowless engine: a deposit is consumed and answered   PASS
    S2-T1  path-rung: the prepared cwd is the named directory      PASS
    S2-T1  path-rung: preparing a directory spawns no bl           PASS
    S13    fixture: the ball is bound to the workspace             PASS
    S13-T4 board: the join binds the claimed row only              PASS
    S13-T2 board: an unclaimed ball sits in ready                  PASS
    S13-T2 board: a bound ball sits in claimed, naming its claimant PASS
    S13-T3 board: a live blocker puts its ball in blocked          PASS
    S11-T4 work-diff: the agent's commit is the ball's diff        PASS
    S14-T5 an answer aimed at nothing refuses                      PASS
    ALL BEATS PASS

- **S13 (board)** — three balls whose STORED facts put them in three columns, and the blocked one's blocker is a LIVE ball, so 'blocked' is balls' own resolution of a real edge, not a field yog could have read. The join half asserts BOTH directions (the bound row names its workspace, the ready row names none) because either alone is satisfied by a board that never joined.
- **S11-T4** — the only rung here whose subject is git: a real 'bl claim' cut a real worktree off main, a commit lands in it, and the query is asserted by identity at every field (the file's name, its +1/-0, both refs, both oids, and that they differ).
- **S2-T1** — the prepared cwd IS the named directory, plus the rung's negative clause (naming a directory is not a ball, so no bl spawns).
- **S14-T8/T5** — the verb's own premise, asserted rather than assumed, and the refusal.

### Every beat MUTATION-PROVED to bite

Not one of these is a beat I hope is real. Each was reddened by breaking the world or the gesture, then restored:

    drop --needs from the blocked ball        -> S13-T3 FAIL nothing blocked
    skip the agent's commit                   -> S11-T4 FAIL no matching diff row
    bare /prepare instead of /prepare dir     -> S2-T1  FAIL cwd is not the dir
    assign --as somebodyelse                  -> S13-T2 FAIL wrong column or claimant
    /seen predicate asked for another name    -> S14-T5 FAIL refused without naming the agent
    assign the READY ball too                 -> S13-T2 FAIL bl-934b is not ready
    assign both balls                         -> S13-T4 FAIL workspace on the wrong rows
    a bl verb between the two ops-row reads   -> S2-T1  FAIL a bl verb fired
    yog headless replaced by a sleep          -> S14-T8 FAIL no engine in 40s

Then three consecutive clean runs, all ten green.

### Two defects the beats found in themselves before they landed

1. **A RACE, caught by the beats going green then red on identical trees.** The board's workspace join lands AFTER 'bl claim' writes balls' store — yog's derivation has to see it — so a single read raced it: run one passed, run two reddened with 'workspace: null' on a row whose claimant already said 'home'. Fixed the harness's own way: '/board' is a pure read and the predicate is monotone, so it is 'await'ed, which is exactly what that primitive's contract asks for. A flaky beat is worse than a vacuous one; it teaches people to re-run.
2. **A leaked engine.** An aborted mutation run left a parked 'yog headless' holding a scratch world — 'set -e' can end the file between the boot and the kill. The teardown is now a trap paired with the boot, the way 'verdict' is paired with 'claim_seat'. Also: 'yog gesture' waits for a consumer with NO deadline of its own, so an engine that never boots would HANG the run rather than redden it; the boot probe is timed out and a failed boot ends the run with a verdict row.

### OUT, and why — these are decisions, not omissions

- **S9 (Settler) — already driven, so the ball's 'ZERO real-substrate beats' is wrong here.** STORIES.md says outright that the W14 clean room 'is this rung's own drive', and beats_s8.sh's S8-T2 already asserts the nested-clone half. S9-T3's '--as $YOG_NAME' stamp is set ONLY for an agent's tool subprocess (actions/verbs/bound.rs); world/seat.rs says the hatches carry no YOG_NAME on purpose, and I verified it: 'yog exec --ws W sh -c echo $YOG_NAME' prints empty. Driving T3 therefore needs a real agent really running a real tool — wire spend and agent nondeterminism for a fold multiplex::bl tests exactly. S9-T4 was discharged by DELETION (W13), so there is nothing to drive.
- **S10 (Historian) — BLOCKED, not skipped.** The rail, transcript, steps and files surfaces have no headless spelling at all (bl-6233), so an S10 beat could only take a screenshot, and a screenshot proves nothing about a spine. This ball cannot be finished for S10 until bl-6233 lands.
- **S12 (Counterfactualist) — the N>1 fan is bl-8746, open and blocked.** A beat written against a half-landed mechanism is a beat that will be rewritten.
- **S15/S16/S17 (Warden, Releaser) — a real adjudication needs a real agent really attempting a real tool and then a real decision:** wire spend, nondeterminism, and the response ladder is bl-02c2, still open.
- **S18 (Admiral, armed) — the armed loop is not live.** Its unarmed clause, 'S18-T1 unarmed-is-today's-board', is exactly what the S13 beats above now assert on the real substrate.
- **S14-T1..T4 (the attention queue) — the right world is run_s3s4s6, not here.** They need a LIVE conversation waiting on the operator, which needs a model call. run_s3s4s6 already lays two real conversations and already reads the same ui.json watermarks from the WINDOW side; the line's read of that same fact is a 4-line addition there and closes a genuine two-writers-one-file seam. I did not add it, on purpose: I cannot execute run_s3s4s6 (seat + wire), and this cluster's whole lesson is that an unexecuted beat is a beat nobody knows is broken. It is the obvious next slice and it is cheap.

### Doc changes

DESIGN §12.2 gains a beats_headless.sh row and the corrected counts; STORIES.md's real-substrate section gains the paragraph that says not every run verb drives a window, and points at the file's own head for the rung-by-rung scope record. The scope decision lives in the code it governs, not only here.

### Found, not fixed

- **'yog gesture' blocks forever when no engine consumes the deposit.** I bounded it in my own boot probe, but the shared 'gesture' helper (stories.sh) has no deadline, so a yog that dies mid-run hangs every windowed verb too instead of failing. One 'timeout' in one helper; I left it because I cannot execute the windowed verbs to check the change.
- **The bootstrap workspace has no headless mint.** '/prepare' with no workspace in context refuses ('focus one, or use the envelope'); naming a not-yet-existing '--ws <path>' is what mints it, which is what this verb does. Worth knowing, and adjacent to bl-9b52.
