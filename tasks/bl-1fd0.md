+++
title = "the start pane invites a goal that cannot possibly run: on a wall with no usable provider, the first rung must be provider sign-in, not a text box"
created = 1787548547
updated = 1787549232
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

---

**One deviation from the ruling's predicate, and why.** The ruling's stated test
— *"any row `stored` or `not required`"* — is vacuous as written. brazen merges
its built-in table under **every** config there can be, so `ollama` and
`claude-code` read `credential = "not required"` on every wall, including a bare
one and including the table this box actually carries. A predicate they satisfy
is a predicate nothing ever fails: the rung would not have appeared on the wall
the ruling was filed from. Verified against a real `bz --list-providers --json`
with an empty config home — five keyed rows `missing`, `ollama` and
`claude-code` `not required`, the built-in oauth row `missing`.

Nor is a keyless row what a doomed start was routed to: both claim no model
prefixes and are reached only by an explicit `--provider`, so a start whose role
names an uncredentialled row dies exactly as described with both of them sitting
in the table.

So the shipped predicate is **a row carrying a credential a run would spend**,
read off brazen's own `credential` column (its `fetch_cred` answer minus the
network) rather than off a second file-existence read. `stored`, `ambient`,
`inline` and any spelling this build cannot read all ready the wall — the last
because refusing on an unanswered question would block a working setup — and
only `missing`/`not required` do not. The keyless rows still change what the
rung *says*, so an operator looking at two rows marked "no credential needed" is
told why they do not count.

**What the gate still cannot see, and did not pretend to:** *which* row a start
routes to. That is `roles.<r>.provider` on the config branch — several git reads
and §9.4's subject — so the gate judges the wall, not the route. Exactly right
when nothing at all is signed in; conservative for an operator whose roles all
name a keyless row, who is told to sign in when their setup needs no sign-in.
DESIGN §8.1 carries all of this.

**Filed out of this ball:** bl-fef7 — the rung guards the start pane only, and
the docked message composer and the empty-world bootstrap box fire the same
doomed start through §3.4's bare rung. That is probably the seat the live
incident used.
