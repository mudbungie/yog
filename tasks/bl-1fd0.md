+++
title = "the start pane invites a goal that cannot possibly run: on a wall with no usable provider, the first rung must be provider sign-in, not a text box"
created = 1787548547
updated = 1787548558
claimant = "Grommet"
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
## Operator ruling

The start pane opens on a fresh workspace with a goal box and Send. On a wall
holding no usable provider credential, typing a goal and hitting Enter will
work **zero percent of the time** — the conversation is born, immediately dies
on no-models, and the operator learns it from a dead row (or from nothing at
all). The pane is inviting the one act that cannot succeed while hiding the one
act that must come first. **The start flow must begin with provider
registration when the wall has none.**

This was hit live, twice in one evening: once on a fresh local wall, once on a
freshly founded remote workspace. Both times the operator's first typed goal
was wasted.

## What exists to build from — this is wiring, not invention

- The pane already has rungs (seed / workspace / ball, §11's start flow) — a
  provider rung is a new first rung, not a new surface.
- The provider table is an in-process read:
  `BzRunner::providers` (`src/config_edit/brazen/`) yields the effective rows
  with their auth state (`stored` / `missing` / `not required` /
  `login_blocked`), per wall. "This wall can run something" is a pure predicate
  over that table: any row `stored` or `not required`.
- The Login affordance exists (`src/shell/login_pane.rs`, §8.3): per-provider
  buttons, streamed sign-in, refusal rows for rows that cannot sign in. The
  rung REUSES it aimed at the start's target wall — one capability, not a
  second login.

## Shape

- Pane opens; the target wall's provider predicate is read.
- **No usable provider** → the first rung is the provider roster with Login
  buttons and one sentence saying why the goal box is not yet the point. The
  goal box may stay visible (draft freely) but Send states the real blocker
  rather than firing a doomed start.
- **Usable provider** → today's flow, untouched. The predicate is per-wall, so
  a box signed into `ops` still gets the rung on a fresh wall.
- On sign-in success (the streamed run's outcome), the rung dissolves and the
  goal box takes focus — the operator's typed draft intact.

## Boundaries and cautions

- **Remote walls**: the provider read and the login act must aim at the wall
  where agents run. For a remote workspace that is bl-61bf's design (in
  flight); this ball lands the LOCAL arm and paints the remote case honestly
  (the rung shows the host wall's state if readable, else says it cannot yet
  read it) rather than lying with the local table. Do not block on 61bf; leave
  the seam it will fill named in a comment.
- The predicate must not add a network round trip to the pane — `providers` is
  an in-process table read; keep it that.
- §8.3 rule 4 stands: a keyless/api-keyed row gets no button, only the reason.
- Acceptance coverage: fresh wall → rung painted, goal Send refuses with the
  reason; signed wall → rung absent, flow byte-for-byte today's; sign-in
  completing mid-pane → rung dissolves, draft preserved.