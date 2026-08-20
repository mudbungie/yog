+++
title = "the whole-tree ceiling nobody set: workspaces born before lernie's template fix froze budgets: into config/default, so a yog-dispatched conversation dies against a cap invisible everywhere yog looks"
created = 1787206006
updated = 1787206007
claimant = "Zircons-Budget"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
## What the operator hit

A conversation dispatched from the window dies against a ceiling the operator
never set and cannot see. Ruling: eliminate the conversation budget entirely —
the default must be no cap — and a cap must never again be invisible.

## Where the number is

lernie's `workflow.yaml` carries an optional `budgets:` block (lernie ARCH §6):
`max_total_tokens`, `max_wall_seconds`, `max_depth`. It is a **whole-tree**
consumable, not a per-agent allowance — a root and every subagent below it
spend one shared allowance — and every axis defaults to None = unbounded in
lernie's own config type. So the cap is SET, never defaulted.

It is set by lernie's **shipped template**, as it stood before lernie bl-8dea.
The seeded block reads:

    budgets:
      max_total_tokens: 2000000
      max_wall_seconds: 3600
      max_depth: 4

One hour of accumulated whole-tree wall, and a dispatch tree four deep. Both
bind on ordinary work: a fan of subagents crosses depth 4 by its third hop, and
an hour of tree-wall is a morning's chat.

lernie already ruled the same way and fixed it upstream (bl-8dea, "ship the
template with budgets off by default"), released in 0.0.11 — the pin yog
already carries. So **nothing yog dispatches into a fresh workspace is born
capped.** What is still capped is every workspace born BEFORE that release: a
workspace's `config/default` is a committed git lineage, so the old seed is
frozen there forever, and every agent forked off it inherits the ceiling. That
is not a lernie defect any release can reach; a template only ever reaches
workspaces born after it.

Evidence that it fires: a long-lived workspace's repo carries dozens of
`refs/lernie/budget-exhausted/<agent-id>` marks, concentrated on the deepest
descent ids — the `max_depth: 4` axis, mostly.

## Why yog is the right place to fix it

yog already converges `config/default` at every start, for exactly this reason
and with exactly this argument (DESIGN §8.6): the §8.6 `tool_control:` block
and §3.7's `instructions/**` manifest glob are authored onto the workspace's
own committed files by a fixed-point transform, driven through the one lawful
writer of `config/*` (the §9.3 `lernie config` drive). §8.6 states the premise
in words: the template would only have reached workspaces born after it, while
this reaches every workspace on its next start. The stale ceiling is the same
shape of problem and wants the same instrument.

So: `control::author::authored` — the one fixed point over `workflow.yaml` —
also strips a top-level `budgets:` block, and authors one comment line saying
it did. Same file, same transform, same drive, no second drift entry, no new
artifact, no state. Fixed point holds: a file with no `budgets:` block strips
to itself.

## Why the strip is unconditional, and what that dissolves

yog does not preserve a smaller-but-still-present default and does not
condition on the seeded values: it holds `budgets:` unbounded outright, the way
it already owns `tool_control:` outright. Two reasons, and the second is the
architectural one.

1. A whole-tree token/wall/depth ceiling **kills a conversation that is
   working**. Early termination is the expensive failure — it destroys
   uncommitted work — which is the identical argument DESIGN §3.5 already makes
   for yog's own ceiling: the ceiling gates spawns and never kills a running
   drone.

2. **yog already has an operator ceiling, and two representations of one fact
   drift.** §3.5's spend ceiling is the `ui.json` `ceiling` key: denominated in
   dollars (the unit an operator actually reasons in, not tokens or
   tree-seconds), absent by default so deleting the key deletes the gate,
   refusing a *birth* rather than killing a live drone, and already spoken on
   the V4 board with the gate's own words ahead of the spawn it will bind.
   lernie's `budgets:` is a second ceiling over the same concern that is worse
   on every axis and visible on none.

That is what makes the second half of the complaint — that no budget shows up
in the agent's config tab as an option — dissolve rather than become a feature.
With the strip in place there is no per-conversation budget for a yog-dispatched
agent to surface: offering a control for a value yog deletes at the next start
would be a knob that lies. The ceiling the config surface owes the operator is
the one that survives, and it has a home already.

Recorded for the record, since it was the alternative design: surfacing
`budgets:` as typed §9.5 rows was costed and is NOT wanted here, but it was
also not cheap. The §9.5 form grammar is three-level — block, entry, field
(`models:` to an id to `context_window`; `cadence:` to `watcher:` to
`debounce_ms`) — and `budgets:` is two-level, block to field. It would need a
new flat-block schema kind in the grammar, plus an affordance the pane has
never had: `read` emits no row for a field the file does not declare, so an
OPTIONAL setting whose absence is the default cannot be offered at all without
an add-the-block and remove-the-block path. Mechanism for a knob that should
not exist.

`workflow.yaml` is still browsable as raw text in the config tab like every
other file in the governing commit, so nothing is hidden from an operator who
wants to read what their agents run under.

## Deliberately out of scope

`manifest.yaml`'s `roles.*.budget_tokens` (150000 worker / 50000 compactor) is
a different thing wearing a similar word: lernie's context-assembly budget,
governing which files fit into an assembled context, not a ceiling that ends a
conversation. It is still shipped by lernie 0.0.11's template and is untouched.

## Done when

- `control::author::authored` strips a top-level `budgets:` block from
  `workflow.yaml` and authors the note; the transform is a fixed point over
  both blocks together.
- A workspace whose `config/default` carries the stale ceiling converges to one
  that does not, on its next start, through the existing single `lernie config`
  drive — no new drift entry, no new ops row.
- DESIGN §8.6 records the second thing that fixed point holds and why the
  config-tab option is dissolved rather than deferred; §3.5's ceiling is named
  as the one that survives.
- Tests, 100% coverage, 300-line cap.