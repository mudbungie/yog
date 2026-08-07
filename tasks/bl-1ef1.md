+++
title = "capability default to 11: open-world passes; only loss and credentials refuse"
created = 1785997673
updated = 1786064433
claimant = "Sluice"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator complaint, 2026-08-05: *"auto-approval should be on by default. I don't
want to approve everything, and neither does the power-user who is using yog.
it's a good knob, but turn it to 11 by default."*

## What ships today

`src/control/judge.rs` `Table::ruling`:

    Effect::Read | Effect::TargetWrite | Effect::Process => Ruling::Pass,
    Effect::OpenWorld                                    => Ruling::Hold,
    Effect::Destructive | Effect::Secret                 => Ruling::Refuse,

with the doc comment *"Shipped safe for an unattended drone: the three classes
that are the job pass, an effect that leaves the world parks for an answer, and
loss or credentials are declined in band."*

`OpenWorld` is what parks the operator constantly — every `python`, every fetch.

## The ruling

**`Effect::OpenWorld` ships `Ruling::Pass`.** Destructive and Secret keep
refusing: the operator's words were "turn the knob to 11", not "delete the
gate", and loss/credentials are the two classes an unattended drone must not
decide for itself. So the shipped table becomes: everything passes except loss
and credentials, which refuse in band.

Nothing else moves. In particular:

- The **workspace override still exists** — `capability.yaml`'s `table:` block
  (`src/control/policy.rs`) already spells `open-world: hold`, so an operator
  who wants the old behaviour writes one line. Severability holds in the
  direction it must: absence is the (now permissive) default, and the file is
  the override.
- The **monitor's revoke rung still works** — the per-conversation floor
  (`src/boundary/control/floor.rs`) *raises* whatever the table said and never
  lowers it, so a floored conversation still holds every class above read.
  That is the safety story after this change: hold is no longer standing
  policy, it is what the alignment monitor imposes on a conversation that has
  earned it.

## Work

1. Flip the arm in `src/control/judge.rs`, and rewrite the `Table` doc comment
   — it currently *argues* for the parked default, and a stale rationale is
   worse than none. Say what the new table is and why the floor carries the
   weight now.
2. Fix the tests that pin the old verdict (`src/control/tests/policy.rs` and
   siblings). A test asserting `open-world → hold` from the *shipped* table
   becomes a test asserting it from an *override*; do not delete the coverage.
3. Docs are the authority here, so the doc edit is part of the ball, not
   after it: `docs/DESIGN.md` §8.6 and `docs/VISION.md` §4.11 item 4 both state
   the shipped table. Amend both, and say the rationale moved to the floor.
4. Check the boundary help table (`src/boundary/help/table.rs`) and any
   `--skill`/help prose that quotes the shipped verdicts.

Verify before editing: this body was written from a read of HEAD on 2026-08-05;
if the table has moved since, fix the ball, then do the work.