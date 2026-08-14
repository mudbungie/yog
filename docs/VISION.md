# VISION — executing the task-first promise across the suite

**Scope:** the four-crate suite — `balls`, `brazen`, `lernie`, `yog`.
**Authority:** this document owns cross-suite *direction*: which promises the
suite is executing, which layer owns each one, and the acceptance stories that
prove them. `docs/DESIGN.md` remains yog's architecture authority and each
sibling repo's own docs remain theirs; when this document and an architecture
authority disagree, one of them is amended deliberately — never coded around.
Sources: two external design notes — *fleet-setup* and *Bounding Agent Lifetime
to the Unit of Work*, whose arguments §4.7 and §4.1 adopt explicitly — and a
survey of all four repos at `balls` 0.5.9 / `brazen` 0.0.5 / `lernie` 0.0.3 /
`yog` HEAD. Later amendments are recorded against the ball that carried them:
the headless invariant §4.8, the logic board §5 and G12; the alignment monitor
§4.9 and V6 (bl-af1a); the responder hook and the anti-reinvention guardrail
(bl-156b); the flag floor grant (bl-7aef); V1's two-edge taxonomy, child card
and streaming tail (bl-83f3 — the same pass records the bl-a693 landing that
supersedes G2/G3's survey truth); the project-delivery contract §4.10
(bl-2b8c); the capability boundary §4.11 (bl-0cea).

---

## 1. The promise

> yog takes balls and lernie and integrates them into an agentic workflow
> engine that treats the task, not the agent, as the principal unit of work.

Unpacked, that is three claims, each with a reason:

1. **The task is durable; the agent is disposable.** Quality degrades
   catastrophically with context size, so the execution model must work with
   small contexts — which means agents that live one unit of work and die.
   Everything an agent knew that matters must therefore survive it as a file:
   a ball, a transcript entry, a message, a skill.
2. **If something must happen, a mechanism introduces it.** Instructions decay
   against a growing context; gates, spawn-time seeding, and blocker edges do
   not. Protocol over prompt.
3. **Mechanism sinks, policy rises.** The three lower crates ship capabilities
   with as few opinions as possible; every judgment — which model, which
   skills, when to spawn, when to reap, what a token costs — lives in yog,
   where it is config, severable, and visible in one place.

The third claim is also the standing self-criticism: lernie today mixes
primitives with usage of them, and the opinion embedded in it needs
extracting. Section 6 carries the extraction ledger.

## 2. The layer law

| Crate | Owns (mechanism) | Never owns |
|---|---|---|
| **balls** | What is true about work: occupancy, blocker edges, worktree lifecycle, squash delivery, the plugin seam | Messaging, liveness, identity minting, residency, quality opinions |
| **brazen** | The wire: canonical request in, `Event` stream out, credentials, token counters | Pricing, model catalogs beyond routing, tool execution, state |
| **lernie** | The agent: branch-per-dispatch, transcript-as-context, the inbox front door, the exec baton, locks, sweep | Schedulers/clocks, model catalogs, workflow *content*, UI |
| **yog** | Every opinion: the clock, spawn policy, model/skill selection, spend, the fleet loop, the whole operator surface | A second implementation of anything below it |

Each lower crate has already ruled its side of this law in its own docs:
balls assigns signaling residency to "the party that already has it" (the
harness), refuses a reaper verb, and outsources identity; brazen ships four
`Usage` counters and no price; lernie ships no scheduler and runs `scan` only
by hand or cron. The law is not aspiration — it is the recorded consensus of
three repos. What remains is to *finish obeying it* (§6) and to build the yog
side that the abstinence below was reserving room for (§4, §5).

## 3. Promise vs. truth

The gap ledger. "Promise" is what §1 and the specs commit to; "truth" is what
the survey found on disk.

| # | Promise | Truth today | Gap owner |
|---|---|---|---|
| G1 | Task, not agent, as principal unit | The phrase appears nowhere in yog; DESIGN organizes by workspace, STORIES by conversation. Balls are start-affordances, not the spine | yog (§4.1, §5) |
| G2 | "Running five versions of the same workflow and having a judge decide" | Survey truth ("**no CLI verb takes a ref** — `prompt` is `(repo, message)`, the only production `fork_point` caller is the verifier") is superseded: lernie bl-a693 shipped `--from <ref>` on `prompt`/`dispatch` (ARCH §2.3 shipped-state note; in the pin since 0.0.6). Remaining gap is the yog surface (V2) and, for fans that mutate project work, §4.10's attempt isolation | yog UI (§4.4, V2, §4.10) |
| G3 | Baseline context, fork N variants off it | Survey truth ("`prompt` hardcodes `config/default`") is superseded by the same landing: `--config <name>` on `prompt`/`dispatch` (bl-a693, pinned 0.0.6). Remaining gap is the yog surface | yog UI (V2) |
| G4 | Judge / consensus fan-in | Verifier gating runs end-to-end config-only; sibling results already arrive as N deposits. No synthesis primitive — and none needed: judging is a workflow shape (§4.4) | yog workflow + lernie exposure |
| G5 | Spend attribution ("$3800/wk and no idea per what") | brazen counts tokens statelessly; lernie sums them into one budget scalar; balls tags every delivery `[bl-id]`; **nobody joins them and nothing prices them** | yog (§4.5) |
| G6 | Model by complexity heuristic, not tag map | yog has a per-role picker; no policy layer, no dataset | yog (§4.6) |
| G7 | Bounded agents, no judging supervisor | bl-3381 as filed implements the *superseded* five-role fleet (long-lived shepherd/sensor) | yog (§4.1 adjudicates) |
| G8 | Unified inbox — "a new prompt, a steer, a tool result, an inter-agent message… all a message in the inbox" | Delivered, with one deliberate refinement: prompts, steers, results, and peer messages all enter by the inbox front door; **tool results commit straight to the transcript** because self-output needs no on-ramp. The unification that holds is: *everything context-bearing is a committed transcript entry* | none — promise met, wording refined |
| G9 | Push-not-poll agent wake-up | lernie's writer/driver totality delivers it in-process; balls' draft ruling: "a poke carries no truth… the poll loop is the correctness floor." yog's level-triggered loop is that floor made cheap | none — polling is the design, not a gap |
| G10 | Agents address each other by name | lernie resolves only `agents/*` ids; display names are yog prose. Already ruled: lernie bl-c8ed (`--name` at creation, `message` resolves id-or-unique-name), yog bl-50f3/bl-08f2 consume it | in flight upstream |
| G11 | Policy out of the primitives | lernie ships `models.yaml` (a model catalog, in git — bl-35e2 rules it out), budget numbers, an opinionated soul, and a hardcoded default config ref | lernie extraction (§6) |
| G12 | An agent in yog can operate yog (§4.8) | The dispatch layer is already `pub` and driven headlessly by every story test; but there is no headless entrypoint, no parity enforcement, and the drive harness steers pixels | yog (§4.8, V5) |
| G13 | Project work is one forkable, deliverable fact | **Ruled (§4.10, bl-2b8c); item 2 landed.** The target no longer rides goal prose — yog passes it typed as lernie's `--cwd` at fire (yog bl-6654 over lernie bl-d0b4) — but project edits still stay outside agent refs, child inheritance, bundle/replay, and comparison, so the rest of the chain stands: balls bl-a1a4 → bl-4eac → yog bl-8746 | §4.10's filed balls |

## 4. The execution model

### 4.1 A drone lives one ball — and bl-3381 is re-scoped

The *Bounding Agent Lifetime* argument is adopted: binding an agent's lifetime
to a single task **deletes** the coordination machinery instead of improving
it. An idle bounded agent is not ambiguous — it is either working or finished —
so the role that existed to disambiguate idleness (the shepherd) is retired,
not audited. Reaping becomes three timestamp/pid comparisons, which is also
lernie's own convergent design (`scan` derives silent death from refs, locks,
and framing — "no status field anywhere"). Early closure, not runaway, is the
failure to engineer against, and gates — not nudges — are the engineering.

**bl-3381 is therefore re-scoped, not discarded.** Its recorded decisions
stand: *yog becomes the clock*; cadence lives in yog; lernie stays
schedulerless. What changes is what the clock drives. The five-role cast maps
onto the bounded model as follows:

| Fleet-setup role | Disposition |
|---|---|
| Coordinator | The **admiral**: yog's level-triggered loop (§4.3) plus the operator conversation. Long-lived because the human talks to it |
| Shepherd | **Dissolved**: its disambiguation job vanishes with bounded lifetime; its verification job (did `origin` move, did the close land) becomes close gates and the loop's derived reads; its reap job becomes comparisons |
| Sensor | Survives where an outward surface exists (Slack): a read-only relay with one speaker per surface — unchanged from fleet-setup §6, which earned its rules empirically |
| Builder | The **drone**: one ball, claim → work → commit → discharge gates → close → exit. Handoff (commit, unclaim, exit) covers gated-and-waiting without a waiting process |
| Steward | The human plus yog's config surfaces; doctrine is skills and templates, seeded at spawn (§4.2), proposals are balls |

### 4.2 Protocol over prompt

Three mechanisms, all existing, carry the discipline:

- **Gates are blocker edges.** Review, docs, adversary — a gate is a child
  ball close-blocking its parent, minted by a plugin at the moment it applies
  (bl-chore's pattern). Never a lifecycle document.
- **Skills are seeded at spawn, keyed on ball tags.** A drone's context is
  fresh exactly once; that is the one moment a standard cannot be crowded out.
  yog selects skills and model from the ball's tags at fire. No standing
  overseer injects anything mid-flight — bounded lifetime already prevents the
  decay the overseer would exist to fight. A skill states a *bar*, never a
  procedure; a skill that grows into a lifecycle document has failed.
- **Depth is data.** Fractal decomposition is bounded by a tag written at
  create (a pre-create plugin refuses past it), never derived by walking
  ancestors — a closed ancestor has no file and would silently truncate the
  walk.

### 4.3 The clock and the loop

yog's backend owns the only clock in the system (bl-3381 decision, standing).
The loop is **level-triggered**: each tick reads the board (`Catalog` load),
reads liveness (locks, pids, leases), brings the drone count to match ready
work under a policy cap, reaps by comparison, and surfaces decisions to the
operator. A missed tick is self-healing because the loop converges from
whatever state it finds. The loop **spawns and reaps; it never diagnoses.**
I7 (yog never mutates except on explicit user action) is preserved by making
fleet mode itself the explicit action: the operator arms the loop per
workspace; an armed loop's spawns are the user action, continuing.

### 4.4 Fork, fan, judge, consensus

The **agent-history** experimentation verbs have shipped (G2/G3, pinned
0.0.6); the **project-work** promise is ruled but not yet built (§4.10). G13
is the distinction the earliest version of this section missed: a lernie
sibling ref carries agent context/transcript, while a yog ball's code edits
live in a separate balls worktree. Those refs are not candidate project
branches and balls cannot squash one of them.

- **Agent fork remains the ordinary fork with a ref argument** (lernie ARCH
  §7.2, verbatim). The narrow upstream ask has landed (lernie bl-a693, ARCH
  §2.3 shipped-state note; pinned 0.0.6): `--from <ref>` on
  `prompt`/`dispatch`, plus the `--config` branch selector. An agent's
  history carries two distinct facts, never conflated: *context* is git
  ancestry, *provenance* is the descent id plus the dispatch notch (the
  two-edge taxonomy, V1.3).
- **A read-only/context fan is N agent forks off one ref.** That is already a
  lawful comparison of model/config/goal outcomes. A fan whose candidates
  mutate project work additionally needs N isolated project attempts and a
  typed binding from each agent to one of them — §4.10 items 1–3; built by
  balls bl-4eac and yog bl-8746 over lernie bl-d0b4.
- **A judge is still a dispatch, not a fan-in primitive.** It may compare
  terminal responses and agent refs now. A project diff is a pure git read
  once §4.10's pointer join lands; a synthesizer that writes project bytes is
  itself an ordinary attempt on the candidates' own target (§4.10 item 1).
- **Acceptance is delivery, and there is no Adopt.** A candidate is accepted
  by ordinary source-to-target delivery under bl-a1a4's law; the "winner" is
  a query over the target's history, never a stored mark (§4.10 items 5–6).
  V3 renders **Deliver candidate** over that mechanism.

The refusal of a new fan-in primitive still stands. So does the refusal to
paper over project work with a bare cwd flag: §4.10's binding is lawful
because it arrives *with* attempt isolation, per-step provenance pointers,
and the delivery law — a cwd alone would make the off-record path less
visible, not make project work forkable.

### 4.5 Spend attribution

Cost per ball is now a *query* (bl-afc4), honoring single-source-of-truth:

- brazen keeps counting tokens and never learns prices (its stated boundary).
- lernie keeps committing usage into step records (already on disk, already
  folded by yog's budget inspector).
- balls keeps tagging every delivery `[bl-id]` and stays metric-free — its own
  doc reserves exactly this seam: "a `*.post` plugin observes the lifecycle
  and writes to its own territory."
- **yog owns the join and the price table.** Price-per-token is yog world
  config (severable: deleting it deletes a column, not code). Cost per ball =
  Σ(step usage over the agents whose claim stamp names the ball) × prices.
  Per-conversation and per-ball figures now render at the workspace altitude;
  the V4 board/epic rollup remains future. The ceiling, when built, gates
  *spawns* and never kills a running drone — killing mid-ball destroys
  uncommitted work, and early termination is the expensive failure.

### 4.6 Model selection

Tag→model is the functional heuristic and stays available. The vision's
target is one rung up: a **complexity estimate on the ball** (a tag, written
at decomposition time) mapped to model by a yog policy table — because the
estimate-outcome-spend triple (§4.5's join plus delivered/reverted facts) is
the dataset that lets the mapping be tuned instead of vibed. Policy table,
yog config, severable. No crate below yog ever names a model.

### 4.7 Fleets talking to fleets

Adopted from fleet-setup §6, which paid for its rules: coordinator-to-
coordinator over a channel **both humans stand in**; one speaker per outward
surface; sensors read-only with enumerated write prohibitions; findings travel
with method and numbers. yog's part is rendering the channel and the sensor's
relays as inbox facts — the mechanism is lernie messages plus whatever
transport (Slack MCP today) the deployment chooses.

### 4.8 Every gesture is a verb — the headless invariant

**Every interface in yog, no exceptions, is drivable headlessly, and yog has a
headless mode — an agent in yog can operate yog.**

The foundation already exists: yog's story tests "drive the dispatch layer —
the same `pub` functions the shell's click-glue calls… never egui widgets,"
and yog already multiplexes verb namespaces (`yog bl`, `yog lernie`, `yog
bz`). What this vision adds is the invariant that makes that foundation
binding: **the GUI is one serialization of the dispatch surface; headless mode
is another serialization of exactly the same surface** — brazen's lib↔CLI
parity discipline, applied to yog.

**The boundary is formal, and it carries queries as well as actions.**
The ruling is a formal control boundary for every operation that *does
something*: focusing an input box does not cross it, hitting enter does;
switching tab does not, changing a value does — and so does the call that
populated the tab's contents. Actions and queries cross the boundary; views do
not. Three families, and the classification is decidable per gesture:

- **Actions** — anything that mutates (send, stop, close, claim, apply, arm):
  through the boundary, logged in `ops.jsonl`, headlessly spellable.
- **Queries** — anything that populates (the read that filled a tab, the board
  load, a snapshot derivation): through the boundary, returning the same typed
  data both frontends render.
- **Views** — per-instance ephemera (focus, scroll, tab selection, unsent
  drafts): never through the boundary. They are exactly I6's closed RAM
  whitelist, already enumerated.

The taxonomy is the existing invariants made structural — views are I6's
whitelist, queries are I1's derivations, actions are the ops trail's rows —
which is why the rewrite is a re-plumbing, not a re-conception. Its cost is
acknowledged and accepted: the click-glue and the view-models route through
the boundary types, because parity enforced anywhere short of the boundary
re-creates two surfaces. The boundary's carrier is a datum, not a convention:
one typed surface (action and query variants carrying their parameters) that
both frontends construct and one chokepoint dispatches — so a new gesture
without a headless spelling fails to compile before it fails review.

Consequences:

- Every operator gesture — start, message, stop, close, assign, arm the loop,
  fork, judge, acknowledge — is a boundary variant first and a widget second.
- Reads obey the same law, and I1 already pays for them: disk is the app, so
  every query is the snapshot derivation run without a frame, emitted as
  typed JSON.
- Teleop follows for free. An agent operating yog is a driver of the headless
  surface; the connective tissue — Slack, a phone, a peer fleet's coordinator
  — lives outside yog and can live anywhere.
- The drive harness stops steering pixels: story beats address the headless
  surface, and screenshots become what they always should have been — visual
  confirmation, not the transport.

### 4.9 The alignment monitor — the check is a call, the judge is a dispatch

The concept (bl-af1a): a second, cheaper model reads what an
agent is doing — thinking, output, tool calls — and continuously answers one
question: **does the recent work serve the stated goal?** On divergence, a
graduated ladder of responses fires, each rung an existing verb. The research
ground is the trusted-monitoring protocol (Redwood Research, *AI Control:
Improving Safety Despite Intentional Subversion*, 2023 — a weak trusted model
scores every action of a strong untrusted one, with an audit budget for
escalation) and OpenAI's CoT-monitoring result (*Monitoring Reasoning Models
for Misbehavior*, 2025 — a weak model watching a reasoning model's thinking
catches misbehavior that action-only monitoring misses, and a model optimized
against the monitor learns to obfuscate). yog builds the production instance;
no crate below yog learns the monitor exists.

The suite's own disciplines make the design nearly free:

- **Intent already has a durable home.** An agent's assignment is its
  branch's `goal.md` plus its principal's messages in the transcript (the
  user's for a root, the parent's for a child) — committed, immutable,
  per-branch. The monitor quotes them verbatim; no extracted-intent store
  exists to drift (single source of truth). The concept's original three
  questions collapse to one classification.
- **The live stream is already on disk.** lernie streams every model event
  into the step record as it arrives, and the committed transcript carries
  assistant text *and thinking blocks*; watching an agent is reading files
  yog already derives from (the in-flight strip, bl-905f). No wire tap, no
  new transport — disk is the bus.
- **The verdict is replayable.** A check reads (goal, transcript window) from
  the branch's read-state commit — lernie's own assembly-from-commit
  discipline — so any verdict re-runs against the sha its ops row names. A
  verdict that cannot be reproduced cannot be tuned or tested.

**The check.** One bounded, tool-less, cheap-model call per checkpoint — a
brazen call composed by yog (the embedded adapter, DESIGN §16.7). Request:
the goal verbatim, the transcript delta since the last checked sha, the
standing verdict. Response: `aligned | drifting | diverged` plus a
one-sentence reason. Level-triggered off the step spine by yog's clock: a
tick checks an armed agent only when its branch tip has moved. The
last-checked sha derives from the monitor's own `ops.jsonl` rows — every
check writes one row naming the sha, verdict, and reason, which is at once
the audit trail, the level-trigger's memory, and (joined with §4.5's
delivered/reverted facts) the tuning dataset. No new durable artifact; policy
and arming ride `cadence.yaml` (I2 holds at three).

Tool-lessness bounds prompt injection: transcript content is quoted strictly
as data, and a poisoned check can at worst emit a wrong verdict — a false
`aligned` degrades to today's baseline, a false `diverged` fires a rung whose
severity the ladder config caps and whose harshest action (stop) is
non-destructive.

**The ladder is a hook** (bl-156b). Verdict→response is
a logic-board tie-point (§5): config an operator rewires, never code. A
wiring takes one of two shapes, and both spend only existing verbs:

- **Direct** — a verdict mechanically fires one verb; yog executes a config
  row and decides nothing.
- **Hooked** — a verdict dispatches a **responder**: an ordinary lernie
  worker on an operator-named workflow, its goal carrying the flagged window,
  its granted tools yog's own boundary verbs through the world's tool shims.
  This is V5's promise cashed early — an agent operating yog drives the
  headless surface — and it is symmetric by construction: **tool selection is
  ladder selection.** Grant `stop` and the responder can kill; grant
  `message` and steering falls out. **The floor grant is `flag`** (bl-7aef):
  signaling out is itself a tool call — a boundary action writing the ops
  row attention already derives from — because the alternative is yog
  parsing a verdict out of terminal-response prose, a protocol-over-prompt
  violation inside the monitor's own machinery. A responder granted only
  `flag` is a pure judge (§4.4 — a judge is a dispatch, not a primitive),
  its verdict typed by the tool's schema. yog names the workflow and the
  grants; lernie runs the tools; no rung executor exists in yog.

| Verb | Mechanism | Notes |
|---|---|---|
| Flag | attention item + ops row; a boundary variant, so it is also grantable as a responder tool | the default floor twice over: tier-0's direct wiring writes it mechanically from the verdict, and it is the responder's minimum grant — the signal-out verb |
| Notice | `message` — an inbox deposit quoting the goal and the divergence, delivered at the next step boundary | lernie's own front door; **reveals the monitor to the agent**, inviting in-context masking (the obfuscation result), so it is a deliberate wiring, never the default |
| Dispatch | the responder hook above, on a stronger model | the audit budget: deliberative response only on flags |
| Revoke | `/revoke` — a boundary action writing the per-conversation capability floor (§4.11 item 7) | the fifth rung, landed bl-94b4: every class above read adjudicates to a hold, over the conversation and its whole descent, until `/restore`. Non-destructive like stop and *less* blunt — the drone keeps running and keeps reading, and each call it wants can still be walked through with `/answer pass` |
| Stop | the existing stop verb | non-destructive by lernie's design — a stopped agent is a persisting branch (ARCH §2.9); the operator reviews, then resumes with a message or abandons |

Responder verdicts weigh **actions over prose** — tool calls and worktree
diffs are evidence; thinking is early warning that a monitored model can
learn to launder.

**The check never becomes a second lernie.** The anti-reinvention guardrail,
stated as law: the tier-0 call has no tools, no retry machinery, and no
transcript. Its retry is the level-trigger itself — a failed check leaves
the last-checked sha behind the tip, so the next tick simply re-fires; its
audit is the ops row; and any response requiring a decision is a dispatch.
The moment the check wants a second lernie feature — a tool, a multi-step
chain, a memory — it has become an agent, and the design's answer already
exists: dispatch one.

A fifth rung — **revoke tool auto-approval** — is ruled (§4.11 item 7,
bl-0cea) and **landed** (bl-94b4, over bl-765d's policy surface): an
alignment verdict becomes one input to the capability policy — a boundary
action writing a per-conversation floor (every class above read adjudicates
to hold) into the ops-row fold the capability control reads. No lernie-side
approval seam needed filing: the tool-control seam had already shipped
(§4.11 item 2). It is a **verb the operator may wire, never a behavior that
switched on**: nothing fires it by default, because the monitor still ships
flag-only (a false `diverged` that acts visibly teaches the operator to
distrust the monitor), and wiring it is a `cadence.yaml` tier-0 row or a
responder grant — tool selection is ladder selection. The division of labor
is permanent: **the capability boundary rules what an agent may ever do; the
monitor rules whether what it is doing serves the goal.** The monitor defends
against sustained drift, not against a single hostile tool call — that is the
capability boundary's job — and neither substitutes for the other. Which is
also why the rung is a *capability* verb rather than a monitor one: a verdict
must never become a capability, nor a capability a judgement.

**Reconciliation with the standing rules, by name:**

- *No long-lived judging supervisor* (§8) stands: there is no monitor
  process. The clock fires bounded calls; a responder is a bounded dispatch;
  everything durable is an ops row or a commit.
- *The loop never diagnoses* (§4.3) stands: the loop's own spawn/reap actions
  remain comparisons. The diagnosis lives in a model call's verdict, quoted
  in its ops row — data the operator audits, never a silent opinion.
- *I7* holds the fleet-mode way: the operator **arms the monitor per
  workspace**; an armed monitor's checks and rungs are the user action,
  continuing. Unarmed — the default — the mechanism does not run, and
  severability is deleting a `cadence.yaml` entry, not editing code.
- *No standing overseer injects anything mid-flight* (§4.2) stands as the
  default: the notice rung is off unless wired, and its recorded cost
  (revelation → masking) rides with it.
- Spend is attributed, not hidden: §4.5's join folds monitor ops-row usage
  into the watched conversation's cost, so watching a ball is a visible line
  in that ball's spend.

**Staging.** v1 checks read the committed transcript only — step-boundary
latency, replayable evidence. A mid-step fast path (scoring the streaming
text the in-flight strip already snapshots) is a permitted later increment,
restricted to the stop rung and quoting its evidence verbatim in the ops row,
because staging text is retry debris with no commit to replay against. The
earliness it buys is one step; buy it only if step-boundary latency proves
too slow in practice.

### 4.10 The project-delivery contract — one recursive graph (bl-2b8c)

Project execution is **one recursive delivery graph**: task and subtask, root
and child, single try and fan candidate differ only by **delivery target**. An
agent operation has **N ≥ 1 delivery attempts**, each starting from an exact
target commit; with N > 1 each alternative is a **candidate** — a policy name
yog uses, never a balls verb, task kind, status, or Git primitive. An attempt
may be **reworked** before delivery, **rejected** with its target unchanged,
or **accepted** by ordinary source-to-target delivery. Acceptance advances
the parent obligation's branch; it never closes that parent — the parent
later delivers recursively through ordinary close. The layer law, applied:
balls owns project refs, worktree lifecycle, delivery, and safe cleanup;
lernie owns agent context/transcript and the attempt binding; yog owns N,
variants, comparison, accept/reject/rework policy, retention, and
presentation.

1. **Source and target.** Every write-capable attempt is a
   balls-materialized private source (ref + index + worktree). The N = 1
   ordinary ball path is exactly today's `work/<id>` claim; N > 1
   alternatives use the same capability in a namespace distinct from
   `work/*` (balls bl-4eac) — one mechanism, no special candidate path. The
   target is derived, never chosen: a ball attempt targets what `bl close`
   already derives (the parent's ref for a close-gating subtask, the
   integration branch for a root); a write-capable *child agent's* attempt
   targets its parent attempt's source ref — the same operation at every
   depth (bl-a1a4's fractal law). A judge is a dispatch that reads immutable
   objects (commits by OID in the shared project store) and needs no
   checkout; a synthesizer that writes project bytes is itself an ordinary
   attempt on the same target as the candidates it merges.

2. **The binding is typed and mechanical, never prose.** lernie's
   working-directory mark (`refs/lernie/cwd/<agent-id>`, pinned 0.0.8 —
   today written only by the model-driven `cd` tool, read by the executor at
   every tool spawn) becomes seedable at creation: a parameter on
   `prompt`/`dispatch`/the `dispatch` tool writes the mark before the first
   step (lernie bl-d0b4). yog's fire passes the attempt worktree and the
   goal-prose location preamble retires (yog bl-6654) — the goal keeps
   payload (ball title and body), never location. Validation lives where the
   binding is consumed: creation refuses a missing directory; an executor
   read whose directory has vanished declines the tool call loudly, never
   silently falling back to the agent worktree; and balls owns the paths
   (computed, never stored), so unclaim/re-claim and moves re-materialize
   rather than dangle. Inheritance is mechanical or absent: the mark is
   id-scoped and a child's is unset unless its dispatch names one; what a
   write-capable child inherits is the *target* (its parent attempt's source
   ref) through a fresh attempt of its own — never the parent's mutable
   checkout. No goal parsing anywhere: nothing ever derives a target from
   prose.

3. **Writer isolation.** Snapshot identity and mutable checkout are
   distinct: attempts may share immutable Git objects and an exact base
   commit; no two write-capable lineages may share a mutable ref, index, or
   worktree. A checkout is shared only when mechanically read-only, and a
   bash/build-capable agent counts as a writer until bl-0cea's capability
   enforcement proves otherwise — which is why a child born today inherits
   no binding, and why every fan candidate gets its own attempt with a
   single-writer lease (bl-4eac).

4. **The join is by pointer, never by copy.** Project bytes live in the
   project repo only. The agent side records pointers: the binding mark plus
   each step record's observed HEAD OID of the bound repo (bl-d0b4). Agent
   history, the V1 rail's "project as of", transcript, bundle/replay, and
   crash recovery all join project state by resolving OIDs against the
   project repo; the project diff is a pure git read (`target..source`); a
   missing project repo renders as a named absence, never a guess; crash
   recovery is balls' worktree convergence plus lernie's committed facts —
   nothing reconciles because nothing was duplicated.

5. **One source-to-target delivery law** (bl-a1a4). The source owner
   incorporates the current target before delivery; delivery pins the target
   tip, requires it to be an ancestor of the source tip, refuses a stale
   source before any merge, gates the exact source tree with the repo's own
   hook, mints the tagged squash, and CAS-advances the target — it validates
   and lands, never reconciles. Accepting a fan candidate is this same
   delivery; after one lands, every sibling is stale by construction and
   must rework (incorporate the new target in its own worktree) before it
   can deliver — sequential synthesis falls out of the law instead of
   needing a primitive. The parent's own later close is the same operation
   one level up.

6. **N, candidates, and outcomes are yog policy over derived facts.** yog
   decides N and per-variant overrides at fire; comparison, selection,
   rejection, rework, and retention are boundary actions (§4.8) spending
   existing verbs — deliver (balls), message/fork (lernie), cleanup (balls'
   separated worktree-release vs source-ref retention). There is no Adopt
   verb and no stored winner: the accepted candidate is the attempt whose
   delivery the target's history records (balls returns the exact
   base/source/target/delivered identities); cohort = attempts sharing
   (target, base); provenance is ancestry; rejection is the absence of a
   delivery. Losers stay inspectable until yog's retention policy (world
   config, severable) asks balls to clean them.

7. **The science projection is a query** (yog bl-40ab): frozen inputs (goal,
   pins, governing config commit — model and skills ride it),
   base/source/target/delivered OIDs, terminal response, usage, wall time,
   project diff, verdicts (messages), and the accepted/rejected/reworked
   outcome — all derived at read time from lernie step records, balls
   delivery identities, and git ancestry. Nothing stored.

8. **Bare, path, and ball.** A bare start binds nothing — tools run in the
   agent worktree, the general path with empty inputs. A path start binds
   the named directory (typed, not prose) with no delivery obligation — no
   target, no attempt materialization, no delivery; work lands where the
   agent puts it, exactly as honest as today. A ball start derives its
   target from balls and is an N = 1 attempt; fanning any delivery
   obligation — ball-derived, or an explicit repo + ref via bl-4eac's
   non-task attempts, bare project repositories included — is the N > 1
   case of the same path.

9. **Implementation and release sequence.**
   - lernie: bl-d0b4 (binding parameter, validation, per-step OID) →
     publish → yog bl-6654 pins it, passes the binding typed, retires the
     prose channel → unblocks bl-aa8b (frozen project instructions anchor
     at the typed target).
   - balls: bl-a1a4 (source-owned reconciliation) → bl-4eac (the
     policy-blind attempt capability) → publish → yog bl-8746 pins it and
     builds the mutating fan → unblocks bl-c2bd (V3) and bl-40ab (the
     projection).
   - bl-0cea (capability boundary) proceeds from this ruling now: the
     writable root is the bound attempt worktree.
   - V4's armed code-writing drones wait for bl-6654 + bl-0cea (§5's
     precondition, unchanged in substance, now named by ball).

### 4.11 The capability boundary — policy before execution (bl-0cea)

The verified premise (bl-7fc8): pinned lernie's shipped `worker` role grants
the **entire** tool pool — `[apply_patch, bash, cd, dispatch, load_skill,
message, multi_tool, read_file]` — yog grants nothing on top and has no grant
path at all, so an unattended drone has unrestricted `bash`/`apply_patch`/`cd`
from its first step. The comparison target is Claude Code's allow/deny/ask
*permission mediation*, not a sandbox claim — its OS sandbox is separately
optional and defaults off, and so is ours (item 8). This section rules the
mediation.

1. **The effect vocabulary classifies invocations, never tool names.** Six
   classes: **read** (observes only), **target write** (mutation confined to
   the writable root, or to the world's own substrates through their gated
   verbs), **destructive** (irreversible loss: history rewrite, forced ref
   updates, deletion beyond git's recoverability), **process** (minting
   agents or processes beyond the invocation — `dispatch` lands here, and
   stays governed by lernie's own budget/depth gates), **open-world**
   (effects beyond the root and the world: network egress, host filesystem
   writes, a `cd` out of the root), **secret** (credential and environment
   access: brazen config/credentials, key material, `bz` itself — a drone
   spending ambient credentials outside lernie's budget derivation). Built-ins
   carry an intrinsic class map; `cd` and `apply_patch` classify against the
   writable root at consult time; `bash` classifies by an operator-editable
   ruleset over its command — and an **unmatched bash invocation is
   open-world**: classification error fails toward the wider class, so
   obfuscation never lands in `read` and is what an override or a floor bites
   on. External `lernie-tool-*` binaries default to open-world until the
   ruleset classifies them.

2. **The enforcement point is lernie's shipped tool-control seam — no new
   primitive anywhere.** The pinned 0.0.8 already carries it (lernie ARCH
   §3.3 *Tool control*, bl-de6d): `workflow.yaml`'s `tool_control:` names one
   executable consulted before every granted invocation executes, answering
   `pass` / `refuse` / `hold`, failing closed. yog supplies that executable —
   a world-tools re-exec shim of the yog binary (DESIGN §16.4's pattern) —
   and authors the block onto each workspace's `config/default` at every
   start, through lernie's own config verb — the nested *template* route this
   item first named cannot carry it (bl-fec8 verified the pin: `prime` never
   seeds that override root, the override is a whole-file copy that would
   delete `events:`, and lernie's embedded default is private), and the
   per-workspace route is stronger anyway: it reaches workspaces already on
   disk, not only ones born after. The §6 lernie item-7 contingency ("if bl-0cea wants a
   synchronous pre-tool approval seam, that files as its own ask then") is
   answered: **it shipped before we asked; the ask is zero.** Role grants
   stay exactly as shipped — grants are lernie's structure, the control is
   yog's policy — so bl-7fc8's deletion of the grant path stands; tool-name
   narrowing returns nowhere, because the workhorse tool (`bash`) is every
   class at once and only per-invocation adjudication can tell them apart.

3. **The writable root is §4.10's answer, restated:** the bound attempt
   worktree plus the agent's own worktree. Inside it, writing is the job
   (target write, pass). The world's substrate verbs — the `bl`/`lernie`
   shims, `message`, `dispatch` — keep their own gates (the delivery law,
   the front door, budgets and depth) and are never re-adjudicated: the
   control rules host effects, not deliveries.

4. **Every fact is durable, and the control writes none of them.** The
   *request* is lernie's hold mark (`refs/lernie/held/<agent-id>` — id,
   tool, reason). *Standing policy* is per-workspace config, severable:
   absence is the shipped defaults of item 6. *Answers and floors* — a hold
   approved or denied once, a conversation's auto-approval revoked — are
   boundary actions whose `ops.jsonl` rows are at once the audit and the
   fold the control reads (the §4.9 monitor's own pattern: no new durable
   artifact). The control is side-effect-free per consult — the seam demands
   it, since release is re-adjudication — so it reads everything and writes
   nothing. Every fact survives driver exit and revival by construction:
   marks, config, ops rows. GUI and headless answer through the same §4.8
   boundary variants; parity is the compile gate, not review.

5. **"Ask" with no human at the window is a park, and a deny never stops an
   agent.** A `hold` parks the branch: the driver exits, nothing at or past
   the held call ran, the drone costs nothing while it waits, and the board
   renders the ball waiting at its gate — attention, not deadlock. A `refuse`
   is an in-band decline the model reads and steps past. **No enforcement
   path ever calls stop**: `lernie stop` mid-tool-window permanently wedges
   the branch (lernie bl-b98d), so a deny that stopped would brick the drone
   it means to protect — the wedge is routed around, not depended on. And no
   modal exists anywhere: attended and unattended are one flow — the
   attention item is answered in seconds or in hours, so attendance is
   latency, not a mode, and the interactive/armed policy split dissolves.

6. **One default table, and it passes everything but loss and credentials**
   (amended by bl-1ef1; it shipped open-world → hold): read, target write,
   process, open-world → pass; destructive, secret → refuse. The parked
   open-world default made the operator answer for every `python` and every
   fetch, and an approval given by reflex is the failure mode a gate exists to
   avoid — so **a hold is imposed, never standing**: a workspace writes one
   `table:` row to get the park back, and the §4.9 monitor's floor aims one at
   the conversation that earned it. Destructive and secret keep refusing;
   those two are what an unattended drone must not decide for itself.
   Once-answers are scoped to the held `tool_use` id (provider-unique, so a
   once-grant needs no consumption and cannot race); persistent answers are
   policy rows. Revocation binds at the next consult — a verdict already
   passed runs its one call, because recalling it mid-window is the wedge
   above.

7. **The monitor's fifth rung now has its substrate** — and, since bl-94b4,
   its writer. Revoke-auto-approval (§4.9) is a boundary action writing a
   per-conversation floor — every class above read adjudicates to hold, over
   that conversation and its whole descent — into the same ops-row fold,
   spelled `/revoke` and `/restore`, latest row winning. The division of
   labor is permanent: the capability boundary rules what an agent may ever
   do; the monitor rules whether what it is doing serves the goal.

8. **The threat model, stated honestly.** The ambient PATH rides beneath the
   world's prepend; the network is unconfined; brazen credentials are
   ambient and shared by §16.2's own deliberate ruling; `cd` and absolute
   paths reach the whole host. Rule-based classification bounds *accident
   and casual drift*, not adversarial evasion — an allowed `cargo run`
   executes arbitrary `build.rs` under a pass. The defense is layered by
   design: unmatched effects fail to a hold (this section), sustained drift
   is the monitor's job (§4.9), and evasion is the OS-confinement layer's —
   lernie's reserved v1.1 sandbox seam (ARCH §3.6) or platform facilities,
   later and platform-explicit. A workspace policy may declare confinement
   *required*, and arming then refuses where the platform cannot provide it
   — never a silent fallback, never a promised wall that isn't there.

9. **Implementation** (yog only — lernie, balls, and brazen file nothing):
   bl-fec8 (the control shim, classifier, judgment fold, template
   authoring) → bl-765d (the policy config, hold-answer boundary actions,
   parked-branch attention, the confinement-required refusal; blocks
   bl-66fb's armed loop) → bl-94b4 (the §4.9 revoke rung, landed as the floor
   writer over bl-fec8's reader, coordinating bl-8da1 by ruling — its ladder
   config gains the rung; its internals are untouched). V4 arms behind
   bl-765d.

## 5. The operator's view of it — story rungs

STORIES.md is the acceptance ladder and keeps that authority; these are the
vision rungs that graduate into it (S10+) as their enabling verbs land. They
answer the question the ladder currently stops short of: S7 lets the operator
*read* every byte; nothing yet lets them **navigate commits, act on history,
or steer a fan**. Written in the ladder's idiom; tests get filed with the
graduating rung.

**The rungs are projections of a logic board, not built-in flows.** The
lernie/yog workflow is a logic board for how tasks and agents get handled, and
the thing worth spending time on is how they are tied together. Every flow a
rung renders — which skills a tag
seeds, which model a complexity maps to, which gates a close mints, what shape
a fan takes, what a judge reads — is a **tie-point**: config an operator
rewires without anyone editing a crate. The UI renders the board and its
wiring; it never *is* the wiring. A rung that hardcodes a flow has failed the
same way a skill that grows into a procedure has.

### V1 — Historian: the step spine is a commit spine

Machine state: a conversation with history worth walking.

1. The transcript grows a **step spine**: one notch per step, each notch the
   step's read-state commit (already recorded in `meta.json` — "every step's
   read state is a real git commit"). The spine is the timeline; the newest
   notch is where S7 already lives. It is drawn **through** the chat — one
   horizontal rule per operable commit — not beside it (bl-1802, below).
2. Selecting a notch pins the agent-history inspector to that commit:
   transcript as of, **agent-context** files as of, config-frozen-at, budget
   folded to that point. The Raw toggle keeps showing verbatim bytes of the
   pinned tree. Project files are not in that tree and never will be (§4.10
   item 4 — the join is by pointer): "project as of" resolves the step
   record's observed project OID against the project repo, and renders only
   once the bl-d0b4 pin lands; until then the surface stays honestly absent.
3. **An agent's graph edges are two distinct facts, never conflated:**
   - The *context edge* — what the child inherited — is git ancestry.
   - The *provenance edge* — who dispatched it — is the descent id plus the
     parent-transcript notch where the dispatch landed.
   A clean child (`dispatch --from config/<name>`, shipped in lernie bl-a693)
   has provenance only — no ancestry edge to its parent. "Clean vs fork" is
   one spawn gesture with one parameter — the fork point — per lernie ARCH
   §2.3 ("Any ref is a legal fork point"); the spine renders both edge kinds,
   never just ancestry.
   **Amended by bl-1802 on how they are *drawn*, not on what they are.** The
   original wording said solid and dashed strokes, which the gutter had room
   for and a rule across a chat does not. The distinction is now carried by the
   card's own fork label in words — `from here` / `from <Name>@<oid>` name an
   ancestry, `from config/<name>` names a clean child that has none — and the
   strokes are gone, because two renderings of one fact is one too many. The
   taxonomy is unchanged and still derived.
   **Ruled by bl-5cf8: a drawn descent graph earns no seat — the words are
   the rendering.** Each edge already has its one home. The *provenance*
   relation is drawn, as the §11 conversation list's indentation (since
   bl-fa82 the list renders the descent-id forest itself, and since bl-8905
   that is its only rendering); the *context* edge is worded, on the child
   card's fork label, at the moment an operator asks where a child came
   from. A two-edge graph would therefore be a second rendering of both
   facts at once — the exact debt bl-1802 paid down for one of them. Its
   cheapest candidate seat, the conversation list, also folds: a picture
   whose edges vanish behind a collapse is not a picture of a shape, and
   pinning the list open to keep the picture honest would spend the fold on
   a stroke nobody asked for. And it is unassertable: acceptance tests hold
   the paint layer to account through its text (bl-bc06), so a
   solid-vs-dashed stroke between rows is a fact no test could catch lying
   — what can't be tested mustn't be built. If an operator one day asks the
   question the labels cannot answer — the shape of a whole descent at a
   glance, ancestry diverging from provenance across many children — that
   is a new surface argued on its own evidence, not this taxonomy's missing
   rendering.
4. **The dispatch notch carries a live inline child card**: the child's name,
   its fork-point label ("from here" / "from config/default" /
   "from <Name>@<notch>"), a state chip, a spend figure, and a **streaming
   tail** — the last line or two of the child's in-flight inference text.
   Grounding is §4.9 verbatim: "lernie streams every model event into the
   step record as it arrives … watching an agent is reading files yog already
   derives from (the in-flight strip, bl-905f)" — the tail is a second
   consumer of that same derivation (DESIGN §5.1 #10, the in-flight strip's
   own fold), pointed at the child's agent id. Moving text means active;
   still text means tool-wait or quiescent — the point is to show that
   the agent is active, not idle. Following the card to the child's branch
   is the same selection gesture as everywhere else on the rail.
5. **The card's spend figure is the per-agent fold of `steps/<id>`** —
   workspace-root and id-namespaced, so nothing double-counts: a fork's
   shared prefix cost stays with the ancestor. A subtree rollup is a
   descent-id prefix match; no registry.
6. Everything here is a **pure read of the lernie workspace repo** — refs,
   trees, commits — **derived, never pushed**: the card updates via the
   fs_watcher on the workspace repo plus an off-thread snapshot read, like
   every other I1 derivation, and the frame renders snapshots only
   (UI/backend isolation). No new verb, no new transport anywhere; the rung
   is derivation and rendering only. A project-history rail remains behind
   §4.10's pointer join (the bl-d0b4 pin) rather than guessed from an
   unrelated repo.
7. **The fan anchor exists here.** V2.3's sibling group renders anchored at
   the birth notch — one card, N columns — so when V2 lands the fan is item
   4's card grown wide, not a new seat.

Burden check: no dispatches → no cards, no edges; the spine collapses to
today's transcript for anyone who never clicks a notch — the S0 stranger
sees today's transcript exactly.

**Landed (bl-98da), graduated as STORIES §S10 with its six tests.** It needed
no verb: both edges fall out of `Agent::steps`' shared commit prefix, the
notch spine reuses the Steps view's `meta.commit`, and the one new disk read
is the pinned Files tree.

**Re-seated by bl-1802.** History riding alongside the chat was right, but as
its own window it was wrong: every operable commit is a horizontal rule across
the chat rather than a panel beside it, and the fork overlay rises when one is
clicked. The `SidePanel` gutter is deleted; the notch **is** the boundary rule the chat
was already drawing above each commit crossing (§5.1 #29) — one fact that had
been rendered twice, from the same `meta.json`, in two places. Clicking a rule
pins, clicking the pinned one releases, and the V2 composer (seated on the pin,
dying with it) rises on the same click, which is that fork overlay
with no new mechanism. Three rulings this rung's first wording is now wrong
about, and which are the authority over it:

- **The release lives on the pin banner, not on a gutter.** The old ruling
  ("the rail is the *inspector's* gutter, because its release has to be
  reachable from all four pinnable tabs") named a real obligation and met it
  with a panel. The banner already paints above every pinnable tab and already
  names the commit, so the banner carries the release: one existing gesture
  given the seat it needed, no new verb, no routing between tabs. Its old
  sentence, *"Pick the same mark again to come back"*, was a lie the moment the
  mark lived in another tab.
- **The crossing set and the notch set were never the same set, and that was a
  shipped bug.** Both derivations paired *the i-th delivered run* with *the
  i-th step*, on the ground that drains and steps serialize under lernie's
  executor lock. Serialization gives order, not a bijection: a lernie step is
  one model call and a tool loop is many steps behind one delivered run (lernie
  ARCH §2.3), so from the second tool-using turn onward every rule carried the
  wrong commit and every pin cut the transcript in the wrong place. What pairs
  exactly is **one sealed model-output entry per completed step** — a call that
  reaches `Finish` commits `messages/NNN-<model-id>.json` and one that does not
  commits nothing — and that is the pairing both the rules and the pin now ride.
- **"The rail paints only when it has more than one notch" is retired.** That
  gate existed to keep a gutter from claiming width; with the rules in the chat
  the burden check needs no gate at all, because the rules are the ones bl-929d
  already shipped. An operator who never clicks one sees exactly today's
  transcript, which is what the check always asked for.

### V2 — Counterfactualist: fork from any notch

Machine state: V1, plus a step the operator wishes had gone differently.

1. A pinned notch offers **Fork from here**: a goal composer seeded empty,
   firing the ordinary fork with the pinned commit as ref. The new agent
   appears as a sibling row; its context edge is ancestry and its provenance
   edge the firing notch, both rendered by V1's rail (V1.3).
2. The composer's fire-time controls are yog policy made visible: model
   (existing picker), config branch, skills. **×N** fires the same fork N
   times with per-variant overrides — the fan is N rows born of one notch.
3. A fan renders as a group: siblings-of-one-ref, each with its state badge,
   terminal response preview, and usage figure side by side — anchored at the
   birth notch (one card, N columns; V1.7 reserves the anchor).
4. Upstream landed (lernie bl-a693, in the pin since 0.0.6): `--from <ref>` and the
   config-branch selector on `prompt`/`dispatch` (G2/G3). A **mutating** fan
   additionally needs §4.10's attempt isolation and binding (balls bl-4eac +
   yog bl-8746 over lernie bl-d0b4); until those pins land the affordance
   does not render (no capability theater). No new fan verb exists anywhere.

Burden check: no fork, no fan, no change — the composer is reachable only
from a pinned notch.

**Landed (bl-dc0c), graduated as STORIES §S12 with its six tests (S11 went to the Auditor rung, bl-3746, which landed the same day).** It needed
no verb either: the attempt is `lernie dispatch <role> <ws> <parent> --goal
<text> --from <ref> [--pin …]`, already shipped. Three implementation rulings
amend items 2 and 3 above and are the authority over their first wording:

- **×N is the gesture repeated, not a gesture of its own.** The boundary grew
  one attempt-shaped `Fork`, and Fire crosses it once per candidate. A fan
  gesture would have to *name* the fan, and a name is a stored fact the refs
  already imply — where item 3's "siblings-of-one-ref" is derived by grouping
  V1's cards by the notch they were born at. So `N == 1` and `N > 1` are one
  path in the strongest sense available: the same single gesture, counted. The
  §4.2 trail gains N rows where one would have gained one, which is more
  provenance, not less.
- **"Model" is the *role*, and that is what makes it honest.** lernie binds a
  provider and a model id to a role in the `providers.yaml` of the config
  commit governing the fork point, and nowhere else. The composer therefore
  lists the roles that ref declares *with the model each names*, read from the
  very file the run will resolve against. Giving an attempt a model no config
  declares is a config write (§9.4's picker), not a dispatch flag — item 2's
  "model (existing picker)" as a fire-time control would have been a dropdown
  that could lie, which is exactly the capability theater item 4 forbids.
- **Item 3's group anchors at the birth notch, not at a ref.** Candidates fired
  from one mark off *different* refs are still one cohort — they share the
  question, which is what an operator compares — and the group simply has no
  common ancestry to state, so each column states its own. That dissolves the
  case rather than special-casing it.

### V3 — Adjudicator: the fan resolves

Machine state: V2's fan, finished.

1. A fan group offers **Judge** and **Synthesize**: both dispatch one more
   agent whose goal carries the candidates' terminal refs (V2's fire path,
   nothing new). The verdict arrives as a message and renders on the group —
   approve/reject per candidate, or a synthesis child holding the merged
   result.
2. **Deliver candidate** — never Adopt — is the acceptance action: ordinary
   source-to-target delivery of one attempt (§4.10 items 5–6, over
   bl-8746's surface). The UI's mark is a rendered consequence of the
   target's history, never a yog-owned winner field. Losers stay navigable
   as retained attempt refs until yog's retention policy cleans them.
3. The group renders the comparison the operator actually judges by: response
   diff, usage per candidate, wall time, and the ruled project diff
   (`target..source`, §4.10 item 4).

Burden check: judging is a dispatch the operator can also simply not do;
reading the fan costs nothing.

### V4 — Admiral: the board runs the fleet

Machine state: a project with a decomposed backlog; fleet mode armed.

Precondition: code-writing drones do not arm until every drone has the
mechanical, isolated project target §4.10 rules (landed as yog bl-6654 over
lernie bl-d0b4) and the §4.11 capability policy runs live — bl-fec8 (the
control) plus bl-765d (the policy surface), which close-gates the armed
loop (bl-66fb). A fleet of agents merely *told* where to
work, with unrestricted host shell authority, violates §1 before it scales it.

1. The balls section becomes the **board**: ready / claimed / gated / blocked
   columns derived exactly as `bl list` derives them, each claimed ball
   showing its drone (a conversation row — the same object as everywhere
   else), each gate showing what mints it.
2. The armed loop renders as facts, not magic: cap, current count, last tick,
   next tick, and every spawn/reap it performed as ops rows. Reap reasons are
   the comparisons themselves ("lease expired 14m ago"), never diagnoses.
3. Spend is a column: cost per ball from §4.5's join, rolling up the epic
   tree. The ceiling renders where it will bind — on the next spawn.
4. A drone's exit-with-handoff (committed, unclaimed, gated) renders as the
   ball waiting at its gate, not as a dead conversation — the ball is the
   unit the board tracks, which is the promise of §1 made visible.

Burden check: fleet mode is armed per workspace and off by default; unarmed,
the board is today's balls section.

**Landed (bl-9dd4).** Items 1, 3 and 4 as bl-9dd4 (DESIGN §11's board
paragraph, STORIES S13); item 2 — the armed loop itself — as **bl-66fb**
(`src/fleet/`, STORIES S18), once both preconditions above closed. It shipped
**disarmed**, and the burden check is mechanical rather than promised: an
unarmed world derives no loop facts, its tick returns before it reads anything,
and the board reply omits the key rather than answering an empty list. Two
rulings were made in the building and are recorded in DESIGN §11: the tick
renders as a **period**, not a countdown (a level-triggered loop has no phase on
disk and must not grow one), and a **reap releases a claim and never stops a
conversation** — the ceiling's own no-killing-mid-ball ruling, applied to
claims. Whether and when to arm remains the operator's explicit act.

### V5 — Teleoperator: yog without the window

Machine state: the operator is remote — a phone, a peer fleet's coordinator,
an agent; yog runs headless.

1. **Headless mode is the same binary, minus the window**: the backend loop,
   the clock, and the dispatch surface — every gesture addressable as a verb
   (§4.8), every rendered fact readable as derived JSON.
2. **The admin surface is the attention strip made addressable.** Escalations
   and decisions-needed form a queue an agent can read, answer, or forward;
   answering headlessly writes the same watermarks the GUI writes, and I0
   guarantees the two frontends converge over one disk.
3. **An agent operating yog is unremarkable.** The coordinator that
   decides-or-escalates is itself a lernie agent whose tools are yog's
   headless verbs; the human on the phone talks to the coordinator, and the
   coordinator drives yog.
4. Nothing here is a second implementation. Parity (§4.8) is the rung's whole
   content — which is why it costs an entrypoint and a test, not a subsystem.

Burden check: the windowed operator sees nothing new; headless mode adds no
widget anywhere.

**Landed (bl-f6fe), and what it does not include.** Points 1–4 are
in, as DESIGN §8.5's "the decision queue" and "one engine, two faces": the
assembly both faces boot is one `Engine::boot` in the library (it was two copies
inside coverage-excluded `main.rs`, which is exactly the drift point 4 forbids);
the queue is `Query::Attention` and its answer `Action::MarkSeen`, in all three
serializations, writing the very watermarks the window's focus tick writes from
one evidence definition; and forwarding needed no verb, being `/message` aimed
elsewhere. **Point 1's "backend loop" is not delivered and was not faked.** The
clock is real (`cadence.yaml`, DESIGN §7.2) and the dispatch surface is real,
but bl-9dd4 established that no fleet loop existed in `src/` at all — no arming
gesture, no cap, no spawn, no reap — and §8's no-capability-theater refusal plus
this document's hold on fleet mode (§4.3, then pending bl-0cea) both forbade
building one under a teleoperation rung. It landed on V4's own rung instead
(**bl-66fb**), off by default — an unarmed headless yog is exactly
the one this rung shipped.

### V6 — Invigilator: the fleet is watched

Machine state: a workspace with running agents; the monitor armed beside (or
before) the fleet loop.

1. **Arming is a gesture**: a boundary action per workspace with a headless
   spelling (§4.8), recorded as the `cadence.yaml` monitor entry — the same
   arm-is-the-explicit-action pattern as fleet mode (§4.3).
2. **Alignment is a rendered fact**: each armed conversation carries its
   standing verdict — aligned / drifting / diverged, with the reason and the
   checked sha — derived from the ops tail, never a stored flag.
3. **Rungs render where their verbs already render**: a notice is a message
   in the transcript, a responder is a conversation row, a stop is a
   stopped agent — plus the ops rows naming which verdict fired each. Nothing
   the monitor does is invisible (the no-silent-guardian refusal, §8).
4. The V4 board composes: a drone flagged `diverged` is attention on the ball
   the board already tracks; the operator steers or stops from there.

Burden check: unarmed there is no monitor — no chip, no rows, no calls; the
S0 stranger and the unarmed operator see today's yog exactly.

## 6. Upstream asks — the extraction and exposure ledger

Filed as balls in each repo; this table is the index, the balls are the work.

**lernie** (the big one — exposure and extraction):
1. `--from <ref>` on `prompt`/`dispatch` + config-branch selector (G2/G3).
   **Landed** (bl-a693, ARCH §2.3 shipped-state note; pinned 0.0.6); what
   remains is yog's V2 surface — landed in bl-dc0c.
2. ~~Generic caller-supplied pinned documents~~ **shipped** (bl-fb5c, 0.0.4;
   pinned 0.0.6). **First consumed by V2's skills control** (bl-dc0c):
   `--pin skills/<name>/SKILL.md=<pool>/<name>/SKILL.md`, which the shipped
   worker manifest composes through its own `order: skills/**` glob. The
   project-instructions consumer is still bl-aa8b, behind bl-6654.
3. ~~bl-c8ed naming~~ **shipped and consumed** (0.0.4 → pin bumps landed).
4. The §4.10 binding: a creation-time working-directory parameter seeding
   the existing cwd mark, its validation, and per-step observed project
   OIDs in the step record (bl-d0b4). Policy-blind — lernie never learns
   balls, targets, candidates, or delivery.
5. Extraction per bl-35e2's ruling: `models.yaml` out of git —
   protocols and login triggers only, brazen's `defaults.toml` as the
   reference shape. Same sweep audits the other §3/G11 items: budget numbers,
   the shipped soul's stale claims, `dispatch` skill/schema drift ("handle",
   "compacted result" — retired vocabulary still on the wire).
6. Judge/consensus need **no fan-in primitive**. Mutating candidates ride
   §4.10's target/delivery composition, not a new judge operation.
7. Nothing for the §4.9 monitor's v1 — notice, stop, and judge are the
   existing front door, stop verb, and dispatch. ~~If bl-0cea's capability
   ruling wants a synchronous pre-tool approval seam, that files as its own
   ask then~~ **Answered (bl-0cea): the seam had already shipped** — the
   tool-control connection point (ARCH §3.3, bl-de6d, in the 0.0.6 pin) is
   the enforcement point §4.11 builds on, and the capability ruling files
   zero lernie asks. One standing defect matters to it: `lernie stop`
   mid-tool-window wedges a branch (lernie bl-b98d) — §4.11's refusal paths
   route around stop entirely, but the monitor's stop rung still wants that
   fix.

**balls**: two tasks, both inside its own territory — no new opinion, verb,
or metric. bl-a1a4 corrects the delivery law (the source owner incorporates
the target; delivery validates and CAS-advances, never reconciles) and
bl-4eac exposes the policy-blind attempt capability (§4.10 items 1, 3, 5–6).
Liveness, identity, messaging stay refused per its own design records; the
§4.5 spend seam is already reserved and needs no core change. (An earlier
draft of this section floated a "tap" poke verb — withdrawn; the poll loop is
the design, per bl-6c84.)

**brazen**: nothing. The token-counter boundary is correct and load-bearing.
(Housekeeping continues on its own track — the 0.0.5 `--thinking` defects are
already on main.)

**yog** (the build): the §4.8 control boundary — a declared interface
rewrite: the typed action/query surface, the single chokepoint, the headless
entrypoint, the parity tests; the §4.9 alignment monitor (tier-0 check +
ladder; only its auto-approval rung waits, on bl-0cea); the §4.10 contract's
yog side — the typed binding at fire (bl-6654), the mutating fan (bl-8746),
the attempt projection (bl-40ab) — plus frozen project-instruction policy
(bl-aa8b) and capability policy (bl-0cea); the V-rungs in ladder order (V1's
agent-history read first, project diff/fan/deliver behind the §4.10 pins,
V5 riding §4.8); the §4.3 loop under bl-3381 as re-scoped; §4.5's
board/epic rollup and ceiling; §4.6 policy table; the §5 tie-points surfaced as
config, not code. Each files as its own ball; DESIGN.md is amended where a rung
touches declared architecture (the board reframes §3.5's join surface; the
loop amends I7 as §4.3 states; §4.8 amends the §8 hatches into a full surface).

## 7. Sequencing

1. **Now:** bl-2b8c has ruled the project-work graph (§4.10); bl-0cea has
   ruled the capability boundary from it (§4.11 — implementation is bl-fec8
   → bl-765d → bl-94b4, no upstream asks); the §4.8 control boundary work
   proceeds because every later query/action routes through it. The §4.5
   conversation/ball join is already landed; its board/epic rollup and
   ceiling follow the board/control boundaries. V1 may ship its
   agent-history rail, but project files/diff stay explicitly absent until
   the §4.10 pins. The §4.9 monitor's tier-0 check and ladder are
   now-eligible — the clock, the embedded adapter, and the boundary all
   exist; arming lands as a boundary variant first (§4.8's compile gate).
   Only the auto-approval rung waits, on bl-0cea.
2. **Behind the release trains** (§4.10 item 9): lernie bl-d0b4 → publish →
   yog bl-6654 → bl-aa8b; balls bl-a1a4 → bl-4eac → publish → yog bl-8746 →
   bl-c2bd (V3) and bl-40ab. V2's read-only fan needs neither train; its
   mutating path waits for bl-8746.
3. **Behind work authority + capabilities + V-rungs/gates:** V4's armed loop —
   the clock existed first as cadence for watcher cycles (bl-3381's shipped
   scope), and grew spawn/reap only once drones had mechanical isolated project
   targets (bl-6654) and an explicit noninteractive capability policy
   (§4.11: bl-fec8 + bl-765d). **Landed (bl-66fb), off by default**:
   the sequencing held, and what shipped is the mechanism, not an armed fleet.
4. **Continuous:** lernie extraction (G11) — each item severable, none
   blocking the rungs.

## 8. Refusals

Named so they stay refused:

- **No central queue, dispatcher-relay, or review stage.** Agents solve their
  own merge problems; review is a gate ball, not a pipeline stage; the
  admiral assigns work but never sits between peers.
- **No long-lived judging supervisor.** Anything that must notice something
  continuously is either a comparison (the loop) or a read-only sensor on an
  outward surface. The §4.9 monitor complies: bounded checks fired by the
  clock, bounded judge dispatches, no resident process.
- **No silent guardian.** Every §4.9 check writes its ops row; every rung
  fired names its verdict and evidence; an intervention the operator cannot
  audit is a defect. The monitor renders as facts (V6) or does not run.
- **No warm drone pool, no retirement thresholds.** If cold starts cost too
  much, the answer is coarser balls.
- **No pricing, catalogs, or policy in the lower crates.** The severability
  test governs: removing a default must delete config, not edit code.
- **No new lernie primitives for fan-in, registry, or scheduling.** The
  message, the dispatch, and the yog clock already cover them.
- **No second project-work path** (§4.10's rejected set, named): no goal
  parsing as a binding channel; no yog-side project index; no duplicate
  project bytes in any agent workspace (the join is by pointer); no Adopt
  verb; no stored winner, cohort, or outcome field; no special candidate
  delivery path — N = 1 and N > 1 land through the one source-to-target
  law.
- **No capability theater.** A UI affordance for a verb that does not exist
  yet does not render; the V-rungs gate on their verbs by construction. The
  §4.11 corollary: OS confinement is never promised where unavailable and
  never silently substituted by policy mediation — where a workspace
  requires it and the platform lacks it, arming refuses.
- **No permission modal and no second adjudicator** (§4.11). One enforcement
  seam (lernie's tool control), one control (yog's shim), one policy home
  per fact. "Ask" is a park rendered as attention, never a prompt that
  blocks a fleet; "deny" is an in-band decline the model steps past; no
  enforcement path ever stops an agent (the stop-mid-window wedge, lernie
  bl-b98d, is routed around, not depended on). Role grants stay whole-pool:
  tool-name narrowing is theater when `bash` is every effect class at once,
  so no grant path returns (bl-7fc8 stands).
- **No GUI-only capability — and no headless-only capability.** One dispatch
  surface, two serializations, never two implementations; the §4.8 parity
  test holds the line, not review.
- **No flow in code.** The logic board's tie-points (tag→skills,
  complexity→model, gate minting, fan and judge shapes) are config an
  operator rewires; a rung or a crate that hardcodes one is a defect.
