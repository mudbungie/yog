+++
title = "per-agent task branches: each agent tracks on its own balls branch by default, settable at launch, inherited by subagents, amendable by the agent"
created = 1786508785
updated = 1786509919
claimant = "Bract"
priority = 1
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-9dde"
on = "claim"

[[blockers]]
id = "bl-c0e2"
on = "claim"
+++
Operator ruling 2026-08-11: "The balls interface is broken right now. The
default is going to be that each agent gets its own balls branch for tracking,
but that an agent can have its branch set at launch, and that subagents, by
default in yog, get their parents' space passed to them. Obviously an agent can
amend their own branch, to change their config (necessary when an agent is
launched but told to work on a project)."

Implement the amended DESIGN §16.3: per-agent tracking branch as the default,
a launch-time branch parameter, parent-space inheritance for subagents, and a
runtime path for an agent to amend its own branch. The bl conf surface
(world::marks) is the existing write path — drive it, don't duplicate it.

Also: "the balls interface is broken" is an observed failure, not a metaphor —
before building, exercise the current balls board/section against a real store
and file (or fix here, if in scope) what is actually broken. Verify every
premise against the tree; the world env composition will have just moved under
the blast-radius ball.

---

BROKEN-BOARD INVESTIGATION (run before building, per the ball) — root-caused and
reproduced against scratch worlds under /tmp, never the live world.
`make drive-preflight` green first.

**Finding 1 (the big one): yog's world nests balls' STATE but not balls' CONFIG,
so every balls checkout yog founds is silently remote-less.**

DESIGN 16.2's override set is LERNIE_HOME / XDG_STATE_HOME / PATH.
XDG_CONFIG_HOME is left ambient — and balls reads TWO things from it
(layout::Xdg): $XDG_CONFIG_HOME/balls/config.toml (the balls-4 layer-2 config,
which OUTRANKS the landing) and $XDG_CONFIG_HOME/balls/default-config/
(the seed template `bl prime` founds a landing from).

This box's ~/.config/balls/default-config/plugins.toml is a June-7 copy that
names the RETIRED plugin `tracker` (renamed `bl-tracker`, balls bl-27bf). balls'
seed prunes any scheduled plugin with no binary beside `bl` — the world tools dir
ships `bl-tracker`, not `tracker` — so every entry prunes.

Observed, `bl prime` inside yog's world on a fresh project:
  landing plugins.toml = ONLY bl-delivery on 8 hooks; NO bl-tracker anywhere,
  and no `show` hook.
Consequences: the store NEVER fetches or pushes (create/claim/close/update all
had a tracker phase), and `bl show <id>` prints no worktree line.
Also observed bleeding in: `clock-provider /home/u/.local/bin/bl-workhours
xdg (global)` — the operator's own machine config, inside the "nested" world.

Proof by control: same yog binary, same project shape, XDG_CONFIG_HOME pointed
at an empty dir => the landing wires bl-tracker at 8 phases + `show`, and
balls/tasks is pushed to the project's origin (hub had `main` + `balls/tasks`).
With the ambient config home: hub has `main` only.

This is squarely inside this ball: tasks_branch — the thing this ball binds —
resolves through exactly that layer (conf_resolve::scalar: cli > xdg > landing
> default), so a tasks_branch in the operator's ~/.config/balls/config.toml
would today override every landing in yog's world and make `bl conf set
task-branch` a silent no-op. FIXING HERE.

**Finding 2: the marks knob writes a bogus store remote.**
world::marks::plan writes `bl conf set task-remote origin` for BOTH Shared
and CustomBranch. `bl conf set task-remote` takes a URL; the literal word
`origin` lands in the clone's binding.toml as if it were one, and the binding
tier outranks the "project repo's origin" tier it was meant to restore.
Observed on a project with a real origin: before /marks shared,
`task-remote <hub-url>  origin`; after, `task-remote origin  binding`. On a
project with no origin at all, `/marks branch X` invents one out of nothing:
`(none) stealth` -> `origin binding`. FIXING HERE, by subtraction — the ruling
re-keys the knob to the agent's BRANCH, and the remote is the project's fact,
so the marks plan stops writing task-remote at all (which also retires
Mode::Stealth, whose framing 16.3 already supersedes).

**Filing, not fixing: already-founded landings stay broken.**
A landing's committed schedule is never re-seeded (`bl prime` rebinds but does
not prune/rewrite; balls' converge is LANDING-only and deliberately declines to
touch the XDG layer — "an old name in the XDG layer is the user's file"). So
nesting the config home repairs every NEW clone and leaves every clone the live
world already founded tracker-less. Filed separately.

Not fixed, not yog's file: this box's ~/.config/balls/default-config/plugins.toml
is stale for the operator's OWN `bl prime`s too (same `tracker` name). Operator's
to update; yog is out of its blast radius after this ball.
