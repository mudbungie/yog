+++
title = "per-agent task branches: each agent tracks on its own balls branch by default, settable at launch, inherited by subagents, amendable by the agent"
created = 1786508785
updated = 1786510248
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

---

DESIGN DECISION, taken here because DESIGN 16.3 said it was mine: "how the
per-agent key meets bl's per-checkout config". It does not — it meets bl's
SPACE. The reasoning, so the next reader does not re-derive it:

`bl conf set task-branch` is scope-keyed to the LANDING, and a landing belongs
to a CLONE, which balls keys on (state home, invocation path). Three
consequences kill "drive `bl conf set task-branch` per agent":
 1. two agents working one project share one landing (per project, not agent);
 2. one agent running `bl` in three directories has three landings;
 3. a clone holds ONE store worktree, so two branches in one clone thrash it
    (`substrate::materialize` re-points the checkout every op) — and yog's board
    reads exactly that checkout, so it would render whichever agent ran last.

So the unit that can be per-agent is the space the clone lives in. One var,
YOG_MARKS, naming balls' state home AND config home together; absent = the
world's space (state stays <world>/state so every existing clone is still the
one the board reads; config becomes <world>/config, which is finding 1's fix);
present = <wall>/marks, the agent's own clone bundle and own balls config home.
Keyed by the 3.1 name that is already the ball claimant (3.2), so claimant and
space cannot disagree.

The branch inside a space is `tasks_branch` in balls' own layer-2 config
(<space>/balls/config.toml) — the one layer that covers every clone in a space,
which is what "the agent's branch" means, and a layer balls ranks above the
landing and names by layer on every read. yog writes that one key and nothing
of its own shape. That is a deliberate reading of the ball's "drive bl conf,
don't duplicate": the file is balls' file, balls' key, balls' precedence; the
alternative (`bl conf set`) provably cannot express a per-agent binding at all.
`bl conf` remains the authority on what a checkout resolves.

WHAT IT COSTS, stated plainly: a `bl conf set task-branch` an agent runs itself
writes the landing and is then shadowed by its own space's layer. Not silent —
`bl conf task-branch` prints the winning layer by name (`xdg`) — and it is the
intended relationship (the agent's space outranks any project binding), but it
is the one wart, and it is recorded here rather than hidden.

The seam: balls' library does no env reads (its bl-bfa8 rule) — the host builds
the Edge with balls' two home directories — so yog folds the space in at
`multiplex::bl::edge`, the one place it already folds YOG_NAME into balls'
default actor. Same exit 16.2 takes for brazen's credential/cache seams.

CLAUSE 2 needed no new mechanism. 3.4's payload ladder already distinguishes
the case: the BALL rung was offered on a project's board and its `bl claim`
landed there, so it carries no YOG_MARKS and its `bl` IS the board's own —
instantly consistent, no sync hop. Every other rung is launched onto its own
space. So "set at launch" and "amend" are one gesture differing only in when it
fires, which is why no launch axis, flag or verb was added.

VERIFIED END TO END against a scratch world (never the live one), with the
operator's stale ~/.config/balls left untouched:
 - `bl prime` in the world now wires bl-tracker at 8 phases + the `show` hook,
   and pushes balls/tasks to the project's origin (it did neither before);
   `clock-provider` reads `(none) default` instead of bleeding in from
   ~/.config/balls/config.toml.
 - `/marks --ws <ws>` reads `balls/tasks` beside the space root; `/marks
   balls/agents/home` writes it; `/marks balls/config` refuses.
 - with YOG_MARKS standing, the agent's `bl conf` reports `task-branch
   balls/agents/home  xdg` in its OWN clone, while the board's space still
   reports `balls/tasks  landing`. Four clauses, observable.
