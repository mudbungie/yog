+++
title = "run_s7's S6 stop beat asks an already-settled conversation to stop, so x correctly dispatches nothing — a fixture/ordering defect, NOT the wall and NOT the selection"
created = 1786515941
updated = 1786601813
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
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