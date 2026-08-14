+++
title = "run_s7's S6 stop beat asks an already-settled conversation to stop, so x correctly dispatches nothing — a fixture/ordering defect, NOT the wall and NOT the selection"
created = 1786515941
updated = 1786683393
claimant = "Ingot"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["drive"]
+++
**THIS BALL'S ORIGINAL DIAGNOSIS WAS WRONG, TWICE OVER. Do not implement the
title.** Two workers have independently falsified it with baseline re-runs;
what follows is the corrected reading.

## The title's claim is retired

The title says `run-s7`'s first conversation dispatches before `seed_wall`, so
its first prompt resolves providers against an empty wall. That *was* true, and
**bl-1851 fixed it** (`c60ed08`): `seed_wall` is now keyed by §3.1's bootstrap
name constant `home` and laid with the world seed, before the launch. This
ball's primary red — `S7 fixture: wire reply on disk` — went red→green with
that delivery. That half is done and needs no further work.

## The `S6 stop` half was never caused by the wall

**Falsification 1 (bl-1851's worker).** Restoring the pre-fix scripts and
re-driving on the same binary left `S6 stop` red on *both* baselines, so the
wall neither caused nor cured it. `s6-07-inflight.png` showed the selection
sitting on the laid child, not the root the predicate names.

**Falsification 2 (bl-5cce's worker), which also retires the selection theory.**
Deriving the coordinates and landing the gestures correctly did **not** fix it,
so the selection was not the cause either. The actual reason:

`stop_enabled` (`src/actions/mod.rs:170`) requires the selected agent's state to
be `Live | InFlight`. World C's root settles long before `s6_attention` runs —
the beat waits on `reply_exists`, lays fixtures, then runs three more beat
groups — so `x` correctly dispatches nothing. `ops.jsonl` for the run holds
**zero** `stop` rows.

## What this actually is

A **fixture/ordering defect in `run_s7`**: the beat asks a conversation that has
already settled to stop. It is not a selection defect, not a wall defect, and
not a yog defect — yog is behaving correctly. `run_s3s4s6`'s copy of the same
beat passes precisely because its root is genuinely in flight when the beat
fires.

## The work

Re-seat the `S6 stop` beat in `run_s7` so it fires against a conversation that
is actually in flight, or move it to a world that has one. Whoever takes this
should first confirm the above against a fresh baseline rather than trusting
this body — it is the third diagnosis this ball has carried.

Related: bl-1851 (delivered), bl-5cce (delivered), and the standing lesson that
a beat whose only assertion is negative passes when its gesture lands on
nothing.

---

## RE-DIAGNOSED FROM SCRATCH ON THE MERGED TREE (Ingot, 2026-08-13). Fourth and final reading: THE DEFECT DISSOLVED. Closing with NO CODE CHANGE.

The third diagnosis was right about the mechanism and wrong about the culprit. It said: world C's root settles long before the stop beat fires, stop_enabled (src/actions/mod.rs) wants Live|InFlight, so x correctly dispatches nothing and ops.jsonl holds zero stop rows. All true. What it could not see is that RUN_S7 WAS NEVER SUPPOSED TO FIRE A STOP BEAT AT ALL.

## The proof, from the source at three points in history

bl-2d45 (7a91ed69, 2026-08-11) moved the stop-and-ack stage out of run_s3s4s6 into beats_s6.sh and named it s6_attention — a name that file's world-C S6-T1 stage already had. Every beats_*.sh is sourced into ONE flat bash namespace and the later definition silently wins. Function definitions in beats_s6.sh:

    7a91ed69^  (before bl-2d45)     s6_attention:26  s4_overflow:56  s4_uncoloured:90  goal_stamped:114  s6_converges:122
    7a91ed69   (after  bl-2d45)     s6_attention:31  s4_overflow:61  s4_uncoloured:95  goal_stamped:119  s6_attention:144  s6_converges:180
                                    ^^^^^^^^^^^^^^                                     ^^^^^^^^^^^^^^^ collision

beats_s7.sh has called 's6_attention "$wid" "$out" "$agent"' since before bl-2d45 (line 132 then, 123 now) and that call has never changed. Before bl-2d45 it reached the S6-T1 budget/conflicted/mail stage. After bl-2d45 it reached the stop-and-ack stage instead — handed world C's settled root as its $3 'in-flight' argument. THAT is the beat that went red, and it went red for exactly the reason the third diagnosis gave.

## Why the ordering fix the body asks for would have been wrong

The body's work item is 'Re-seat the S6 stop beat in run_s7 so it fires against a conversation that is actually in flight, or move it to a world that has one.' It is already in a world that has one: run_s3s4s6 lays two roots and hands the stage the in-flight one, which is why the body itself observes that run_s3s4s6's copy passes. Building a second in-flight fixture inside world C would have added a duplicate beat to satisfy a symptom that had no cause.

## What actually landed the fix

bl-0e44 (844ffd45, on main), which I closed immediately before this one. It renames the newcomer to s6_stop_ack, points run_s3s4s6 at it, leaves run_s7's call resolving to the real S6-T1 stage, and adds one_name_one_definition (harness.sh) so stories.sh refuses to run at all on a duplicate top-level beat name — mutation-tested both directions. On the merged tree the labels 'S6 stop' and 'S6 ack' now exist only inside s6_stop_ack (beats_s6.sh:160), which only run_s3s4s6 calls. run_s7 emits no S6 stop beat. There is nothing left to observe: the red row cannot be produced.

## Evidence I did NOT take, and why

No live ladder run. run_s7 no longer contains the beat, so a drive could only confirm an absent row — it cannot distinguish 'fixed' from 'deleted', which is precisely the blindness bl-0e44 exists to close. The source-level fact above is stronger than the absence a drive would report.

## Found but not fixed, filed as bl-1061

The flip side of the collision: for two days run_s7's three S6-T1 beats (budget, conflicted, mail) DID NOT RUN, and a beat that never runs writes no verdict row, so every ladder in that window said ALL BEATS PASS about them. bl-0e44 restored them unexercised against a yog many deliveries newer than their last real execution. Their first run is unproven and any red there is fresh drift, not this ball.
