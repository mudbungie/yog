+++
title = "multiple armed projects have no world-level concurrency or whole-day spend ceiling"
created = 1787206353
updated = 1787207265
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["missing", "design", "agentic"]
+++
Each armed fleet fact names one workspace, one project and one cap. Multiple projects require multiple independently armed workspaces. Each planner enforces only its own cap.

Spend ceilings are also workspace-local; the architecture says the compared figure is the target workspace's. A nominal ceiling can therefore be consumed once per workspace. There is no world/portfolio limit on concurrent drones, total tokens, wall time or spend, and no ordering across projects.

For a whole-day operator this is the safety boundary above every individual queue: starting several fleets must not multiply an intended global allowance silently.

## Required design result

Decide whether yog owns a world-level cap/budget or whether an external supervisor is explicitly the only authority. If yog owns it, derive usage from existing durable facts rather than storing counters, define precedence with workspace ceilings, and drive simultaneous projects proving the total never exceeds the declared world allowance.

---

## Verified at HEAD: the premise holds, with two corrections

**1. Arming really is one workspace, one project, one cap.** `cadence.yaml`'s
`fleet:` block carries one entry per workspace path with `project:` and `cap:`
both required (`src/fleet/arming.rs`: *"An entry with no `project:` or no
readable `cap:` is **not armed**"*) and `lease_min:` optional, absent meaning
never reap. Two projects are two entries and two independent caps. Note that
`cap` counts **claimed balls**, not drones and not money.

**2. The dollar ceiling is one world-global number compared against a
workspace-local figure.** This is the ball's sharpest point and it is exact.
DESIGN §4.1: *"`ceiling` — the §3.5 spend ceiling: one number, **USD**, the
bound a workspace's own spend must stay under for yog to start a *new*
conversation in it."* DESIGN §3.5: *"**The figure it compares is the target
workspace's**, the same `Attribution::Workspace` sum this section already
accepts as a ball's honest upper bound: a workspace is the sphere a drone lives
its ball in, and it is the one scope a spawn names outright without inventing a
linkage fact nobody stores."* In code, `Ceiling::refusal(workspace, prices)`
folds `spend::of_workspace(workspace, prices)` and nothing wider. So: one
operator number, N armed workspaces, N times the intended allowance.

**3. Correction — the world is neither unordered nor unrated.** There is ONE
pilot thread for every armed workspace, and it makes **at most one move per
tick, world-wide**: `PilotCtx::pass` iterates `board.fleet` and `return`s on
the first workspace whose `plan` yields a move. The order is `cadence.yaml`
entry order, and a workspace at its cap yields the tick (`Facts::has_room` is
`count < cap && ceiling.is_none()`, and `plan`'s spawn arm returns `None`
without room), so it is a priority rather than starvation. The tick is the
full-sweep cadence, 15 s by default, so the whole world births at most ~4
drones a minute however many projects are armed. What is genuinely absent is a
**standing** bound on the total — not a rate bound, and not an ordering.

**4. Correction — the existing ceiling is not a daily allowance, and
re-scoping it would not make it one.** The compared figure is the workspace's
**lifetime** priced spend: `budgets::bills` over the whole workspace, folded by
`spend::figure`. Nothing windows it and nothing resets it. Today's ceiling
already means *"this sphere has spent enough, ever"* rather than *"enough
today"*. A whole-day ask is therefore a **second denominator**, and that — not
the scope — is where the drift risk in this ball actually sits.

**5. Nothing else bounds the world.** VISION V4's burden check is *"fleet mode
is armed per workspace and off by default; unarmed, the board is today's balls
section"*; no V-rung proposes a portfolio bound. And since bl-56af the §3.5
dollar ceiling is the *only* ceiling a yog-dispatched conversation has — §8.6's
workflow fixed point strips lernie's whole-tree `budgets:` block from every
workspace's `config/default`, so there is no token-denominated bound underneath
it either.

## Does §3.5's ceiling generalize? Yes, and it costs no mechanism

The scope of the **comparison** is the only thing that would change. The gate
has exactly one seat — DESIGN §3.5: *"Every drone yog ever births is fired by
that one function… **There is no second gate anywhere**, which is the whole
point of seating it at a chokepoint"* — so there is one call site to re-aim.

A world figure is a query over facts yog already publishes, not a counter. The
snapshot carries per-workspace bills, and `spend::figure` is already `pub` for
this exact reason: *"**Public because a rollup crosses workspaces** (§3.5,
bl-9dd4): the board selects one slice per workspace, concatenates them, and
folds the whole here — one fold"*. A world total is that concatenation taken
over every workspace instead of over one ball's members. No stored counter, no
new durable, no new config key, and severability is untouched — delete
`ceiling` or delete `prices` and the gate is gone, exactly as today.

## The minimal non-drifting shape

**Re-scope the number that exists; do not add a second one.** `ceiling` stops
meaning "one workspace's spend" and starts meaning "everything this world has
spent". One key, one comparison, one gate, one home — and the multiplication
this ball names disappears by construction rather than by a second rule
policing the first.

The cost is real and belongs in the ruling: an idle workspace is refused
because a busy one spent, and an operator who wrote `25` meaning *per sphere*
now has a world allowance of 25. Because the figure is lifetime-cumulative, a
long-lived world eventually latches the gate for good; the remedy is raising
the number, which is the remedy it has today, arriving sooner.

**Deliberately not proposed:**

- *A per-workspace ceiling AND a world ceiling, with precedence between them.*
  That is two ceilings over one concern — the exact shape bl-56af's ruling
  deleted. If both are ever wanted, the second is a new fact needing its own
  ruling, not a derivation off this one.
- *A world-level concurrent-drone cap.* Sum-of-caps is already arithmetic over
  entries the operator wrote themselves, the birth rate is already bounded at
  one per full sweep, and a drone count is a poor proxy for the thing being
  protected when yog can price the money directly.
- *A day window.* It needs a clock origin, a reset and a stored or derived
  boundary — a genuinely new fact, and the first thing here that would not be
  severable by deleting a key.

## This needs an operator ruling before any code

Three questions, none of which yog should answer for itself:

1. **Should `ceiling` become world-scoped?** It changes the meaning of a number
   already sitting in `ui.json` on live boxes. (Recommendation: yes — it is the
   only shape that adds no second ceiling.)
2. **Is a lifetime-cumulative bound the intended semantics at all?** That is
   what ships today at workspace scope, and this ball's "whole-day" framing
   suggests it may have been read as a rate. If a window is wanted, it is a
   different ball and a new durable fact.
3. **Or is an external supervisor explicitly the only authority?** The ball
   offers this outright. If so the deliverable is a sentence in DESIGN §3.5 and
   this ball closes with no code: yog already refuses to invent a token proxy
   for dollars it cannot price, and refusing to invent a portfolio policy is
   the same discipline.

No mechanism written and nothing claimed; this comment is the design result the
ball asks for, pending the ruling above.
