# yog — Architecture & Design

Status: normative. This is the repo's living architecture document, tracked and
amended like code; the decisions here are settled unless amended here.
Deliberate interpretations of the requirement that the user may veto are
collected in §13; rejected alternatives in §14.

**Amendment doctrine (bl-43cd).** The doc states current rulings; git and the
balls store are the record. An amendment *replaces* the prose it amends and
cites its ball id — it never narrates the path to the ruling (keep at most one
sentence of why, where the why changes what an implementer does; relitigation
guards go to §14). Keyed tables stay sorted by their key. A completed plan
retires to git history behind a tombstone heading — a section number, once
cited, resolves forever (machine-checked: `tests/design_citations.rs`).

---

## 0. What yog is

yog is a desktop manager for lernie loops organized around **named
workspaces**; balls are the work items workspaces pick up. One yog window
shows every project on the machine, every ball in each project, every lernie
workspace, every agent and subagent in each workspace, and every byte of
diagnostic data lernie produces — and lets the operator start work from
nothing, a path, or a ball (§3.4),
drive agents, and edit brazen and lernie configuration, all through the
substrates' own write paths (`bl`, `lernie`, `bz`-validated file writes).

**yog is an application that owns a nested world, not a layer draped over the
user's.** The naïve reading — yog as a thin viewer over the ambient
`bl`/`lernie`/`bz` state a human already runs — inverts: yog composes its own
nested environment (§16.2) for itself and every child it spawns, so the
substrate state yog drives is *yog's*, under yog's data root. Playing on top of
the user's *direct* tool usage stays possible — an agent's task branch can be
pointed at the project's shared store branch at launch (§16.3) — so
**compatibility with an ambient workflow is the user's decision, not a
structural given.** The world is the subject of §16.

**Governing invariant (I0): two yog instances running side-by-side faithfully
replicate the same data, with nothing RAM-only except unsubmitted input text.**
yog is a pure renderer of disk plus a small set of user-action dispatchers. The
only durable state yog itself owns is one UI-state document (`ui.json`), one
action-outcome log (`ops.jsonl`), and one clock-settings document
(`cadence.yaml`, §7.2, bl-3381). Everything else already has an authoritative
home in balls, lernie, brazen, or git, and yog derives it.

Crate `yog`, bin `yog`, repo `github.com/mudbungie/yog`, published on
crates.io. Runtime dependencies: the eframe/egui stack, clap, libc, notify,
serde_json, thiserror — plus the three embedded substrates `balls`, `brazen`,
`lernie` as **exact-pinned** crates, the pin being the version mechanism
(§16.5; the pin authority is `Cargo.toml`).

---

## 1. Taxonomy

lernie bans the term "session" (TAXONOMY §3: "underdefined, per-framework
overloaded, and colliding with the transport/connection sense"). yog does not
re-mint it — **the concept dissolves into existing nouns**: "start a session"
is *prompt into a workspace (created if none exists — §3.4; claim optional)*;
"the session" is *the conversation — a root agent plus its descent subtree*;
"the session list" is *the §11 conversation list*. (Amended with §11's
conversation-first rework: the original mapping read "the session is the
workspace", from the era when the workspace roster was the organizing unit.)
The word "session" appears
nowhere in yog code or UI. yog's vocabulary, exhaustively:

| Noun | Definition | Authority |
|---|---|---|
| **project** | A repo path with a balls clone: one entry under `$XDG_STATE_HOME/balls/clones/<pct-enc-path>/`, percent-decoded | balls |
| **ball** | A task: `tasks/<id>.md` in a project's store, read in-process through the linked balls crate (`reads::Catalog`, §16.7 W8); the closed listing alone is still `bl list -s closed --json` | balls |
| **workspace** | A lernie workspace: a directory containing `repo.git`; yog-started workspaces live at `$XDG_DATA_HOME/yog/workspaces/<name>/` (§3.1) | lernie (contents), yog (location) |
| **name** | Two names at two altitudes: a **workspace name** is operator-chosen at creation (validated shape, §3.1) — the sphere wall's label, the dir leaf, **and** the claimant stamped on every ball claim the workspace makes (§3.1, §3.2); a **conversation name** is minted (a single word from an embedded wordlist, bl-d12f) — yog draws it at preview and at fire and passes it via `--name` (§3.3, bl-08f2), **from `lernie::mint`'s embedded list** since bl-cd38 consumed the bl-aca4 ruling (§3.3's "state of the move") — the context window's own identity, durably lernie's `name` blob beside `goal.md` | lernie (the stored fact, and the mint); yog (the seed, the fire-time draw + preview) |
| **binding** | The derived association between a ball and a workspace: ball claimant = workspace name (§3.2). Balls-owned metadata, explicitly late-mutable via `bl claim`/`bl unclaim` — never a yog-stored fact | balls (claimant field); yog joins |
| **agent** | `agents/<id>` branch; the id is a chain of `<ts>-<short>` descent segments (ARCH §2.3 — two hyphen-free tokens each), which is where the hierarchy lives | lernie |
| **conversation** | A root agent plus its descent subtree — the §11 organizing unit. Its **identifier** is the root agent id; its **name** is a minted wordlist label lernie stores on the root's branch (§3.3, reversing bl-68d9's no-name rule) — minted for agent self-identity, rendered as the row title | lernie (the agents, the stored name, the mint); yog (the seed and the derived view) |
| **exchange** | lernie's presentational span (ARCH §2.4: root agent's history between a user message and the terminal response) | lernie |
| **attention** | A derived per-agent predicate (§6): unacked notify / stop / budget / conflict, or pending mail with no driver | yog (pure function) |
| **seen / pin / collapse** | The operator's durable, converging UI facts (§4.1) | yog (`ui.json`) |
| **draft** | Text typed but not sent, **for one target** — a new conversation in a workspace, or a message to an agent (§11) | RAM (the requirement's carve-out) |
| **world** | The nested substrate environment yog composes under its data root — the `LERNIE_HOME` / `XDG_STATE_HOME` / `PATH` override fold that redirects `lernie` and `bl` state into yog-owned roots and fronts the search path with yog's own `bl` shim (§16.2, §16.7 W9; brazen state resolves per workspace since the blast-radius ruling, §16.2) | yog (composed) |

**Rejected:** a first-class session/loop record (registry file, ball field, or
git ref mapping ball↔workspace) — a second name for a workspace path, i.e. a
stored duplicate of a derivable fact, which drifts. balls' unknown-key
writeback seam was considered for storing the workspace path in the ball and
rejected: machine-local paths in a shared store are the same mistake balls
itself refuses for worktree paths (balls arch §11). The claimant field is
neither: it is balls' own first-class metadata under balls' own merge
discipline, and a *name* is a machine-neutral identity, not a path — which is
why binding lives there (§3.2).

---

## 2. Invariants (the durability skeleton)

- **I1 — Disk is the app.** Every rendered fact is a pure function of (files
  on disk, probe observations). Restart is equivalent to re-read. This already
  holds for the read path (`tests/integration/pluggability.rs` proves N concurrent
  `GitTree::from_repo` calls converge); yog extends it to all state. The files
  are the *nested world's* files: every path derivation resolves through the
  composed world env (§16.2), so "disk" means yog's world, not the ambient one.
  **Two named exceptions, both display-only and both licensed by the operator**
  (§7.2): the pending echo (bl-915e) and the live tail (bl-54f7). Each is a
  fact the *painter* holds and nothing else may read — no derivation, no
  gesture, no §8.5 reply — so restart is still equivalent to re-read for
  everything anyone acts on. What they buy is latency; what they cost is
  nothing, because they are dead ends. What neither waives is I1's other half:
  **the frame still does no IO.** Both are published from off-thread into RAM
  the frame paints from.
- **I2 — Three durable yog artifacts, plus the monitor's policy while it is
  armed (amended bl-3381, bl-8da1).** yog owns
  exactly `$XDG_STATE_HOME/yog/ui.json` (§4.1), `$XDG_STATE_HOME/yog/ops.jsonl`
  (§4.2) and `$XDG_STATE_HOME/yog/cadence.yaml` (§7.2, absent by default —
  absence *is* the defaults), and — **only where the alignment monitor is
  armed** (VISION §4.9) — the policy file that monitor entry names, seeded
  beside them at arming. VISION §4.9 expected arming and policy to ride
  `cadence.yaml` alone and I2 to hold at three; implementation found the
  anchored block grammar (§9.4) cannot carry a multi-line prose policy in a
  four-space field without becoming the YAML parser yog refuses to have, and
  the policy *must* be editable prose — it is the operator's tuning surface,
  not a Rust constant. So the entry names a file and the file is the fourth
  artifact, with the arming it belongs to: unarmed there is no file, and
  deleting the file disarms as surely as deleting the entry does — these `$XDG_STATE_HOME` paths resolve through the composed world env
  (§16.2), i.e. nested inside yog's data root, which the world leaves anchored
  to the ambient `$XDG_DATA_HOME`. Every other write goes through the owning
  substrate's contract: task
  state via `bl` verbs; workspace state via `lernie` verbs only (never a direct
  write inside a workspace — ARCH §3.5; removing a *whole* workspace dir is a
  write to yog's own names root, not inside one — the §3.6 delete verb); brazen
  config as a
  `bz --dump-config`-validated atomic file replace; lernie global config as an
  atomic file replace (lernie declares these hand-edited).
- **I3 — All yog file writes are temp-in-destination-directory + `rename`.**
  Never in-place truncation, never a temp on another filesystem (EXDEV). Temp
  names are dotfiles (`.<name>.yog-tmp-<pid>`) so no substrate reads them;
  leftovers older than 24 h are swept at startup. `ops.jsonl` is the one
  exception: O_APPEND lines ≤ 4096 bytes (PIPE_BUF), atomic per line.
- **I4 — Watches are latency, polls are correctness.** fs-watch (notify:
  inotify/FSEvents) triggers re-derivation fast; a periodic sweep re-derives
  regardless (§7.2: 2 s cheap sweep, 15 s full sweep, clock-injected). A
  dropped event, a stale watch, an overflowed inotify queue never causes
  divergence — only delay bounded by the sweep interval.
- **I5 — Convergence discipline by state class.** `ui.json`: last-writer-wins
  whole-file with echo suppression. `ops.jsonl`: append-only, order-free
  union. Config files: optimistic hash guard (refuse to overwrite a file
  changed since load, §9). `bl` operations: converge-on-retry (balls' own
  recovery rule, arch §13). lernie repo state: lernie's writer/driver
  discipline (ARCH §2.11); yog is a pure reader.
- **I6 — The RAM whitelist is closed.** Only the items in §5.3 may exist
  without a disk home. Every addition requires amending this document.
- **I7 — yog never mutates any substrate except on an explicit user action.**
  No auto-prime, no auto-scan, no auto-push, no background repair. Two
  instances can never race a spontaneous mutation because neither has any.
  *Composing* the world env (§16.2) is pure — no mutation, always safe;
  *materializing* the world (creating the subtree, writing the `world/tools/bl`
  agent-tool shim, seeding `LERNIE_HOME` via
  lernie's own bootstrap verb, priming the nested balls clone, binding an
  agent's task branch, §16.3) is a mutation and so happens only on an explicit action — the
  first Start seeds a missing world exactly as it seeds a missing workspace
  (§3.4), never at idle. **The one write that answers to no user action is
  §7.2's own drift line** (`yog-drift <kind>`, exit `-4`, §4.2): yog recording
  that its watcher missed a change, or that a derivation pass outran its
  cadence. It is an *accusation about yog's own fidelity*, not a change to the
  operator's world — nothing is minted, seeded, claimed, pushed or spawned — and
  it exists because the alternative is a backstop that repairs silently, which
  is what I7 forbids most (§7.2: "it no longer repairs silently"). So a beat
  that reads I7 as "the ops tail is empty" is reading it in a unit it does not
  control: on a loaded machine a pass **is** late and the line **is** correct.
  INV-1 asserts I7 in the units that are yog's own — no process spawned, nothing
  minted or seeded, no verb's line — and drives its own clock so that elapsed
  time is the test's fact rather than the gate's (bl-9006).
- **I8 — A probe never perturbs the observed.** Liveness probing is read-only
  observation (lsof / procfs scans), never lock acquisition (§10, §14).
- **I9 — Determinism substitutes for persistence.** All ordering is derived:
  projects sort by path, workspaces by ball id, agents by descent order. Two
  instances render identical order without sharing any ordering state.

---

## 3. The organizing unit: the named workspace

Both roots below — `yog_data_root`, `lernie_data_root` — resolve through the
composed world env (§16.2), so the paths name locations *inside yog's nested
world*. The balls state root enters only through `bl` verbs (claims and
listings), never through path arithmetic: **the workspace tree encodes no
project paths and no ball ids** (§3.2 supersedes the original path-convention
binding).

### 3.1 Names: an operator-chosen sphere label, a flat root

**A workspace is long-lived and low-volume** — a sphere of work (personal,
corporate, a client) whose wall is lernie's isolation boundary; conversations
are root agents *inside* it (lernie §7.3), and balls flow through it (§3.2).
**The wall holds everything.** A workspace is an entirely separate space —
essentially an app-wide blast radius: different sets of conversations,
settings and providers, all of it. A workspace's conversations, its settings, and its providers — brazen's rows
included — are its own, behind the wall, never another workspace's (§16.2 as
amended; the per-workspace brazen config is bl-c0e2). Nothing spans the walls
but the roster of workspaces itself.
A new user never meets the concept: the first start bootstraps one (§3.4),
and further workspaces are raised only to wall spheres off from each other.
**The name names this wall and nothing else** — it is the dir leaf and
the ball claimant (§3.2), a *boundary's label*, not anybody's identity: no
conversation bears it (a conversation bears its own minted name, §3.3).

Every yog-started workspace is created at:

```
$XDG_DATA_HOME/yog/workspaces/<name>/
e.g.  ~/.local/share/yog/workspaces/cobalt-gecko/
```

- **The name is chosen at creation, by the operator** (bl-df65): a short
  explicit label for the sphere — `ops`, `dev`, an employer — typed into the
  New-workspace affordance (§11); a sphere wall is the rare, deliberate thing
  the human names on purpose. yog validates: lowercase ASCII
  alphanumeric words joined by single hyphens (`^[a-z0-9]+(-[a-z0-9]+)*$`),
  ≤ 32 bytes — path-safe on every §10 target — and never the literal
  `unknown` (bl's terminal `--as` fallback; a workspace so named would
  false-join every unstamped claim). A name equal to an existing leaf under
  any of the three roots is refused outright — no suffixing, no prompt-loop:
  the operator retypes. Equality with `$USER` is *not* refused: a hand-run
  unstamped `bl` claim would then join that workspace, which renders only and
  never mutates (I7) — the same accepted altitude as §3.2's cross-machine
  caveat. **The dir's existence is the registration**: the name namespace
  *is* the readdir; no registry file. Validation governs **creation only** —
  enumeration classifies by path and never validates, so pre-reversal minted
  leaves (`native-alarm`) and foreign leaves remain lawful names.
- **The bootstrap names without asking.** The empty-world start (§3.4)
  creates its workspace under the fixed default name **`home`** — a
  constant, not a config (severability: there is nothing to delete), and not
  a mint (the wordlist names conversations, §3.3). Zero-workspaces is
  the only state that takes the default, so it cannot collide locally, and
  the first Enter meets no name picker. The deliberate New-workspace verb,
  by contrast, refuses an empty name — raising a sphere wall is exactly the
  moment the operator has a name in mind. A workspace name is chosen or
  `home`, never minted.
- **Enumeration / reverse derivation:** readdir the root for directories
  containing `repo.git`; the leaf is the name. Workspaces under
  `<lernie-data-root>/workspaces/` are **foreign** (lernie's auto-id
  territory — rendered, unnamed, never created by yog);
  `<lernie-data-root>/replays/*` render as read-only replay workspaces. Three
  roots, one shape, classification by path alone.
- **Severability:** the root is yog's own territory, *not* lernie's
  machine-populated `workspaces/` tree. Deleting `$XDG_DATA_HOME/yog` erases
  yog's entire workspace footprint and leaves lernie and balls untouched —
  the same choice balls made in placing delivery worktrees under its own
  plugin territory (arch §1). **With the nested world (§16.2) this widens:** the
  nested `LERNIE_HOME`, the nested balls state root, and yog's own artifacts
  all live under `$XDG_DATA_HOME/yog`, so one `rm` erases the whole world and
  leaves the *ambient* lernie/balls/brazen untouched.
  Rejected: `<lernie-data-root>/workspaces/yog/…`
  — squats in lernie's retention-governed, auto-id-populated territory.
- lernie accepts any path for `lernie new [path]` (CLI §1); yog `mkdir -p`s
  the root (outside any workspace — the ban is on writing *inside* one) and
  passes `<root>/<name>`.

**Renaming, and the pre-reversal leaves (migration).** There is no rename
verb: a sphere with the wrong name is **replaced, not renamed** — raise the
chosen-name workspace (New workspace), move its bound balls across (§8.2
Move), and let the old workspace's conversations age out under lernie's
30-day retention; a hand `mv` of the dir plus per-ball unclaim/claim through
the hatches (§8.4) is the same operation spelled by hand, lawful while
nothing runs there. A ball whose claimant still names the old leaf renders
**claimed-elsewhere** — §3.5 already enumerates "a deleted workspace"; a
rendered fact, never a wound. Pre-reversal minted leaves need no migration
event: they remain valid names until their operator replaces them. (The
path-convention binding this section replaced is a §14 rejection: a location
is congenital where assignment must be late-mutable.)

### 3.2 Binding is the ball's claimant

The ball↔workspace association is **metadata on the ball**: a ball is bound
to a workspace iff its claimant equals the workspace's name.

- **Forward derivation:** ball → workspace = the dir named by the claimant.
  **Reverse:** workspace → balls = `bl list --json` filtered on claimant =
  name. Both are joins over facts balls already owns, mutates, and syncs —
  yog stores nothing (I2 intact).
- **Every claim a workspace makes is stamped with its name:** yog's own start
  flow claims `bl claim <id> --as <name>`, and the world's `bl` shim defaults
  `--as` to `$YOG_NAME` (§3.3, §16.7 W9), so the balls an agent authors and
  picks up register the same way — **"agents write the balls" is the normal
  case, not a deviation.** The ball-pickup event *is* the assignment record.
  The claimant is the **sphere binding**, not the speaker: it records which
  workspace a ball flows through, never which conversation spoke (the honest
  per-conversation limit below). A conversation's minted name (§3.3) never
  appears as a claimant — §8.1's load-bearing order claims *before* the root
  exists, the constraint bl-68d9 identified and bl-df65 preserves.
- **Explicitly late-mutable, by design:** assign = `bl claim <id> --as
  <name>`; move = `bl unclaim <id>` + `bl claim <id> --as <other-name>`;
  release = `bl unclaim <id>`. All are first-class UI verbs (§8.2), legal
  whenever balls allows them, at any point in a workspace's life.
- **N balls per workspace** over its life; a ball has one claimant, so at
  most one workspace at a time. Stop-scope, budget-scope, and retention-scope
  ride the workspace, not the ball.
- **Cross-machine caveat (accepted; re-argued at bl-df65):** the store branch
  is shared (§16.3), so claimants stamped by another machine's yog are
  visible here — and operator-chosen names collide far more readily than the
  minted two-word pool did (`ops` meets `ops`). The combinatorics defense is
  gone and is **not replaced**: a same-name workspace on two machines is
  either the *same sphere* deliberately spanning boxes — a true join, not a
  false one — or the operator's own naming to fix. The backstop is
  unchanged: a join only renders — it never mutates (I7).

**Justification:** single source of truth — the assignment's one
authoritative home is the ball's claimant field, owned by balls, mutated only
through `bl` verbs, synced by the task branch the claim rode (§16.3 — per
agent by default; a claim on the project's shared store branch is the one the
ambient `bl list` sees). **Rejected:** (a) a yog-side registry file — drifts, needs its own
merge discipline (the claimant is not a registry: it is balls' first-class
metadata under balls' discipline); (b) storing the workspace *path* in the
ball — machine-local paths in a shared store (balls arch §11); a name is a
machine-neutral identity, not a path; (c) binding in `goal.md` content only —
per-root prose, unusable as an enumeration source; (d) workspace inside the
delivery worktree (`<worktree>/.lernie/`) — captured by close's squash into
the project repo, disqualifying.

Parallel attempts, reprompts, and follow-ups remain *multiple root agents in
the one workspace* — exactly lernie §7.3's concurrent-exchange model ("new
question → `lernie prompt` forks new root"); the workspace stays lernie's
isolation boundary (§2.2).

**Two altitudes of ball attribution, both derived, honestly scoped.** The
claimant equality above binds a ball to a *workspace* — the enumeration source
for the workspace's bound balls (the roster/header). A *conversation* (a root
agent, §1) is finer than a workspace, and its association is derived from a
different fact, with a different reach:

- **Conversation → ball (start-flow only):** the start flow composes the ball
  header into the conversation root's `goal.md` (`Ball <id>:`, §3.3); parsing
  that stamp back is the conversation↔ball join. It is the **inverse of the
  compose**, so one module owns both (compose and parse live together) and the
  format has a single home. A start-flow conversation stamps **exactly one**
  ball, so a conversation carries **at most one** derived ball — never a set.
- **Agent-picked balls have no conversation-level record.** When an agent runs
  `bl claim` mid-conversation (the normal case, §3.2 above), the claim stamps
  the *workspace* name — there is no fact recording *which conversation* picked
  it up. Such balls therefore bind at the workspace altitude only. **This is a
  real limit, not a rendering choice:** per-conversation badges come *only* from
  the goal stamp; every other bound ball renders in the workspace header. A
  conversation-level pickup record does not exist yet, and until a fact carries
  it, none is invented (single source of truth — no yog-side registry, §3.2).

Both joins are pure reads over facts already owned elsewhere (the claimant on
the ball; the `Ball <id>:` line in `goal.md`), stored nowhere by yog (I2).

### 3.3 Work-target and conversation identity ride in the goal — through an editable composer

Lernie has no target-repo concept, and its tools do **not** inherit the
driver's cwd: the executor runs every tool subprocess in the agent's working
directory — the agent worktree by default, movable only by the agent's own `cd`
built-in, which writes the id-scoped mark (`refs/lernie/cwd/<agent-id>`) the
executor reads back at every spawn. **The work target is a typed parameter
(bl-6654, landed on lernie `=0.0.8`):** the fire passes the rung's binding as
`lernie prompt --cwd <path>` (upstream bl-d0b4), which seeds that mark at
creation — so *every* tool step of every later turn runs at the target, not
just the initial process. One channel, per rung: **ball** binds the
claim-derived `work/<id>` worktree, **path** binds the directory box's value,
**bare** binds nothing and lets lernie's own default (the agent worktree)
stand. Nothing is inherited by children.

Two weaker spellings were retired with it, because a fact with three channels
is three facts:

- **The goal-prose preamble is gone.** The ball prefill used to trail its body
  with "The project repository checkout for this work is the git worktree at:
  …", an absolute path the *model* had to read and obey — a location channel
  made of content. The goal is payload now: the ball's title and body, and the
  `Ball <id>:` headline, which stays because it is the §3.2 conversation→ball
  join, not a location. (The path rung's `Working directory: <dir>` first line
  stays too: it is that rung's headline — the display ladder's rung two, §3.3 —
  and the operator typed it.)
- **The per-target `current_dir` is gone.** Yog set the initial `lernie prompt`
  process's directory per rung. It reached that one process and no tool step,
  which made it look like a binding while binding nothing; DESIGN recorded it
  as misleading redundancy. `Prepared` carries no cwd field at all now — the
  detached driver simply stands in the workspace it drives.

Edits at the target still land in balls' external project worktree, outside
lernie's agent branch, commit-per-side-effect history, child inheritance,
bundle, and replay. That is the verified two-Git limit, and the binding does
not dissolve it: **ruled (bl-2b8c — VISION §4.10 is the cross-suite
authority)**, per-step observed project OIDs join project commits to agent
history by pointer, and attempt isolation and delivery ride balls
(bl-a1a4/bl-4eac, consumed by bl-8746).

The start flow (§3.4) opens a **prompt composer prefilled** per payload rung.
**The workspace stamp is the harness's, not the model's:** every
workspace-scoped spawn carries `YOG_NAME=<workspace name>` in its env (§8),
and the world's `bl` — yog's own shim, first on the agent's `PATH` (§16.7 W9)
— defaults `--as` to `$YOG_NAME` whenever the caller omits it. The agent
cannot forget what it never had to remember.

**The goal fires verbatim (bl-6920).** Yog
prepends **nothing** to the payload: the first user message reaches the model
exactly as the operator (or a ball body) wrote it. Identity's one channel is
`--name` — lernie commits the name beside `goal.md` and **states the stored
fact in its assembled context** (lernie bl-d55f, released 0.0.4:
`compose_system` derives `Your name is <name>.` into the system slot from the
`name` blob, never a second copy). The retired `You are <name>.` stamp — one
line, no instruction, the bl-df65 interim channel — survives only as a
**legacy parse** (below); no live path composes it.

**The workspace never enters the prompt (bl-df65).** The workspace name is
*operative* in exactly one channel, the harness's: `YOG_NAME` rides every
workspace-scoped spawn and every tool subprocess inherits it (§8), the shim's
`--as` default stamps it on every claim (§3.2). The spawn's `current_dir`
selects the initial prompt process's directory (§3.4), but pinned lernie
rebinds each tool to its own agent worktree (the limitation above). A prose
line restating the workspace identity would still be a second copy of a fact
the env already carries; an agent that needs its sphere — to tell its own
workspace's claims from a sibling sphere's in `bl list` — reads `$YOG_NAME`,
the one home, at need ("identity rides the env, not the argv", §16.4).

**A conversation bears its own minted name (bl-df65).** The name is not a
display label — it is **agent self-identity**: two agents collaborating
without a discriminator confuse themselves for each other, or blame their own
acts on the operator or a parent; the name is what lets a context window know
which one it is. The first payload line cannot do that job (two conversations
spawned off one ball share it verbatim), and the workspace name did the
opposite of it (N conversations in one sphere all claimed to *be* the
sphere) — so the name names the context window itself. The ball claimant
stays at the workspace altitude: §8.1's order claims before the root exists
and never reads the root id back (§3.2); a conversation's name never becomes
a claimant.

The mint — **lernie's `lernie::mint`** since bl-aca4, consumed by bl-cd38 (see
"state of the move"): a **single word** from a wordlist embedded in that crate
(bl-d12f, retiring the two-word compound), a pure function over an injected RNG
and an occupied set. On collision with the occupied set the candidate is
discarded and the next word tried — the retry is bounded by the wordlist itself
(one wraparound scan), erroring loudly on an exhausted pool rather than
looping. The pool is 541 words, sized against the occupied set it actually
races (one workspace's living agents — tens, recycled by retention), not
against a birthday bound. Yog calls that one function at preview and at fire.

- **The occupied set is per-workspace, and already derived (bl-08f2):** the
  names the target workspace's living agents wear, read off the same name
  fact the display ladder reads — the lernie-stored `name` blob, with the
  legacy goal-stamp parse as fallback while pre-0.0.4 roots live. Children
  count too, and must: lernie refuses a name any living agent already wears,
  so a mint blind to a named child would fail at fire. No cross-workspace
  enumeration: workspaces are isolation walls — two agents in different
  spheres never meet, so global uniqueness would buy nothing.
- **The pool does not burn:** occupancy is one workspace's live roots
  (dozens at most), lernie's 30-day retention recycles it, and a recycled
  name is lawful — the name discriminates the living; the root id stays the
  identifier.
- **Nothing is stored that can be computed — and yog stores nothing:** the
  name's one durable home is lernie's `name` blob on the agent's own branch
  (below; formerly the goal's first line, the bl-df65 interim). No yog
  registry, no `ui.json` field, no second home — reading a name is
  `git show agents/<id>:name`, a query.

**Ruled (bl-50f3): the name's durable home moves to lernie.** Prose-only
naming failed live: agents message peers through lernie's `message` tool,
which resolves only `agents/*` ref ids, while the operator and every UI
surface speak names — an agent told to message `shudder-storeroom` could not
resolve it, twice. The name is agent identity, agents live in lernie, so the
fact becomes lernie's: `lernie prompt`/`dispatch` grow an optional `--name`
(the name committed under the agent beside `goal.md`, so `agents/*` refs stay
the only registry and retention recycles names with zero cleanup; uniqueness
among the living is refused at creation), and `message` resolves exact id
first, else unique living name (lernie bl-c8ed; released as 0.0.4, lernie
bl-4c15). yog stays the minter — the mint above is unchanged, still previewed
before spawn (I7; the name exists before the agent id does, which is also why
deriving the name *from* the id was rejected as impossible — and name-as-id
was rejected because ids must never collide while names lawfully recycle) —
and becomes a pass-through and reader — **wired by bl-08f2**: the fire
spawns `lernie prompt --name <minted> <workspace> <goal>` (the goal stays
last in the argv, which is also what keeps the ops-log clip and the
detached-sink join key positional-from-the-tail), on a lost-race re-mint the
re-derived name is what passes, and the ladder's rung one reads the lernie
fact back (`Agent::name` at enumerate time, `git show agents/<id>:name`).
The `You are <name>.` stamp is no longer a fact home — and by the later
bl-6920 ruling (**landed**) it is retired as a channel too: the first user message reaches the model unmutated; self-identity
belongs in the harness-assembled context, which is lernie's job (lernie
bl-d55f, verified in the pinned 0.0.4: `compose_system` states the stored
name fact in the system slot). The stamp's compose is deleted. Its parse
survives as the one legacy rung (`Agent::name_fact`'s fallback), kept
**solely** because pre-0.0.4 roots carry no `name` blob — the stamp's first
line is the only record of their name — until 30-day retention ages them
out; then the rung is deleted. New roots never match it: nothing composes
the shape anymore. A workspace registry file yog writes and lernie reads
was rejected outright: two representations of the living-agent fact that
must drift, in a format neither crate owns.

**Ruled (bl-aca4): name is an exposed dispatch parameter; omission
auto-mints — and the mint's one home moves to lernie.** A name belongs on the
dispatch command itself: it already tells the depth, it keeps subagents'
identities and tasks clear, and it simplifies the whole naming question —
an omitted name is generated dynamically, a supplied one is honoured, and
either way the parameter is exposed. This **amends bl-50f3's "yog stays
the minter"** — the storage half of that ruling (the name's one durable home
is lernie's `name` blob) stands untouched; what moves is the mechanism's
address. bl-50f3 could leave the mint in yog because only yog's fire minted;
the moment *every* creation path must mint on omission — the `dispatch`
tool, `lernie dispatch`, `lernie prompt`, none of which pass through yog — a
yog-resident mint would leave lernie either calling up the stack (inverting
the dependency) or growing a second list. So the wordlist + draw + bounded
retry move **into lernie beside the uniqueness check they race**
(`agent_name::require_available`): one settle-the-name seam per creation
pre-flight — a supplied name validated, an absent one minted against the
same living-names scan (`agent_name::named`) uniqueness reads — and no fork
can end nameless. Unnamed stays a *readable* state (pre-aca4 agents, the
ladder's lower rungs, until retention ages them out) but stops being a
creatable one.

- **The contract:** `name` is a prominently exposed parameter on every
  creation surface, and the `dispatch` tool schema teaches it — the
  operator's stated purpose is that dispatching models keep child
  identities clear in subagent trees, so the description says what a name
  buys (a `message`-addressable, tree-readable child) *and* that omission
  mints silently and validly. The mint's interface is
  `mint(rng, occupied) → name | exhausted`: RNG injected as a trait (tests
  script the draw; production seeds from entropy), the wordlist an
  implementation detail *behind* the function, never an exported surface.
- **Yog is a consumer, not a second minter:** yog deleted its local
  mint (wordlist and all; `src/names` keeps only the §3.1 workspace-name
  validation) and calls lernie's through the crate it already links (the
  §16.7 multiplex proves the linkage). **Landed at bl-cd38** — see
  "state of the move" below. Everything operator-facing is
  unchanged: the composer still previews the predicted name before anything
  spawns (I7), the seed lives exactly as long as the prediction it backs
  (bl-28ba — preview timing and seed lifetime are yog *policy* over
  lernie's *mechanism*), and the fire still mints and passes `--name`,
  because the fire's return — the minted name — is the §3.4 focus-claim
  handle that must exist before the agent id does. Yog never omits the
  parameter; omission-minting is for the callers that have no preview to
  keep.
- **Preview parity:** preview and spawn draw from the same function. The
  occupied sets differ only by yog's legacy goal-stamp rung, which makes
  yog's a *superset* of lernie's living-names scan — the safe direction
  (yog may avoid a word lernie would allow, never predict one lernie would
  refuse) — and the discrepancy dies with the legacy rung. The lost-race
  story is bl-50f3's, unchanged: the preview is a prediction, the fire's
  fresh mint is the truth, and `require_available` at creation stays the
  actual uniqueness gate — the residual race (two mints landing the same
  word between scan and commit) is refused there, loudly, exactly as a
  hand-typed collision is.
- **Rejected — lernie fallback-mints with its own list, yog keeps its
  own:** two lists and two draws are two representations of one behavior
  and must drift; yog's preview could predict names lernie's fallback would
  never produce, and every wordlist curation lands twice or diverges. The
  mechanism is one function or it is two facts.
- **Rejected — `name` required, every dispatcher always supplies:** directly
  against the ruling ("if a dispatch command omits it, we can provide it
  dynamically"); it multiplies minters — every model, script and human
  becomes one — and turns a forgotten name into a refused dispatch, a
  failure apiece where one default dissolves the class.
- **Rejected — lernie mints always, yog stops passing `--name`:** the
  preview dies or I7 does — the name would exist only after the spawn, yog
  would read it back on a race-prone poll, and the fire would return
  nothing to hold the §3.4 focus claim by.
- **Depth ("already tells the depth"):** dispatch implies parent→child, so
  depth is already in the descent tree (the `<parent>-<sub-id>` branch
  shape) and stays **derivable — no depth field anywhere**: not in a name,
  not in a blob, not in a schema.

**State of the move: landed (bl-cd38, on lernie 0.0.8).** The paragraphs above
describe the tree, not a plan. Yog's wordlist and draw are **deleted** —
`src/names/words.txt` is gone, and `src/names/mod.rs` holds the §3.1
workspace-name validation and nothing else. Preview
([`start::identity_preview`]) and fire ([`start::prompt::execute_prompt`]) both
call `lernie::mint::mint(rng, occupied)` over the crate's `Rng` trait and its
`SplitMix64`; the wordlist stays behind the function, unexported, per lernie
ARCHITECTURE §3.4. Yog's own seat in the seam is the **seed**: `shell::clock`'s
`entropy_seed` is still the one home for "now" and feeds
`SplitMix64::from_seed`, which is what makes the §3.3 preview and its fire
agree and what bl-28ba's per-prediction seed lifetime is expressed in. The
corpus that lands is lernie's clean-room 541-word list (lernie bl-b59c), which
replaced an EFF-derived CC BY 4.0 list — the licence and the hostile-word
problem left the tree with yog's copy of it, rather than moving into lernie.
The acceptance fixture pins the pair `MINT_SEED`/`MINTED_FIRST`, so a corpus
change in the crate fails loudly at `shell::acceptance::mint_seed` and names
its cause.

**Scope: every creation names; the ladder names whatever lernie names.**
Since bl-aca4 no creation path yields a nameless agent — a dispatch child, a
hand-run `lernie prompt`, and yog's fire all end named, supplied or minted.
The name fact rides any `agents/*` ref and the enumerate read
(`Agent::name`) makes no root/child distinction, so the ladder shows every
new agent by rung one with no yog special case (bl-08f2's read, unchanged —
which had already lifted the bl-df65 honest-scope limit: back then no fact
carried a child's name, so none was invented); the lower rungs keep naming
the pre-aca4 stock until retention ages it out.

**Display: the name is the title; the first payload line is the preview.**
One function derives what a conversation is called, as a ladder (bl-08f2):
the lernie-stored name fact → the legacy goal-stamp parse → the first
payload line (the goal with the stamp stripped) → the root agent id. The two
name rungs fold in one place (`Agent::name_fact`), so retiring the legacy
rung is one deletion. The §11 row title, the center header, the descent-tree
member row (bl-df72 — that seat painted the raw id, the operator's
"incoherent timestamp"), the in-flight strip, and the composer's
`message <x>` target label (bl-2f30) all read the fold and fall through;
foreign, hand-typed, and post-bl-6920 unnamed roots land on the payload
line or the id. **No seat formats an agent id as a display name** — the id
is a fact whose display seats are the ladder's own floor and the hover (the
member row and the center header both keep it there); an acceptance source
scan holds the rule the way the §11 hover invariant is held. **The floor
spells the terminal generation only** (bl-63a1): a lernie child id embeds
the full ancestry chain — one `<stamp>-<hash>` pair per generation — and
the descent tree's indentation already states the lineage, so when the
ladder bottoms out at the id it renders the agent's own trailing
`<stamp>-<hash>` segment (a root id, one generation, is its own terminal
segment; an id the stamp grammar does not recognize is spelled whole), and
the full id's seat stays the hover. **The floor has more than one seat**
(bl-3aa1): an inbox deposit's `from` is an agent id too, and those rows led
with the depositor's whole four-token chain — 52 characters heading a row
whose other content is a timestamp and a subject — because the acceptance
naming scan named `agent_id`/`root_id` and a deposit carries the fact under
`sender`. That is bl-63a1's own lesson repeating verbatim, so the fix is
both halves: `inboxview::header_line` reads `nav::convs::id_floor` like
every other seat, and the scan's vocabulary is now every identifier the fact
travels under, with a new carrier belonging in the change that introduces
it. The strip is
the retired compose's other inverse — parse and strip live in one module
(`start::identity`), where the retired shape has its one written record.
The legacy rung has **one** shape to recognize (bl-2706): `You are <x>.` A
legacy root whose `<x>` was a workspace name parses by the same rule — an
accepted, bounded, display-only misread until retention ages the goal out;
for every goal without the stamp — which is every new one — the strip is
the identity function, the general path. **A legacy-rung title says it is
display-only** (bl-8068): lernie resolves a message target by exact id, else
unique *stored* name — never the goal-stamp prose — so a title with no `name`
blob behind it (`Agent::name_display_only`) is unaddressable. Every seat that
shows it says so: the §11 row title and the centre header hover
`theme::NAME_DISPLAY_ONLY`, which names the agent id as the address that
works, and **the boundary withholds the `name` key entirely** for such a row
(`display` still carries the ladder's answer) — the machine-facing surface
never hands a peer a name lernie will refuse. Diagnosed from the field: an
operator read "marbling-lake" off a pre-fact row, told a peer to message it,
and lernie refused with `no agent "marbling-lake" in this workspace`. The
headline invariant is what
makes rung two worth showing: **every prefill yog composes
leads with its headline** — the ball rung's first line is
`Ball <id>: <title>`, the path rung's is `Working directory: <dir>`
(reworded from a sentence that buried the path on line two), and the bare
rung is the operator's own ask, which leads with intent by nature. Two
conversations with the same preview stay distinguishable by name, state,
age, and the root id (the true identifier): a preview is a subtitle, not a
key — the same distinction balls draws between a task's title and its id.

**The `--as` stamp is applied where balls reads the default, not by rewriting
argv:** the multiplex `bl` arm resolves `Edge::default_actor` as `$YOG_NAME`
→ `$USER` → balls' `"unknown"`, so an explicit `--as` still wins and a verb
that takes no `--as` is untouched. One rule, no per-verb flag table.

The path rung appends a target preamble whose first line is the headline —
`Working directory: <dir>`, the directory verbatim — followed by the
work-there-by-absolute-path sentence; the ball rung composes the header and the
ball body verbatim, and nothing else (bl-6654 retired the worktree preamble;
the path is the typed `--cwd` binding now, not prose):

```
Ball <id>: <title>

<ball body verbatim>
```

**Transparency, scoped by ownership:**
the operator sees and edits exactly **the whole payload** that is sent — their
own text and the target/ball prefills, fired verbatim (bl-6920) — before
`lernie prompt` fires; **identity is the harness's fact, passed as `--name`
at fire time** (the same ownership line as `YOG_NAME`/W9: the harness carries
identity, the model reads it from lernie's assembled context, the operator
never types it). The composer *previews* the predicted name greyed above the
box (`will be named <name>` — worded as the prediction it is, never as a goal
line) — the mint is a pure read (the target workspace's derived name facts +
RNG, drawn through `lernie::mint` since bl-cd38 landed the bl-aca4 move), so the predicted
name renders before submit with
nothing spawned (I7 intact) — and on the rare lost race (another instance
took the name between preview and Enter) the mint re-derives and passes
the fresh name; the preview is a prediction, the fired mint is the truth.
**A seed
lives exactly as long as the prediction it backs** (bl-28ba): the RNG seed is
held across frames so preview and fire agree, and re-rolled the instant a fire
lands, because that prediction has been spent. Held longer — one seed per
session — every fire took the same single draw, so every start after the first
landed on an occupied slot and walked forward off it; the pool is
first-word-major, so the walk paid out siblings (`recite-a`, `recite-b`,
`recite-c`): unique names, but a fleet the operator cannot read apart. A
refused or failed launch minted nothing and keeps its seed. The
binding mechanic stays transparent, and no goal-template config file exists
(the visible editable prefill is the severable version; deleting nothing
changes no code path). Yog sets **no** per-target `current_dir` on the initial
spawn any more (bl-6654): it was never a work-target mechanism — every later
tool cwd is the agent's own working directory (the agent worktree, unless its
`cd` or the `--cwd` seed moved the mark) — so a second, weaker spelling of the
target is deleted rather than kept beside the real one. `Prepared` carries no
cwd field; the detached driver stands in the workspace it drives, the same for
every rung, and the creation-seeded mark (VISION §4.10 item 2) is the one
channel — no misleading redundancy.

The worktree path is never stored (balls arch §11: "computed, never stored");
it is recomputed by the bl-delivery formula
(`$XDG_STATE_HOME/balls/plugins/bl-delivery/<mirrored-project-path>/<id>/`)
and cross-checked against `bl claim` stdout at claim time; `git worktree list`
in the project repo is the standing ground truth. Because the path is a pure
function of (project, id), an unclaim/re-claim re-materializes at the same
path — the preamble never goes stale.

### 3.4 Lifecycle: the start flow

**Two orthogonal axes, one composer.** *Where* a prompt goes: the focused
workspace — and a world with zero workspaces creates one first under the
default name (`home`, §3.1), so **bootstrap
is the empty case of the general path**, never a wizard or a concept the new
user meets. *What* it carries: the payload ladder, each rung the one below
plus inputs.

| Payload rung | Input | Extra steps | Composer prefill (the prefill fires verbatim, bl-6920; the name is previewed grey and passed via `--name`, §3.3) | Typed work target (`--cwd`, §3.3) |
|---|---|---|---|---|
| **bare** | — | (none) | (none — the empty composer) | (none — lernie's default, the agent worktree) |
| **path** | a directory | (none) | target preamble, path verbatim | the directory |
| **ball** | a ball, picked or freshly created | `bl claim <id> --as <name>` | `Ball <id>: <title>` + body | the work worktree |

- **Creating a workspace is the rare, deliberate verb** (New workspace,
  §11): raising a sphere wall — a client, corporate vs. personal — not a
  per-conversation act, which is why it is also where the operator types the
  name (§3.1). The everyday gesture is the composer: a new prompt is
  a new root in the focused workspace (lernie §7.3), and a ball pickup claims
  `--as` that workspace's name (§3.2). **The raise ends at the composer**
  (bl-9acf): it is the bare rung, so its prefill is "none" per the table above
  and §8.1 step 2 opens no start draft over it — what the operator gets is the
  sphere focused and one box holding the keyboard, not two boxes one of which
  is empty.
- **A start focuses what it started.** The *where* axis is decided
  once, by the start flow, and the focus **is** that decision — never a second
  one the operator has to repeat by hand. So New workspace selects what it
  raised,
  ▶ Continue selects the resumed ball's own claimant workspace, and the
  bootstrap selects the workspace it just founded; a start into the
  already-focused workspace re-focuses it, which is a no-op. One unconditional
  rule, not four cases: the tab bar, the conversation list, and **both**
  composers (the start pane's editable goal and the pane-docked one, §11)
  read the single focus, so they cannot name two workspaces at once
  (bl-2826).
  The rule runs all the way down to the **conversation** (bl-49cb): a fire
  selects the root it started, so the transcript with the streaming tail is
  what the center renders after Enter — STORIES S0.3's *"the reply streams into
  the focused view"*, which the workspace half alone left one ↓ short. Its one
  wrinkle is timing, not policy: the started root has no `agents/<id>` ref
  until the detached driver writes one, so the fire cannot name an agent. It
  claims the conversation by the **minted §3.3 name** — the name fact the
  derivation reads back off every root (lernie-stored, legacy goal stamp as
  fallback), unique per workspace by the
  mint's own occupied set — and the frame spends that claim through the
  ordinary `focus_agent` path (the one ↓ takes, acknowledging §6 identically)
  on the first roster that carries it. A claim whose root never appears is
  inert, and no claim survives being spent, so the operator's own later
  selection stands. The claim is per-instance RAM like the focus it becomes
  (§13.1); nothing about it is written down.
  **The claim carries the operator's text with it** (bl-915e): a handle that
  paints no row left the message with no representation anywhere in yog between
  Enter and the driver's first write, which is what the operator saw as the UI
  waiting for the send. It is that one claim extended — §7.2's pending echo, not
  a second pending concept beside it: the same value names the conversation,
  holds the goal, and is retired by the one predicate that also spends the
  focus.
- The path rung's directory need **not** be a bl-primed project — it is the
  typed working-directory binding the fire passes as `--cwd` (bl-6654,
  consuming bl-2b8c's ruling, VISION §4.10), and a path start binds the
  directory while carrying no delivery obligation: no target, no attempts, no
  delivery.
- **The directory is a birth parameter, and it is pre-filled** (bl-7927): an
  editable text box with the default pre-chosen, at the top in the config
  block rather than at the bottom beside the message. Its one carrier is the §11 birth-config block's
  editable box — never the composer, which carries the message and nothing
  else. The box is seeded at boot with the **bare rung's own resolution**, the
  operator's home dir, spelled as the absolute path it is rather than a `~`
  nothing in yog expands. Two consequences: the path rung stops being a mode to
  opt into — leaving the box alone runs exactly where the bare row above says,
  so the rung the operator gets follows from the one visible fact — and the box
  survives a send, because a parameter the block *states* is not a draft (§5.3
  governs the message, not this). A box emptied by hand is the bare rung, which
  resolves to that same home: a value, not an error. **"At the top" and the
  settings-seat ruling do not collide** (bl-2e18): what bl-7927 refused was the
  box riding the composer as `dir (optional)` — a birth parameter loose in the
  drafting seat — and its remedy was the config block. The block has
  since moved as a unit to the settings seat (§11), so the row is still in the
  block and still not in the message box; only the block's own seat changed.
- **A directory that is not there is refused at the field** (bl-6191), in §3.1's
  own idiom: the box flags in ichor with the reason and the composer's Enter
  disarms, before anything spawns. The question the field asks is the *spawn
  boundary's* (`cli_outbound::work_dir_fault`), asked one step earlier — one
  reading of "lawful cwd", one sentence for it, so a pre-flight refusal and a
  forced spawn failure can never word the same fact differently.
- Re-opening is the same path as opening: an existing dir skips `lernie new`,
  dissolving any "resume" special case. During work, everything is lernie's
  normal surface; yog renders and dispatches verbs (§8.2), including
  late assignment of further balls (§3.2).
- **`bl close`:** the ball file is deleted; the closed listing's claimant
  still names the workspace, so *that query is the "delivered" status* —
  delivered balls group under their claimant workspace on demand
  (`bl list -s closed --json`; obituary via `bl show <id> --json`;
  delivered-in commit via `git log --grep "[<id>]"`). yog deletes nothing *at
  close*: workspace retention is lernie's (§9.2, 30-day default); `lernie
  bundle` is the archival verb (deferred, §8.3). The one deliberate destroyer
  in the whole surface is workspace deletion (§3.6).

### 3.5 Join states (the edge-case-dissolving enumeration)

The join is the claimant equality (§3.2), enumerated once, as a table; every
combination renders as a row state, never an ad-hoc branch. All derived, none
stored:

| Ball (derived status) | Claimant | Rendered as |
|---|---|---|
| ready | — | ready ball: ▶ Start (ball rung) or Assign to an existing workspace |
| blocked | — | blocked ball (blocker edges shown from bedrock JSON) |
| claimed | = a local workspace name | **bound** — the normal working row, grouped under its workspace |
| claimed | ≠ every local workspace name | **claimed-elsewhere** — badge shows the claimant verbatim (a human, another machine, or a deleted workspace) |
| closed (absent from live set) | (from the closed listing) | **delivered** — grouped under the claimant workspace when one matches, else visible in the on-demand closed listing |
| (none claim it) | workspace exists, zero bound balls | **unassigned workspace** — the bare/path-rung general case: full rendering, no ball column |
| any | project clone gone | **orphaned-project** — the project's balls unlistable, marked missing; workspaces unaffected (they encode no project path) |

**Conversation-level rendering is an overlay on this same table, keyed by the
goal stamp (§3.2).** A conversation row's ball badge is the ball its `goal.md`
stamps (`Ball <id>:`, §3.3), coloured by *that ball's* row-state above — bound
green, delivered ash, blocked/claimed-elsewhere brazen, orphaned ichor. The
join table is unchanged: the overlay looks the stamped id up in it. Two honest
gaps follow from §3.2's scoping, each a plain `None`, never a fabricated row:
(a) a conversation with no start-flow stamp (bare/path, or a hand-typed root)
shows no badge; (b) a stamped id the join does not know here (its project
unfetched, or the ball since deleted) still shows the id from the stamp, but
uncoloured — the badge is source-1 truth, the colour is the join's when it has
one. The **grouped-by-ball** organizing view is this overlay inverted: each
stamped ball heads its conversations, the stamp-less ones trailing in one
unassociated group — a pure, stable partition of the recency-sorted list.

**Spend rides this same join, and stops exactly where the join does (bl-afc4).**
Cost per ball is a *query* over facts that already have owners, which is the
only reason it can exist without a new store: brazen counts tokens and never
learns a price (its stated boundary); lernie commits that count into the step
record; balls tags every delivery `[bl-id]` and stays metric-free. **yog owns
exactly two things nobody below it may hold — the price table and the join** —
and stores neither the table's product nor the figure (I2 intact; a figure is
re-derived from disk like every other §5.1 fact).

- **The price table is world config**, one `ui.json` object keyed by model id
  (§4.1 `prices`), read-only and hand-edited. **Severable in the strong sense:
  deleting the key deletes a column, not a code path** — an absent table is the
  default, the one gate is "is the table empty", and a yog that was never
  priced renders precisely the token figures it always did. No crate below yog
  ever learns a rate.
- **The join is `Σ(step usage over the agents tied to the ball) × prices`,**
  priced *per step by that step's own model* — the model is read back off the
  step's `request.json`, so a conversation whose model changed mid-flight bills
  each step at what it actually ran on. There is one steps-tree walk feeding
  both figures (`budgets::bills`): the token figure drops the model, the spend
  join groups by it.
- **Attribution is the §3.2 two altitudes, said out loud.** A ball a
  conversation's goal *stamps* attributes at conversation granularity — the
  stamped roots and their whole descent, two stamps in one descent resolving to
  one root so a tree is never billed twice. A ball an agent claimed
  **mid-conversation** stamps only the workspace name, and **no fact anywhere
  records which conversation picked it up.** The ruling:
  **accept workspace-granularity attribution for such a ball** — sum the whole
  workspace and *label the figure as that*, an upper bound rather than a
  claim. **No linkage fact is invented.** A yog-side conversation↔ball registry
  is exactly what §3.2 already refused, and refusing it again here costs one
  honest label instead of a second home for someone else's fact.
- **Unpriced is reported, never rounded to free.** A step whose model the table
  has no rate for contributes to an `unpriced_tokens` count rendered beside the
  money, so a partial table reads as a floor — which is true — instead of a
  number that is quietly wrong. Money is micro-USD integers end to end; the one
  `f64` is the operator's decimal rate at parse time and it does not survive it.

**Where it renders:** the conversation's settings rows at the bottom of the
altitude-1 surface (§11 — moved off the header by bl-2e18), which seat the whole-tree token figure — one line per bound
workspace ball plus the open conversation's own; and, since bl-9dd4, the V4
board's spend column and epic rollup (§11's board paragraph). The follow-on this section recorded — *"a
rollup crosses workspaces (a ball's children may be claimed anywhere), so its
enumeration source is the board's join, not this one"* — landed on exactly that
term: `board::rollup` folds over `Snapshot`, and `spend` still knows one
workspace at a time.

**The walk is the worker's; the join is anyone's (bl-9dd4).** `budgets::bills`
used to run wherever a figure was asked for, which meant *on the frame thread*
— tolerable for one workspace's header line, and a guaranteed freeze for a
board that wants a figure per row per frame (§7.2: the frame renders snapshots
and reads no disk). The fold now runs once per workspace per derivation pass
and rides out as `Snapshot::bills`; every figure is a filter over it. Three
consequences, and they are the whole change:

- **A bill names its conversation.** `StepBill::conv` is the step's conv-id
  dir, so `Scope` became a predicate applicable *after* the walk rather than a
  parameter to it. One rule ("is this conv in that tree"), one home — a second
  copy is how a row's figure and a rollup's would drift.
- **The fold sits outside the tree's equality gate.** A step's `response.json`
  growing is spend that changed while every git ref stood still; folding behind
  `old == new_tree` would freeze the column at whatever it read when the refs
  last moved. `steps/` is already in §7.1's Workspace allowlist, so the change
  is announced and no sweep has to find it.
- **Nothing is stored.** The bills are derived-from-disk like every other §5.1
  fact, published inside the immutable snapshot and re-walked on the next pass.
  I2 is intact: this is a cache of a *read*, on the worker, invalidated by the
  same watch that invalidates the tree beside it — not a second home for a
  figure.

**The ceiling landed with the boundary, and it refuses exactly one thing: a
birth (bl-56d5).** The shape was fixed before the code — *it gates spawns and
never kills a running drone*, because killing mid-ball destroys uncommitted
work and early termination is the expensive failure. What was missing was the
refusal's *seat*, since a gate covering one spawn path and not the others is
worse than none. §8.5 supplied it.

- **The seat is the `Prompt` door** (`boundary::dispatch::prompt`, gate in
  `boundary::ceiling`). Every drone yog ever births is fired by that one
  function — the §8.5 dispatch match's `Prompt` arm delegates to it and the
  frame's `fire_prompt` calls it directly — so one gate covers the click, the
  slash line, the deposit and `yog gesture` at once. **There is no second
  gate anywhere**, which is the whole point of seating it at a chokepoint.
- **A birth is the only thing gateable, and that is a ruling, not an
  omission.** `Message` is *not* gated: refusing to answer a drone that is
  mid-ball strands exactly the uncommitted work the ceiling exists to protect,
  which is the same expensive failure as killing it. `Prepare` is not gated
  either — a claim spawns nothing and is releasable, so the refusal belongs at
  the one irreversible step rather than one step early. The bound on a drone
  that is *already alive* is lernie's own `max_total_tokens`, one layer down,
  where the loop that spends it runs; yog's ceiling bounds the fleet's
  births and says so.
- **The value is one `ui.json` number** (§4.1 `ceiling`), USD, beside the price
  table it is denominated in — **severable in both directions**: delete
  `ceiling` and the gate is gone; delete `prices` and it is gone too, because a
  ceiling is a dollar figure and yog refuses to bound dollars it cannot compute
  rather than inventing a token proxy for them. No new config artifact, no
  setter, no verb, no flag.
- **The figure it compares is the target workspace's**, the same
  `Attribution::Workspace` sum this section already accepts as a ball's honest
  upper bound: a workspace is the sphere a drone lives its ball in, and it is
  the one scope a spawn names outright without inventing a linkage fact nobody
  stores. At-or-over refuses. The comparison is against the figure's **floor** —
  unpriced tokens are reported (above) and never guessed at — so the gate
  refuses only on spend it can actually name, and a literal `0` is honored as
  written: the deliberate hard stop.
- **A refusal renders where refusals already render.** It writes the §4.2
  `["yog-step","ceiling"]` failure line before it rides back, carrying the
  start's own `Origin`, so it banners at the rung that fired it (§7.3) and
  counts toward §6 attention like any other failed action; the composer keeps
  its goal and its unspent mint seed, exactly as any other failed fire does.
  The text names both figures and the key to edit. Nothing is refused
  silently, and nothing running is touched.

### 3.6 Deletion: unmaking a workspace (bl-ef89)

**Deletion is the raise's inverse, at the raise's altitude.** Raising a
workspace is `mkdir -p` the names root + `lernie new <root>/<name>` under the
operator's chosen name (§3.1, §3.4); deletion is the
release of the sphere's live claims followed by removal of the workspace
directory. lernie has no workspace-delete verb, and that is not a gap to work
around: `lernie new` accepts any path — the caller owns placement, so the
caller owns disposal. (An earlier draft grounded this in "lernie's retention
is branch-level GC inside a living workspace" — **false against the shipped
crate**: lernie through 0.0.4 ships no retention or GC of any kind, and since
0.0.4 it ships an *agent*-scoped `lernie delete` — the verb the
one-conversation delete below spawns. The conclusion stands on the placement
argument alone, corrected bl-f17a.) Removing the whole dir is a write to
yog's **own names root** (§3.1: "the root is yog's own territory"), never a
write *inside* a workspace, so I2's lernie-verbs-only rule stands untouched.
And because the dir's existence is the registration (§3.1), the removal **is**
the de-registration — no registry to update, nothing to sync, the name free
for re-use by the same readdir that always was the namespace.

**What it destroys — everything inside the wall, irrecoverably:** the
workspace directory whole, which is `repo.git` (every conversation: agent
branches, transcripts, config lineage, marks), `steps/`, `inbox/`, and every
agent worktree — **and the sphere's wall** (§16.2): its brazen config, its
sign-ins and its model cache, removed in the same step as the directory. A
wall left standing would hand a dead sphere's credentials to the next
workspace that takes its name, which is the one thing §3.1's name-reuse case
must not do; a sphere that never configured a provider has no wall dir, and
that absence is the same end state, not a failure. That is the point — the verb takes a sphere wall down, and
the wall's contents go with it. Until `lernie bundle` is surfaced (§8.3,
v1.1) there is no archive step to offer, so the confirmation *states*
irrecoverability instead of mitigating it; when bundle lands, archive-first
composes in front of this verb without changing it.

**What survives — everything whose home was never the workspace:**

- **Balls and their history.** Task state lives in balls' store, not the
  workspace. Live claims are **released by the verb itself** (step 1 below),
  returning them to ready; delivered balls keep the dead name as claimant
  forever — the closed listing's obituary record (§3.4) — rendering in the
  on-demand closed listing once no local workspace matches (§3.5). Project
  repos, clones, and delivery worktrees are untouched: a workspace encodes no
  project path (§3.1).
- **The trail.** `ops.jsonl` rows and detached stderr sinks naming the
  workspace stay — the deletion is itself an ops event, and a trail that
  deletes with its subject is not a trail (§4.2).
- **The world.** Deletion is per-workspace; the nested world (§16.2) and
  every other workspace stand.

**The plan (§8.1's planner idiom — pure steps, order load-bearing,
convergent on re-run):**

1. `bl unclaim <id> --as <name>` for each live bound ball (the §3.5 claimant
   join enumerates them; each a logged short-piped verb, §8.2). Released, not
   stranded: claimed-elsewhere (§3.5) exists for foreign claimants and true
   ghosts, not for a wall the operator is deliberately taking down — leaving
   N claims on a dead name is just N manual releases later. A refused unclaim
   (balls' own semantics, e.g. a store race) surfaces from its ops row and
   aborts before the removal.
2. Prune the workspace's `ui.json` keys — its `seen[ws]` map, `pinned` entry,
   and `collapsed` override — yog's own file, one ordinary debounced write
   (§4.1). Not mere hygiene: the name-reuse case below must not inherit a
   dead sphere's acknowledgement watermarks or pin.
3. Remove the directory — logged as the non-spawn step
   `["yog-step","delete-workspace"]` (§4.2's sentinel convention).

Releases first, removal last: a crash anywhere leaves a live workspace with
some claims released — benign, re-runnable — never a removed workspace with
steps unfinished. Even the worst residue is legal: §8.1 already rules a claim
whose workspace was deleted a lawful claimed-elsewhere, so the plan degrades
into a state the join table renders, never an error class.

**The gate: no live drivers.** The verb refuses while any of the workspace's
agents probes Live or InFlight (§5.1 #9), with the §10 "?" uncertainty
counting as live — fail closed. An `rm` under a flock-holding driver is a
race with a running process, and folding a kill into a delete would let one
gesture destroy running work across two substrates; Stop keeps its own
semantics (deposits, epitaphs — lernie ARCH §2.9), and the refusal names the
live conversations so the operator stops them first. Verbs stay orthogonal.

**Confirmation doctrine — destructive vs recoverable (normative for every
future verb).** A verb is **destructive** iff it destroys facts no derivation
can recompute. Everything else is **recoverable** through the ordinary
primitives — Stop resumes by message, Release re-claims, Move is two of
those, Close is a *delivery* gated by the repo's own hook — and fires
unconfirmed, the ops trail its record. A destructive verb:

- **confirms explicitly, scaled to blast radius (amended bl-f17a)**: every
  member opens a dialog that enumerates concretely what dies, and the arming
  is **typed-name iff the verb destroys objects beyond the one named on
  screen** — a principle, not a per-verb answer. Workspace deletion satisfies
  it (the wall's contents are not the tab under the pointer): the operator
  types the workspace's name — obscurity is not the safety mechanism; the
  typed name is, and since bl-df65 settled §3.1 on operator-chosen names, the
  typing is the short word they chose themselves (`ops`). A **leaf** agent
  delete does not satisfy it (one conversation is the row the operator is
  pointing at, its name in the dialog's own title): a plain explicit confirm.
  A **subtree** agent delete does again (it destroys conversations that are
  not that row): the typed conversation name.
- **takes no keyboard binding, ever** — §11 rule 3 taken to its limit: not
  even a bare letter, because no reflex may reach it (the same reasoning that
  left `Ctrl+R`/`Ctrl+W` unbound, §11).
- The class has two members: workspace deletion, and the one-conversation
  delete below (bl-f17a).

**Scope: named workspaces only.** Foreign workspaces are lernie's auto-id,
retention-governed territory (§3.1) — yog may not delete what it did not
place; replays are read-only by definition. The verb renders only on
yog-named workspaces.

**Name reuse is re-adoption, and with chosen names it is the point.** The
claimant join is a string equality (§3.2), so raising a new workspace under a
dead name re-adopts the name's history: delivered balls group under the new
workspace, and any claim the deletion could not release re-binds. Since names
are typed rather than minted (§3.1), reuse is never accidental — it is the
operator typing `ops` again, and that *means* continuity. The old
combinatorial argument (a random pool makes accidental reuse negligible) died
with the workspace mint and needs no replacement: a name chance never hands
out cannot be collided with by chance (§3.2's cross-machine caveat, re-argued
there). The `ui.json`
prune (step 2) keeps re-adoption clean: history rejoins, stale
acknowledgements and pins do not.

**Rejected:** (a) *soft-delete / a trash state* — a second lifecycle state
with its own retention is mechanism where a statement of irrecoverability
plus the (deferred) archival verb suffices; (b) *requiring zero bound balls
before delete* — N manual releases the plan performs with the same logged
verbs; (c) *stop-and-delete as one gesture* — the gate above, for cause;
(d) *an upstream `lernie delete` verb* — **scoped to workspace disposal**
(amended bl-f17a): disposal belongs to whoever owns placement, yog places
workspaces, and that is the whole of the rejection. lernie places *agents*,
and since 0.0.4 ships exactly that verb for them — which is why the
one-conversation delete below spawns it rather than re-deriving it. The two
rulings are one principle applied at two altitudes.

Implementation: bl-0ccc.

#### Deleting one conversation (bl-f17a)

The same class, one altitude down: right-click a conversation row → `delete
this conversation…`, or the same worded, ichor row at the foot of the
inspector's Config tab — the visible carrier the §11 doctrine requires, the
menu its accelerator. Both open one confirmation dialog; no key opens or arms
it. Scope is the workspace verb's: **named workspaces only** — yog offers no
deletion inside a workspace it did not place.

**The removal is lernie's verb, spawned** — never a yog write inside the
workspace (I2): `lernie delete <ws> <agent> [--children]` (0.0.4), short,
piped and logged like every §8.2 lernie verb, cwd the workspace,
`Origin::Conversation`. What it removes is lernie's own subtree cut — the
agent's branch and worktree, its `steps/` and `inbox/` slices, its
`refs/lernie/*` marks, and (under `--children`) the `<id>-*`
hyphen-descendants — and an absent agent is a quiet success (delete's
postcondition already holds), so the verb is convergent on re-run.

**The gate is the workspace verb's, member-scoped:** refuse while the root or
any member of the conversation probes Live/InFlight, the §10 "?" counting as
live — fail closed, naming them, so the operator Stops first (rejected (c)
holds here too: no kill is folded in). lernie's own `Driven` decline is the
substrate's independent fail-closed under the race; yog gates first, at fire
time, off the published snapshot.

**The census is the substrate's, not a yog re-derivation.** The dialog
enumerates what dies from `lernie delete --children --dry-run` — the
descendants by name and the pending-deposit count, straight off lernie's
`DeleteReport` (its `--dry-run` is documented as "the census a caller's
confirmation enumerates"). One source of truth: the process that performs the
act computes what the act takes. The dry run mutates nothing and is fetched
once, at dialog open (an explicit gesture, I7) — unlogged, the `bl conf` read
seam's idiom, so opening a dialog does not append trail rows.

**Arming is the amended doctrine above, and it is also the argv:** the typed
conversation name is the *only* thing that fires `--children`; an unarmed
fire is the **bare** verb, which lernie declines for an agent with
descendants (`HasDescendants`, naming them). So the subtree/leaf decision is
re-made by the substrate at the moment it acts — a descendant born after the
dialog's census cannot die unconfirmed, and the race needs no yog-side lock.

**What survives:** everything the workspace verb's table already says, plus
the workspace itself — and the conversation's bound ball, if any: a ball is
claimed by a *workspace*, not an agent (§3.2), so a single-conversation
delete releases nothing. **What yog prunes** after a clean removal: the dead
subtree's `seen[ws][id]` watermarks in `ui.json` (§4.1) — the same
not-mere-hygiene as the workspace prune (a re-used id must not inherit a dead
conversation's acknowledgements) — and a focus that pointed into the subtree
clears rather than naming a gone branch. Until `lernie bundle` is surfaced
(§8.3) there is no archive step to offer; the dialog states
irrecoverability, and bundle-then-delete composes in front later without
changing the verb.

Implementation: `delete::agent` (gate, arming, census parse, the two spawns),
the `DeleteAgent` boundary action (§8.5 — gated in dispatch exactly as the
dialog gates, whichever frontend fires), the §11 seats in `nav::menu` /
`shell::delete_agent`.

---

## 4. Durable yog state

### 4.1 `$XDG_STATE_HOME/yog/ui.json`

`ui.json` holds **only genuinely-converging data** — user assertions with no
other authoritative home, where both instances *should* agree. Live focus,
selection, and scroll are deliberately **not** here: they are per-instance
viewport ephemera (§5.3, reasoning in §13.1).

```json
{
  "v": 1,
  "seen": {
    "/abs/ws/path": {
      "a-b": { "notify": "<ref-oid>", "stopped": "<tip-oid>",
               "budget": "<ref-oid>", "conflicted": "<ref-oid>" }
    }
  },
  "pinned":   ["/abs/ws/path", "..."],
  "collapsed": ["proj:/home/u/dev/brazen", "ws:/abs/ws/path"],
  "transcript_expand_responses": true,
  "transcript_expand_others": false,
  "zoom": 1.0,
  "panels": { "conversations": 260, "activity_trail": 200, "start_goal": 240 },
  "identity_last_used": "op@example.invalid",
  "prices": { "opus": { "input": 15, "output": 75,
                        "cache_read": 1.5, "cache_write": 18.75 } },
  "ceiling": 25,
  "notify_unfocused": true
}
```

- **`seen`** — the attention-acknowledgement watermarks (§6), one per signal
  kind, keyed to ref oids (notify/budget/conflicted = the `refs/lernie/*` ref
  target; stopped = the branch tip oid at acknowledgement, stamped for any
  ***at-rest*** agent, not only a stopped one — §6 rule 2 as amended bl-2194.
  The key keeps its historical name because the watermark's identity is the tip
  oid, unchanged: that is what makes the widening migration-free — every
  `ui.json` already on disk stays exactly as valid). lernie's marks are
  level-triggered and yog may not delete refs ("the UI is a pure reader"; no
  ack verb exists) — so "the user has seen this" is a yog fact: **the mark is
  lernie's, the acknowledgement is yog's.** A moved ref re-notifies.
- **`pinned`** — ordered float list of workspaces (a user assertion, no other
  home).
- **`collapsed`** — explicit user expansion overrides only; default expansion
  is derived from attention, so the file stays tiny. A persisted *view* (§13.0),
  not durable data.
- ~~**`show_internal`**~~ — **deleted (bl-e3e7).** It was the global
  nested-delivery ("internal") clone view filter (§5.1 #1). Nested-delivery
  clones are now hidden unconditionally, so no key backs the view and none is
  read: a stale `"show_internal"` in an existing `ui.json` round-trips as an
  unknown key and means nothing. See §5.1 #1 for the ruling.
- **`notify_unfocused`** — the §6 desktop escalation (bl-e160): may an
  **unfocused** window tell the desktop that something new needs the operator?
  Default `true` — the strip is invisible exactly when the operator needs it,
  so a notifier off until you find its switch is a feature nobody has. The
  severability test in one line: delete the key and the default returns, set it
  `false` and the behaviour is gone, and no code path knows either way.
- **`transcript_expand_responses` / `transcript_expand_others`** — the §11
  transcript-density automatics: which row classes arrive expanded (defaults
  `true` / `false`, the operator's ruling — the conversation open, the
  machinery around it folded). The key names are the durable ones and stay as
  written; bl-6ec6 widened what `responses` *covers* (delivered messages
  joined it) without touching what an existing `ui.json` means. Policy, not data — and the reason
  they are *here* is that `ui.json` is the durable UI-state artifact yog
  already has; inventing a second file for two booleans would give one fact
  two homes. Absent ⇒ the default; a non-bool value ⇒ the default (the
  forgiving read). Deleting them restores the ruling, deletes no code path.
- **`zoom`** — the whole-UI scale factor: the operator's **text size**
  (Ctrl+`+` / Ctrl+`-` / Ctrl+`0`, §11). Absent or non-numeric ⇒ `1.0`; the
  read clamps to egui's own 0.2–5.0 domain, so a hand-edited value can never
  open a window nobody can read, and the write snaps to a hundredth so the
  `f32` round-trips exactly. **This document is the authority and the egui
  context is its projection** — `src/shell/keys.rs` re-asserts
  `ctx.set_zoom_factor(model.zoom())` every frame, and egui's own built-in
  keyboard zoom is switched off in `theme::apply` (bl-42e7). The alternative —
  letting egui hold the live factor and mirroring it here — gives one fact two
  homes: it is lost at exit (the symptom filed: set the size, quit, relaunch,
  it is 1.0 again), and two running instances would each write their own back
  over the other's adoption, forever. Derived-from-one-home has no such
  fixpoint: an adopted change simply lands on the next frame.
- **`panels`** — the sizes the operator dragged the resizable panel boundaries
  to, in **logical points**: the conversation column's width, and the heights of
  the expanded activity trail and the start-goal composer (§11). One object
  keyed by panel, so a new draggable boundary is one member, not a new
  top-level key. Absent, non-numeric, or a `panels` that is not an object all
  read as *never dragged* ⇒ that panel's default; a value below the panel's
  floor is raised to it, so a hand edit can never open a panel with no boundary
  left to grab, and one above the panel's **ceiling** — half the window along
  the panel's own axis (§11 rule 5) — is lowered to it *on read*, so a width
  stored on a wide screen opens usable on a small one without the document
  losing what the operator actually dragged. Sizes are in points and therefore independent of `zoom`: a text
  size change rescales what a panel holds, never how wide the operator made it.
  **The same authority ruling as `zoom`** — this document is the authority and
  egui's panel state is its projection: the shell hands each panel its size
  every frame and writes back only what a *released* boundary settled on (one
  drag, one write; an unmoved boundary writes nothing, so a still window never
  touches the disk). A view kept durable for convenience (§13.0), like
  `collapsed` — never required to replicate the way state is.
- **`identity_last_used`** — prefills `--as` for verbs dispatched *outside*
  any workspace (manual `bl create`/`bl update` from a project row; default
  `$USER` when absent). Workspace-scoped verbs never consult it: they stamp
  the workspace's name (§3.2). The severability showcase: no yog config file exists;
  deleting `ui.json` restores defaults and deletes no code path.
- **`prices`** — the §3.5 spend-attribution price table: model id → the four
  brazen counters' rates, quoted in **USD per million tokens** the way a
  provider's price page prints them, so the operator transcribes rather than
  converts. Model ids are the ones `models.yaml` declares (the string the step's
  own `request.json` carries), so the key is the same name the picker shows.
  **Read-only — no setter, no editor, no verb**, because the rates are operator
  policy with no other authority and a hand edit is already live within a tick
  (the whole-file `adopt`, I5). It is *here* rather than in a config file of its
  own for the same reason the density knobs are: `ui.json` is the durable
  artifact yog already has, and a second file for one object would give one fact
  two homes. Absent ⇒ no cost figure renders anywhere; a malformed row or a
  non-numeric rate degrades to absent rather than refusing the document (the
  forgiving read), so a typo costs a column, never the window. Deleting the key
  deletes the column and no code path — the severability §3.5 demands.
- **`ceiling`** — the §3.5 spend ceiling: one number, **USD**, the bound a
  workspace's own spend must stay under for yog to start a *new* conversation
  in it. Read exactly like `prices` and for the same reasons — read-only, no
  setter, no editor, no verb, live within a tick through the whole-file
  `adopt`. Absent, non-numeric or negative all read as **no ceiling**: deleting
  the key deletes the gate, not a code path. An empty `prices` deletes it too
  (§3.5: a ceiling in dollars needs the table that makes dollars). A literal
  `0` is honored — the hard stop that starts nothing new. It bounds *births*
  only; nothing already running is ever stopped by it.

**Write discipline: write-through.** Every mutation lands on disk before the
mutating call returns — temp-in-dir + rename (I3) — and is **elided outright**
when the new bytes hash to what is already there (the same content hash that
suppresses echoes, below). There is no debounce, no pending-write state, and
therefore no flush: no dispatch site and no exit hook has anything to do.

*Why (bl-b54e):* any debounce window loses a gesture to a signal — `pkill -x
yog` reaches no eframe `on_exit`, a SIGKILL reaches nothing at all, and
"durable except when the process is signalled" is not durable. With nothing
ever in flight there is no shutdown path to get right. Coalescing is bought
by the hash rather than by a clock — a gesture that changes no byte writes
nothing, so re-acknowledging a seen agent, or holding an arrow key across
already-seen rows, costs no writes at all. A failed write leaves the
last-known hash alone, so the next mutation simply rewrites the whole
document; this is last-writer-wins whole-file state, never a delta that could
half-apply.

**Convergence:** both instances watch `$XDG_STATE_HOME/yog/`; on an external
change, read; if the content hash equals our own last write it is an echo —
ignore; otherwise adopt wholesale (LWW at file granularity). A missing/corrupt
`ui.json` is the fold identity — all defaults, never an error (brazen's
forgiving-read stance for its model cache). Unknown keys are preserved on
writeback (additive schema, balls' discipline).

**Startup focus derivation:** with focus out of `ui.json`, each instance
derives its initial focus deterministically — the next-attention workspace
(§6), else the first workspace in derived order (I9). Nothing is lost on
crash; focus re-derives.

### 4.2 `$XDG_STATE_HOME/yog/ops.jsonl`

One JSON line per **attempted** yog-initiated action. A spawn failure —
missing binary, exec error — appends a synthetic line with the intended argv
and the failure in `stderr`; a non-spawn step failure — mint pool exhaustion,
`mkdir`, worktree cross-check drift — appends a line whose `argv` is the
logical step name, e.g. `["yog-step","mint"]`, with a sentinel `exit`.
**The sentinels are the `src/opslog` consts (the authority): `-1` a piped
verb whose status was unobservable, `-2` a detached spawn that **handed off**
(`lernie prompt`; its own `stderr` is always empty, and a post-launch death
folds in from the sink), `-3` a synthetic failure line — a spawn that never
launched, piped or detached, or a non-spawn `["yog-step",…]` step.** An error
class with no ops row is an error the UI cannot render — the §7.3 failed-action
row depends on this:

```json
{"ts":"2026-07-17T12:00:00Z","argv":["bl","close","bl-4db6"],"cwd":"/home/u/dev/brazen","exit":0,"origin":"balls","stdout":"…","stderr":"…"}
```

- **`origin` is the §7.3 attribution** — `balls` / `conversation` / `world`,
  the [`opslog::Origin`] tokens (bl-48f8). A *fixed* field like `ts`/`cwd`/
  `exit`: never truncated, and the one thing a banner surface filters on. It is
  **stored, not derived, because it cannot be derived**: `bl close` and `lernie
  message` are told apart by their `argv`, but a ball-rung start and the
  composer's Enter write byte-identical `lernie prime` / `lernie new` /
  `["yog-step","mkdir"]` / `lernie prompt` lines, so any read-time
  classification is wrong for one of them every time. The writer knows — the
  verb knows its own subject, the start knows its own rung — so the fact is
  recorded once, where it exists. A line without the field (an older yog) reads
  as `conversation`, the surface that is always on screen in some form, so a
  legacy failure still banners exactly once rather than nowhere (INV-2).
- Written with O_APPEND; **each line is capped at 4096 bytes (PIPE_BUF)** so
  concurrent appends from two instances are atomic and never interleave;
  `stdout`/`stderr` are truncated to fit with an explicit
  `"truncated":true` marker. A pathological `argv` element is the one field the
  capper cannot shrink, so the caller pre-clips the sole large one — the detached
  `lernie prompt`'s composed goal — to a bounded head with an explicit
  `… [+N bytes elided]` marker before logging (the *spawned* goal is unclipped;
  full fidelity is never the log's job, the goal being derivable from the
  workspace).
- Both instances append; both tail it (fs-watched). The ops pane renders the
  shared history.
- **The operator's own two lines (bl-c417).** Dismissing an alarm and ending a
  trail are *actions with outcomes*, so they are ops lines like every other
  action — never a stored flag, never a second state home. Both are ordinary
  completed step lines (`["yog-step", …]`, exit `0`, origin `world`, so neither
  banners anywhere and neither reads as a failure) written through the same
  ≤4096-byte capper, and both carry only fixed fields, so the atomicity bound is
  structural for them:
  - `["yog-step","ack-failures"]` — the **ack**, a *global seen-watermark*. Every
    failure-derived alarm considers only the rows **after the newest ack line**:
    the §7.3 banner on every surface (`AppModel::last_failure`) and the §11
    chip's ⚠ and drift counts (`opslog::activity`) both read one derivation,
    `opslog::since_ack`, so a banner and a chip can never disagree about what has
    been seen. One line shape, no per-origin variants — dismissing from anywhere
    means "I have seen what is on screen now". **A NEW failure lands after the
    watermark and re-alarms**, which is why this is a watermark and not a mute.
    **Drift is quieted with the failures** (§7.2 files its catches as *alarms*,
    and an alarm the operator has said they have seen is exactly what an ack is
    for). **The trail is untouched:** the expansion still renders every row —
    the acked failures and the ack line itself — and the chip's `N ops` still
    counts the whole tail, because that number names the rows the pane lists.
    Slicing a *prefix* is sound against §6's retirement, which only ever looks at
    rows *later* than the one it judges.
  - `["yog-step","clear-trail"]` — the **clear**, and the one place anything is
    ever removed from this file. It truncates `ops.jsonl` and logs itself as the
    new trail's first row. **The durability promise (below) is that yog never
    loses an outcome *silently*** — it was never a promise that the operator may
    not discard their own history. A discard the operator asked for, which
    leaves its own record where the history was, loses nothing silently. Still
    no line is ever *rewritten*: the write goes through an `O_APPEND` handle
    truncated with `set_len`, so a concurrent instance's append between the
    truncate and the write survives at the front rather than being clobbered by
    a positioned write, and the reader is stateless (it re-reads the file) so a
    file that shrank needs no handling of its own. It is offered only inside the
    *expanded* §11 pane — reaching it costs opening the pane, which is the whole
    guard a destructive verb gets here.
- This closes the one durability leak of a pure derive-everything stance:
  gate/close output, `lernie scan` summaries, and error text are *not* on disk
  anywhere else — without this log they would be RAM-only and non-convergent
  (and the current shell's "stderr printed and dropped" hole would persist).
- **The alignment monitor's two row shapes (VISION §4.9, bl-8da1).** Both are
  ordinary lines in the schema above; neither adds a field. A **check** is
  `argv = ["yog-monitor", <verdict>, <agent id>, <sha>, <model>, <input
  tokens|->, <output tokens|->]`, `cwd` the workspace, `stdout` the model's one
  sentence, `exit` `0`, `origin` `world` — the same attribution a drift
  observation carries, so a verdict never banners on a surface that did not ask.
  A **flag** is `argv = ["yog-flag", <agent id>]`, `stdout` the reason, `exit`
  `0`, `origin` `conversation` — its subject is a conversation, and raising
  attention is not a failure. A check that *did not complete* writes the
  ordinary `["yog-step","monitor"]` synthetic-failure line (`-3`) and
  **deliberately names no sha**, which is the whole retry mechanism: the level
  trigger still sees that tip as unchecked and the next tick re-fires. These
  rows are the monitor's only durable state — the last-checked sha and the
  standing verdict are `latest`/`worst` queries over them (`src/monitor/row.rs`),
  never stored fields — and, joined with the delivered/reverted facts VISION's
  spend attribution already assembles, they are the tuning dataset the
  calibration concern asks for.
- **The armed loop's one row shape (VISION §4.3, bl-66fb).** An ordinary line in
  the schema above; it adds no field. A **spawn** and a **reap** are both
  `argv = ["yog-fleet", spawn|reap, <ball id>, <what it produced>]` — the
  conversation a spawn minted, or the claimant name a reap released the ball
  from — with `cwd` the armed workspace, `exit` `0`, `origin` `balls` (the board
  is the surface the loop acts on, so a loop row banners where a ▶ Start row
  would). A reap's `stdout` is **the comparison that decided it**, verbatim
  ("lease expired 14m ago"), and there is deliberately no way to put a diagnosis
  there: the loop spawns and reaps, it never diagnoses (§4.3). A spawn's
  `stdout` is empty. **A move that did not land writes nothing here**, because
  every executor it composes — the start flow's `bl` steps, the §3.5 ceiling
  gate, `bl unclaim` — already left its own failure row, and the loop is
  level-triggered, so a second row per tick saying "and the loop wanted that"
  would double every failure on the trail forever. These rows are the loop's
  only durable: the "last tick" the V4 board renders is `last_act` over them
  (`src/fleet/row.rs`), never a stored field.
- **The capability boundary's two row shapes (§8.6, bl-765d/bl-94b4).** Also
  ordinary lines, also no new field, and — unlike every other row here — they
  are *read back*: the capability control folds them on every consult, so these
  rows are at once the audit and the policy's memory (no fourth durable; I2
  holds at three). An **answer** to one parked invocation is `argv =
  ["yog-control","answer",<tool_use id>,"pass"|"hold"|"refuse"]`, `stdout` the
  control's own reason off the hold mark. A **floor** over one conversation is
  `argv = ["yog-control","floor",<conversation id>,"raise"|"lower"]`, `stdout`
  empty — the reason is the row before it, the monitor verdict or flag that
  prompted it, and the trail is read in order. Both carry `cwd` the workspace,
  `exit` `0` and `origin` `conversation`: the subject is a conversation, and a
  policy decision is not a failure. The fold is **latest row wins** per key, so
  the floor's two directions need no ordering rule of their own, and a floor
  matches its conversation's whole hyphenated descent — one row, a subtree.
- No rotation in v1 (documented; balls' own multi-MB per-clone `log` sets the
  precedent). The clear line above is not rotation and must not become it: it is
  an operator gesture, never a size or age policy, and yog runs none. Detached long-lived drivers (§8.1) do **not** stream here —
  their outcomes are derived from disk, not parsed from pipes.
- **One field is not stored here: a detached spawn's `stderr`.** The `-2` line
  is written at fire, when the child has said nothing yet. Its stderr is
  captured to the per-spawn sink `detached/<ts>-<workspace leaf>.err` (§8.1,
  §5.2) and folded into the row **at read time**, on the tail the ops sweep
  re-reads. The sink is the authority and the row a projection, so the text is
  never stored twice and no line is ever rewritten; the sink's name derives from
  the `ts` and workspace the line already carries, so the schema gains no field
  to join them. A row whose folded `stderr` is non-empty is a rendered failure by
  the `-2` rule above — that is how a driver that died *after* launching stops
  being invisible.
- **One sentinel, one fact — and the field is never rendered raw (bl-afa9).**
  `-2` used to be written for both a detached `lernie prompt` that handed off
  *and* one whose fork never landed (a nonexistent work directory), the error
  merely riding `stderr`: two opposite facts under one encoding, and the trail
  could not tell "running fine" from "never ran". The encoding was the lie, so
  it is fixed at the source — a spawn that never launched writes the ordinary
  `-3` synthetic-failure line, whatever its spawn shape — leaving `-2` to mean
  exactly *the handoff happened*. Rendering follows from that: the `exit` field
  is projected through **one** classification, `opslog::exit::ExitKind`
  (`OpRow::failed` / `OpRow::drift` / `OpRow::exit_label` all read it and
  nothing re-reads the integer), which says every sentinel in words — `detached
  — handed off, no exit to observe`, `failed to spawn — never started`, `step
  failed — nothing was spawned`, `ran; exit not observable`, `drift observation
  — not an attempted action` — and reads a real `128+n` status as the signal
  death it is. No surface prints a bare sentinel: a negative integer in an exit
  column reads as a signal death to anyone who has used a shell.
- **The badge/rollup half owes the same honesty (bl-8433).** bl-afa9 fixed the
  *expanded* row's wording; the *collapsed* row's badge is a separate
  projection — `theme::op_badge` over `opslog::OpOutcome` (§6, §11) — and had
  no detached case, so a handoff fell through to `OpOutcome::Clean` and wore
  "ran clean" for an exit nobody observed. Fixed by building on `ExitKind`
  rather than re-reading `-2`: `OpRow::detached` (`pub(crate)`, `src/opslog/
  exit.rs`) is true exactly when the row's kind is `ExitKind::Detached` *and*
  it is not `OpRow::failed` — a `-2` row with stderr folded in from its sink is
  already a rendered failure and never reaches this arm, so the two stay
  mutually exclusive. `opslog::live::outcomes` (§6's retirement projection)
  reads it to produce a fourth outcome, `OpOutcome::Detached`, and
  `theme::op_badge` gives it its own glyph/hue/phrase (`↳` brazen — the same
  carriers `flight_badge` already wears for `Flight::Subagents`, "a dispatched
  child is running", the nearest existing fact to "handed off, running
  elsewhere"; phrase matches `ExitKind::Detached::label()` verbatim so the
  badge and the expansion never disagree). **The ruling, both halves:** (1) a
  handoff is **neither** `Clean` nor `Failed` for the §11 activity chip's ⚠
  count — nobody observed the exit, so it must not count as a failure, and it
  must not silently inflate "clean" either, hence its own bucket; (2) a
  handoff **does** retire an earlier failure of the same verb exactly like a
  clean run does — the handoff is the newest fact yog has about that verb in
  that `cwd`, and a stale failure under it is no longer the live story, so
  `outcomes` inserts a detached row into the same retirement set a clean row
  would. No reason surfaced to treat retirement differently for a handoff than
  for a clean run — both are "the last thing yog knows about this verb went
  fine, or at least didn't fail," which is all retirement asks.

---

## 5. The complete state inventory (normative)

Every piece of state in the application, classified. **This table is
normative: code review rejects any state not placeable in it.**

### 5.1 Derived-from-disk (fact → home → derivation; never stored by yog)

Every path fold below resolves through the composed world env (§16.2), and
every **balls** fold through the §16.3 space standing in it — the world's when
no `YOG_MARKS` is layered on, which is every read yog itself makes:
`LERNIE_HOME` and `$XDG_STATE_HOME` name nested locations, and
`$XDG_DATA_HOME` stays ambient as the world's anchor. Brazen's three folds —
config (#19), credentials (#22), model cache (#23) — resolve **per workspace**
(§16.2 as amended, the blast-radius ruling): a provider is a
workspace setting, so each fold reads inside the wall of the workspace being
asked about, never the machine's shared brazen state. The three are one fold
(`BrazenPaths`, §16.2's layout) over one var, and a seat that names no
workspace answers `None` for all three — the general path with an empty input,
not a special case.

| # | Fact | Source of truth | Derivation |
|---|---|---|---|
| 1 | Project list | `$XDG_STATE_HOME/balls/clones/*` | readdir + percent-decode basename; decoded paths under `plugins/bl-delivery/` are nested-delivery clones and are **never** shown (`projects::visible`, unconditional). **The "internal clones" toggle is deleted (bl-e3e7)** — it was a checkbox at the top of the §11 balls section over a `ui.json` boolean the §7.2 worker re-read as a filter over which clones to walk. Such a store exists only because something ran `bl` with its cwd inside a `work/<id>` worktree, which balls' own guide says addresses "a *different* (usually empty) store"; the clone dir sits under `clones/`, outside the worktree, so it outlives the `bl close` that tears the worktree down. Revealing them put phantom projects on the roster — each with a new-ball form that would file a ball into a throwaway store, each turning into an orphaned-project row once its worktree was gone — and no yog verb ever acted on one. A rename could not rescue a view whose ON state is a trap, so the whole knob went: the checkbox, the `ui.json` key, `visible`'s bool parameter, and the worker's only `ui.json` read (`adopt_ui` now ferries the bytes and interprets nothing) |
| 2 | Balls per project | project store | **in-process** typed catalog load — `balls::reads::Catalog::load` over the clone's store checkout, projected to `Ball` from `task::Task` (§16.7 W8; no spawn, no `--json`, no serde_json). **Unlistable ≠ empty:** a clone with no founded landing (`config/`) leaves the project *unkeyed* in the cache → the §3.5 orphaned-project row; a founded clone with no task files keys it to an empty vec → a listable project with no balls (the two are distinct states, not both "no balls"). Foundedness replaces "the `bl` process exited non-zero" as the unlistable test — the read no longer runs a process to fail |
| 3 | Ball status | ball frontmatter | balls §3 ladder: claimant ⇒ claimed; else unresolved claim-blocker ⇒ blocked; else ready. Closed = absent from live set |
| 4 | Delivered/closed balls | store history | `bl list -s closed --json` on demand — the ONE read still spawned (as `yog bl …`, §16.7 W12), because balls' dead-ball history walk is not on the promoted read surface; the live-ball detail (`show`) is the in-process `Catalog::get`. Its result is **published in the snapshot beside the live map** (bl-3c28), so the §3.5 Delivered rows and the §8.5 search corpus read one cache rather than each fetching: sparse by design, because the fetch is on demand and never on the cadence |
| 5 | Work-worktree path | formula + git | bl-delivery formula recompute; `git worktree list` ground truth; `bl claim` stdout cross-check |
| 6 | Workspace list + names + foreign + replays | `$XDG_DATA_HOME/yog/workspaces/*`, `<lernie-data>/workspaces/*`, `replays/*` | readdir for `repo.git`; leaf = name per §3.1 |
| 7 | Join state per (ball, workspace) | #2–#6 | the §3.5 claimant join, a pure function |
| 8 | Agent set, descent, tips | `repo.git` refs | `git for-each-ref agents/*`, then the §2.3 grammar over the ids (existing `git_tree`): an id's parent is that id less its last **descent segment** — `<ts>-<short>`, exactly two hyphen-free tokens — and it is a child only if that derived parent is **present in the ref set**. Absent (an id outside the grammar, or a deleted intermediate) ⇒ a root row, never a re-attach to some shorter prefix; the registry intersection is lernie's own ruling (ARCH §8) and the two must not fork. **The id-derived tree is the *provenance* fact** — who dispatched whom; since lernie bl-a693 a `--from` child's *context* (git ancestry) can diverge from it, and the two are distinct edges, never conflated (VISION V1.3's two-edge taxonomy): §11 membership stays this descent-id tree, while V1's spine states the distinction in the child card's fork-label wording (`from here` / `from <Name>@<oid>` vs `from config/<name>`) — the solid/dashed strokes went with the gutter (bl-1802), and drawing the descent as a graph is bl-5cf8. **Since bl-fa82 the §11 conversation list is a rendering of this tree, and since bl-8905 the only one**: every visible row is the subtree rooted at its agent, `children_of` is the row's `direct` count and the subtree size less one is its `total`, and the all-collapsed case is the root-only list this seat had before. The strict rule is what the list obeys — the Stop menu's looser `+children` prefix test is a different question and stays where it is |
| 9 | Agent state {Live, InFlight, Quiescent, Stopped} | inbox flock + response.json framing | LockProbe + WriterProbe (tri-state, §10) + `last_segment_complete` (ARCH §3.5/§4.4) |
| 10 | Streaming text, tool calls | `steps/<id>/NNN/` | existing `streaming.rs` / `tools.rs`. The response fold is **one read yielding one value** (bl-b768, bl-54f7): a `Stream` carrying the answer text, the reasoning text and the *kind of the last content delta* (#28b), and `Agent::stream` holds that value whole rather than splaying it into fields that could be filled from three reads. Two reads would cost a second syscall per agent per tick and could catch two different mid-write states of one file, so the fold returns all of it or it is not one file's answer. **On the focused conversation the same fold runs again, off-derivation, at frame cadence** — §7.2's live tail, the one thing on a rendered `Agent` that no derivation put there |
| 11 | Pending messages, inbox contents | `inbox/<id>/*.md` | count + parse `---from/deposited_at/epitaph---` frontmatter |
| 12 | Transcript, and **how many messages have landed** | `agents/<id>/messages/NNN-<origin>.*` | readdir + sort; origin from filename; "tool in progress" = tool_use with no tool_result. The **count** (`Agent::messages`) rides on the same readdir the §11 recency fact (`Agent::last_action_unix`, bl-cad5) already performs — one directory walk yielding both, the #10 discipline — because it is the one honest observation of "the message I just sent has landed" that the §7.2 pending echo reconciles against |
| 13 | Step diagnostics | `steps/<id>/NNN/{meta,request,response,staging}.json`, `stderr.log`, `tools/` | full-file reads, jsonview rendering — every byte inspectable. The §7.3 **no-response wound** derives from the same bytes: an empty-or-absent `response.json` with no `meta.json`, on an agent nobody is driving (#9) — and since bl-55d8 its **reason** is a third read of that same directory, `stderr.log`, gated on the predicate so a healthy step pays no syscall for it. Three readings, one derived value (`Wound::{None, Mute, Spoke}`), nothing stored |
| 14 | Marks ×4 | `refs/lernie/{conflicted,budget-exhausted,abandoned,notify}/*` | `for-each-ref` (marks.rs extended from 2 to 4 namespaces) |
| 15 | Attention (per agent / rollups / totals) | #9, #11, #14 + `ui.json.seen` | §6 predicate — pure |
| 16 | Budget *spent* | Usage events across `steps/<root>*/` | fold; limits displayed only as raw `workflow.yaml` text (no YAML dep) |
| 16a | Spend **attributed and priced** (per conversation, per ball) | #16's fold + each step's `request.json` model + `ui.json.prices` (§4.1) | the §3.5 join — Σ(usage over the agents tied to the ball) × the table, priced per step by that step's own model. Attribution is #8's goal stamps when any name the ball, else the workspace (the §3.5 ruling, labelled as such). Derived per ask, never stored; an empty table yields no cost at all |
| 17 | Governing config per agent | git ancestry | nearest ancestor of agent tip reachable from any `config/*` ref (merge-base over config refs, ARCH §2.2) |
| 18 | Config branches + contents | `repo.git` `config/*` refs | `for-each-ref` + `git show <ref>:<path>` |
| 19 | brazen file config | `<wall>/brazen/config.toml`, the focused workspace's own (§16.2's wall layout) | raw text. No workspace in focus is **no path at all** (`BrazenPaths::of` answers `None`) — the surface renders its guard rather than falling back to the machine's `$BRAZEN_CONFIG`/`$XDG_CONFIG_HOME`, which is the whole of the blast-radius ruling in one row |
| 20 | brazen effective config | `bz` itself | `bz --dump-config` stdout verbatim (bz is the authority on the value fold; yog never re-implements TOML semantics). *Since §16.7 W10 this is a **function call**, not a spawn — the linked `brazen` at yog's exact pin, driven through `src/bz_host.rs`; same bytes, same authority, no foreign binary* |
| 21 | brazen built-in rows | compiled into bz | static read-only hint beside the dump (which drops the defaults operand and so can never show them). *Since W10 the rows are additionally **listed**: `bz --list-providers` keeps that operand, so the login surface offers built-ins and file rows alike — one in-process read (§5.1 #20's sibling), and the source of the credential-presence rows too*. **When** it is read: at the login surface's RAM construction (bl-e290), so the pane opens with its roster already in it; `↻ providers + credentials` is the re-ask for the two things a start-up read can go stale against — a config edit made since, and a credential written since (bl-402f: it re-reads #22 in the same gesture). Never on first click — a surface that shows nothing until you refresh it reads as a surface with nothing in it |
| 22 | Credential presence | `<wall>/brazen/credentials/<provider>.json` existence — the focused workspace's own | existence only; contents never read, never written. Paired with #20's `auth` column it is the whole of what a surface may say about a provider, so the *words* are derived once (`brazen::row_views`) and rendered by both seats — the §9.5 config rows and the §8.3 Login rows (bl-402f). Read at the same gesture that asks #21, never per frame |
| 23 | Model cache | `<wall>/brazen/models/*.json` — the focused workspace's own | read-only display; refresh = `bz --list-models`, whose write lands in the same wall because the caller carries it — the picker's spawn in its env, the in-process runner in its `Env` (bl-dff8) |
| 24 | lernie global config | `<config-root>/models.yaml`, `workflows/*.yaml` | raw text |
| 25 | Action history | `ops.jsonl` | tail + parse (§4.2); ambient error prominence is the §6 retirement projection over that tail — never a stored flag |
| 26 | A provider's offerable model roster | the provider, through `bz` | `bz --list-models --provider <row> --json`, fired **on every picker open** and never stored (§9.4) — and, since bl-dff8, on every `Query::Models` (§8.5), which is the same read in-process through the linked brazen. Distinct from #23: that row is brazen's on-disk cache rendered read-only; this one is the live answer the picker offers from |
| 27 | Role → (provider, model) per workspace | `providers.yaml` in a config commit | `git show <commit>:providers.yaml` (#18) + the §9.4 anchored-block read. Read at the **config-branch tip** for what the picker is about to change, and at the agent's **governing** commit (#17) for what the open conversation is frozen on — the same file at two commits, which is exactly why the model line states both (§11's bottom settings rows) |
| 28 | Conversation live-activity class {inference, tools, subagents} | #9 + #10, over the §2.3 subtree | `nav::convs::flight` — **inference** = any member `InFlight` (#9: the open `response.json` fd); **tools** = a *running* member holding a tool call whose `output.json` has not landed (#10); **subagents** = any non-root member holding a driver. The three overlap by construction and the operator's priority resolves them to ONE: **inference > tools > subagents** (§11). The classes are a query per tick and could not be a stored flag — yog never observes a start or an end, only disk at this tick. **Three seats read this one derivation** (the list row's pulsing name, the altitude-1 chip, the bottom in-flight strip, bl-905f); only the strip adds characteristics, and every one of them is a field of the same snapshot (`Agent::stream.text`'s length, `ToolCall::name`, a running-child count, and the two **structural starts** below) — never a second derivation. **Since bl-b768 the class is a *fold over #28b*, not a second reading of #9/#10:** `inference` = any member whose `Doing` is a model call, `tools` = any member whose `Doing` is `Tools` (the live-driver guard moved there with it). The priority is unchanged and is applied to those answers |
| 28b | What **one agent** is doing {waiting, thinking, inference, tools, idle} | #9 + #10 + the last content delta of the live `response.json` | `nav::convs::doing` — the finest live-activity fact, and the whole vocabulary of the §11 live mark (one circle per agent). Under an open response fd (#9) the **last content delta** splits the call three ways: none yet = *waiting on the API*, a `thinking_delta` = *thinking*, a `text_delta` = *inference*. Else a running member holding an unfinished tool call (#10) = *tools*; else *idle*. **Idle is not "stopped"** — a quiescent agent, a killed one and a circle with no agent in it all read the same, because whether a branch ended well is a different question with its own carriers (the §3.5 badge, the §6 marks); putting it on the same circle would be two facts on one carrier. The delta kind rides the snapshot as `Agent::last_delta`, off #10's one fold, and is consulted **only** while the agent is `InFlight` — so a settled step's trailing delta is never read and no expiry rule is needed. #28 folds over this; nothing folds the other way |
| 28a | When the call in flight **began** — the strip's elapsed (bl-9dfb, amending bl-905f) | the world record that opens the call: `steps/<id>/<NNN>/request.json` for a model call, `…/tools/<tool-id>/input.json` for a tool call | `Agent::call_start_unix` / `ToolCall::start_unix`, both stamped at enumerate time beside the presence checks they ride with (bl-cad5's rule). Each file is written **once, immediately before the call it opens, and never rewritten**, so its mtime is the call's *start*, not its last sign of life — verified against the pinned lernie: both drivers (`run_exchange`'s loop and the `lernie advance` hop) land `request.json`, then take the very timestamp they will later write as `meta.json`'s `started_at`, then invoke the adapter; the tool executor writes `input.json` atomically, then takes `output.json`'s `started_at`, then spawns. `meta.json`/`output.json` cannot serve: each lands only *after* its call returns, so it is absent for exactly the call being timed. Nothing under `steps/` is git-tracked (§2.3), so neither file has a commit timestamp to prefer. **Elapsed = now − start, per frame, nothing stored** — the wall clock is minted at the shell boundary (`shell::now_unix`, the same one the §11 list's ages use) and the label is `age_label`'s, so the two seats cannot drift |
| 29 | Operable-commit rules — the persistent faint lines through the chat, one per step, each the gesture that pins it (bl-929d, re-seated bl-1802) | #12's transcript entries + `steps/<id>/NNN/meta.json` `commit` and `response.json` framing (#13) | `rail::place`, folded onto #30's notches. **Position pairs a model-output entry to a completed step, one for one** — a call that reaches `Finish` seals `messages/NNN-<model-id>.json` and one that does not seals nothing (lernie ARCH §2.3), so the transcript's `Model` entries are the `Framing::Complete` steps in step order. Each step's rule sits at the first entry it read that its predecessor had not (its predecessor's tool results, then the boundary drain's deliveries — ARCH §2.11 orders the drain after the tool entries), and the index of its own output is #31's cut. **This replaces the ordinal alignment bl-929d shipped**, which paired the i-th delivered run with the i-th step: a lernie step is one model call and a tool loop is many steps behind one drain, so every rule after the first tool-using turn carried the wrong commit. Absence of a commit = no line; a call that sealed nothing takes a seat only as the last step (its read state is the tail of the chat). Derived per snapshot, never stored |
| 30 | The **step spine** — one notch per step, its two edge kinds, its seat in the chat, and each dispatched child's card (VISION V1, bl-98da; bl-1802) | #13's `meta.json` `commit` (the notch), #8's `Agent::steps` (both edges), #8/#9/#10 (the card's identity, state and streaming tail), #16 folded per agent (its spend) | `rail::build`, a **pure** fold with no git call of its own, over #13, #8 and #12's entries. Each notch carries its chat seat (#29's `rail::place`) — the row its rule paints above and the cut its pin reads to, so the spine and the transcript's rules are one derivation rather than two. The notch spine is #29's read — the Steps view's `meta.commit` in step order, reused not re-derived, so the spine and the transcript's boundary rules can never disagree about what a step read. The **two edges** (VISION V1.3) come off one fact, and since bl-1802 the context edge is spent on the fork label's wording rather than kept as an index — a chat rule has no gutter to stroke, and the label states in words what the strokes drew (`from here` / `from <Name>@<oid>` vs `from config/<name>`); drawing the descent as a graph is bl-5cf8: `Agent::steps` is `git log --first-parent <branch> --not --branches=config/*`, so a *fork* child's list opens with its parent's commits up to the fork point and a *clean* child's shares nothing with it — the longest common prefix **is** the fork point (the *context* edge, git ancestry) and its emptiness **is** cleanness. The *provenance* edge is located for both kinds alike: the last notch whose commit is no later than the child's own first commit, which is the rule the card hangs under. §11's descent tree stays the descent-**id** tree (#8); this is a label on a card, never a second membership tree. Derived per snapshot, stored nowhere. The old `Rail::navigable` gate is gone with the gutter it kept from claiming width: the chat's rules are the ones bl-929d already drew, so an operator who never clicks one sees today's transcript exactly — the burden check, with nothing to gate |
| 31 | The inspector **as of** a notch — transcript, agent-context files, config-frozen-at, budget (VISION V1.2) | #30's pinned commit + #12 / `ls-tree` at that commit / #17 / #16 | one commit, four reads, no new mechanism per tab (the shape STORIES §S7 point 3 named when it declined a per-tab checkbox). **Transcript** is a *prefix* of #12, cut at the notch's own `Place::cut` (#29) — everything ahead of that call's own model output. Exact, not convenient: `messages/` entries are append-only under a monotonic counter, so the pinned tree's entries and today's leading entries are the same bytes, and the Raw toggle keeps showing verbatim bytes with no `git show` per message. **Files** is the one new disk read: `git ls-tree -r -l <commit>` for the listing (blobs with their sizes as of then — a `--name-only` listing could not state a size, and a zero would be a lie) and `git show <commit>:<path>` for one file, both memoized per snapshot with the commit in the key, never per frame. **Config-frozen-at** is #17 asked at the pinned commit instead of the tip — no new code at all. **Budget as-of** is the notch spine's own per-step tokens summed through the pin. Nothing here is stored; the selection itself is §5.3 viewport ephemera. **The release** is the pin banner, which paints above every pinnable tab: the gesture that raises the pin is a rule in the chat, so the way back has to be reachable from the tabs that have no chat (§11) |
| 32 | The **project work-diff** — what a workspace's delivery attempts have actually changed (VISION §4.10, bl-3746) | #2's live balls (the §3.2 claimant binding, the ball graph) + the project repo's own refs | a pure git read of the project repo, `target..source`, spelled exactly as the §4.10 ruling spells it. The **source** is the claim's branch, `work/<id>` — balls' own `delivery_path::work_branch`, never a literal here. The **target** is balls' own `target::derive` re-run over facts the snapshot already carries: the parent ball's work branch when this ball close-gates a *live* parent (`parent = X` AND `X` carries `{this, on: close}`), else the project's integration branch — `git symbolic-ref --short HEAD`, again balls' own spelling, so yog and `bl close` can never name two targets. The listing is `git diff --numstat` (counts, not bytes) bounded at the Files tab's own cap; one file's patch is `git diff <range> -- <path>` read only when picked, classified by `files_view::classify` — the one "what this file is" vocabulary. **Three declines, never one silent empty listing:** a project repo that names no branch is *unreadable*, a ref that does not resolve is *absent* naming which end, and a resolved pair with nothing between them is an empty diff. A workspace holding two balls has two attempts and both are shown — there is no rule that picks one. Nothing stored, no index, no verb spent; memoized per snapshot by the seat that asks (§7.2), never per frame |

| 33 | A **cohort** — the candidates dispatched from one notch, and the ancestry they share (VISION V2.3, bl-dc0c) | #30's cards, grouped | `rail::cohort::cohorts`, a **pure** grouping of #30 by `provenance_notch` — the birth notch V1.7 reserved as the fan anchor. **Membership is not a fact anybody records.** Firing the fork twice from one mark *is* firing a cohort and firing it once *is* a cohort of one, so there is no fan registry, no fan verb and no winner field to keep: the group is a `group_by` over cards yog already derived. The **common ancestry** is the fork label when every member wears the same one and nothing when they differ — the same fact said once at whichever level owns it, and absence is a value (the columns then say their own). The four side-by-side facts a candidate is judged by need no new derivation either: state is #8's, usage is #16 folded per agent, and the **terminal response** is #10 — `Agent::stream.text` is the latest step's accumulated text re-read every tick, so while a candidate runs it is the live tail and once it settles the same bytes are the last thing it said. A second "terminal response" reader would be two readers of one open file, disagreeing at every moment it matters |
| 34 | The **fire-time policy** a pinned notch may fork into — the fork points, and the model each role names at each of them (VISION V2.2, bl-dc0c) | #18's config branches + the pinned commit; #17 asked at each; `providers.yaml` from that commit's tree | `fork::choices`. The points are **here** (the pinned commit — a fork carrying the conversation's own history) then every `config/<name>` head (a clean start): one control with two kinds of value, which is VISION V1.3's *"one spawn gesture with one parameter — the fork point"*. Each point's roles are read from the `providers.yaml` of the config commit that will **govern** it — the very file lernie resolves the run against — through §9.4's own `grammar::roles`, so the picker and the fork composer can never disagree about what a config says. **That is what makes the model visible without yog owning a model list**: a role *is* a model binding, and giving an attempt a model no config declares is a config write (§9.4's `PickModel`), not a dispatch flag. A ref whose config yog cannot reach declares no roles and the point paints as offering nothing — a fact about the workspace, never a silence. Memoized per snapshot with the pinned commit in the key (it is `for-each-ref` + a `merge-base` walk + a `git show` per point), never per frame |
| 35 | **How full a conversation's context is** — the percentage §11's settings rows state per chat (bl-a48b: the context-window percentage is shown per chat) | the **last `Usage` line** of the root agent's **latest** `steps/<root>/<NNN>/response.json`, the model that step's `request.json` names (#16's walk), and the `context_window` §9.2's global `models.yaml` declares for it (#24) | `context::of_conversation`, a pure filter over `Snapshot::bills` — no second disk pass, no stored counter, nothing cached. **Fullness is not spend, and the difference is load-bearing:** #16 sums every counter of every attempt of every step of a whole descent (what exhausts `max_total_tokens`, and what keeps growing after a compaction empties the context), while this is one number off ONE step — so the walk carries two extra columns, the step's own `seq` (making "which is the latest" an in-memory question, exactly as bl-9dd4 made scope one) and its **last** attempt segment's counters (a step retried three times must not read as a context three times its size). **The root's own latest step, never the descent's** — a dispatched child runs its own context in its own tree. **The prompt is `max(input, cache_read + cache_write)`**: brazen's canonical `Usage` is deliberately unnormalized about overlap because its providers disagree — Anthropic reports the three as **disjoint** slices of one prompt while OpenAI's `prompt_tokens` and Google's `promptTokenCount` already **contain** the cached slice beside them — so summing over-states one shape and taking `input_tokens` alone under-states the other by nearly everything (brazen marks Anthropic prompts for caching unconditionally). The maximum is exact where the slice is contained, degrades to plain `input_tokens` where no cache counters are reported at all, and is a **floor** where they are disjoint. It never over-states; normalizing that overlap is brazen's job, not yog's to guess at. **The denominator is the declaration, not a discovery:** brazen serves `Model.context_window` only for the providers that publish one (Google), which is its own empty-set rule — *"a harness hand-configures only what no provider serves"* — and `models.yaml` **is** that hand-configuration, written by the §9.4 picker and edited by the §9.5 form. Reading brazen's cache as well *here* would be two representations of one fact — which is why bl-848f moved the discovery to the WRITE path instead: the picker seeds the declaration from the `Model.context_window` its own roster carried, so the one authority this figure reads starts out true wherever a provider serves the number at all, and this reader still consults nothing but the declaration. **No window, no figure** (no step, no model on the step, an undeclared or zero window): the row is absent, never a percentage of a default — the same no-capability-theater rule §3.5's unpriced remainder keeps. The windows ride the snapshot (`Snapshot::windows`), read at boot and on the 15 s full sweep like the ball fetch: one hand-edited world-global file, so a fifth watch root would buy latency nobody can perceive |
### 5.2 Durable-on-disk, yog-owned

Exactly `ui.json` (§4.1), `ops.jsonl` (§4.2), `cadence.yaml` (§7.2, bl-3381 —
present only once the operator tunes the clock or arms the monitor; deleting it
is the reset), the alignment monitor's policy file (`monitor.md` by default,
whatever the armed entry names — present only while armed, I2/§7.2), and
the detached-spawn stderr
sinks `$XDG_STATE_HOME/yog/detached/<ts>-<workspace leaf>.err` (§8.1, §13.3) —
each written *by the detached child itself*, not by yog, and each the sole
authority for what that driver said, projected into its `-2` ops row at read
time. Like `ops.jsonl` they are not rotated in v1.

Plus two *transient scratch* artifacts that exist only inside an operation and
are swept: config staging temps (`.<name>.yog-tmp-<pid>` in the destination dir)
and the scripted-editor staging directory `$XDG_STATE_HOME/yog/stage/<nonce>/`
(§9.3). Neither is an authority; leftovers >24 h old are swept at startup.

Five more yog-owned files exist and are deliberately **not state**: the world's
tool shims `<yog-data-root>/world/tools/{bl,lernie,bz,bl-delivery,bl-tracker}`
(§16.7 W9, extended to all three agent tools at W11 and to balls' two sibling
plugin binaries at bl-2930). Each is a *generated artifact* — a pure function
of the `Cli` yog resolves that tool through — re-derived and rewritten on any
drift at every start and every hatch (bl-44a5), holding no fact yog cannot
recompute.
Deleting it loses nothing; that is the test this row asserts, and why it is
listed here rather than beside `ui.json`.

### 5.3 Legitimately RAM (the closed whitelist, I6)

| Item | Why RAM is legitimate |
|---|---|
| Unsubmitted input text (prompt/message bars, claim-dialog fields, the goal composer, config editor buffers before Apply, the two §3.6 delete confirmations' open target + typed name — the agent dialog also holds the dry-run census it fetched at open (bl-f17a), a derivation cache the fire never trusts: the argv is re-armed from the typed name and the subtree re-cut by the verb itself) | the requirement's explicit carve-out: "text typed in a box can live in RAM until sent". RAM, but **per target**, not per box (§11, bl-a69a): the docked composer keys its drafts by what it is pointed at, so switching the selection switches drafts rather than re-addressing one. **The target of a config editor buffer is the workspace wall that holds the file** (§16.2 as amended, bl-5894): brazen's `config.toml` is a workspace's, so its draft is keyed by wall and survives A → B → A, while the lernie-global and cadence files are the world's and keep exactly one draft each. Re-loading one box on focus change is not per-target keying — it is the box relabelled, and it discarded the draft |
| **Live focus/selection and scroll position** — including **which tab each of the two strips shows**: the altitude-2 inspector tab, and the §11 center tab (`keymap::CenterTab`, bl-1ca2) | per-instance viewport ephemera — *which data you look at*, not data; loses nothing on crash; re-derives at startup (§4.1). Deliberate interpretation, §13.1. Scroll is *represented* as content anchors (topmost visible message ordinal / step number), never pixels, so the viewport stays stable across live re-derivation of the tree beneath it |
| Subprocess handles, drain threads, `Stream`s, the detached reaper threads | a process is not data; the fact each represents lives on disk (driver running = flock held; op outcome = `ops.jsonl` line + substrate state). Long-lived drivers are spawned fully detached (§8.1) so yog's death cannot kill or starve them — the handle a detached spawn keeps lives in its reaper thread, never in app state, and carries no fact (its whole job is to take the status and drop it) |
| Watcher registry, notify channels, dirty flags | reconstructible plumbing; the sweep (I4) makes their loss harmless |
| Memoized derived snapshots (`HashMap<PathBuf, GitTree>`, ball lists, parsed transcripts) | caches of §5.1 facts; discarded and rebuilt at will |
| Live window geometry, egui layout/font caches, GPU state | instance-physical, not data (§13.0: window arrangement is a view, not data). **The panel proportions inside that window are not this row** — where the operator dragged a boundary is an assertion with no other home, kept durable in `ui.json.panels` (§4.1) on the same terms as `zoom`; the window's own size and position stay the desktop's business, and yog neither reads nor writes them |
| Probe result TTL cache on macOS (§10) | a cache of an observation with a 2 s bound |
| The model picker's open/closed flag, its selected role, the half-made pick beneath it, and the in-flight `bz --list-models` run with the roster it produced (§9.4) — **all of it per workspace wall** (bl-5894) | the run is a live subprocess (the row above); the roster is #26 — *a query's answer, held only as long as the surface that asked*. Storing it would make yog a second authority on a set the provider owns, and the picker re-asks on every open by design. The wall scoping is the blast-radius ruling read onto RAM: a roster listed against workspace A's providers, and a role/provider/model chosen from it, may not paint or be clicked under B — a pick there would write B's config lineage from A's candidate set |
| Live streamed-verb output (§8's streamed-piped class: the `bz --login` sign-in lines, off stderr — §8.3 as amended), **held with the wall it was fired in** (bl-5894) | instance-local by nature (a device code is for the human at *this* keyboard); it converges to its `ops.jsonl` outcome line at exit, so the other instance renders the durable fact from the pane, never diverges. **The last failure is *not* here** — it was a RAM item until bl-4895 proved a cached copy cannot be right: the banner is derived from the ops tail every frame (§7.3), so it has no RAM home to lose. **Its attribution is on the row too, never in RAM** (bl-48f8): which surface a failure belongs to is a fact of the durable line's `origin` (§4.2), so it survives a restart, reads the same in both instances, and cannot be lost with the frame that dispatched it. **Parked, never dropped, when focus leaves its workspace** (bl-5894): the run is writing *that* sphere's credential, so it may not paint under another — and dropping it would SIGTERM a sign-in the operator is halfway through, which is why the wall owns the holder rather than a focus change clearing it |
| What this window has already told the desktop (§6 as amended, bl-e160): the last observed alert set | a desktop belongs to a **window**, so two instances each announce their own and neither should converge — the §13.1 viewport-ephemera argument carried one step out of the frame. Losing it loses nothing: a restart is a new window, and a new window announces nothing it merely inherited (its first fold is its baseline), which is the same rule that keeps a fresh launch from flooding. Nothing here is a fact — the facts are the `refs/lernie/*` marks and the `ui.json` watermarks that decide the queue it is a difference against |
| The §3.4 start claim and the **pending echo** it carries (§7.2, bl-915e): the workspace, the target (a minted §3.3 name or an agent id), the operator's text, and the landed-message baseline it reconciles against | the unsent-input row above, one instant later: a message yog has *sent* and the driver has not yet flushed is still only this instance's word for it, and the moment disk says so the derivation says it instead. Losing it loses nothing — a restart re-derives from disk, which is the same convergence a landed message already has. It is deliberately not durable and deliberately not in `Snapshot`: writing it down would make yog a second authority on a fact lernie owns, and two authorities on one message is exactly the drift §7.2 measures |
| The last line's answer (§8.5): a reply's own JSON, or the refusal the reader gave | what a *typed* control said back, and only until the next line — an action's durable record is the `ops.jsonl` row it wrote anyway (§4.2), a query's answer is #26 (a query's answer, held only as long as the surface that asked), and a refusal never crossed the boundary at all. It is replaced by the next command and cleared the moment the draft is edited, because an answer about something the operator is no longer saying is worse than none |
| The §11 conversation list's **expanded set** (bl-fa82): the agent ids whose subagent rows are unfolded | which rows you have opened — *which data you look at*, the jsonview collapse set's argument on a list instead of a JSON tree. It is deliberately not `ui.json`'s `collapsed` (below): that array names a fixed handful of sections, while this is keyed one-per-conversation and would accrete a stale key for every conversation that ever existed, and mirroring it would drag a second instance's list open under the operator's hands. An empty set is the whole list collapsed, which is the seat's own pre-bl-fa82 rendering — so losing it loses nothing |
| Per-row transcript **fold overrides** (`tx/<file>#<block>`, §11), and the inbox-composer's pending-item folds (keyed by the deposit's inbox path, bl-929d) | which rows you have flipped away from their configured auto-state — viewport ephemera by the same argument as the jsonview collapse set. The *policy* (which classes auto-expand) is durable in `ui.json`; the hand-flips over it are not, so a restart returns to the configured reading and nothing is lost. A pending item's fold dies with the pending row: the delivered transcript entry it becomes is a different fact under a different key |

| The composer's **prompt recall** (bl-f908): how far back ↑ has paged, the draft it displaced, and the caret row the gate reads | one step further out from the draft above, which is already the requirement's carve-out — the displaced text is unsent input, and the depth is *which* recalled turn you are looking at, the §13.1 viewport-ephemera argument on a box instead of a list. Nothing else is held: the prompts themselves are §5.1 #11 + #12 read back, so the walk survives a restart without yog storing a single one of them, and the caret row is last frame's galley — a measurement, not a fact |

**The left panel's `collapsed` overrides (§4.1) are the deliberate
counter-example:** a persisted *view* — which sections (the balls section)
you keep folded — that converges benignly for convenience, intentionally
*unlike* this RAM-only jsonview collapse set and the RAM-only activity
accessory (§13.0). All are "collapse" state; only the section override is
worth persisting. Don't "fix" the asymmetry.

---

## 6. The attention model

`attention(agent)` is a derived predicate, true when any of:

1. `refs/lernie/notify/<id>` exists and its oid ≠ `seen[ws][agent].notify`.
2. State **at rest** (Quiescent or Stopped), no `refs/lernie/abandoned/<id>`,
   and tip oid ≠ `seen[ws][agent].stopped` — a conversation waiting at a tip you
   have not seen. Rest is the general condition; a stop is only the wounded way
   of coming to rest. The clean end and the failed end differ in the state
   badge, never in whether your turn has come *(widened bl-2194)*.
3. `refs/lernie/budget-exhausted/<branch>` oid ≠ `seen[ws][agent].budget`.
4. `refs/lernie/conflicted/<id>` oid ≠ `seen[ws][agent].conflicted`.
5. Pending inbox > 0 **and** lock Free — mail nobody is driving (the
   writer/driver stall case). **Not seen-gated**: it is actionable (flush via
   `lernie scan`), self-clears when a driver picks it up, and hiding it would
   hide a stall.
6. `refs/lernie/held/<id>` exists — the capability control parked a tool
   invocation before it executed (§8.6, bl-765d). **Not seen-gated**, for rule
   5's reason carried one step further: a park costs the drone no process and
   no tokens and *nothing but an answer releases it*, so a watermark could only
   ever hide a conversation that cannot move. It self-clears when lernie lifts
   the mark, which happens exactly when the answer's re-adjudication runs.

Signals 1–4 are seen-gated on the `ui.json` watermarks (§4.1); focusing an
agent records the current evidence oids as seen. Because `seen` converges,
**acknowledging in one instance acknowledges in both — attention is data, and
it converges.** Live focus does not.

**The acknowledgement is a state, not a gesture (bl-aa1f).** A focused agent's
evidence oids are stamped on **every frame it stays focused**, not once at the
focus transition. A one-shot stamp covers only evidence that predates it, so a
signal landing on the conversation you are actively reading raised the flag at
the very thing you were looking at, and kept it raised until you clicked away and
back. Holding the stamp is what buys the contract the whole model rests on:
**attention is evidence that arrived while you weren't looking.** It is free —
§4.1's write-through elides a write whose bytes are unchanged ("re-acknowledging
a seen agent … costs no writes at all"), so the held ack writes only on the frame
evidence actually lands. Two consequences are accepted rather than fixed: the
jump control acknowledges its own destination (arrival puts the conversation on
screen, which *is* seen — and it is what makes the `⏭` walk empty the queue
rather than circle it), and keyboard transit acknowledges the rows it passes
through (each did render focused; the only knob-free alternative is a dwell
timer, a clock threshold with no §5.3 home).

**Every signal's fact has a carrier the ack cannot reach (bl-efa2).**
"Acknowledging clears the signal, not the fact" only holds where the fact is
*rendered* somewhere the watermark does not touch. Rule 2's carrier is the state
badge; rule 5's is the `✉n` accessory and the Inbox tab; rules 1, 3, 4 and 6 are
the agent's `refs/lernie/*` **marks**. The marks are one closed set
(`git_tree::AgentMark`, total over the namespaces `marks.rs` reads —
`abandoned` included, since the assertion that *suppresses* rule 2 is exactly why
a quiet stopped branch is quiet, and `held` since bl-765d), each with its label
and its sentence in one
mapping (`theme::mark_badge`, §11's badge-seat pattern), worn at two seats:
**outright under the focused conversation's header** (§11 altitude 1 — the
surface a jump-to-next-attention always lands on, so arriving acknowledged never
leaves *why am I here* unanswered) and inline-with-hover on each descent-tree
member row.

Rollups: workspace attention = max over its agents; the top strip shows totals
across all workspaces with a **jump-to-next-attention** control (also the
startup focus derivation, §4.1). Sort within a workspace's **agent roster**:
**attention > running > idle**, then
derived order (I9). "Idle" now means *at rest at a tip you have already seen*:
under rule 2 as amended, an unacknowledged rest is attention, so the idle rank
is what is left after the ack. **The conversation list is no longer one of
these groups** — since bl-cad5 it sorts by recency alone (§11); attention there
is a **badge only** — not a count, and not a rank.

**The rank orders the jump, not the walk (bl-fa82).** This roster used to be the
keyboard order too — ↑/↓ stepped its flattening across every workspace. Since
the unfold ruling (§11) ↑/↓ walk the focused workspace's **visible list rows in
paint order** instead, so the attention rank now serves exactly two seats, and
they are the two it was always for: the **jump** control above — which still
crosses workspaces, because naming the next thing that needs you is the whole
job — and the §8.5 decision queue, which shares its one roster build so a
headless reader and the jump can never disagree. A jump whose target sits inside
a collapsed subtree **reveals it** (§11's visible-selection invariant): arriving
somewhere you cannot see would leave *why am I here* unanswered, which is the
same reason the landing surface states the marks outright.

**The row wears a flag, not a tally (bl-b9e3).** The tally comes out of both
the spec and the implementation; what stays is the flag, on the right side
rather than the left, because a conditional mark on the left makes the list
fail to align. Two things fall to it, and the second is this
section's own argument. *The seat* is §11's — a conditional mark before the
title moves the name column, so the flag rides the trailing right-pinned group.
*The number* dies here: a per-row tally is a **second, lossier encoding** of
what that same row already renders through the carriers just above — the state
badge, the `✉n` accessory, the `refs/lernie/*` marks — and a row that says the
same fact twice says it once as a word and once as a number that means less.
The queue-depth defence below (*"the number is a queue depth, and a depth that
stays high is itself the information"*) is an argument for the **strip's one
global total**, which owns the question *how many conversations are waiting on
you*; the row owns *is this one waiting*, and that answer is a boolean. So the
count keeps every seat where it is the whole point — the strip's total, the
workspace tab badges (§11 altitude 0), and `ConvRow::attention` itself, which
stays a `usize` because the headless `conversations` answer has no width bind
and no column to break. Only the *paint* drops to a `> 0` read.

**What the strip answers (ruled bl-2194).** Attention is *evidence that arrived
while you weren't looking* — both kinds: wounds (notify / stop / budget /
conflict / mail) and **turns** (a conversation come to rest at an unseen tip).
The strip owns one question — how many conversations are waiting on you — and
answers with one number; *which way* each is waiting is the row's job (the
per-kind badges, the state badge — the bl-e266 amendment in §11). `⚑ nothing
stirs` is therefore inbox-zero: everything is running, abandoned, or already
seen — not merely "nothing has failed." The rule set stays at **five**: the queue
is not a sixth signal, it is rule 2 with its special case removed, so no new ref,
no `ui.json` schema change and no new verb exists. It requires the ack to be a
state rather than a gesture (bl-aa1f above): with rule 2 firing on every rest, a
one-shot stamp at focus would raise the flag on the conversation you are actively
driving at each of its turn-ends.

**The strip escalates to the desktop when the window is buried (bl-e160).** §6
is yog's core promise — *does anything need me?* — and it answers only where the
window is visible, which is precisely not when it matters. So when a new thing
needs the operator **and the window does not have focus**, the desktop is told:
one notification per conversation, naming its workspace wall, its §3.3 display
name, and the firing rules in words (`AttentionKind::says`, the one home for
that sentence).

*Nothing new is modelled.* An alert is a projection of the §8.5 decision queue
(`boundary::answer::queue`) — already "what needs you" made addressable — so the
count the strip paints and the ask the desktop names cannot diverge, and the
acknowledgement that empties one empties the other. **No sixth signal, no ref,
no `ui.json` `seen` key, no verb.** An alert is *render output*: it writes
nothing, so I7 stands and `ops.jsonl` (§4.2, the log of what yog did to the
world) gains no row — including on failure.

*What counts as new is the row's own sentence.* A conversation is announced once
per **set of firing rules**, not once per frame and not once per rule: a second
rule firing is a changed sentence and says itself, while the same sentence again
is the same unanswered ask. That is the flag's own behaviour — it stays raised,
it does not re-raise — and it makes the dedupe ride the acknowledgement that
already exists, since a row leaves the queue exactly when the operator focuses
the conversation or spends `/seen`. A ref that moves puts the row back and so
re-announces for free (§4.1's "a moved ref re-notifies", one layer out).

*The baseline is per window, and it advances unconditionally.* What has been
announced is §5.3 RAM (a desktop belongs to a window; two instances each own
theirs and neither converges), and the fold runs on **every** frame — focused or
not, knob on or off — with only the announcing gated. That is what stops a burst
of stale news the moment a window loses focus or the knob is switched on. It
also dissolves the first-boot flood one level up: a window that has just opened
has witnessed no *arrival*, so its first fold is the baseline and says nothing —
the general path with no prior observation, not a first-run branch.

*The mechanism is a spawned binary, and had to be.* AGENTS.md rule 6 forbids a
new dependency, and every Rust crate that raises a freedesktop notification
(`notify-rust` and its `zbus`/`dbus` stack) is one. The spec's own reference
client is a binary every desktop already ships — libnotify's `notify-send` — and
yog is a program whose substrate is spawned binaries, so this is the discipline
it already has pointed one process further out. It is spawned **bare**
(`git_env::command`, not `Cli`): the notifier is the operator's desktop session
reached through the session-bus address yog inherited, not substrate, so the
§16.2 world fold must not ride it. The frame never waits on it — the spawn and
its wait go to a thread of their own (bl-ee0a) — and every failure, an absent
notifier included, is **silent**: a desktop that cannot take a notification is
not an event, and the feature degrades to the world before it existed.

*Severable in one key* (§4.1 `notify_unfocused`, default `true`). Deleting the
key restores the default; setting it `false` deletes the behaviour without
touching a line of code. Armed by default because a notifier that is off until
you find its switch is a feature nobody has — and this one is the operator's own
request.

**What the ruling accepts, and must not "fix".** (1) **First-boot flood** — on
upgrade every parked conversation with an unstamped tip stirs at once; the `⏭`
walk is the designed emptier (STORIES S6-T4) and one wrap of the roster stamps
them all. No bulk-ack verb: new verbs are a smell. (2) **Flicker** — an
autonomous loop that releases the flock between steps rests briefly and stirs the
strip for that moment. Attention is a level-triggered per-frame derivation and
rule 5 already self-clears the same way; transient truth is truth. (3) **Muting**
needs no mechanism — an acked tip that never moves is silent forever, and an
`abandoned`-marked conversation never fires rule 2 at all. (4) **A sustained high
count** is not noise to threshold or decay: the number is a *queue depth*, and a
depth that stays high is itself the information — you are over-subscribed.

**Ops error prominence retires the same way — derived, never stored.** The
activity surface (§4.2's tail, §11's chip) is the ops-side analogue of this
model. `ops.jsonl` is append-only and keeps every failure forever, so the
*ambient* ⚠ count is a **projection over the tail at read time, never a stored
flag**: walking newest-first, a failed line is a **live failure** unless a later
line with the same (`cwd`, verb) did not fail. The verb is the leading two argv
tokens — binary plus subcommand (`bl close`, `lernie prime`, `yog-step mint`) —
because the argv tail carries per-run operands (a ball id, a composed goal) that
never repeat, so keying on the whole argv would retire nothing; `cwd` scopes it,
so a clean `bl close` in one project leaves a failed one in another alone.
Success is the pane's own failure classifier negated (`OpRow::failed`), so no
second definition of success can drift from the one it paints. A retired failure
keeps its row and its ⚠ in the expanded accessory — it loses only ichor and the
chip's count. **Absence of a live failure is the record; the log is the
history.** The wound this closes: a three-day-old `lernie prime` failure, since
fixed and re-run green, read as THE error when an unrelated action failed,
sending diagnosis down a false trail.

A conversation whose latest step **failed** stirs the strip through rule 2: a
failed or killed latest `response.json` classifies the agent Stopped
(§4.4/§3.5) — an auth-failed step included — so an unseen dead conversation is
never "nothing stirs". Acknowledging it clears the *signal*, not the fact: the
conversation list's state badge and the §11 Login affordance keep rendering
the settled failure (the badge is state, not attention).

**The prompt that never became a conversation (bl-a649).** Rules 1–5 are all
per-*agent*, so they can only stir once a conversation root exists. A detached
`lernie prompt` that dies before writing one — a tool version-skew refusal at
startup — has no agent to attach a signal to, and used to stir nothing at all.
It is not a sixth rule: that failure is an **action** outcome, not an agent
state, and it surfaces on the action path it already belongs to — the child's
stderr sink folded into its `-2` ops row (§4.2, §8.1), which makes the row
`failed()` and therefore lights the §11 activity chip's ⚠ count and the §7.3
ichor-red banner at the firing surface — *the* firing surface, the one the row's
`origin` names (§7.3, §4.2). The two paths stay disjoint on purpose:
attention is about agents that exist; the ops surface is about actions that were
attempted. A prompt whose driver dies mid-life crosses over — it *has* an agent
by then, and rule 2 fires. Rule 2 only says *this one is waiting on you* though
(and since bl-2194 not even that much distinguishes a wound from a clean
turn-end): the **cause** is not attention's job. What that conversation shows once focused
is the §7.3 no-response wound (§11), and what the driver actually said the
wound itself now states — the tail of the step's own `stderr.log` (bl-55d8).
The ops surface answers the same question for the *spawn* yog fired; the two are
different drivers and only the step's copy exists for a `lernie message` turn.

**The alignment monitor's hook, and why it is not yet a sixth rule (VISION §4.9,
bl-8da1).** An armed conversation carries a **standing verdict** — the worst of
its members' latest monitor checks, with the reason and the sha, derived from
the `ops.jsonl` tail on every build (`ConvRow::verdict`), never a stored flag.
It renders as a badge on the row it belongs to and in the headless
`conversations` answer, so the monitor is never silent. It deliberately does
**not** enter the five-rule predicate yet. The reason is this section's own
doctrine: attention is a queue depth an operator empties, and a signal they
learn to distrust is worse than no signal. A `diverged` verdict is a *model's
opinion*, and the design ruling's own calibration concern (recorded at bl-af1a's
close-out) is that a false one, fired visibly, teaches exactly that distrust —
so the wiring ships flag-only and the ops rows accumulate as the dataset the
verdict quality is measured against **before** a verdict is allowed to raise a
flag. The seam is already the right shape when that measurement says go: a sixth
rule reading the standing verdict, seen-gated on the checked sha exactly as
rules 1–4 are seen-gated on their oids, so a moved tip re-notifies for free.

---

## 7. Watch / re-render architecture

### 7.1 Watch roots

A `WatchSet` (new module `src/watch/`) owns one `fs_watcher::Watcher` per
root, with a per-root-kind allowlist (generalizing the existing hardcoded
single allowlist):

**One watcher per root, one backend instance per process (bl-908c).** A
`Watcher` is a *subscription*, not a backend: `src/fs_watcher/hub.rs` owns the
process's single `notify::RecommendedWatcher` and fans each raw event out to
every registered root that contains it. The reason is a kernel budget, not
tidiness — `fs.inotify.max_user_instances` (128) is **per user**, shared with
every other process the operator runs, while `max_user_watches` (65536) is what
one instance can hold. One instance per root spent the scarce budget at the rate
of one per workspace, and running out is silent: `WatchSet::reconcile` skips a
root it cannot arm and retries (below), so exhaustion surfaced as watches that
were simply never armed — reproducibly, whenever a few test binaries or yog
instances ran at once. Two consequences are deliberate: a backend *error* has no
root attribution once the instances are one, so it desyncs every live root (a
superset of the truth, which is what `Desynced` already means); and because
`notify` unwatches a whole subtree, retiring an enumeration root re-arms every
live root nested inside it rather than leaving it deaf (§7.3).

| Root kind | Path | Allowlist |
|---|---|---|
| Workspace (×N) | each workspace dir | existing: `steps/`, `inbox/`, `agents/<id>/{goal.md,soul.md,summary,messages,descriptions,skills}`, `repo.git/HEAD`, `repo.git/refs`, **`repo.git/packed-refs`** |
| NamesRoot | `$XDG_DATA_HOME/yog/workspaces/` (top-level — flat by construction) | dir create/remove (new/removed named workspaces) |
| WorkspacesRoot | `<lernie-data>/workspaces/`, `replays/` (top) | dir create/remove |
| BallsClones | `$XDG_STATE_HOME/balls/clones/` | clone dir create/remove; per-clone `tasks/tasks/*.md` and `config/config/**`; the per-clone `log` (multi-MB, no rotation) is **filtered out** to avoid event storms |
| YogState | `$XDG_STATE_HOME/yog/` | `ui.json`, `ops.jsonl`, `cadence.yaml` (the `detached/` sinks are **not** watched: a chattering driver would storm the watch, and the 15 s sweep's re-read of the ops tail folds them in anyway, §8.1) |

`packed-refs` is not decoration. yog reads refs through `git for-each-ref`,
which reads the loose tree **and** the packed file. `git gc` runs `git
pack-refs`, which empties `repo.git/refs/` into `repo.git/packed-refs`; deleting
a ref that is only packed then rewrites `packed-refs` **alone**, touching nothing
under `repo.git/refs/`. Without the entry that deletion is invisible to the
watcher and reaches yog only via the 15 s sweep — a reproducible dropped event
(bl-49f4; proven in `src/fs_watcher/drift_tests.rs`).

Rejected: one recursive watcher over the whole lernie data root — an agent
building a large tree in its worktree inflates the inotify watch count and
fires on every git object write; per-workspace scoped watchers with allowlists
are the existing, tested shape.

**Config files are deliberately NOT watched (bl-9130).** Three reasons, each
sufficient:

1. **Nothing derives from config.** §7 exists to keep `AppModel`'s snapshot a
   pure function of disk. A config file is not in that snapshot: it is an
   *operator-authored draft* in the §9 editors, whose lifecycle is load → edit
   → Apply, not derive → render. Marking a config root dirty would reach no
   re-derivation, because there is none to reach.
2. **§9 already answers the concurrency question, and answers it better.** The
   optimistic hash guard refuses an Apply whose on-disk snapshot moved since
   load, and it is the answer *because* blind LWW over operator-authored text is
   rejected there. A watcher could not adopt an edited draft anyway — that is
   precisely the discard §9 forbids — so the watch would buy notice, never
   correctness.
3. **`LernieConfig` cannot be armed as specified without becoming the watcher
   this section already rejects.** §16.2 collapses `LERNIE_HOME` onto
   `world/lernie`, and `lernie_config_root()` and `lernie_data_root()` both
   return it — so the "config root" *is* the lernie data root, `workspaces/` and
   `replays/` and every agent worktree included. `Watcher` arms
   `RecursiveMode::Recursive`, so that root is exactly the "one recursive
   watcher over the whole lernie data root" rejected two paragraphs up.

What a watch would have been reaching for is real and is kept: an external
edit must not leave a config pane showing startup content forever. That is
solved where it belongs — **the §9 editors re-read on the operator's own
attention gesture** (opening the Config pane re-reads every editor whose draft is
pristine; an edited draft is left alone, and the hash guard is its answer). A
read on demand, not a watch root, and no fact stored twice.

### 7.2 Repaint and re-derivation

**The frame thread renders and captures input. It derives nothing** (bl-ee0a).
The frame's whole interface to everything that does is the cells in
`src/state.rs`:

| Thread | Owns | Does |
|---|---|---|
| **frame** (eframe) | `UiState` (`ui.json`, write-through), `Focus` (§13.1), an `Arc<Snapshot>` | renders the latest completed snapshot; captures input; marks a root dirty when a dispatched verb changed something |
| **worker** (`app::Worker`) | `ProbeStack`, the sweep `Schedule`, the `BlRunner`, every derived cache | one pass at a time: drain dirty → route → sweeps → re-derive → **publish** a `Snapshot` → request repaint |
| **bridge** (`watch::Bridge`) | — | polls the `WatchSet` and marks announced roots dirty |
| **searcher** (`search::Searcher`, §8.5) | — | takes the frame's outstanding search, runs it over the latest published snapshot + the world's bytes, publishes the answer; abandons when the ask is superseded |
| **follower** (`app::Follower`, §7.2 live tail) | one open `response.json`, a byte offset, and the fold so far | reads the bytes appended to the **focused** conversation's step file, absorbs them into its accumulator, publishes into the `TailCell` and wakes the face — display-only, under the §7.2 in-memory carve-out |

- A dirty root is a `(path, Mark)` pair in the shared `DirtySet`. The bridge
  fills it from the watchers; the frame fills it when it *caused* a change the
  watch would only find later (a `bl` verb against a project, an unmaking).
  **There is deliberately no request channel
  beside it** — one vocabulary in, so a new frame→worker need is a root name,
  never a new verb.
- The §8.5 `SearchCell` is the fourth resident and the second frame⇄worker
  direction, and it does **not** break the "no request channel" rule above: a
  search is not a derivation the dirty vocabulary can name (it carries a
  parameter — the text), and running it on the derivation pass would let a long
  search delay a re-derivation. It is one cell in both directions because a
  search is one question at a time; the serial in it is the supersession *and*
  the cancellation.
- A completed derivation is an immutable `Snapshot` behind an `Arc`, swapped
  into a cell the frame clones out **once per frame**. Both locks are held for
  exactly one pointer move, so "the frame never blocks on the worker" is true by
  construction: a pass that takes ten seconds delays the next *snapshot*, never
  a frame.
- **The frame renders the derivation plus exactly two non-derived facts: the
  operator's own last send, and the focused conversation's live tail.** One
  function folds both onto the snapshot a frame paints (`echo::compose`), so
  "what does a frame see that disk does not say?" has one answer to read and a
  third such fact would be a third argument there rather than a third
  mechanism. The first is below; the second is **The live tail** further down.
- **The pending echo: the operator's own last send** (`src/app/echo.rs`,
  bl-915e). A snapshot is what a
  completed derivation read off disk, and that was the *only* source — so
  between Enter and the detached driver's first write, the text the operator
  had just typed existed nowhere in yog's model — a sent message was simply
  missing until the reply arrived, and the window read as though it were
  waiting on the send. The UI must be immediate: a message written goes into
  the inbox and pushes the inbox line up. Nothing was ever blocked; there was nothing
  to render. The answer is **not** a synchronous write and **not** a spinner —
  the frame still does no IO and still renders a completed derivation. It is an
  **optimistic echo**, reconciled by the next snapshot.
  - **Where the pending fact lives.** In `AppModel`, **as the §3.4 start claim
    itself** — one value, not a second pending-state concept beside it. It
    cannot live in `Snapshot`: a snapshot is derived-from-disk, and this is a
    fact yog caused and disk has not yet shown. Per-instance RAM (§5.3, §13.1),
    like the focus it becomes; nothing about it is written down.
  - **One fold, in one place, never per seat.** `AppModel` holds two snapshots:
    `derived`, the worker's, and `snap`, the **rendered** one — `derived` with
    the echo folded in. That fold is the single place snapshot and pending meet;
    every render seat reads `snap` and no seat knows an echo exists. The
    partition is the rule: **paint reads the fold, gestures read the
    derivation.** `boundary_deps` — every §8.5 dispatch and every machine-facing
    query — and the reconciliation itself take `derived`, so nothing yog *does*,
    and nothing a headless reader is told, is ever decided by a fact that is
    only optimistic. The fold recomputes only when one of its two inputs moved,
    so the rendered `Arc` stays stable and the `SnapMemo` below does not rebuild
    per frame.
  - **What the fold writes is the fact an unflushed message already is: a
    pending deposit** (§5.1 #11). The echo becomes a trailing `InboxEntry` on
    the target agent's `pending` listing and lifts its `last_action_unix`, so
    the three seats that already render pending mail carry it with no new code —
    the `✉n` badge, the Inbox tab, and the §11 inbox-composer queue directly
    above the box, which is the seat the operator named. A **start** has no
    agent to hang it on, so the fold mints one: a **pending conversation**,
    keyed by the minted §3.3 name — the only identity a start has before its
    branch, and the same handle the focus claim is already held by — carrying
    the operator's goal as its preview. That is one row in the §11 list, at the
    top by recency, in the operator's own words. This is deliberately not a new
    "pending" widget class: a message yog has sent and the driver has not
    flushed **is** an undelivered deposit, and yog already had that concept.
  - **The reconciliation key is the landed message count, never the text.**
    Every echo records how many `messages/` entries its target carried when it
    was made (§5.1 #12) — zero for a start, whose root does not exist yet — and
    is superseded on the first derivation whose count exceeds that baseline: the
    file the echo stood in for has landed. Matching on message text is fragile,
    and every cheaper proxy is a conflated predicate of the kind §7.2's own
    drift instrumentation exists to avoid: `last_action_unix` moves on a single
    streaming token and `tip_oid` on any step commit, so either would retire the
    echo while the operator's message was still missing — the defect, restored.
  - **Landing and the §3.4 claim are one event, not two.** One predicate retires
    the echo and spends the focus claim, in `adopt_started` — so the pending
    value has one lifetime and there is no state in which yog holds half of it.
    Only a *conversation* target moves focus (§3.4: a start focuses what it
    started); an *agent* target does not, because the operator was already
    there, and their own message landing must not yank them back from wherever
    they have since navigated.
  - **A pending echo is *visibly* pending, and that is what makes expiry cheap.**
    Showing a send in-memory in faded colour, brightening when it is actually
    locked in as a statement, is exactly what the case wants. So an
    echo paints at reduced solidity (`theme::tone_solidity` over the existing
    `Tone::Weak` — no seat mints a second colour vocabulary and no render site
    restates an RGB, §11) and brightens to full strength the instant the
    derivation carries it. Which state a row is in is a **query, never a flag**:
    a pending conversation has no branch and therefore no tip oid
    (`Agent::in_memory`), and an echoed deposit has no file and therefore no
    name (`InboxEntry::in_memory`) — a derived agent always has the first and a
    listed deposit always has the second, so neither predicate can be satisfied
    by anything real. Brightening is that same row at full strength, not a
    repaint into different hues.
  - **Expiry, therefore, is two rules and no timer.** A fire that **never
    launched** leaves no echo at all: the claim is taken only on a successful
    spawn, so a failed fork is the §4.2 synthetic-failure line and nothing else.
    A **launched-but-silent** driver leaves a faded row standing, and that is
    correct rather than a phantom: the row never claims the message landed, so
    it cannot go stale, and a hard timeout would only replace an honest faded
    row with a vanishing one — the original defect wearing a clock. It is RAM,
    so it dies with the window and can leave nothing behind, and the next send
    replaces it: one send is in flight at a time, exactly as the §3.4 claim
    always was. What says the launch *failed* is the §4.2 trail, which is where
    a failure has always been said.
  - **The second and later messages ride the same mechanism, and must.** The
    §8.2 `message` verb is piped and its deposit becomes `NNN-user.md` only on
    the driver's next step boundary, so the identical hole was open there. One
    `Echo` with two target arms covers both; there is no start-only path, and no
    second thing to build when the same complaint is made about a reply.
- **The live tail: the focused conversation's open `response.json`, followed at
  frame cadence** (`src/app/live.rs`, bl-54f7). Text did not stream back in
  while the model was thinking or writing: yog reads directly from the
  stream-in file in the lernie workspace and shows every character as it
  lands. The mechanism was already there — the fold (§5.1 #10), the virtual transcript
  entry, the `Doing` split (#28b). Its **cadence** was the defect: the fold ran
  only as a byproduct of a whole-workspace derivation, so it inherited the
  watcher poll, the `DirtySet` announcement, the 100 ms debounce and the
  re-derive. Characters arrived in clumps at the watcher's rhythm.
  - **The carve-out, and its exact width.** This is a deliberate, explicit
    violation of the no-state-in-memory rule: it is purely display, a dead
    end. So the accumulated text
    lives in RAM and is **not** re-derivable. What that waives is the
    derivation-is-the-only-truth rule. What it does **not** touch is I1: the
    frame does no IO and never blocks. The licence is to *hold* the text, not to
    *fetch* it on the UI thread — so the shape is a **follower thread**
    (`Follower`, on the engine beside the worker) publishing into a `TailCell`
    the frame paints from, exactly the hand-off the snapshot and the §8.5 search
    already use.
  - **Dead end means one reader, enforced by an absence.** The follower
    publishes into the cell; the only thing that reads it is `echo::compose`,
    which builds `AppModel::snap`. `AppModel::derived` — what `boundary_deps`
    hands every §8.5 dispatch and every machine-facing reply, and what the
    reconciliation and the memos take — never carries it. There is **no accessor
    from the model to the tail**, so a second consumer cannot be added without
    first deleting that absence; the regression is
    `app::live::tests::the_derivation_the_gestures_read_is_untouched_by_the_tail`.
    The moment a second reader appears the carve-out is void, because the fact
    would then need a home.
  - **Superseded, never merged.** When the step commits, `NNN-<model>.json`
    lands and the derivation carries it; the tail is dropped whole at the next
    subject change or step advance. The two texts are never reconciled
    character-by-character — the committed entry is the truth and the tail was a
    preview of it.
  - **Scope is the focus, and focus moving is the whole retirement rule.** One
    conversation is open; that is the one that streams. Tailing every agent in
    every workspace at frame rate is the version of this that burns the machine.
    When focus moves the follower drops its accumulator and opens the new
    subject's file, and the conversation just left reverts to the derivation's
    own fold of the same bytes on the sweep's cadence — which loses nothing,
    because nobody is reading a conversation they navigated away from at
    character rate. Three things reset the accumulator and they are one rule
    (*this is a different stream now*): the focus moved, the latest step
    advanced, or the file shrank. Nothing here expires on a clock.
  - **Following, not re-reading.** The follower holds the response file's path,
    a byte offset and the trailing partial line; each pass reads only what was
    appended, folds the complete lines through the **one shared parser**
    (`git_tree::fold_stream`) and absorbs the result (`Stream::absorb`, whose
    contract is `fold(a).absorb(fold(b)) == fold(a ++ b)` on any line boundary).
    Re-reading and re-folding the whole file every frame is the naive shape and
    it degrades as the answer grows; the in-memory carve-out is exactly what
    makes the incremental one legal. Partial-write tolerance is structural
    twice: the bytes after the last newline are held back, and the parser skips
    a line it cannot read.
  - **The fold writes the field the derivation already fills**, `Agent::stream`.
    No seat learns a new vocabulary: the §11 live mark, the flight strip's
    `N chars streamed`, the roster preview and the transcript's live rows all
    quicken together off one assignment. A tail for an agent the snapshot does
    not carry writes nothing — minting a row is the pending echo's job (§3.4),
    not this one's.
  - **The memos key on the derivation, not on the fold.** `SnapMemo` caches a
    read of *disk*, and neither non-derived fact is on disk — so keying a memo
    on the rendered snapshot would rebuild every disk read whenever a character
    landed, which is bl-e90a's cost restored with a new trigger. The transcript
    therefore splits: `transcript::build` reads the committed `messages/` and is
    memoized per published derivation, and `Transcript::with_live` appends the
    live tail per frame from the rendered snapshot's own `Stream`. Merging the
    two into one build is what made the tail as slow as the derivation.
  - **Reasoning streams too, as its own row** (the thinking ruling). The
    complaint says *"thinking/writing"*, and during a long reasoning phase the
    old answer was a `Doing::Thinking` badge — a mark that never grows, which
    cannot tell a model thinking hard from a driver that has hung. "Nothing is
    happening on screen" is precisely the complaint, so a badge is not an answer
    to it. The fold therefore accumulates `thinking_delta` text beside
    `text_delta` text (`Stream::thinking`), and the live entry projects to **up
    to two rows: `thinking:` then `live:`** — the same two a *committed* model
    turn already has (`Block::Thinking`, `Block::Text`). Each keeps its
    committed counterpart's `RowClass`, so the §11 fold knobs mean one thing on
    either side of the commit; what differs is the tone, and `Tone::Live` is
    what auto-expands them while the step is happening. The badge stays — it is
    still the one-glyph answer at roster altitude — but it is no longer the only
    thing moving. An empty half is no row: a model that has only thought so far
    shows one growing row, not one growing row and one blank one. The two are
    accumulated separately rather than interleaved, so a model that alternates
    reasoning and answering within one step shows all its reasoning above all
    its answer; the committed entry, one step later, restores the true order,
    and buying that ordering live would cost a per-fragment block list for a
    preview that is about to be replaced.
- **A frame-side build keyed on frame-owned selection is memoized per
  snapshot** (`SnapMemo`, `src/app/memo.rs`, bl-e90a). The Altitude-2
  view-models (transcript, steps) are functions of disk *and* of §5.3 RAM the
  worker cannot see — which agent is focused, which tab is open — so the worker
  cannot derive them ahead. The honest middle: the frame builds one **at most
  once per published snapshot per key**, sound because every disk fact those
  builds read is change-tracked by the snapshot itself (`last_action_unix`
  folds the newest `messages/` mtime, `Agent::stream` the live tail — a change
  that matters forces a publish). **The memo keys on the *derivation*, never on
  the fold** (bl-54f7): a memo caches a read of disk, and neither non-derived
  fact is on disk, so keying one on the rendered snapshot would rebuild every
  disk read whenever an echo or a live character moved — this cost restored
  with a new trigger. Before this rule the transcript's `messages/`
  and the steps tree's every `response.json` were re-read and re-parsed **per
  frame** (and the steps twice: the auth and wound banners each rebuilt it) —
  measured at 35 ms/frame for a 50-step conversation, 144 ms at 100 steps,
  which the operator felt as sluggish, sticky chat scroll. A pulse repaint or a
  scroll frame now costs a pointer compare; the rebuild cadence is the
  snapshot's own.
- Re-derivation granularity is **whole-root rebuild with a 100 ms coalescing
  debounce** (a streaming `response.json` append storm collapses to ≤10
  rebuilds/s of one workspace). Rebuild is always correct; incremental
  streaming-only refresh is a listed optimization task, not a correctness
  feature. `GitTree: PartialEq` suppresses no-op replacements, and a pass that
  changed nothing publishes nothing.
- **Poll floor (I4):** every frame schedules `request_repaint_after(2 s)`, so a
  published snapshot never waits on a mouse move. The **2 s cheap sweep** re-runs
  the cheap enumerations (readdir of clones/, workspace roots, config dirs),
  reconciles the WatchSet (a watcher whose directory was deleted/recreated — e.g.
  a re-primed clone — is rebuilt), and performs the **targeted liveness re-probe:
  only agents currently Live/InFlight are re-probed** — a released flock emits
  *no* fs event, so silent driver death is only observable by polling, and
  probing only the agents that could have died bounds the cost. Every **15 s**
  the sweep marks *everything* dirty. Correctness never depends on an event
  arriving. The per-project ball read runs on its root's dirtiness or the 15 s
  sweep, never per frame (since §16.7 W8 it is an in-process catalog load, not a
  `bl list --json` spawn — the cadence is unchanged, the cost is a directory
  walk).
- **The three periods are settings, not constants (bl-3381).** The 100 ms
  debounce and the 2 s / 15 s sweeps above are the *defaults* — the live values
  are a `Cadence` (`src/app/cadence.rs`) read from `cadence.yaml` under the yog
  state root and surfaced as §9.5 `Number` rows. yog's backend owns the only
  clock in the system (VISION §4.3), and this file is that clock's one setting.
  The worker adopts a change on the file's own announced §7.1 event (never a
  per-tick read), re-tunes the `Schedule`, and rides the value out on every
  `Snapshot`, so each period *derived* from the bases — the wound grace
  (§7.3), the late-pass and stale-snapshot thresholds, the I4 poll floor —
  follows the tuning without a frame reading disk. The parse is total: a
  missing file or field falls to the default and an out-of-range value clamps
  to the control's own bounds, so deleting the file is the reset and a
  hand-broken one degrades to the shipped rhythm, never to a stall.
- **`cadence.yaml` carries a second block: the monitor's arming (VISION §4.9,
  bl-8da1).** The file's schema is two column-0 blocks read through the same
  §9.4 anchored grammar. `cadence:` holds the clock's one `watcher` entry
  (above). `monitor:` holds **one entry per armed workspace, keyed by the
  workspace path** — the same key `ui.json` uses for its §4.1 watermarks — with
  `model:` (the cheap model every check is pinned to; required, because
  guessing it would spend the operator's money on yog's opinion), optional
  `provider:` (a brazen provider row; absent means brazen's own effective
  config resolves the model), and `prompt:` (the leaf name of the policy file
  beside this one, `monitor.md` by default). Presence *is* armed; absence is
  the default and under it no call is made, no row is written and nothing
  renders. Reading is as total as the clock's: an entry with no `model:` is not
  a watch, and a named policy file that is missing or empty reads as
  **unarmed** — the policy is the mechanism, so severing it severs the
  mechanism rather than falling back to a compiled-in prompt. Severability is
  deleting the entry (the `/disarm` gesture) or deleting the file.
- **The monitor's tick is its own thread, at the full-sweep period (VISION §4.9,
  bl-8da1).** `src/monitor/sentry.rs` runs the derivation worker's shutdown
  shape (stop flag, park loop, joining `Drop`) beside it, never inside it: one
  check is an HTTPS call measured in seconds, and the worker's pass is a
  correctness floor measured against the cheap-sweep cadence, so a call in it
  would read as yog being late. It is **level-triggered off the step spine** —
  a tick reads the published `Snapshot`, and checks an armed agent only when
  its branch tip has moved past the sha its last `ops.jsonl` check named — and
  fires **at most one check per tick**, so a storm cannot become a spend storm.
  Its period is the *full-sweep* cadence, re-read each turn: a checkpoint is a
  step boundary, not a poll, so the monitor ticks with the slowest thing yog
  does and follows the operator's clock tuning with it. Unarmed, a tick is one
  file read and nothing else.
- **The armed loop's tick is its own thread, beside the sentry (VISION §4.3,
  bl-66fb).** `src/fleet/pilot.rs` takes the same shutdown shape and the same
  full-sweep period, for the same reason one layer over: a spawn runs `bl` and
  forks a driver, and a fork inside the worker's pass would read as yog being
  late. It is **level-triggered off the published snapshot** — a tick reads the
  board the worker last published, decides **at most one move** (reap, else
  spawn) and stops — and it keeps no memory of the tick before, because it does
  not need one: the next tick reads the world the last one left, so a missed
  tick, a crashed yog and a second instance all converge. **Unarmed, a tick is
  one read of a snapshot it already has**: it returns before it builds a board,
  opens `ui.json` or looks at the trail, which is V4's burden check made
  structural rather than promised.
- **All sweep/debounce/heartbeat timing is clock-injected** (a `Clock` trait,
  same injection pattern as LockProbe/WriterProbe) so every time-gated branch
  is testable to 100% without sleeps. `Clock` also mints the wall-clock
  `ops.jsonl` stamp: the worker writes its own drift lines off the frame, and a
  second time seam for the string would be a second thing to inject and fake.
- The existing ~30 fps tool-pulse repaint is unchanged; effective repaint
  delay each frame is `min(pulse, sweep)`.
- **A pulse is scheduled by the fact it paints, never by a timer.** Every
  pulsing seat — the tool chips and the §11 live-activity indicators, all three
  of them — asks for
  its ~30 fps repaint *inside the branch that painted it*, so a window whose
  conversations are all at rest schedules nothing beyond the 2 s poll floor.
  This falls out of §5.1 #28 being a query rather than a flag: `None` is both
  "paint nothing extra" and "ask for nothing extra", one decision at one site,
  which is why no seat can drift into a busy loop by forgetting a guard. The
  §11 **bottom in-flight strip** (bl-905f) is that rule at its strictest: the
  `None` arm returns before the `TopBottomPanel` is constructed, so an idle
  window does not merely paint an empty strip — it has no strip, and the
  repaint request is on the far side of the same early return. Its
  characteristics change nothing here: every one is a field of the snapshot the
  frame already holds (`Agent::stream.text`'s length, `ToolCall::name`, a member
  count, and the §5.1 #28a call starts), gathered at enumerate time like the
  bl-cad5 recency mtimes, so a strip that ticks its elapsed once a frame still
  stats no disk — the only per-frame input is the wall clock the shell mints. The
  frame still renders a snapshot and the backend still never drives paint — the
  animation clock is egui's own (`input().time`), read, never advanced.

**Why this replaced "the frame IS the derivation" (bl-ee0a).** The original
ruling here rejected "a dedicated derivation thread pushing snapshots" as an
extra concurrent stateful component, and kept derivation inline because it made
"pure function of disk at this tick" inspectable. That trade was wrong in one
specific way: **derivation cost scales with the workspace's branch count, and
nothing bounds that count.** One conversation went from 2 branches
to 227 in ~90 s while every one of those branches streamed step files that marked
roots dirty; every 100 ms debounce release and every 15 s full sweep re-walked
the whole set inside `App::update`. A frame that does not return does not pump
events, and a window that does not pump events is what the desktop calls
unresponsive — so the only "non-responsiveness" signal the operator got was the
desktop's, accusing yog of a storm lernie was creating. Per-frame work caps were
rejected as the fix: a roots-per-tick cap cannot bound the cost of re-deriving
**one** root, which is exactly what happened. The standing rule is now that UI
and backend operations are totally isolated: the UI never freezes, which means
it does as little as possible.

**What that costs, and how it is paid honestly.** The frame no longer renders
this instant's disk; it renders the last *completed* derivation. That is a real
loss of a real guarantee, so it is measured rather than assumed:

- Every `Snapshot` carries `derived_at`. The §11 ops accessory renders
  `derivation N s behind` once the age exceeds two full-sweep periods — silent
  below that, because a full sweep re-stamps the snapshot every 15 s even when
  nothing changed, so exceeding two of them means *passes are not completing*,
  not that the world is quiet.
- A pass that takes ≥ **the period of the sweep it ran** writes a `yog-drift
  late` line (§4.2), attributed to the yog state root with the duration in
  `stderr`. Everything rendered while it ran was that stale. This is the same
  instrumentation as the dropped-event kinds below, and for the same reason:
  **drift is divergence between what the frame renders and what is on disk**, and
  a late derivation is one way to get it.

  **Two amendments, both from one storm (bl-4b28).** A 4472-row trail whose
  4447 rows were this line — one every 15 s, unbroken for a day, burying the
  operator's entire real history in 25 surviving rows — was not reporting an
  anomaly. It was reporting the schedule working:

  - *The bound is the pass's own promise, not one bound for every pass.* Every
    row was a **full** sweep (they arrive on the full-sweep tick, and only
    there): the pass that re-enumerates, re-fetches every project's balls and
    re-derives every workspace. In a real workspace — 110 agent branches, a
    `bl` fetch per project — that costs 2–3 s, and it was being judged against
    the *cheap* sweep's 2 s, a period it neither owes nor was ever budgeted.
    A full sweep is now held to the full-sweep period, which is what that
    longer period is *for*; a cheap pass, and one that swept nothing, still owe
    the 2 s poll cadence they ride.
  - *Late is an edge, not a level.* The row is written on the pass that stops
    keeping cadence and not again until one keeps it — so a permanently-late
    derivation is one dated event to correlate against what changed, not a
    restatement per sweep. Nothing records the recovery, because *are passes
    late now* is already answered, derived and current, by the staleness line
    above: a state belongs to a query, an event to the trail. The worker holds
    one bool for the edge (its own observation about its own passes; a second
    instance holds its own and reports its own).
- Ingest stays its own thread rather than folding into the worker. An
  *announcement* and a *derivation* are the two halves the drift instrumentation
  compares; one thread doing both makes "disk moved and nothing said so"
  unobservable, and therefore untestable.

**Naming the storm (bl-ee0a).** A dispatch storm is a fact about a *conversation*,
and yog could not say it: the drift instrumentation reported *that* a sweep found
unannounced change, never the shape of it. Each re-derivation now diffs
per-conversation branch counts against the previous snapshot and carries the
growth on the snapshot; the ops accessory renders `<conversation> +N branches`.
Nothing is stored — it is a diff between two derivations, held for as long as the
snapshot that found it, exactly like a `Drift`. One glance now names lernie
instead of yog.

#### What the sweeps are FOR (ruled bl-49f4)

The paragraph above justified the 15 s sweep as a **latency bound**. That was
the wrong frame, and it hid something: the sweep re-derived everything on a
timer, so a dropped filesystem event cost latency and *left no trace*. The drop
rate was unmeasurable and the sweep was load-bearing over a defect nobody could
name. **A change should be picked up automatically; a sweep that catches
something is evidence of a bug.** The two sweeps are therefore justified
separately, and one of them is now instrumented rather than trusted.

**Provenance is the mechanism.** Every dirty root carries a `watch::Mark` naming
*why* it is dirty, and marks merge with `max` over a
weakest-explanation-first order — `Sweep < Poll < Watch < Desync` — so the
strongest available explanation always wins:

| Mark | Set by | A re-derivation that changes the snapshot means |
|---|---|---|
| `Watch` | an allowlisted watcher event (§7.1) | the watcher working |
| `Desync` | the backend announcing it **lost** events (inotify `IN_Q_OVERFLOW`, a watch it could not arm) | a real drop, **announced** — re-derived at once, not on the sweep |
| `Poll` | the 2 s cheap sweep's targeted liveness re-probe | the poll working: no filesystem event exists for a released flock |
| `Sweep` | the 15 s blanket mark | **nobody announced it — a dropped event** |

A fourth kind carries no mark because no root announced it: `late`, the pass on
which derivation stopped keeping the cadence it promised (above). Same log, same
chip, same doctrine — evidence that the frame and the disk diverged. It is the
one kind counted **per run rather than per occurrence**, and that is not an
exception to the doctrine but the doctrine applied: the other three name a root
that diverged, of which there can be many; this one names the derivation itself,
of which there is one, and it either keeps cadence or does not.

**The 15 s full sweep is a deliberate backstop whose catches are alarms, not
routine.** It stays because residual drop sources are genuinely outside yog's
reach (below), but it no longer repairs silently: a `Sweep`-marked re-derivation
that changes a snapshot, and a `Sweep`-driven re-enumeration whose workspace-set
membership changed, are written to `ops.jsonl` as `yog-drift <kind>` lines
(exit `-4`), one per kind, roots in `stderr`. The §11 activity chip counts them
(`activity · N ops · K drift`) as a **query over the tail** — no stored counter,
no fourth surface. A drift line is deliberately not an `OpRow::failed` row: it
accuses the watcher, not the operator's last action, so it never hijacks the
§7.3 failure banner. **A quiet sweep writes nothing, and that silence is the
target state.**

**An accusation requires that an announcement was possible (bl-f726).** A drift
line accuses the watcher, so it is only honest where the watcher had a chance:

- **`unannounced` requires a baseline.** Drift is *divergence*, and divergence is
  measured against something. A root taking its **first** snapshot has nothing to
  have diverged from — and no watch was armed over it either, since watches are
  armed from the enumerated set, so a root yog has never derived is a root yog
  has never watched. Its appearance is the enumeration side's question, asked
  once there.
- **`unenumerated` requires a watched enumeration root.** The membership delta is
  filtered to the §7.1 enumeration roots that were **already armed** when the
  delta happened (read before the pass re-arms). A root nothing was watching
  dropped no event; it was never given one.

The case that forced the rule is the *pristine* world: `$XDG_DATA_HOME/yog/
workspaces/` does not exist until the §8.1 start flow's own `create_dir_all`
founds it, so the very first workspace is minted under a directory no watch is
armed on and cannot be armed on. The first sweep to meet it used to file **both**
kinds against it — two red rows, permanent (the trail persists), on a healthy
first run. Cry-wolf is worse than silence here: it buries the real findings the K
count exists to surface. Neither rule is an amnesty — the same workspace's *next*
unannounced change is accused like any other, and both directions are proven in
`src/app/tests/drift.rs`.

**The 2 s cheap sweep survives on its own terms and is not a filesystem
backstop.** Its liveness re-probe polls *process* state — a released flock emits
no event any watcher could have dropped — so its roots are marked `Poll` and its
changes are never counted as drops. Its reconcile half is filesystem-facing and
*is* instrumented (`unenumerated`), keyed on the workspace-set membership delta
so a finding is reported once rather than re-accused every 2 s.

**Residual sources the sweep still covers (why it is not deleted):**

1. **Filesystems inotify does not cover** — NFS, overlayfs, container bind
   mounts, FUSE. `RecommendedWatcher` arms successfully and then simply never
   fires for changes made outside the local kernel's view. There is no error, no
   rescan flag, nothing to detect: the sweep is the only cover, and drift lines
   on such a workspace are the operator's signal that the workspace is on a
   filesystem yog cannot watch.
2. **notify's recursive watch-add race** — a directory tree created faster than
   the backend adds watches to each new subdirectory loses the events inside it
   before its watch is armed. Not addressable above the library.
3. **A workspace holding a Live/InFlight agent** is marked `Poll` every 2 s, so a
   genuine drop there is explained away by the liveness poll. The drift count is
   a **lower bound**, exact on the 15 s tick (which re-probes no liveness) and
   for quiescent workspaces.

**Fixed at the watcher layer instead of covered** (bl-49f4, each proven in
`src/fs_watcher/drift_tests.rs` and `src/app/tests/drift.rs`): the backend's own
loss announcements are no longer discarded (`Desynced` on the root → immediate
whole-root re-derivation); `repo.git/packed-refs` is allowlisted (§7.1);
`WatchSet::reconcile` rebuilds a watcher whose root inode was replaced; and
`Deriver::boot` arms the watches **before** taking the first snapshot, closing a
startup window that was a dropped event by construction. `boot` enumerates and
derives directly rather than through `reconcile`, because reconcile opens a
debounce window per un-snapshotted workspace and a window opened at startup would
still be pending on the first real pass — outranking that pass's own `Mark` and
making a genuine dropped event read as the watcher working.

### 7.3 Failure modes (enumerated)

| Failure | Handling |
|---|---|
| inotify queue overflow (`IN_Q_OVERFLOW`) | the backend announces it as a rescan-flagged, path-less event; surfaced as a `Desynced` change **on the root** → the ordinary whole-root re-derivation, at once, plus a `yog-drift desync` line (§7.2). It is no longer discarded by an ingest loop that iterated `event.paths` (empty on a rescan) |
| Watch cannot be armed mid-tree (`fs.inotify.max_user_watches` exhaustion) | notify sends `ErrorKind::MaxFilesWatch` on the event channel; same handling as overflow. It is no longer discarded by `Ok(Err(_)) => {}` |
| Watch cannot be armed at all (missing root, exhaustion at construction) | `WatchSet::reconcile` skips it (absent, not stored) and retries every 2 s; the consequence — an unwatched root — surfaces as repeated `yog-drift unannounced` lines the moment anything there actually changes. An unwatched *enumeration* root is the exception, and deliberately so (§7.2 bl-f726): a workspace appearing under a root nothing was watching is enumerated at birth, not drift — the pristine world's first `EnsureWorkspace` founds that root itself |
| A drop with no announcement at all (uncovered filesystem, notify race) | 15 s full sweep re-derives **and names it**: `yog-drift unannounced`, attributed to the root (§7.2). Bounded staleness, no longer silent |
| Watched dir replaced (clone re-primed, workspace deleted) | 2 s reconcile rebuilds the watcher: the backend watches an *inode*, so a same-path replacement leaves a deaf watch a name-keyed desired-set diff would keep forever. `Watcher::is_stale` compares the armed `(dev, ino)` against the path's current identity |
| Startup read/arm ordering | `Deriver::boot` arms the watch set **before** the first snapshot: a watch armed after the read is blind to everything that landed in between |
| A derivation pass that outruns its own cadence (a branch-count storm, a loaded machine) | the frame is unaffected — it renders the last completed snapshot and keeps pumping events (§7.2). The lateness is named: a `yog-drift late` line, plus the §11 `derivation N s behind` line once the rendered snapshot passes two full-sweep periods |
| Atomic-rename inode swap on a watched file (`ui.json`, a task file) | the watch is on the *directory*, which sees Remove+Create; the allowlist matches by name, so the new inode is admitted under the same entry. (Config files are not watched at all — §7.1's bl-9130 ruling — so this row is about the yog-state and balls-clone roots) |
| Event storm from streaming response.json | 100 ms coalescing debounce per root; balls `log` files excluded from allowlists |
| Concurrent `ui.json` writers | LWW + echo-hash (§4.1) |
| Concurrent `ops.jsonl` appenders | O_APPEND + ≤PIPE_BUF lines — kernel-atomic, no interleave |
| Concurrent config-file editors (two instances, or instance + vi) | optimistic hash guard: Apply refuses if on-disk content ≠ content loaded into the buffer; user reloads and re-applies (§9) |
| Crashed `bl` op | converge-on-retry: re-run the verb (balls arch §13); stderr in `ops.jsonl` |
| Crashed/killed driver | agent classifies Stopped from framing; attention rule 2 fires; `lernie scan` deposits died epitaphs and flushes inboxes |
| yog crash mid-write | rename atomicity: whole-old or whole-new; dotfile temp debris swept at startup |
| yog signalled (SIGTERM from `pkill`, SIGKILL) or crashed between a §4.1 gesture and disk | cannot arise: `ui.json` is **write-through** (§4.1) — the gesture is on disk before the call returns, so there is no in-flight window and no shutdown hook to miss (bl-b54e) |
| yog crash with drivers running | drivers are detached into their own process group, holding no yog-owned pipe (stdin/stdout null, stderr on a file) — unaffected; next launch re-derives their state from locks/refs |
| Detached driver dies right after launch (tool version skew, missing model config) | its stderr sink (§8.1) is non-empty; the ops sweep folds the tail into the `-2` row, which becomes a rendered failure — banner + ⚠ chip. Without the sink this was invisible: exit `-2`, empty stderr, a prompt that "does nothing" (bl-a649) |
| Probe backend unavailable (lsof missing) | tri-state `Unknown` → uncertainty badge, never a false definite state (§10) |
| Driver dies leaving an empty step (version skew, OOM, kill before the first event) | the step is a **no-response wound**, not a quiet one: an empty-or-absent `response.json` **and** no `meta.json` **and** no driver on the agent (§3.5) renders "driver produced no response" in ichor beside the step and banners it at Altitude 1 (§11). Framing alone reads this `Killed` — the ash "stopped" badge over a `0 attempts · 0 tok` row, which is how it read as a quiet step (bl-7f2e). **The banner states the *reason*, in the adapter's own words** (bl-55d8): the tail of that step's `stderr.log`, which lernie ARCH §2.3 defines as *"the adapter subprocess's stderr, appended once per attempt across the model call. **Empty on an ordinary run**: brazen speaks every failure in-band on stdout, so bytes here mean the adapter failed outside that contract — a startup failure (a malformed brazen config, an unreadable credstore) that produced no events at all"* — which is this row's class exactly, so the file is not a hint about the cause but the cause itself. The predicate is unchanged and the reason is not a second fact: one derived value carries both (`Wound::{None, Mute, Spoke}`), and the `stderr.log` read is gated on the wound so a healthy step never pays for it. A wound whose `stderr.log` is empty too (a SIGKILL mid-call) is `Mute` and **says so** — "nothing on disk says why" — never a bare glyph and never a pointer at somewhere that has nothing (§11 glyph doctrine). **What this retired:** the banner used to say *"the driver's own stderr is in the activity trail below"*, which was wrong for the class the operator actually hit. A turn continued by `lernie message` is driven by a child **lernie** launched, not by a yog detached spawn, so no §8.1 per-spawn sink exists for the ops sweep to fold into a `-2` row at all — the falsifying run's two sinks belong to its two `lernie prompt` starts and the one matching the wounded turn is zero bytes. The step's own `stderr.log` was the only copy of the answer, and yog was reading past it. The operator's whole signal was *"it looks like the second message in a conversation always fails"* — the absence of a reply. **Residual, recorded not hidden:** the drill-in's record picker (§11 Altitude 2) still lists the five JSON records only, so the *whole* of a long `stderr.log` is not browsable in-window; the banner quotes its tail on the two bounds the crate already had (`opslog::detached::captured`'s 4 KiB of file, then `opslog::rows::stderr_tail`'s last three lines — the same tail every other §7.3 surface shows) and names the file for the rest |
| A **healthy send** classified as that wound for a moment (bl-90bf) | the wound's two halves do not share a clock: the disk half is read through the §7.2 per-snapshot memo (once per published snapshot since bl-e90a; per frame before that), the liveness half rides the probe cache inside that same snapshot, and a driver *taking* its flock emits no fs event — so between the send and the §7.2 poll that finds the lock, a genuinely-in-flight empty step reads as a wound. The predicate is right on the inputs it is given; the cache is what is behind. The banner therefore holds a **grace window** before it paints (`src/app/grace.rs`, `WoundGrace`): a wound that clears inside the window never reaches the screen, one that outlives it banners and stays. The window is `Cadence::wound_grace` — cheap sweep + debounce, the catch-up bound itself (one sweep tick to mark the root, one debounce window before the mark is due), spelled as that sum over the **live** cadence off the rendered snapshot (bl-3381) so a re-tuned period carries the grace with it, never a magic number. The gate is **render-layer RAM on the injected clock**, mirroring `Schedule`'s debounce; the predicate stays pure and Clock-free (§5.1 #13). A genuinely dead driver is therefore banner-ed *late*, never not at all — by the window plus the frame cadence's own ≤2 s poll floor (I4), which applies to it exactly as to every other rendered fact. Only the **banner** is graced: the §11 Altitude-2 Steps row paints the same flag ungated, because a cell in a table you opened is as fresh as the rest of that table, while a banner is an unrequested alarm and an alarm that retracts itself teaches the operator to distrust it |
| `bz --list-models` fails, or returns an empty roster (§9.4) | the picker banners in ichor with the captured stderr and the exact command to run by hand (the §8.3 fallback grammar); an empty roster is named as itself ("the provider offered no models"), never rendered as a picker with nothing in it. The current assignment stays on screen throughout, so a failed query never looks like a lost model. *An **auth-shaped** failure additionally names what this row needs (`login_blocked`'s own sentence) and carries the control that goes there — Login for a row that signs in, the Config tab otherwise (bl-91f1). The banner and the fallback command are unchanged beneath it* |
| A config file the §9.4 anchored-block grammar does not recognize | the picker declines **loudly** (ichor, naming the file and the shape it expected) and points at the §9.2 / §9.3 raw editors. yog never guesses at YAML it cannot recognize, and never half-writes: the models.yaml half is refused before the providers.yaml half is attempted |
| A spawn whose requested cwd is not an existing directory | **the directory is named, never the program** (bl-6191): `std::process` fails such a child *between fork and exec*, and reports the resulting ENOENT against the **program path** — so a start into a typed-wrong work directory read `failed to spawn <yog binary>: No such file or directory`, telling an operator their binary was missing. Every spawn shape routes its failure through one constructor (`CliError::spawn`), which asks the cwd's own question first (`work_dir_fault`) and answers `work directory does not exist: <path>` — or `is not a directory` for a path that is plainly there, since a second lie fixes nothing. Not gated on the OS error kind: a cwd that is not a directory could not have forked for any other reason. The same question is what the §11 field pre-flights, so this error is the *unreachable-by-the-form* residual — a directory deleted between the flag and the Enter, or a non-form caller |
| Failed action (short verb or start step) | **a rendered fact, never stderr-only, and rendered exactly once**: the full `ops.jsonl` entry (argv, cwd, exit, origin, stderr) is expandable at the ops pane, *and* the **originating surface** — and only it — renders the failure in ichor red with argv + stderr tail. The banner is **derived every frame** from the refreshed ops tail (`AppModel::last_failure(origin)`), never cached at dispatch: the dispatch handler runs microseconds after a detached spawn, before the child can die, so a snapshot taken there is `None` forever and the sink row above surfaces to nobody (bl-4895 — three live prompts, three populated sinks, zero banners). No `eprintln!`-only error path may exist in `src/shell/` (STORIES INV-2). **The originating surface is the op's `origin` field (§4.2), stamped at dispatch — see the row below for the rule it obeys** |
| **How a banner ends** (bl-c417) | Two ways, and until bl-c417 only one existed. (1) **Retirement**: a newer op of the same `origin` that did not fail — the §6 rule, per-surface (the row below). (2) **The operator's ack**: `AppModel::last_failure` queries `opslog::since_ack`'s rows, so the §4.2 ack line quiets every surface's banner at once, whether or not anything was retried. That second exit is the whole of the complaint — *"I need a way to make the failed notification go away"* — because an operator who reads an error and decides **not** to retry could not otherwise put it down: the only exit was a *successful re-run of the same verb*, so a failure the operator has understood and chosen to leave alone banners forever. The ack is not a widget flag: it is a durable line, so the dismissal converges to the other instance and survives a restart exactly as the failure did. It is also not amnesia — a **new** failure of that origin lands after the watermark and banners again. The dismiss control sits on the banner itself and in the §11 ops pane, both spelled from one home (`opslog::operator`), both explaining themselves on hover (bl-68ac) |
| Which surface is "the originating surface" (bl-48f8) | **The op's *subject*, recorded at dispatch, one banner surface per subject.** Three subjects exist and each has exactly one seat: a **ball** op (every `bl` verb, and every step of a ball-rung start — `bl create`/`bl claim`, but equally its `lernie prime`/`lernie new`/`["yog-step","mkdir"]` substrate steps and its detached `lernie prompt`) banners in **the roster's balls section**, where the ▶ Start / ▶ Continue / Create-&-Start row that offered it is (§11, bl-6ad8); a **conversation** op (`lernie message`/`stop`/`scan`, and every step of a bare- or path-rung start) banners at **the composer** — the empty world's bootstrap box being that same box before a workspace exists (§3.4), never a second seat; a **world** op (the §9 config writes, the §16.3 space knob, the §8.3 login flow, the §3.6 unmaking, and yog's own §7.2 drift lines) banners on **no §7.3 surface at all**, because each of those surfaces states its own outcome in place and a config-write failure is not news the composer has any business breaking. §6's retirement is per-surface with it: a surface's banner clears when *that surface's* next action runs clean, and never when someone else's does. **The subject is not the pointer's position:** one gesture has one body however many hands reach it (`ball_bar::close_ball` is the composer's Close button, the §11 `c` key and the row menu), so a ball verb is about a ball wherever it was clicked — forking that body per hand to record a pixel would record a distinction no operator makes. What this closes: `last_failure()` was one global query over the ops tail and three surfaces rendered it unconditionally, so a single failed start painted the balls fold, the composer *and* the bootstrap box at once, a config-editor failure accused the composer, and any surface's clean run wiped every other surface's live banner |

---

## 8. The action surface (v1) and exact argv

All spawns go through `cli_outbound` (generalized: binary resolution
parametric over env var — `LERNIE_BINARY`, `BL_BINARY`, `BZ_BINARY`, default
PATH names — plus `current_dir` support and a detached-spawn mode). **Every
spawn carries the composed world env (§16.2)** — the overrides layer over the
inherited environment via the existing `run_env` seam — so every child, the
detached driver included, runs *inside* the nested world; an agent's own tool
processes inherit the driver's nested `$XDG_STATE_HOME` **and its `PATH`**, so
the `bl` they invoke is yog's own shim (§16.7 W9) against the right nested
paths. Workspace-scoped spawns additionally carry `YOG_NAME=<name>` (§3.3) — the
claimant name that shim defaults `--as` to. Binary
resolution is unchanged. Every attempted action appends its line to
`ops.jsonl` (§4.2). Spawns come in exactly three classes: **short-piped**
(run to completion, outcome logged), **detached** (own process group,
stdin/stdout→null, stderr→a per-spawn sink file, a **handoff** logged with the
`-2` sentinel and a failure to launch with the ordinary `-3` one), and
**streamed-piped** *(added with the §8.3
login amendment: **both** output streams line-buffered and rendered live at the
invoking surface, each line tagged with the stream it came from — a
§5.3-whitelisted instance-local stream — with the outcome line appended at exit.
The tag is load-bearing (§8.3 as amended by bl-b4e5): `bz --login` writes its
whole flow to stderr while `bz --list-models --json` writes its payload to
stdout, so a rendering consumer paints both and a parsing consumer filters.
Members: `bz --login` and the §9.4 `bz --list-models` roster query)*.

### 8.1 Start (the composite verb — §3.4's axes, as argv)

1. Resolve the target workspace: the focused one. **Zero workspaces in the
   world → take the default name (`home`, §3.1), `mkdir -p` the names root,
   `lernie new <root>/home`** — the bootstrap is this empty case, not a
   separate flow.
   The explicit **New workspace** verb (§11) runs the same `lernie new` with
   the operator's typed, §3.1-validated name, deliberately. **The resolved
   workspace then becomes the focused one**
   (§3.4) — every rung, before step 2 opens its composer, so the composer and
   every surface beside it name one workspace.
   **Birth judges only what exists at birth (bl-c3a9, retired by bl-00ee).**
   `lernie prime` lays down `<world>/lernie/template/providers.yaml` and every
   `lernie new` commits it verbatim as the workspace's first `config/default`,
   so that one file decides which provider row every conversation in the new
   workspace dispatches through. bl-c3a9 therefore ran §9.2's provider gate over
   it *before* `lernie new`, and while brazen's table was the machine's that was
   a true statement made early. §16.2's wall made the table the **workspace's
   own**, born empty with the workspace and filled by the operator's
   per-workspace sign-in afterwards — so the gate judged a template against a
   wall that cannot exist until the birth it was refusing, and refused every
   workspace naming a row brazen does not ship, permanently. The gate is retired
   rather than relocated (§9.2 records the full before/after): a start creates
   the workspace whatever its template names, and a dead row is faulted in the
   §9.5 pane and surfaced at the first dispatch through §8.3's auth-shaped
   step failure — both against a wall that is a fact by then.

   **The pinned template already grants the complete worker pool; yog grants
   nothing on top (bl-7fc8).** Pinned lernie (`=0.0.8`) authors
   `worker.tools: [apply_patch, bash, cd, dispatch, load_skill, message,
   multi_tool, read_file]` — the entire shipped pool, `message` and `dispatch`
   included — so a root agent in a workspace yog just created can already
   message a sibling and dispatch a subagent with no second write. An earlier
   creation-time rewrite (`grant_worker_tools`) once re-asserted those two
   names against a stale template that lacked them; against every template
   since, it read the role, found both already present, and authored no
   commit — a no-op path kept alive only by its own stale comments. bl-7fc8
   deleted it, its staging/editor call and the tests that existed only for it.
   The first `config/default` is lernie's/operator's one home from the start,
   read and edited through §9.3/§9.4 like every later config. **No grant path
   returns** (VISION §4.11): tool-name narrowing is theater when `bash` is
   every effect class at once — what an invocation may *do* is adjudicated
   per call by the §8.6 capability control, not per name at creation.

2. Open the **editable goal composer** (§3.3), prefilled per payload rung —
   **and only when that rung composed a prefill** (bl-9acf). The table above
   gives the ball and path rungs one and the bare rung none, so the bare rung
   opens no start draft at all: the raise founds its sphere, focuses it, and
   hands the keyboard to the docked composer, which *is* the goal box for a
   rung with nothing to edit (§11: one box, one Enter). A draft over a blank
   prefill is not a lighter version of this step — it is a second goal box
   stacked on the first, with its own name preview, and its Send fired
   nothing but an empty payload onto the wire. One predicate decides
   it, `actions::goal_present`, and the same one arms the fire: **a blank goal
   never sends**, from the start draft's Send, from the §11 Enter binding, or
   from the composer (the goal half of `new_prompt_enabled`, bl-6191). On
   confirm: the conversation name mints (§3.3 — a pure in-process step;
   exhaustion is a `["yog-step","mint"]` row, §4.2), and
   `lernie prompt --name <minted> <root>/<name> <goal>` — the goal **verbatim**
   (bl-6920) and last in the argv (the ops-log clip trims exactly the final
   element, §4.2) — is **spawned
   detached** (own process group, stdin/stdout→null, **stderr→the per-spawn
   sink file**, `YOG_NAME=<name>` layered per §8), cwd per
   the §3.4 column. The agent id on stdout is
   **deliberately not read**: the new root materializes in the watched repo
   within a tick — *derive, don't parse*. Detachment also removes the failure
   mode where yog holding a driver's stdout pipe ties the driver's lifetime
   to yog's (`Stream`'s Drop SIGTERMs; a SIGPIPE after yog death would kill
   the loop).

   **Detached is not parentless (bl-3016).** yog remains the driver's parent
   for as long as yog lives — dropping the child handle neither signals nor
   reparents it — so each spawn hands its handle to a thread that blocks in
   `wait` and discards the status. Reaping is an obligation *separate* from
   detachment, and skipping it left one zombie per fire for yog's whole
   lifetime. The wait is not a leash (no signal, no pipe): the driver still
   outlives yog, and yog's death takes the thread, leaving init to adopt a
   live driver exactly as before.

The ball rung inserts, between 1 and 2: (optional) `bl create <title> …`
(cwd = project; stdout = id), then `bl claim <id> --as <name>` (cwd =
project; stdout = worktree path). **The order is load-bearing:** every
substrate step (the seed, `lernie new`) precedes every `bl` mutation, so a
failed or missing substrate aborts before anything half-commits — *the start
flow* can never mint an orphaned claim. (A claimed ball whose workspace was
later deleted remains a legal state; §3.5 renders it claimed-elsewhere.)

The planner (`start::plan`) is a pure function returning the command sequence;
the executor runs it step-by-step with per-step outcomes in `ops.jsonl`.
Steps are individually idempotent-or-convergent: re-running after a crash at
any step converges (double-claim refuses benignly; `lernie new` skipped when
the dir exists; prompt just adds a root; **a ball already claimed by a local
workspace name re-plans as a prompt into that workspace — resume, not a
second mint**). A **new** ball defers the id: the plan is a single
`bl create`, and the freshly-minted (ready, unclaimed) ball is re-planned as
an existing one — the new→existing transition *is* the convergence, not a
special case. The claim's stdout worktree path is cross-checked against the
bl-delivery formula (both the `<id>` and `<id>-<claimant>` variants match;
anything else is a convention drift surfaced loudly, never silently
accepted). The short steps (`lernie prime`, `bl create`, `bl claim`,
`lernie new`) log their piped outcome; the detached `lernie prompt` logs only its **spawn** — argv,
cwd, and a `-2` sentinel exit, since the row is written at launch and never
rewritten, while the driver's status lands arbitrarily later and is discarded by
the reaper. A fork that never landed is **not** that row (bl-afa9): it writes
the §4.2 `-3` synthetic-failure line with the error in `stderr`, so "handed off"
and "never started" are never the same fact twice.

**The detached child's stderr sink (bl-a649, amending §13.3).** A spawn failure
is not the only way a prompt fails: the child can launch cleanly and *then* die
(a tool version-skew refusal, a missing model config), which under the original
stdio→null left `-2` + empty `stderr` — indistinguishable from a healthy launch,
and the operator saw a prompt that did nothing. So the detached child's stderr
is bound to a **per-spawn sink file**,
`$XDG_STATE_HOME/yog/detached/<ts>-<workspace leaf>.err`, and the ops row's
`stderr` is **derived from that file at read time** (§4.2, §7.2) instead of
being copied into `ops.jsonl`. Consequences, each load-bearing:

- **The sink is the authority; the row is a projection.** The fact lives once.
  `ops.jsonl` records the *launch* and is never rewritten; a driver that keeps
  writing surfaces more on each sweep, with no second durable copy to diverge.
- **The join key is computed, not stored.** The sink's name derives from the ts
  and workspace the ops line already carries, so no field is added to the §4.2
  schema to point a row at its file — the path *is* the id.
- **A file, not a pipe.** yog holds no descriptor on it, so §8.1's whole reason
  for detaching is untouched: the child outlives yog and keeps writing.
- **Only the tail is folded**, bounded, from a line boundary — a long-running
  driver's sink is unbounded and this read runs every sweep.
- **Nothing new stirs.** A non-empty capture makes the row `failed()` by the
  rule already written for `-2` (§4.2), which is what the §7.3 banner and the
  §11 activity chip's ⚠ count read. No new signal, no new surface.
- **An unopenable sink degrades to `/dev/null`** and the launch proceeds: the
  driver is the point, the capture is the diagnosis.

### 8.2 Per-workspace / per-agent verbs

| UI action | argv (cwd) | Spawn mode |
|---|---|---|
| New prompt (new root) | `lernie prompt <ws> <text>` | detached |
| Message agent (also the resume gesture — no resume verb exists, ARCH §2.9) | `lernie message <ws> <agent> <text>` | short, piped (it self-detaches its driver) |
| Stop | `lernie stop <ws> <agent>` (+ `--stop-children` toggle) | short, piped |
| Fork an attempt from a pinned notch (VISION V2, bl-dc0c) | `lernie dispatch <role> <ws> <parent> --goal <text> --from <ref> [--pin skills/<s>/SKILL.md=<pool>/<s>/SKILL.md]…` (ws) | short, piped (it detach-launches the child's own driver). **Piped, not detached, on purpose:** a refusal — an undeclared role, a ref the workspace has not got — must come back in lernie's own words as a rendered failure, not as a click that did nothing. A cohort is this row fired N times, one ops line each |
| Scan / flush | `lernie scan <ws>` | short, piped; summary line surfaced |
| Move a live conversation onto the current config (§9.4's drift exit, bl-2d19) | `lernie retarget <ws> <agent>` (ws) | short, piped. It writes `refs/lernie/retarget/<agent>` and returns; the conversation's **own** executor lands the re-fork at its next step boundary (lernie ARCH §2.2), so nothing yog spawns advances the branch. No `--config`: lernie's default lineage is the one §9.3 writes and the one the drift is measured against |
| Close ball | `bl close <id>` (project) | short, piped; capture/fold/gate/squash output in `ops.jsonl`, gate failures verbatim (claim+worktree stay up, bl's own semantics) |
| Assign ball → workspace (§3.2) | `bl claim <id> --as <name>` (project) | short, piped |
| Move ball → other workspace | `bl unclaim <id>` then `bl claim <id> --as <other-name>` (project) | short, piped ×2, both logged |
| Release ball | `bl unclaim <id>` (project) | short, piped |
| New ball | `bl create "<title>" [--body B] [flags]` (project) | short, piped; new id captured on stdout |
| Update ball | `bl update <id> …` (project) | short, piped |
| Delete workspace (§3.6 — typed-name confirm; refused while any agent is Live/InFlight) | `bl unclaim <id> --as <name>` per live bound ball (project), then the `ui.json` prune, then the dir removal logged as `["yog-step","delete-workspace"]` | short, piped ×N + the §4.2 non-spawn step line; the §8.1 planner idiom, order load-bearing (§3.6) |
| Delete conversation (§3.6 one deep, bl-f17a — confirm scaled to blast radius; refused while any member is Live/InFlight) | `lernie delete <ws> <agent>` (+ `--children` iff the typed name armed it); the dialog's census is `lernie delete <ws> <agent> --children --dry-run` beforehand (unlogged — a read, the `bl conf` seam's idiom) | short, piped; a clean removal then prunes the subtree's `seen` keys (`ui.json`, §4.1) |
| Refresh models | `bz --list-models --provider <row> --json` | streamed-piped where a frame paints it (§9.4's picker, physically `yog bz …` since §16.7 W10 — the logical argv logged is unchanged); **in-process** through the linked brazen where nothing paints it, which is `RealBzRunner` and therefore every off-frame caller (§8.5's `Models`, bl-dff8) |
| Login provider | `bz --login --provider <row> --browser` (Login pane; also offered beside an auth-failed step) | streamed-piped (§8.3); same physical retarget. `--browser` is unconditional and the row is offered only when brazen's table says `auth = "oauth2"` (§8.3 as amended, bl-b4e5) |

Short verbs show a busy indicator (RAM — the underlying fact is the
`ops.jsonl` line plus substrate state both instances see).

**Claimant rider (Z4; "identity rider" pre-bl-68d9).** Every `bl` claim/close/unclaim yog issues is stamped
`--as <workspace name>`, **not** the operator's `$USER` — the claimant delivers
its own ball (§3.2's ownership line, the same fact as the start flow's `bl claim
--as <name>` and W9's `YOG_NAME`). Concretely: **close** and **release** stamp
the ball's *bound* workspace name (its claimant); **assign** and a **move**'s
claim stamp the *target* workspace name; a **move**'s unclaim stamps the ball's
current (source) workspace name. The operator identity survives only as the
*author* of a standalone `bl create`/`bl update` (§8.2 New ball / Update ball),
where a workspace is not the reporter.

**The workspace-bound rider (bl-bf79).** Every `lernie` row in the table above
is *about one workspace*, and such a spawn carries two env facts: the
workspace's **wall** (`YOG_WALL`, §16.2) and its **name** (`YOG_NAME`, §3.3).
Both are laid **at one seam** — the bound `lernie` a workspace verb takes
instead of a bare command handle — and never per verb. The rule is normative
and total: *a §8.2 `lernie` verb takes a workspace-bound spawn, so there is no
per-verb judgement about which facts it owes.* `stop` launches nothing and so
gains nothing from the wall; it is bound anyway, because an exemption is a
decision, and it was three such decisions that shipped the bug.

That bug is what this rider records. `message`, `fork` and `scan` each laid
`YOG_NAME` alone (or nothing at all). `lernie message` deposits and then
**detach-launches a driver** when the branch is quiescent (lernie ARCH §2.9 —
there is no resume verb; the deposit restarts a driver), and that driver
inherited yog's fold *without the wall*: its first `bz` died with `no workspace
in this environment — providers, sign-ins and the model cache belong to a
workspace`, and the turn produced a zero-byte `response.json`. The first turn of
a conversation always worked (`lernie prompt` is fired from §8.5's chokepoint,
which did lay the wall), so the operator saw "the *second* message always
fails" — really "any message that has to revive a quiescent driver fails". This
is §16.2's own rule applied where it had not been: *set once, at the edge that
knows the workspace, and no downstream seat has to be told.*

### 8.3 Deliberately not in v1 (with reasons)

- **Manual `lernie dispatch`** — workflow-driven dispatch is the designed
  path; a manual role-dispatch button invites mis-goaled children. If ever
  surfaced: `lernie dispatch <role> <ws> <branch> --goal <text>`, role list
  derived from governing `providers.yaml` roles that carry souls.
- **Fork-from-history** — no lernie CLI verb exists; yog may not write refs
  (ARCH §3.5). Upstream gap, tracked as a lernie ball, not worked around.
- **`bundle` / `replay` actions** — v1.1; replay *results* (`replays/*`)
  already render read-only in v1 since they are just workspaces.
- **`bl conf` editing, `bl prime` of new projects** — v1.1; v1 scope is
  projects already primed (present in `clones/`).
- **brazen `[ingress]`/`--serve`, credentials editing** — out of scope;
  credentials are constitutionally untouchable. The one exception:
  `bz --login --provider <row>` *is* v1 (STORIES S0) — bz's one interactive
  surface, needing no TTY input, run as a **streamed-piped** verb (§8's third
  spawn class) from the Login pane, its live lines rendered in the pane
  verbatim. yog renders the flow; credentials remain bz-stored, never read or
  written by yog. Showing the exact command stays as the fallback when the
  piped flow exits non-zero.

  **The flow-selection rule and the capability source (bl-b4e5), three
  rules:**

  1. **The flow is always the browser flow: `--login --provider <row>
     --browser`.** bz *defaults* to the headless RFC 8628 device flow and
     refuses it — exit 78, "this provider has no device endpoint; use
     `--browser`" — for any row whose `oauth` block omits the optional
     `device_url`, which is most of them. The loopback AuthCode flow (RFC 8252) has no such hole:
     `authorize_url` and `token_url` are *required* fields of every `oauth`
     block while `device_url` is `Option`, so `--browser` is the one flow
     **every** oauth row can serve. yog is a desktop GUI with a browser at hand
     and no terminal to type a device code into, so it is also the right one.
     This is a constant, not a per-row branch: there is no flow yog must guess
     at, and no flow selector on the surface.
  2. **Loginability is brazen's `auth` column, never a yog-side
     reclassification.** `bz --list-providers --json` projects
     `{name, protocol, auth, credential}` per row; brazen's own resolve
     invariant is that `Provider::oauth` is "present exactly when
     `auth = "oauth2"`". So `auth == "oauth2"` *is* "this row has an `oauth`
     block", answered by the crate that owns the invariant. `bz --login` signs
     in oauth rows only, so every other spelling (`none`, `api_key`, `bearer`)
     is a row it can only refuse: those rows get **no Login button at all** —
     see rule 4. The projection carries no device-endpoint fact at the
     pinned brazen — and needs none, per rule 1.
  3. **The streamed class carries stderr, tagged.** bz writes its entire
     human-facing login flow to *stderr* (stdout is reserved for its
     machine-readable discovery output): the authorize URL, and on failure the
     exact reason and remedy. The class line-buffers both streams and tags
     each line with its origin (§8), so the rendering consumer paints both and
     the parsing consumer (`bz --list-models --json`, §9.4) filters to stdout.
     The `ops.jsonl` outcome row's stderr is derived from those same lines —
     the log and the pane can never name different text. The fallback command
     is the exact argv attempted, `--browser` included, so what is offered to
     run by hand is a command that would actually succeed.
  4. **A row states its state in words; there is no dim shape (bl-402f).**
     The pane once rendered ten `Login <provider>` buttons and dimmed the eight
     it could not serve, with the reason behind a hover — so eight rows were a
     verb that silently did nothing, and no row said whether it was already
     signed in. Two rules replace that:
     - **Presence renders** (STORIES S5 point 4). Every row carries its
       credential fact as a sentence, phrased by the credential model that makes
       it true: an oauth2 row is `signed in` / `not signed in`, a keyless row
       needs `no credential needed`, a keyed row's file is `credential stored` /
       `no credential stored`. "Signed in" is a sentence only a login can earn,
       so a keyed row never claims it. The fact behind it is the §5.1 #22
       existence read and nothing more — contents are never opened.
     - **A verb yog cannot honor is not rendered.** A non-oauth2 row shows the
       *reason* where the button would be — "keyless — nothing to log in",
       "api-key provider — set the key in Config", "bearer-token provider — set
       the token in Config", and for an `auth` spelling this build does not know,
       the spelling quoted rather than guessed at. The operator's next move is on
       screen; there is nothing to click to discover it.

     Both come from **one derivation**, `brazen::row_views` beside the column
     projection it reads (`src/config_edit/brazen/providers.rs`) — the §9.5
     config rows render the identical struct, so the two surfaces cannot state
     different things about one provider. The Login pane's `↻` re-asks brazen
     *and* re-reads presence in one gesture (§7.2: never per frame), which is
     also how a just-completed sign-in becomes `signed in`.
  5. **The affordance beside a failed step names its row (bl-8e34).** Detection
     alone offers a verb without its object: `bz --login` takes a provider row,
     and the error line yog classifies carries none — brazen's canonical error
     leaves `provider_detail` null even where the row is exactly the news (the
     motivating line, verbatim: `{"kind":"auth","message":"borrowed OAuth
     credential is expired; refresh it with the tool that owns it"}`). So the
     per-step Login flag is a **three-state** value, not a bool
     (`login::auth::AuthFailure`): `No`, `Unrouted`, `Row(<row>)` — offered or
     not, and when offered, routed or not.

     **The row is a join of two facts already on disk, never a stored one.** The
     failing step's `request.json` names the model it was dispatched with (lernie
     ARCH §4.2 — the id rides the canonical request verbatim); the agent's
     **governing config commit** binds each role to a `(provider row, model id)`
     pair, read through `fork::roles_at` — §9.4's own `roles:` grammar reader, so
     the picker, the fork composer and this affordance can never disagree about
     what a `providers.yaml` says. Matching the one against the other yields the
     row. A model no role binds, or one bound under **two different rows**,
     yields `Unrouted`: a guessed row would send the operator through a browser
     sign-in for a credential that was never the problem, which is worse than the
     unrouted sentence it replaces.

     **Where the cost is paid.** The join is the module's one git read, so it is
     asked **once per agent and only when a step is actually failing**
     (`steps_view::route_auth`), never per step and never per frame — a healthy
     conversation's steps view costs exactly what it did before this rung. The
     per-step half (that step's `request.json`) is read for failing steps alone.

     **Both seats say it, and the pane still opens.** The wordings live on
     `AuthFailure` beside the classification that decides them (the
     `GoverningConfig::frozen_label` discipline, §9.3), so the §11 conversation
     banner and the Steps-tab mark cannot drift: `⚠ the last step failed on
     <row>'s credentials` and `⚠ auth: <row> — Login ↙`, degrading to today's
     unrouted wording where no row was derived. The Login pane still paints
     beneath the banner in both cases — a *wrong* derivation must never become
     the only way out.

### 8.4 World escape hatches (`yog env`, `yog exec`)

Two subcommands of the yog binary — the multi-call pattern beside
`--editor-apply` (§9.3) — expose the composed world to a human at a shell:

- `yog env [--ws WORKSPACE]` prints the world's `export` lines (`LERNIE_HOME`,
  `XDG_STATE_HOME`, `PATH`); `eval "$(yog env)"` drops the current shell *into*
  the world, where a bare `bl`/`lernie`/`bz` is the world's own shim — yog's
  embedded substrate against the nested state (§16.7 W9/W11) — the same tools
  the world's agents get.
- `yog exec [--cwd DIR] [--ws WORKSPACE] <cmd…>` runs one command inside the
  world (world env layered, optional cwd) without touching the caller's shell.
  Because the world's `PATH` fronts `world/tools/`, a bare `bl` here is the
  shim; a **host** `bl` is reachable only by naming it by path.

**`--ws` is the headless workspace binding, and it lives here (bl-b589).** The
hatches hand out the *world*, which names no sphere — and since the
blast-radius ruling (§16.2) brazen's providers, sign-ins and model cache live
inside a **wall**, so a seat with only the world cannot reach any of them. That
made the advertised `yog bz --login` refuse with *"no workspace in this
environment"* wherever a human could actually type it, and left the GUI's own
run-by-hand fallback printing a command that fails. `--ws WORKSPACE` layers
that workspace's wall over the world — literally `wall::pairs` on top of
`world::overrides`, the same layering every workspace-bound spawn the window
makes already uses — so `yog exec --ws WORKSPACE bz --login --provider NAME
--browser` signs in *inside* that sphere, and `eval "$(yog env --ws
WORKSPACE)"` stands the wall for a whole shell.

The binding is at the **hatch**, not at `bz`, and that is the point: no new
flag on a foreign crate's argv, and every wall-needing command is bound by the
one spelling rather than sign-in earning a special case. It is the same flag,
spelled the same way, as `yog gesture --ws` — the workspace's *path*, whose
§3.1 leaf keys the wall — so a seat names a sphere exactly one way. **The
no-wall refusal is kept**: naming no workspace layers nothing, a bare `yog bz
--login` still exits 64, and credentials never fall back to the machine's own
state. What was wrong was the advertisement, not the refusal.

**Both hatches converge `world/tools/` before handing the world out**
(bl-44a5; `world::tools::ensure_tools`, the same convergent seeding the start
flow and the lernie arm use). The tools dir is a generated artifact of *the
world* — the `PATH` override names it unconditionally — so it materializes
wherever the world is handed out, never only at a first Start. Before this,
a pre-first-Start hatch handed out a `PATH` fronted by an empty dir: a bare
`bl` silently fell through to whatever host binary was next on the ambient
`PATH` (§16.4's (b), the wrong-implementation failure W9 exists to prevent),
and in the W14 clean room — no host `bl` at all — it died `No such file or
directory` (the W14 clean-room drive).

Both are pure entrypoints of the yog binary, not substrate spawns — the
operator's hand-hold on an otherwise-encapsulated world (§16.2), and the human
counterpart to the embedded-crate agent tools (§16.4). **They stay hatches:**
yog never fires `yog exec` at itself as a reproduction affordance beside a
failure — the driver's stderr sink already carries the cause (§8.1), so a
re-run button would re-create a held fact and start a second driver (§14).

### 8.5 The control boundary (VISION §4.8)

The ruling (recorded in VISION §4.8) is a formal control boundary for every
operation that *does something*: focusing an input box does not cross it, but
hitting enter does; switching tab does not, but changing a value does — and so
does the call that populated the tab's contents. Actions and queries cross the
boundary; views do not. This section is that ruling made structural; the §8.4 hatches remain the human's shell-level doors, and this is
the full machine surface beside them.

**The taxonomy is the existing invariants, not a new concept.**

- **Actions** are the ops trail's rows (§4.2): everything that mutates a
  substrate. Their carrier is `boundary::Action` — one enum variant per
  gesture, parameters and all — and one chokepoint, `boundary::dispatch`,
  routes every variant to its §8 executor. Today's roster: the §8.2 short
  verbs (message, stop, scan, close, assign, release, move, create, update),
  the trail's own two operator verbs (ack, clear-trail — bl-c417),
  the §8.1 start family as its two real gestures (`Prepare` — the mutating
  seed/new/create/claim half, returning the composer's `Prepared`; `Prompt` —
  the deferred detached fire), the two §3.6 deletes — `DeleteWorkspace`
  and `DeleteAgent` (bl-f17a) — each gated at dispatch exactly as its dialog
  gates it (fail-closed; typed-name armed / blast-radius armed), and the
  alignment monitor's family (VISION §4.9, bl-8da1) — one `Monitor` variant over
  `monitor::Verb`: `Arm` (write the `cadence.yaml` monitor entry pinned to a
  model, seed the policy file beside it), `Disarm` (delete that entry), and
  `Flag` (raise an attention item on one conversation, with its reason, as its
  own ops row). The three share a subject, a config file and a trail, so they
  fold to **one** row in each of the four boundary tables rather than three;
  each still carries its whole parameter set, and each still spells as its own
  verb, envelope `op` and help page — the fold is in the carrier, never in the
  surface. `Flag` is here for a reason beyond the operator's convenience: it is
  the **floor grant** an alignment responder is given (bl-7aef), so an agent
  signalling out does it by calling a verb whose schema types the signal, never
  by writing prose yog would have to parse a verdict from.
  Beside it, the **armed loop's** family (VISION §4.3, bl-66fb) — one `Fleet`
  variant over `fleet::Verb`: `Arm` (write the `cadence.yaml` `fleet:` entry
  naming the project the loop takes work from and the cap it may hold) and
  `Disarm` (delete that entry). It folds for the monitor's reasons exactly, and
  spells as two verbs, `/fleet <cap>` and `/disband`. Disarming is its own
  gesture rather than an arm with a cap of zero, because a zero cap is an armed
  loop that spawns nothing and still reaps — a different instruction. Arming
  seeds nothing and spawns nothing: the first spawn is the *loop's*, on its next
  tick, through `Prompt`'s own door and its ceiling.
  Beside them, the V2 **attempt**, `Fork` (bl-dc0c). **`Fork` carries one
  attempt, never a fan.** V2's ×N fires it N times with per-attempt overrides,
  because cohort membership is derived from the notch the children were born at
  (§5.1 #33) — so a fan gesture would have to *name* a group the refs already
  imply, and firing the ordinary fork N times leaves N ops rows where one would
  leave one. `Prompt`
  carries one further gate, the §3.5 **spend ceiling** (bl-56d5): it is seated
  inside the shared `prompt` body rather than in the match, so the frame's
  typed door and the dispatch arm are gated once between them — it is the only
  gesture that births a drone, and a birth is the only thing a ceiling may
  refuse.
- **Queries** are I1 derivations: everything that populates. Their carrier is
  `boundary::Query` (`Workspaces`, `Conversations`, `Balls`, `Board`, `Ops`,
  `Search`, `WorkDiff`, `Help`, — bl-0164 — `ReadConfig`, `Marks`,
  `Providers`, and — bl-dff8 — `Lineages` and `Models`) and the chokepoint is
  `boundary::answer` — functions over the
  published snapshot (§7.2) plus the durable `ui.json`, *the snapshot
  derivation run without a frame*. Three of them read the world's **bytes**
  rather than this snapshot's derivations — `Search` (§8.5 below),
  `WorkDiff` (§5.1 #32, which asks the snapshot which balls a workspace
  claims and the project repos for the rest) and `Lineages` (§9.3's browse,
  which is the workspace's own git) — and all are answered straight
  through, because every seat that reaches the chokepoint is already
  off-frame. The §9 config family's reads (bl-0164, extended by bl-dff8 to the
  lineage browse and the §9.4 roster) read the world through the
  same `Deps` `dispatch` takes instead, exactly as their writes already do
  (§8.5 below), which is why `answer` returns a `Result` and can refuse as
  `dispatch` can. The frame's view-models delegate to these same functions
  (`AppModel::conversations`, `workspace_stats`, `conversation_names`,
  `delete_confirmation` are thin delegations), which is the parity mechanism:
  one implementation, two serializations.
- **Views** never cross it: they are exactly §5.3's closed RAM whitelist (I6)
  — focus, scroll, tab selection, drafts — **and the §4.1 presentation
  durables beside them** (pins, collapse, zoom, panel sizes, seen
  watermarks). The ruling's own line decides the durable case: "switching tab
  doesn't" — durability does not promote presentation state into an
  operation. Views gain no boundary representation, by design.

**The GUI is one serialization of this surface.** The shell's click-glue
constructs variants and calls the chokepoint (`AppModel::dispatch`, a thin
covered wrapper over `boundary::dispatch`); its serialization is the in-RAM
variant itself. **The headless serialization is the `boundary::codec` JSON
envelope** — one flat object, `op` the discriminant — and both codec
directions plus the dispatch match are exhaustive over the enums, so **a new
gesture without a headless spelling fails to compile**, never review.

**The transport is deposit-based** (I4 — the lernie inbox discipline applied
to yog itself). A gesture is a create-only file in
`<yog-state>/gestures/<id>.json`, delivered by atomic rename (dotfile temps
are never listed). The consumer claims a deposit by renaming it to
`gestures/claimed/<id>.json` — the rename is the mutual exclusion, so two
yog instances never double-run one gesture — dispatches it, and writes
`gestures/replies/<id>.json` (atomic; its existence is the done marker a
depositor polls). The audit is the deposit plus the `ops.jsonl` rows the
executors already write; the claimed file stays as that audit's other half. A
deposit with no reply has simply not converged yet (I0): the next running yog
consumes it. A crash between claim and reply leaves the claimed file naming
exactly what was in flight — re-deposit to re-run. I7 holds: a deposit *is*
the explicit user action; the consumer adds no spontaneous mutation.

**The id is claimed from the world, never guessed locally (bl-aa9f).** A
deposit's id is also its reply key, so two depositors holding one id is two
callers reading one answer — and yog's world is shared by processes that share
neither a clock nor a pid namespace, which is exactly what the old
locally-minted `<seconds>-<pid>` assumed (two containers, one world: one
depositor was refused as a duplicate, and a three-way run had `/workspaces`
print `/board`'s reply). Nothing a process computes alone is unique here, so
the arbiter is the one thing every depositor shares: the world's own
filesystem. `deposit::mint` **reserves the reply slot** —
`gestures/replies/<seed>-<n>.json`, created exclusively (`O_EXCL`), `n` rising
until one create wins — and the reservation it wins *is* the id. The reserved
file is empty, which the depositor's poll already reads as *not yet* (an
unparseable reply is not an answer), and it is never removed, so an id retired
by a claim or an answer stays spent: a later caller can never be handed a name
whose reply is someone else's. The seed — still the clock second and the pid —
buys legibility and time-ordering in the inbox listing, nothing more; ids now
read `<seconds>-<pid>-<n>`.

The consumer (`boundary::consumer`) is its own thread beside the derivation
worker — never the frame (§7.2) — polling the inbox (a latency knob;
correctness is the deposit's persistence, not the poll). **Both faces run
it**: the GUI window and `yog headless` — the same engine with no window, and
since bl-f6fe *literally* the same `Engine::boot` (see "One engine, two faces"
below), parked until a signal — so a deposit converges whichever face is up. `yog gesture '<json>'` is
deposit-and-wait sugar over exactly this path (validate against the codec,
deposit, poll, print the reply; exit 0/1 on the reply's verdict, 2 for an
envelope that never deposited, 124 when no consumer answered — the deposit
remains). The argv verb is a depositor, never a second dispatch
implementation (VISION §8).

**The line is the third serialization (bl-ec8f).** In pursuit of everything in
yog being teleoperable, and with TUI support in view, parity to the control
interface is implemented via slash commands. The two serializations above are both untypable — one needs a
pointer, the other a JSON writer — so an operator at a terminal, in a chat
window, or an agent whose tool is a text field had no spelling. The **line** is
that spelling: `/verb args`, read by `boundary::line::parse` into the same
`Gesture` the click-glue constructs, run by the same `dispatch`/`answer`
chokepoints. Three serializations, one surface, still never two
implementations.

- **Terse and context-bearing, where the envelope is total and context-free.**
  `/message ship it` says what the operator means and nothing about where: the
  workspace and the agent come from the seat's own selection, carried as
  `line::Context` (workspace, agent, project, the §3.2 `--as` name, the focused
  ball, the pending `Prepared`) — the same facts the click-glue resolves off the
  focus before it constructs a variant, derived once in `AppModel::line_context`
  so a typed verb and a clicked one cannot aim differently. **A parameter the
  line omits and the context cannot supply is a refusal naming it**, never a
  guess: a gesture is an instruction, and a guessed target mutates the wrong
  thing.
- **The verb table is the single source.** `boundary::help::TABLE` (verb, usage,
  summary, detail) is what refusals print, what a seat completes or helps from,
  what `/help` answers, and what the parity tests hold against the reader in
  both directions: every advertised verb reads, and every gesture the
  hand-enumerated round-trip table spells has a page. *(This retracts bl-ec8f's
  "there is no `/help` verb, because a verb that is not a gesture would cross no
  boundary". Help does cross it — see below — and the roster moved out of
  `line::ROSTER` with it.)*
- **The compile gate holds here too.** `line::spell` is exhaustive over
  `Gesture`, so a new variant does not build until it can be typed. It writes
  what the line elides as elided, so the round trip is *modulo context* —
  `parse(spell(g), ctx_of(g)) == g` — which is exactly the parity claim.
- **What the line never mutates.** A message's content and a prompt's goal are
  the whole tail, taken verbatim (§3.3, bl-6920), and admit no flags; everywhere
  else a `--flag value…` value is whitespace-normalized. A line is a line.
- **`//` says a slash and means it**, shed at the one place a draft becomes
  something a model reads.
- **The one thing a line cannot state** is an *existing* ball's spec: its title,
  body and §3.5 join are roster facts no text can carry, so `/prepare ball` reads
  the seat's selection, and a seat with no roster spells that one gesture as an
  envelope, which carries them in full. Everything else is typable at every seat.

**Its seats.** The composer (§11): a draft starting with `/` is a command, and
Enter runs it — **no new control** (bl-8aab), one re-labelled button, and the
answer rendered as the reply's own JSON (`reply::encode`), the same bytes the
deposit's reply file carries. The start family goes through the frame's typed
doors (`prepare_start`, `fire_prompt`) rather than the raw dispatch match — not
a second implementation, but the frame-only aftermath beside it: the §3.4
workspace adoption, the held start claim, the §3.3 mint seed a landed fire
spends. A headless consumer must not do those; a window must. And the terminal:
`yog gesture` takes a line as readily as an envelope, with `--ws / --agent /
--project / --as / --prepared` stating the context a seat with no selection
lacks — the line
is read, encoded by the codec, and deposited as the very envelope the JSON
spelling would have been, so the transport and the audit stay one path. A TUI is
then a seat like any other and needs nothing new.

**The terminal's seat has no memory, so the start flow's two steps compose
through the reply (bl-44d8).** `/prepare` returns a `Prepared` and `/prompt`
fires it; a window holds it in the composer between the two, but every `yog
gesture` is its own process, so `Context.prepared` was invocation-local and
argv had no spelling for it — which made `/prompt`, the only next step the
`prepare` page advertises, refuse forever at a terminal or a headless
controller. `--prepared '<json>'` is that spelling, and it is the fifth seat
flag rather than a new mechanism: the seat states what it elides, exactly as
`--ws` does. What it takes is the `prepare` reply's **own `prepared` object**,
handed back unchanged (`--prepared "$(… | jq -c .prepared)"`), read by the very
`codec::decode_prepared` that wrote it — which is the property `reply.rs`
already claimed ("the `prepare` reply's `prepared` body deliberately re-enters
as the next `Prompt` gesture and shares its codec spelling") and had no seat to
exercise. Nothing is stored: a prepared start gains no durable home and no
per-workspace singleton, because the caller carries the token. Anything that is
not a whole `Prepared` refuses at the depositor (exit 2), never a guess.

**Help is a query, and a higher-order one (bl-0ec2).** Every command carries a
help option, top level included. Help is a higher-order operation, and it
threads the same way through all of the interfaces.

Help sits in the §4.8 taxonomy exactly where the others do — it populates
rather than mutates, it answers typed data both frontends render, it has a
headless spelling — so it is a **query**: `Query::Help { verb: Option<String> }`
→ `Reply::Help(Vec<HelpRow>)`, answered by the one `boundary::answer`. What
distinguishes it is its *subject*: **the derivation is over the interface, not
the world**. Two consequences, and they are the whole design:

- **It is asked *about* a command, so it is never a command's parameter.** The
  line reader reads it once, above the verb match (`line::parse`'s `asks_help`):
  `/help`, `/help <verb>`, `<verb> --help|-h`, and a bare `/` are one gesture,
  not eighteen arms. The flag form is recognized **only when the tail is exactly
  the flag**, which is what keeps the two verbatim payloads whole — `/message
  --help` asks about `message`, `/message run --help on it` is a message. The
  argv seat threads it the same way: `--help` *rewrites the invocation* into the
  help line (`--help close` is `/help close`), rather than becoming a flag
  `yog gesture` itself parses.
- **It is the one query with no world to read, so every seat answers it in
  place.** `yog gesture --help` prints and exits **0** without depositing,
  without a consumer, and without the 124 wait: asking what a verb does is an
  answer, not a refusal, and must not depend on a yog being up. The deposit path
  still answers it (parity is not optional, and `{"op":"help"}` is a valid
  envelope) — nothing is *obliged* to go that way.

Its seats, and the rule they share — *every command answers `--help`*: the
composer renders the reply as help rather than as reply JSON (one row is a
page, many are a roster — the shape follows the question); the terminal prints
the same rendering to stdout; and the **top level** answers `yog --help|-h|help`
with the whole surface — the window's own flags (rendered by clap, never
restated), then the windowless face, both §8.4 hatches, `gesture`, and the
§16.7 namespaces, each named by the const its dispatcher routes on and its
column measured rather than chosen. The two balls plugin binaries are
deliberately unlisted: balls' plugin chain spawns them and no operator types
one.

**The argv seat reads help above the router (bl-52ed).** The rule is the line
reader's, lifted one level: help is asked *about* a command, so `multiplex`
answers it once — `yog --help|-h|help [command]` and `yog <command> --help|-h`
are the same question — **before** `main.rs` reaches its own subcommand match.
That ordering is the whole fix: `yog env --help` printed export lines, `yog
exec --help` exited 127 trying to spawn a program called `--help`, `yog
headless --help` booted the engine and parked, and `yog tool-control --help`
waited on stdin, each because the command ran before anything read the flag.
`multiplex::help::COMMANDS` is the single source — the roster, every page, and
the anti-drift test all read it, and a row with an empty summary is answerable
but unadvertised (`tool-control`, a machine seam).

A §16.7 **namespace** is the one thing yog does not answer over: its argv
belongs to the embedded tool, so `yog bl --help` is balls' page and `yog bz
--help` is brazen's. What the ask must not do is reach that page *through the
world*, so both arms step aside for a **discovery probe** — an argv that is
exactly `--help`/`-h`/`--version`/`-V`/`--skill`, the same narrowness the line
reader uses (the flag form counts only when the tail is exactly the flag, so no
foreign crate's option grammar has to be restated to know a token is not
somebody's value). For `bl` that skips the world's shim converge, which used to
write six shims under a fresh world root and to fail outright before help on a
read-only one; for `bz` it is the wall gate's one exemption (§16.2), lawful
because brazen emits a probe before it reads a config file or touches a seam —
so the paths handed in are unread, and they name a root that cannot exist so
that stays true rather than merely intended. Every other `bz` route still
refuses without a wall: credentials never fall back ambiently. Strictness is at the edge, as everywhere else — the codec refuses
`{"op":"help","verb":"enhance"}` by name, so the answer is total and no seat
ever renders an empty page.

**Search is a query, and the one that is asynchronous (bl-3c28).** Source:
bl-e249's Claude Code comparison — yog could inspect any fact once selected but
could not *retrieve* one, so a ball, a conversation, a goal or an old
transcript line had to be found by remembering where it was.

It sits in the §4.8 taxonomy where the others do: it populates, it answers
typed data both frontends render, it has a headless spelling —
`Query::Search { text }` → `Reply::Search(Found)`, answered by the one
`boundary::answer`. Four things decide its shape, and they are the design:

- **Its subject is the world's bytes; the snapshot only says where they are.**
  The published derivation supplies the addresses — every ball of every listed
  project (live *and* closed, from the same on-demand cache §3.5's Delivered
  rows read, now published beside the live one), every enumerated workspace,
  every derived conversation — and the goal and transcript of each conversation
  are **re-read at ask time**. That is the whole no-index discipline: there is
  no second store to drift, so a hit is a statement about the file as it is
  now. Nothing is written, nothing is cached, nothing is mutated (I1, I7).
- **One hit per address, and the addresses are the ones that already exist.**
  A result is a ball `(project, id)`, a workspace, or a conversation
  `(workspace, agent)` — never a coordinate invented for search — and opening
  one is the selection a click on that thing makes (`AppModel::open` →
  `focus_workspace` / `focus_agent` + the Transcript tab). A conversation whose
  transcript matches forty times is **one row**, because one row is what you
  open; that is also the bound, since the hit count cannot exceed the world's
  subject count, capped at `search::MAX`. A ball no workspace holds moves
  nothing when opened — §3.5 selects a ball *through* its workspace, and a
  ball-shaped selection invented for this one case would be a second navigation
  model.
- **Ranking is a total order over facts.** The matched field's tier first —
  `Name` (a ball id, a workspace name, a conversation name or agent id) before
  `Summary` (a ball title, a conversation's goal) before `Text` (a ball body, a
  transcript entry) — then the byte offset, then the address. No score and no
  tie broken by chance, so two runs over one world answer identically. Case
  folding is **ASCII**: a Unicode fold can change a string's byte length, which
  would make the reported offset lie about the bytes it points into.
- **Unreadable sources are named, never dropped.** A workspace whose tree
  failed to derive, a project whose balls are unlistable (§3.5's orphan row), a
  goal file that exists and will not read — each rides back in the reply's
  `unreadable` list beside the rows. A broken corner shrinks the corpus; it
  does not make the world unsearchable, and it is never silent.

**The window is the only seat that does not run it in place.** A search walks
every transcript in the world, and the frame derives nothing (§7.2) — so the
GUI *asks*, through a `SearchCell` in `src/state.rs`, and renders whatever
answer has landed, exactly as it renders whatever snapshot has landed. The
searcher is its own thread beside the derivation worker (a long search must not
delay a re-derivation: staleness is what the §7.3 banner is for, and a search is
not a wound). The cell's serial is the whole protocol: every ask bumps it, a run
carries the seat it started on, publishes only if that is still current, and
**abandons between conversations** when it is not — which is the cancellation.
`yog gesture` and the deposit consumer are already off-frame and simply run the
same `search::run`; one engine, three seats, no second implementation.

Its spellings: `Ctrl+F` at the window puts the composer on a `/search ` line
(no new control, and no "search mode" to enter — the results pane is a view of
the published answer, so it appears when there is one and goes when a `/search`
with no text clears it; its seat is a **center tab focus** since the
overlay ruling — §11, bl-1ca2. The ruling's premise was that it covered
everything; what it actually did was grow out of the composer and push the
conversation off its own pane, which is the same defect in a smaller frame and
takes the same fix. Asking focuses the tab, and the answer clearing retires
it); `/search <text…>` at any seat; `{"op":"search","text":…}`
as the envelope. An empty search is **not** a refusal — an empty query matches
nothing, which is the general path with no input and how a seat clears its last
answer without a second verb to do it with.

**The config family, landed (bl-3f46).** The §9 config editors, the §16.3 marks
knob and the §9.4 model pick were actions by this taxonomy without being
variants — each funnelling through its own chokepoint (`Editor::apply`,
`edit::drive`, `world::marks::apply`) — and are now three variants beside the
rest, spelled by all three serializations and executed by `boundary::config`:

- **`ApplyConfig { file, text }`** — *a config apply is a destination plus the
  full staged text.* `ConfigFile` enumerates where the bytes land (brazen's
  `config.toml` **in a named workspace's wall**, lernie's `models.yaml`, one
  `workflows/<name>.yaml`, yog's `cadence.yaml`, or one file on a per-workspace
  lineage) and the destination
  **decides the pipeline**: `bz` validates a brazen draft (§9.1), brazen's
  provider table gates a lernie-global one (§9.2), `lernie config` commits a
  lineage (§9.3). One variant, not four gestures — and no branch on which file,
  because a workflow and `cadence.yaml` declare no `models:` block and are
  therefore clean under the same gate, the general path with nothing to judge.
  Line: `/config brazen|models|cadence <text…>`, `/config workflow <name>
  <text…>`, `/config branch|orphan <lineage> <path> <text…>`, `/config fork
  <lineage> <source> <path> <text…>` — the destination is *words* and the tail
  is the file, verbatim, because a config file's whitespace is the file. That is
  also why the lineage modes are sibling words and not flags: a `--from` after
  the text would be read as part of it.
- **`SetMarks { workspace, branch }`** — §16.3's branch-binding write path, as
  re-keyed by the per-agent ruling (bl-e47b): it points **one
  agent's own balls space** at a branch, by writing `tasks_branch` in balls'
  own §4 layer-2 config inside that space. Its reply is the branch **re-read
  afterwards** — never an echo — beside the space it is a branch of, because
  "which branch" and "whose branch" are one question. It answers
  `Action::project()` with **`None`**: it moves a clone bundle the §3.5
  projection does not read, so no board row can change because of it (it
  answered a project while the knob was a project's). Line: `/marks <branch>`
  — one word, and it is the value; `balls/tasks` says "the project's board"
  outright, so no mode word survives to disagree with the name beside it. An
  unlawful branch (whitespace, a quote, or balls' own `balls/config` landing)
  refuses at the line and again at the write, in one wording.
- **`PickModel { workspace, role, provider, model }`** — §9.4, which is §9.2 and
  §9.3 *composed by one gesture*; that composition is precisely why it cannot
  decompose into the two above without a caller re-implementing the plan. Line:
  `/model <role> <provider> <model-id>`.

**No headless gate rides "empty table gates nothing."** The rows are brazen's
fact, so `dispatch::Deps::provider_rows()` **asks** the linked brazen through
the composed world at the moment of use rather than carrying a stored copy —
one home, no drift, and both faces gate identically. The consumer's interim
(leave the table empty because brazen was unasked) is retired. bl-3f46 wrote
this for the §9.2 workspace-birth gate as well; that gate is gone (§9.2,
bl-00ee) and the seat-side plumbing that handed the GUI's cached rows into
`prepare_start` went with it, so what asks today is the §9 config family, each
against the wall of the workspace whose file it judges.

**Two frame-only entries remain, and they are the §8.1 pattern, not exceptions.**
The three *file* editors' Apply buttons (§9.1/§9.2 and the `cadence.yaml` pane)
stay on their own `Editor`/`BrazenEditor`, because a pane holds a **long-lived
RAM draft** with a load-time snapshot and the §9 hash guard is over that draft:
it refuses when the file moved under the operator. A deposit has no such draft —
it states its whole text in one atomic instruction, so load and apply are
microseconds apart and the guard degenerates to the must-not-exist check a new
file wants. Both enter the same §9 pipeline, exactly as `prepare_start` /
`fire_prompt` enter the same chokepoint the `Prepare`/`Prompt` arms do: one
implementation, two entries, the frame-only state beside it. Every other config
seat — the lineage Send, the marks buttons, the picker's selection — now
constructs a variant and calls `AppModel::dispatch`.

**§8.3 login stays as it is.** Its headless spelling already exists verbatim as
`yog bz --login` (§16.7 W10), so a variant would be a second spelling of a
gesture that already has one.

**The config family's reads, spelled (bl-0164).** bl-3f46 landed the family's
*writes* and left the reads that populate the same panes unsaid — a headless
seat could SET the §16.3 marks knob and could not ASK what it was; could
APPLY a brazen config and could not READ the one on disk. Three real,
on-demand gestures had a chokepoint and no boundary spelling: the marks
pane's `Read current` (drives `bl conf`, DESIGN §11's config-mode bullet —
"opening it is the gesture that reads everything the pane renders"); the §9.1
brazen and §9.2 lernie-global editors' `Reload`, the hash-guard's own re-diff
of the file under the draft; and the §8.3 login pane's `↻ providers +
credentials`. Each now joins `Query` beside `Workspaces`/`Balls`/`Help`, with
all three serializations and a help page — the pattern bl-3c28's search
landed at (`Query::Search { text }` → `Reply::Search(…)`, one variant, no
per-pane family):

- **`ReadConfig { file: ConfigFile }`** — the same destination enum
  `ApplyConfig` already carries, so a read and its write name the place the
  same way; the destination decides nothing else, because reading is just
  the bytes currently there. A destination not there yet answers empty text
  — the "new file" reading every §9 editor's own `load` already gives, so
  only a real I/O failure refuses. The one destination this refuses outright
  is `ConfigFile::Branch`: which files a config commit holds is the §9.3
  pane's own browse (`git show` over every file in a lineage at once,
  bl-ee0a), a fact one destination cannot carry, so that read stays where
  bl-ee0a put it rather than half-answering it here.
- **`Marks { workspace }`** — which branch this agent tracks on, over the same
  space `Action::SetMarks` re-reads after it writes. It **never refuses and
  never spawns** (bl-e47b): the value's one home is the space's own balls
  config, so a workspace bound to no project — the
  launched-then-told-to-work-on-a-project case §16.3 exists for — answers
  exactly as any other does, with balls' default branch when nothing is
  written.
- **`Providers { workspace }`** — that workspace's effective provider table
  with the §5.1 #22 credential presence, the exact rows `LoginHolder::ask`
  already renders — asked fresh each time (never stored), matching
  `Deps::provider_rows()`'s own "asked, never stored" contract just above.

**One verb, not two families.** Rather than a `ReadConfig`/`ReadMarks` pair
of new spellings beside `ApplyConfig`/`SetMarks`, the read rides the *same*
`/config` and `/marks` verbs the writes already have, discriminated by the
one field that makes a gesture a write: `/config <destination…>` with
nothing after the destination's words reads it (`ApplyConfig` already
refused that shape — "the file's text is required" — so the read costs no
case a write used); `/marks` bare reads the branch (a branch is always
required to set one, so the empty tail cannot mean anything else). The
envelope mirrors it exactly: `{"op":"config","target":…}` with no `text` key
is the read, `{"op":"marks","workspace":…}` with no `branch` key is the read —
one op each, not two, and a line and a deposit agree on what "just the
destination" means because they read the same discriminant. `Providers` has
no write to share a verb with (§8.3's login flow is not itself a variant), so
it is `/providers` / `{"op":"providers","workspace":…}` — a noun, but a
workspace-scoped one (below), not a global like `/balls`.

**A wall-scoped gesture names its workspace (bl-fcd5).** The
blast-radius ruling moved brazen's three locations inside a workspace's wall
(§16.2), which silently made two gestures unaddressable: `ConfigFile::Brazen`
and `Query::Providers` named no sphere, so the executor could only read
whichever wall happened to stand in `Deps::world` — the window's focus. A
headless seat has no focus, so `yog gesture '/config brazen'` refused with *"no
focused workspace"* and `/providers` answered an empty table, and a
teleoperator (VISION V5) could not reach provider config at all. Both now carry
a `workspace`, exactly as `ConfigFile::Branch` and `Action::PickModel` already
did, and **the gesture's own workspace is the single source**: every brazen
fold in `boundary::config` lenses `deps.world` through `wall::env` on that
name rather than trusting a standing `YOG_WALL`, so a windowless seat reaches
exactly the sphere it named and a window reaches exactly the one it focused —
one derivation, and the `--ws` flag a terminal already states every other
elided target with is what states this one.

Three consequences worth stating. **The refusal moved to the edge and stayed a
refusal**: a line with no `--ws` and no focus refuses in `args::workspace`'s own
words (*"no workspace in context — focus one, or use the envelope"*) and an
envelope without the field refuses in the codec, so nothing reaches an executor
that would have to guess a wall — which is why `boundary::config::brazen_paths`
is now infallible and the old `NO_WALL` executor refusal is gone. **The line
spelling is unchanged** — `/config brazen` and `/providers` still read as they
did, because the workspace rides the seat's context flags like `/marks`' and
`/conversations`' do, never twice in one line. And **§9.4's provider gate
follows the pick**: a row that is dead in one workspace may be live in another,
so `pick_model` judges against the wall of the workspace it is picking for
rather than the seat's — judging with the wrong wall would refuse a valid pick,
and judging headless with no wall at all would gate on an empty table and let
anything through.

**The marks pane's `Read current` is now the boundary too** — the click
constructs `Query::Marks` and calls `AppModel::answer`, the same chokepoint
`/marks` and a deposit reach, so the knob's read and write share one
implementation exactly as its write already did. The two file editors'
`Reload` and the login pane's `↻` are not rewired: both already call the same
pure derivations (`load_snapshot`, `row_views` + `credential_presence`)
`config::read`/`config::providers` now also call from their own `Deps`-scoped
site — two call sites over one fact with nothing to drift, the same shape
`LoginHolder::ask` and `Deps::provider_rows()` already were before this ball,
not a second implementation this ball introduces.

**The decision queue, and the one reclassification it forced (bl-f6fe).**
VISION §5 V5.2, verbatim: *"The admin surface is the attention strip made
addressable. Escalations and decisions-needed form a queue an agent can read,
answer, or forward; answering headlessly writes the same watermarks the GUI
writes, and I0 guarantees the two frontends converge over one disk."* Two
variants carry it, and everything else it needs already existed:

- **`Query::Attention` → `Reply::Attention(Vec<QueueRow>)`** (`/attention`,
  `{"op":"attention"}`) — every §6 attention-bearing agent across every
  enumerated workspace, **in §6's attention-ranked roster order** — which the ↓
  key walked until bl-fa82 moved that key onto §11's visible list rows; the
  queue and the jump still share this one build, each row
  carrying its address (`workspace` + `agent`, spelled with the keys the
  gestures take), its display name, its state, which signals fire, what it last
  said, its age and its pending-mail count. Per **agent**, not per conversation
  root: the strip counts agents and the ↓ key lands on agents, so a child that
  raised its hand is a row of its own. It is not a second model of "what needs
  you" — `answer::queue::roster` is the flattened roster `app::focus` steps
  through, and the queue is its `attention` subsequence, which is why its length
  is `attention::strip_total`'s own number.
- **`Action::MarkSeen` → the same `Reply::Attention`** (`/seen`,
  `{"op":"seen",…}`) — the **answer**. It records the conversation's present
  evidence oids as seen, from the one definition (`attention::evidence`) the
  window's focus tick also reads, so a headless acknowledgement and a windowed
  one are the same bytes in `ui.json` and I0 converges the two frontends over
  one disk. Its answer is **the queue that remains** — the `SetMarks`
  precedent (state re-read, never an echo of the write), and it makes the
  teleoperator's loop one gesture per decision rather than a read after every
  write. It refuses a conversation the published snapshot does not carry.

**Why this is an action when §4.8 files seen watermarks under views.** The
views clause above stands where it was written — *at the window, the watermark
rides on focus*, and focusing is looking, not doing; the ruling's own "switching
tab doesn't" governs it. **A seat with no focus has no looking to ride on.**
There, acknowledging is not the byproduct of anything: it is the operator
stating *I have handled this*, and it changes what every other seat is told
needs attention. That does something, so it crosses. The window keeps its
focus-tick entry and gains no widget (V5's burden check: "headless mode adds no
widget anywhere"); the two entries share one implementation, exactly as
`prepare_start`/`fire_prompt` share the `Prepare`/`Prompt` bodies.

**Forwarding needs no verb.** "Read, answer, or forward" is three things to do
with a row and two gestures: forwarding an escalation is `Action::Message`
aimed at somebody else, carrying the row's own text. A `/forward` would be a
second spelling of a gesture that already exists (the §8.3 login precedent).

**What it deliberately does not cover.** The queue's rows are the §6 signals and
nothing else. Capability holds — a drone parked at a tool call awaiting a
verdict — are **bl-765d**'s `PendingHolds`, whose facts come from
`refs/lernie/held/*` rather than from this predicate; when they land they extend
this query's signal set rather than opening a second queue beside it.

**One engine, two faces (bl-f6fe).** V5.1 is verbatim *"headless mode is the
same binary, minus the window"*, and V5.4 *"nothing here is a second
implementation"*. `src/engine.rs` is where that stops being an intention:
`Engine::boot` is the whole of a running yog minus its face — the §5.2 startup
sweep, the §7.1 roots, the model's first synchronous derivation, and the
derivation worker, watch bridge and gesture consumer spawned beside it — and the
two faces are two calls to it with different repaint hooks (`EguiRepaint` /
`NoRepaint`). Before it, `main.rs` carried that assembly **twice**, in the one
file `tarpaulin.toml` excludes, so the copies were free to drift and no test
could notice; `src/engine/tests.rs` now boots a windowless engine into a
hermetic world and reads a deposit's answer back, which is the V5 claim
end to end. What a *window* adds beside the engine is exactly what a window is:
an event loop to wake, the §8.5 searcher (the windowless face needs none — every
headless seat answers a search off-frame already), and the §5.3 RAM surfaces a
pointer needs.

**What V5 deferred, and where it landed.** V5.1 names *"the backend loop, the
clock, and the dispatch surface"*. The clock is `cadence.yaml` (§7.2) and the
dispatch surface is this section — both real at V5's landing. The **fleet loop
was not**: bl-9dd4 established that commit a94ce81 shipped only the watcher
clock, and that no arming gesture, cap, spawn or reap existed anywhere in
`src/`, so building one under a teleoperation rung would have been capability
theater (VISION §8). It landed on its own rung when its own preconditions closed
— **bl-66fb**, `src/fleet/` — and it landed *off*: the loop is armed per
workspace by the `fleet` / `disband` gestures below, and an unarmed world is
exactly the world V5 shipped into.

Every **new** gesture lands as a variant first — the compile gate above is the
enforcement.

### 8.6 The capability control (VISION §4.11, bl-0cea)

The ruling lives in VISION §4.11; this section is yog's side of it. The
enforcement point is **lernie's own tool-control seam** (lernie ARCH §3.3
*Tool control*, bl-de6d — in the 0.0.6 pin): `workflow.yaml`'s
`tool_control:` names one executable consulted before every granted tool
invocation executes, answering `pass` / `refuse` / `hold` on stdout for the
`tool_use` + `role` + `agent_id` JSON on stdin, failing closed. yog's job is
to *be* that executable and to own every fact it reads.

- **The control is a world-tools shim** (§16.4's re-exec pattern, seated
  beside `bl`/`lernie`/`bz`): the yog binary in a consult mode, addressed by
  **absolute path** in the authored block — never PATH-resolved, so no host
  binary can shadow it. It is **side-effect-free per consult** (the seam
  demands idempotence — release is re-adjudication): it writes nothing, ever.
  Two moves per consult: **classify** the invocation into VISION §4.11's
  effect vocabulary (intrinsic class map for built-ins; the workspace
  ruleset over `bash` commands, unmatched = open-world; `cd`/`apply_patch`
  judged against the writable root — the bound attempt worktree plus the
  agent worktree, the agent's cwd read from lernie's
  `refs/lernie/cwd/<agent-id>` mark, the ball worktree computed by the
  §8.1 bl-delivery formula, never stored); then **judge** by the class →
  verdict table folded with per-conversation floors and per-`tool_use`-id
  once-answers. **The root is derived from facts yog owns, never from one the
  agent controls** (bl-fec8): the cwd mark is read to *interpret* relative
  operands and never to widen the root — an agent that could widen its root
  by `cd`-ing would have no root — and the bound worktree comes from the
  claimant join over yog's own `bl claim` ops rows (§3.2's claimant equality,
  §8.1's formula), so no store read and no subprocess is needed. A ball an
  agent claimed for itself mid-conversation leaves no yog-side row (§3.2
  states that limit), so writing there classifies open-world — which the
  shipped table passes, and which a workspace override or a raised floor is
  what turns into a park. Containment is **lexical** — `..`
  folded textually, `~` expanded, symlinks unresolved — because a
  `canonicalize` per operand per tool call would touch disk and answer
  nothing for the paths that matter most (files a patch is about to create);
  symlink escape is out of the threat model by construction (VISION §4.11
  item 8).
- **Fact homes, one each.** The *request* is lernie's hold mark
  (`refs/lernie/held/<agent-id>`), which unlike the other four `refs/lernie/*`
  marks carries a **value**: a blob naming the held `tool_use` id, the tool,
  and the control's reason. *Standing policy* (class→verdict table, bash
  ruleset, the optional confinement-required flag) is per-workspace
  config read at its **live tip** — the control acts for the operator, so
  revocation binds at the next consult, never frozen at the governing
  commit. Absence is the shipped defaults (read / target write / process /
  open-world → pass; destructive / secret → refuse — bl-1ef1: everything
  passes but loss and credentials) — the
  `cadence.yaml` severability pattern. **Nothing the shipped table says is a
  hold**: a park is *imposed*, by a workspace's `table:` row or by a raised
  floor, never by standing policy. A shipped open-world hold made the operator
  answer for every `python` and every fetch — approvals given by reflex, which
  is the failure mode a gate is supposed to avoid — so the safety story after
  bl-1ef1 is the floor: the monitor aims a park at the conversation that earned
  it instead of at all of them. Severability still runs the right way, absence
  being the (now permissive) default and the file the override. *Answers*
  (approve/deny once or as a rule, revoke-auto-approval floors) are boundary
  actions whose `ops.jsonl` rows are both the audit and the fold's memory — no
  new durable artifact, I2 holds at three.
- **Standing policy is one file, and it is an override** (bl-765d):
  `capability.yaml` beside `workflow.yaml` on `config/default`, four keys —
  `confinement:`, `table:` (class → verdict), `rules:` (`<program>
  [qualifying words…]` → effect class, matched *before* the shipped rows) and
  `secrets:` (extra credential-adjacent path fragments, additive only; a
  workspace may widen what counts as a secret, never narrow it). It is read
  from git on every consult, hand-parsed line by line — no YAML dependency and
  no dependency at all — and every line the grammar does not recognise
  contributes nothing rather than failing a consult closed.

  **The shipped ruleset is deliberately not seeded into it.** Materializing the
  ninety default rows as config would make `rm capability.yaml` mean *no rules*,
  which inverts the ruling's own severability claim ("absence IS the shipped
  safe defaults"). Defaults stay code consts; the file is the delta. What stops
  that being blind is §9.5's own answer — the effective policy is readable
  beside the file — not a copy of the defaults in it.

  **It needs no write path.** A config lineage file already has exactly one
  lawful writer, the §9.3 `lernie config` drive, reached by the existing
  `ApplyConfig` gesture on a `Branch` destination (`/config branch default
  capability.yaml …`). The capability boundary therefore adds a *reader* and
  no writer at all, which is what keeps its whole surface one action wide.
- **Authoring rides an existing write class — but not the one the ruling
  named (bl-fec8, verified against the pin).** The ruling authored the block
  into `LERNIE_HOME/template/workflow.yaml`; three premises behind that turn
  out to be false. `lernie prime` never seeds `template/` — it is an
  *override* root, absent by default ("policy lives in config, not code", at
  lernie's own constant). The override is a whole-file `fs::copy`, not a
  merge, so a `workflow.yaml` carrying only `tool_control:` would delete
  `events:` — and with it every dispatch — from every workspace born after
  it. And authoring a *complete* override needs lernie's embedded default,
  whose module is private: there is no lawful read of it. **So the control is
  authored per workspace, onto `config/default`, at every start**, through
  the one lawful writer of `config/*` — the §9.3 scripted-editor `lernie
  config` drive — with the base taken from the workspace's own committed
  `workflow.yaml`, which is exactly what lernie put there. yog still never
  writes inside a workspace. This is *stronger* than the template route
  rather than a retreat from it: the template would only have reached
  workspaces born after it, while this reaches every workspace on its next
  start, and every agent forked after that commit is controlled. Agents
  already running keep the policy they froze — that is lernie's per-branch
  freeze, not a gap authoring could close. Convergence is by comparison, not
  memory: the authoring is a fixed point, so a tip that already names the
  shim reads one file out of git and spawns nothing. A drive that fails
  aborts the start (`["yog-step","control"]`, Z5) rather than handing back a
  workspace whose drones nothing adjudicates.
- **A hold is a parked drone, not a deadlock; a deny is a decline, not a
  stop.** The parked branch is derived state (the mark plus the unpaired
  tail) surfaced as an attention item naming the tool, an input summary,
  the computed class, and the reason; the ball renders waiting at its gate.
  Answering is a boundary action (§8.5 variant, headless spelling by the
  codec) that writes its ops row and fires `lernie advance` — the explicit
  user action continuing (I7); lernie re-adjudicates and the branch moves.
  **No enforcement path calls stop**: stop mid-tool-window wedges the
  branch permanently (lernie bl-b98d), so refusal is always in-band or a
  park. No modal exists in either frontend — attended and unattended are
  one flow, and attendance is latency.
- **The park is §6's sixth signal, and it gets no query of its own**
  (bl-765d). §6 already *is* "what needs you", and the §5 V5.2 decision queue
  is that predicate made addressable; a `PendingHolds` read would be a second
  model of the same question, disagreeing with the first the first time one of
  them changed. So the hold rides the existing machinery: `Agent::held` is read
  once per tick beside the other four marks, `attention::held` fires on it, and
  the queue row carries the parked invocation. One consequence is a rule: the
  signal is **not seen-gated**. Every other mark-derived signal can be
  acknowledged, because acknowledging says *I have seen what it said*; a park
  says nothing — it waits — and a watermark over it would hide a conversation
  that no acknowledgement can move. Mail is the precedent (§6 rule 5); this is
  the same shape with a harder reason, and `attention::evidence` therefore
  names neither.
- **The answer derives the id it is scoped to.** The gesture carries
  `(workspace, agent, ruling)` and nothing else: the held `tool_use` id is read
  off the mark at fire time, not typed and not carried from a snapshot. Three
  things follow. A headless caller cannot quote a stale id. The operator
  answers the thing they are looking at rather than a token. And "derive, never
  store" holds at the boundary as it does everywhere else — the id has one home,
  which is lernie's mark. `pass` and `refuse` both **release** (one executes,
  one declines in band) and so fire the detached `advance`; `hold` is the
  operator saying *stay parked* — a real third answer, because a once-answer
  outranks the table and therefore survives a later policy edit that would have
  passed the call — and it launches nothing, since driving a branch to re-park
  it spends a process to reach the state it is already in.
- **The floor is the same fold written the other way** (bl-94b4, VISION §4.9's
  fifth rung and §4.11 item 7). `Floor { workspace, agent, raised }` appends one
  row — `["yog-control","floor",<conversation id>,"raise"|"lower"]` — and the
  fold's latest row wins, so revoking and restoring are one gesture in two
  directions with no order to get right. Three spellings: `/revoke` and
  `/restore` on the line (aimed by the seat, exactly as `/answer` is), `revoke`
  and `restore` as envelope ops, the variant itself in RAM. **Two ops for one
  variant**, the arm/disarm shape: raising and lowering are two instructions,
  never one read out of an absence.

  Under a raised floor every class above `read` adjudicates to at least a hold,
  so the drone keeps reading, keeps its branch and keeps its history while
  everything it reaches for waits on an operator. The floor **raises and never
  lowers**: a refusal stays a refusal, and a once-answer still outranks it, so
  a floored conversation can still be walked through one call at a time with
  `/answer pass`. It matches by descent prefix, so it stands over the named
  conversation and its whole subtree, children not yet born included — which is
  also why the writer refuses nothing for an id it cannot see: a pre-emptive
  floor is the mechanism working.

  It **writes one row and launches nothing**, in either direction. An answer is
  about one invocation that is waiting, so it drives; a floor is standing
  policy, and §4.11 item 6 binds policy at the *next* consult. A restore that
  also drove would spend a process on a conversation that may not be parked, and
  the branch a floor did park is released by the answer gesture that already
  exists. It carries **no reason**, either: the reason is the row before it on
  the trail — the monitor's own verdict line, or a flag — and the trail is read
  in order. Its receipt is **re-derived, not echoed** (the `marks` precedent):
  the reply says whether a floor *stands* now, so restoring a child under a
  still-floored parent answers "floored" instead of claiming a restore it did
  not make.

  **The rung is a verb, not a behavior.** Nothing here fires it: the §4.9
  monitor ships flag-only by ruling, and this adds a rung the operator may wire
  — a tier-0 direct wiring, or a responder's grant (tool selection is ladder
  selection). Both are config; neither is code, and neither is on by default.
  And the division of labor stays where §4.11 item 7 put it: a verdict is one
  *input* to the capability policy. The monitor rules whether work serves the
  goal; the boundary rules what an agent may ever do. That is why this is a
  capability action beside the answer rather than an arm of the monitor family
  — §4.9's ladder spends existing verbs from the families that own them.
- **The confinement refusal is a birth gate, at the two doors a drone is born
  through** (VISION §4.11 item 8): `dispatch::prompt` (every start, above the
  §3.5 spend ceiling — a birth that will not happen has no spend to judge) and
  the `Fork` arm (every attempt). yog wires no confinement layer today, so a
  workspace declaring `confinement: required` refuses every birth, by name.
  That is the point: never a silent fallback. It is also why the missing layer
  gets no affordance anywhere — the only surface an absent capability earns is
  the refusal that names it. V4's armed loop (bl-66fb) reads this same gate.
- **What this is not.** Not confinement: the ambient PATH, network, and the
  deliberately-shared brazen credentials (§16.2) stay reachable by an
  *allowed* invocation, and rule classification bounds accident, not
  adversarial evasion — VISION §4.11 item 8 carries the honest threat
  model, the alignment monitor covers drift, and the OS layer (lernie's
  reserved v1.1 sandbox seam) is later and platform-explicit, with
  confinement-required workspaces refusing to arm where it is absent.

Shipped-state landed with bl-fec8 (shim, classifier, fold, authoring), bl-765d
(policy config, the hold-answer variant, the sixth attention signal, the
confinement refusal) and bl-94b4 (the floor writer above — the monitor's revoke
rung over the same fold, whose *reader* `Answers::floored` came with the shim).

---

## 9. Config editing write paths

One shared discipline for all three editors: **load → edit in RAM buffer
(carve-out) → Apply = stage → validate (where a validator exists) → hash-guard
→ atomic rename.** The hash guard is the concurrent-edit discipline: Apply
refuses if the on-disk content no longer matches what was loaded into the
buffer (another instance, or vi, wrote meanwhile); the user reloads, re-diffs,
re-applies. Rejected: blind LWW on operator-authored config — silently
discarding a concurrent edit. **The load and the reload both have a boundary
spelling now** (§8.5, bl-0164): `Query::ReadConfig` answers with the same
bytes a `load`/`reload` would read into the buffer, so a headless seat can see
what an Apply would be diffed against before it ever sends one — **every
destination, lineages included** (bl-dff8: a lineage answers `git show
config/<lineage>:<path>`, and `Query::Lineages` is the browse that says which
paths there are to ask for; §9.3).

**Freshness is a read on demand, not a watch (bl-9130).** These files carry no
watch root — §7.1 records why — so the buffer is kept honest by re-reading at
the moment the operator looks: **opening the Config pane refreshes every editor
whose draft is pristine** (still byte-identical to what was loaded). One rule,
no special cases: a pristine editor follows disk, and an editor with unsaved
edits is left exactly as typed, because adopting under it is the blind LWW
above. That editor learns at Apply, from the hash guard, which is the same
answer it would have given anyway — a watcher could only have made the refusal
earlier, never avoided it.

### 9.1 brazen `config.toml`

**§9.5 amendment.** The pane around this editor is now controls over facts: the
effective provider table (row name, `auth` — which is the login capability,
§8.3 — and credential presence) is rendered as read-only rows, and the raw TOML
draft is folded behind them. The editor below is unchanged and stays raw for the
reason this section already gives, restated as §9.5's first justified fallback.
The rows are what stop it being *blind*: they are the facts the file produces,
beside the file.

- Path: `<wall>/brazen/config.toml` — the focused workspace's own (§16.2's
  wall layout). brazen's ambient fold (`$BRAZEN_CONFIG` else
  `$XDG_CONFIG_HOME/brazen/config.toml` else `~/.config/…`) is **not**
  reproduced any more and not consulted: an exported `BRAZEN_CONFIG` in the
  operator's shell would collapse every sphere onto one file, which is exactly
  what the blast-radius ruling forbids, so yog injects the wall's own path into
  the snapshot the linked brazen folds and brazen's fall-through is never
  reached. With no workspace in focus there is no file — the pane says so
  (§11) instead of offering the machine's. The linked brazen still validates
  every write.
- Editor: **raw TOML text**, not form fields. Apply: write buffer to
  `.config.toml.yog-tmp-<pid>` in the same dir → run
  `bz --config <temp> --dump-config`; non-zero exit (MalformedFile/BadValue/
  IncompleteProvider… all exit 78) rejects with stderr shown, draft kept in
  RAM → hash-guard → rename into place. A malformed config can therefore
  never land, so `bz` (and every lernie loop calling it) never breaks.
- Alongside the editor: a read-only "effective config" pane =
  `bz --dump-config` stdout verbatim (the merged, redacted, authoritative
  view including env-layer effects yog could never compute from the file),
  plus the built-in-rows hint (§5.1 #21).
- **Structured view = `bz --dump-config`; no TOML dependency.** brazen's
  schema is versionless, forward-additive, and full of open valves (top-level
  passthrough, `body_defaults`) that a form would corrupt or reject; any
  yog-side parse is a second authority that drifts. bz *is* the parser, kept
  in lockstep with brazen by being brazen. This deliberately contradicts the
  brief's "toml parsing is probably justified" leaning — all three judges
  concurred it is not.
- **§16.7 W10 amendment — the gate is in-process, and the rule got
  *stronger*.** Both `--dump-config` calls (the temp gate and the effective
  pane) now run through the linked `brazen` (`src/bz_host.rs`), not a spawned
  binary. "bz is the only lawful parser" was a discipline enforced by a version
  gate; it is now enforced by the linker — the validator and the thing yog is
  about to hand to every lernie loop are literally the same code. yog still
  declares no TOML dependency of its own; the one `toml` in the graph is
  brazen's, reached only through brazen's API. What did die is the phase-1
  `name = "…"` line scan that stood in for parsing (it fed the login rows and
  the credential-presence rows): both now read brazen's own `--list-providers`
  projection.

### 9.2 lernie global config (`models.yaml`, `workflows/*.yaml`)

One editor per file; Apply = **provider gate** → hash-guard + temp-in-dir +
rename (lernie declares these hand-edited; yog is the hand, minus torn writes).
yog still adds no YAML dep. New workflow = same path, new name; templates
copyable.

**§9.5 amendment: `models.yaml` is edited as controls, not as text.** Each
entry's four declared fields are a typed row — `provider` a picker over
brazen's live table, `model_id` a scalar field, `capabilities` a list over the
flow sequence, `context_window` a bounded number — read from and written back
into the very draft this Apply commits. `workflows/*.yaml` keeps the raw editor
(§9.5's second justified fallback: lernie's workflow DSL is lernie's, and yog
holds no grammar for it), which is the general path with an empty schema rather
than a branch on which file is open — the same non-special-case as the gate
below.

**The provider gate (bl-53be amendment).** The original text said "no validator
exists … the operator's risk is identical to `vi`", and a shipped `models.yaml`
promptly offered two Claude models on `provider: anthropic` — a row that was
uncredentialed, against a table nobody had asked. `models.yaml`'s own header
states the contract it was breaking: *"`provider:` on each model is a brazen
provider-row NAME (§4.1) — endpoints, auth, and wire dialects live in brazen's
own config"*. That makes one field checkable without parsing YAML and without a
second authority on anything: brazen publishes the row set
(`bz --list-providers`, the linked projection of §16.7 W10), so **an entry
naming a row brazen does not have is refused on Apply**, every offending entry
named, the draft kept in RAM and nothing written — the §9.1 posture, minus the
temp file, since the judgement is pure over the draft text and needs no file to
hand `bz`. Nothing else about the YAML is judged; that risk is still `vi`'s, and
a future `lernie config --check` still slots into the same pipe.

Two non-special-cases, deliberately:

- **The gate runs over every §9.2 file.** A `workflows/*.yaml` declares no
  `models:` block, so it has nothing to check and passes — the general path with
  empty input, not a branch on which file the editor holds.
- **An empty provider table gates nothing.** An empty answer from
  `--list-providers` means brazen could not be asked, not that no rows exist; an
  editor must never refuse a draft on the strength of a question that went
  unanswered.

**The gate's retired third site: the world's workspace-birth template (bl-c3a9,
retired bl-00ee).** The gate once had a third site — a judgement made *before*
`lernie new`, over `<world>/lernie/template/providers.yaml`, the file `lernie
prime` authors and every `lernie new` commits as the workspace's first
`config/default`. It was right when it was written and is wrong now, and the
line between the two is §16.2's wall.

**Pre-wall it was honest.** brazen's provider table was the *machine's*: one
`config.toml` under the operator's own `$XDG_CONFIG_HOME`, in existence long
before any workspace. So the fact the gate read was available at gate time, and
the refusal stated something true — a template naming a row that table lacked
births a workspace that dies at its first dispatch, every time, for every
conversation in it. Refusing the birth was cheaper than the wound.

**Post-wall the fact does not exist yet.** The blast-radius ruling
moved brazen's config and credentials **inside the workspace's wall**
(§16.2), and a wall is born *with* its workspace: empty, carrying brazen's
shipped rows and nothing else, until the operator's per-workspace sign-in and
§9.1 edit put rows in it — which is **after** birth by construction, since the
wall is keyed by the §3.1 leaf the birth itself mints. So the gate asked a
question that cannot have been answered and read the answer as a refusal: any
template naming a row brazen does not ship refused **every** workspace,
forever, with no operator move that could clear it (bl-00ee measured it — a
fresh wall answers exactly `anthropic, openai, mistral, openai-responses,
google, ollama, claude-code`). That is the very posture this section forbids
one paragraph above — *"an editor must never refuse a draft on the strength of
a question that went unanswered"* — arrived at from the other direction: not an
empty table, but a table that is merely **early**.

**So the judgement is not moved, it is already elsewhere.** yog has a designed
seat for "this conversation's provider is not usable", and both halves of it
read a wall that exists: the §9.5 pane faults `roles.<r>.provider` with the same
`is_unknown_row` the moment the workspace's own `providers.yaml` is rendered,
and a row still dead at the fire surfaces as the §8.3 auth-shaped step failure
with Login one click from the wound (§11 altitude 1). Birth now judges only what
birth can see. The template still needs no editor for the same reason as before —
its contents are lernie's seed and lernie's to fix — and yog reads it not at all.

The reader is the §9.4 anchored block grammar applied to the other file — the
same primitives `roles:` is read through, so `models.yaml` has exactly one
reader in yog and the picker's write and the editor's gate agree by
construction.

### 9.3 Per-workspace config branches (the scripted `$EDITOR`)

Browsing: `for-each-ref refs/heads/config/` + `git show <ref>:<path>`
(read-only, via the existing env-scrubbed `git_tree::cmd`), including each
agent's derived governing config (§5.1 #17, "policy frozen at `<short-oid>`").

**§9.5 amendment: the pane picks, it does not type.** The lineage is a dropdown
over the branches that exist (ending in *new lineage…*, the escape that reveals
a name field — §9.4's "each list ends in its own escape"), the file is a
dropdown over that commit's own `ls-tree`, and **Load** fills the body from
`git show`. A `providers.yaml` so loaded renders as typed role rows (§9.5); every
other path in a config commit — `souls/**`, `descriptions/**`, `workflow.yaml`,
`manifest.yaml`, `version` — is prose or lernie's own schema and keeps the raw
body (§9.5's third justified fallback). Before this the pane authored a config
commit from a free-text branch name and a free-text path over an **empty**
buffer: a blind write against a file nobody had read.

**The reads are gestures, not frames** (§7.2, bl-ee0a). The listing used to
spawn `for-each-ref` inside `App::update`, once
per frame. It is now read when
the Config pane opens — the same read-on-demand gesture §9 already uses for the
file editors — when the lineage selection changes, and after a `lernie config`
this pane itself drove. Nothing polls.

**Both halves of the browse cross the boundary now (bl-dff8).** The read half
used to stay in the pane, which left a headless operator able to *write* a
lineage file (`ApplyConfig` on a `Branch`) and unable to see the bytes it was
about to replace — an Apply nobody had read, which is the defect the pane's own
dropdowns were built to end. Two spellings close it, and they are the pane's two
gestures:

- `Query::Lineages { workspace }` — `/lineages`,
  `{"op":"lineages","workspace":…}` — the browse: every `config/*` branch with
  its tip and every path that tip holds (`for-each-ref` + one `ls-tree` per
  lineage, **at the tip oid**). One answer rather than two, so the listing and
  the trees are of one moment — the guarantee the pane's single-pass `reread`
  already gives its dropdowns. A workspace whose repo cannot be read refuses in
  git's words, by the `work-diff` rule.
- `Query::ReadConfig` on a `Branch` destination — `/config branch <lineage>
  <path>` with nothing after it — the pane's **Load**: `git show
  config/<lineage>:<path>`. It carries the write's `origin` and ignores it (where
  the next commit lands is not where the current bytes are). Unlike a file
  destination it has no empty answer: git reports a missing ref and a missing
  blob alike, so an absent path refuses rather than reading back as a new empty
  file an Apply would then commit over the real one.

Editing — `lernie config <ws> [name]` is the only lawful writer of `config/*`
and is $EDITOR-interactive, so yog drives it. Since bl-3f46 the drive is the
boundary's `ApplyConfig` on a `Branch` destination (§8.5), carrying the
workspace, the lineage, its origin, the checkout-relative path and the file's
whole text — so the pane's Send, `/config branch <lineage> <path> <text…>` and a
deposit are one gesture. The steps it runs are unchanged:

1. User edits the branch's files in RAM buffers; Apply writes the full
   drafted file set to `$XDG_STATE_HOME/yog/stage/<nonce>/`.
2. Spawn `lernie config <ws> <name>` (plus `--from <src>` / `--orphan` when
   forking) with `EDITOR="<yog-binary> --editor-apply"` and
   **`YOG_EDIT_SRC=<staging-dir>` in the environment** — the staging dir
   rides in env, the checkout path arrives as the shim's argv, because lernie
   composes the editor line through `sh -c` and its exact arg-passing shape
   is a flagged open question. The shim tolerates both `$EDITOR <dir>` and
   per-file invocation.
3. `yog --editor-apply` (a tiny non-GUI mode of the yog binary; its copy
   logic is a pure, fully-tested lib function) copies **only the drafted
   files** over the materialized checkout — **never a full-tree sync**:
   `lernie config` has just refreshed `descriptions/**` from the data-root
   pools at commit time and the shim must not clobber that. Exits 0; lernie
   commits and tears down. Empty diff is declined by lernie; surfaced as "no
   change".
4. Staging dir deleted on completion; leftovers swept (§5.2).

**Diligence task 0 for this feature: source-read lernie's exact `$EDITOR`
invocation shape** (how the checkout path is passed through `sh -c`) before
building the shim, and record the finding in the task.

This is the only path that advances a config branch, honoring "never write
inside a lernie workspace except via the lernie CLI" — yog writes only its own
staging dir; lernie performs the commit.

### 9.4 The model picker — §9.2 and §9.3 composed, not a fourth editor

*Which model am I talking to, and how do I change it?* Before this section the
only answer was `$EDITOR` on a YAML file inside a git
branch, and the offerable set was whatever a hand-maintained file happened to
list: **one** usable entry against the **seven** the provider actually served.

**The candidate set is a query, never a field.** Opening the picker fires
`bz --list-models --provider <row>` as the streamed-piped class (§8 — the very
machinery §8.3's login pane runs) and the UI paints the flight. Nothing is
cached: brazen's own model cache (§5.1 #23) is brazen's, the roster changes
without yog's involvement, and a stored candidate list would be a second
representation of a fact the provider owns. This is the standing instruction —
the list is triggered every time the picker is — and the house rule (AGENTS.md: *don't store what you can compute —
make it a query, not a field*).

**And it is a query at the boundary too, since bl-dff8**: `Query::Models
{ workspace, provider }` — `/models <provider>`,
`{"op":"models","workspace":…,"provider":…}` — answers the same ids the picker
paints, folded by the same `model_ids` parser, so a headless operator picks an
id it was offered instead of guessing one at `/model <role> <provider>
<model-id>`. It names its workspace for `Providers`' reason exactly (bl-fcd5):
the row, its sign-in and its cache all live inside that sphere's wall. It reads
**in-process** through the linked brazen rather than spawning: every seat that
reaches `boundary::answer` is already off-frame, so the fork the picker needs —
it paints each line as it lands, on a frame thread — buys nothing here and costs
a second brazen. An unusable answer is a refusal carrying brazen's own words, or
`"the provider offered no models"` for an exit-0 run that listed none — the
picker's own sentence, single-sourced, never an empty list a reader would take
for a provider with no models.

**Two dropdowns, both sourced from brazen (bl-bd89).** The pick is a *pair* —
provider row and model — and the surface is a provider dropdown over brazen's
own effective table (`--list-providers`, the same answer the role marks are
judged against) and a model dropdown over that row's live roster, whose click
is itself the write (below). `<row>` above is therefore the row the operator
chose, not the row the role happened to be on.

It was originally the row the selected role was *already* on, which made the
picker a mirror of the broken state rather than an escape from it: the
operator's `models.yaml` declared `gpt-5.4 → codex`, brazen's `config.toml` had
since renamed that row to `openai-chatgpt`, and so the roster query — asked of
`codex` — came back `unknown provider \`codex\`` (exit 78) with **no candidates
at all**. Nothing was clickable, so nothing was written (no `lernie config` op
in `ops.jsonl` for any pick attempt): the reported "selector not working" was a
dead end at exactly the moment §9.4 exists for, not a bad write. The row
selection now **defaults to the role's own row while brazen has it, and to
brazen's first row once brazen does not**, saying so above the dropdown; brazen
unanswerable (an empty table) is no answer rather than an empty one, so it
steers nothing.

**Each list ends in its own escape, so neither dropdown is a dead end.** The
provider list's last entry is *add a provider…* — not a row but a **route** to
the §9.1 brazen `config.toml` editor, which is the one place a row is authored;
a second add-a-provider form over the same file would be a second authority on
it. The model list's last entry is *custom model id…*, a free-entry field for a
model brazen does not list. That entry can declare an *unserved* model — the
operator's own call, and `models.yaml` is already the operator's authority
(§9.2) — but never an unroutable one, because the row beside it is still
brazen's.

**Selection is the gesture (bl-fb6b).** There is no Set button. Choosing a
model writes, and the pane holds no button at all. The ruling: the Set button
went unclicked because selecting *is* the choice — a selection applies by
itself. A dropdown that has already been read as
a choice does not need a second confirmation of the same choice, and the button
was where the picker's one gesture went to die unclicked.

bl-bd89 had introduced the button for a real reason — *"the pair is only
meaningful once both halves are chosen"* — and that reason survives without it,
because **the write fires on the gesture that completes the pair.** Picking a
provider row re-scopes: the id in hand came from the previous row's roster, so
the row click drops it, re-fires the roster below, and the model click there
completes the pick. Carrying the id across instead would let one row click
commit a pair the operator never chose — `opus-5` on `openai` — which is the
same class of lie as an unstated scope. So the row selection asks for a model;
it never writes one. The three refusals below are unchanged in substance; they
now surface on the selection itself, in ichor under it, instead of on a click
the operator had already stopped making.

**The custom id commits on confirm, not on keystroke.** The *custom model id…*
field writes when it is confirmed — Enter, or focus leaving it with content —
and the confirmed id then becomes the dropdown's selection, which retires the
field and makes a second confirm of the same id impossible. Per-keystroke would
declare `g`, `gp`, `gpt`… every one of them a `models.yaml` entry and a `lernie
config` commit on the workspace branch. A half-typed id is not a choice.

**One control, and the role strip is its scope.** There is no per-role apply
and no second button row: *"you're setting whichever one you have selected"*.
The model dropdown's label states that scope — *set worker to […]* — so the
pair a click is about to write stays readable before the click, which is what
the button's label used to carry; afterwards the same sentence is the receipt
(*set worker → anthropic · opus-5*). The role strip, the provider dropdown and
the model dropdown are three labelled rows of one form, and only the last one
writes.

Three consequences, all invariants rather than warnings — the picker already
warned, and the warning was the dead end:

- **Nothing offered is unroutable.** Every listed candidate is a model brazen
  listed for a row brazen has, so the `models.yaml` entry a pick writes can
  never name a row the §9.2 Apply gate would reject, and the role mark it
  produces can never be faulted.
- **`plan` refuses a row brazen lacks** (`PickError::UnknownProvider`), before
  either file is touched. The §9.2 Apply gate could not cover this alone: it
  runs only when the `models.yaml` half needs writing, so re-picking an
  already-declared model would have carried a dead row into `providers.yaml`
  ungated.
- **`plan` refuses an id the block grammar cannot hold**
  (`PickError::NotAnId` — blank, or carrying whitespace / `:` / `#`). Only the
  custom entry can produce one; a listed candidate is a string brazen itself
  printed.

**One gesture, two files, in that order.** lernie's cross-check
(`config::cross::check_roles_against_models`) refuses to load any config whose
`roles.<r>.model` is not declared in the global `models.yaml`, and refuses one
whose declared `provider` differs from that model's. A role assignment and a
model declaration are therefore two halves of **one** fact, and the picker
writes both. Since bl-3f46 that gesture is the boundary's own
`Action::PickModel { workspace, role, provider, model }` (§8.5): the pane
constructs the variant and calls the chokepoint, and `/model <role> <provider>
<model-id>` or a deposit reaches the same executor — the composition below has
one implementation, whichever seat fires it. The two writes are:

1. `<config-root>/models.yaml` — the §9.2 pipeline (hash-guard + atomic
   rename); skipped when the id is already declared **on the picked row**. An
   id declared on a *different* row has its one `provider:` line repointed
   (bl-bd89) — lernie refuses a config whose `models.<m>.provider` and
   `roles.<r>.provider` disagree, so a skip there would brick the workspace the
   pick just repaired. Everything else in the entry is the operator's and is
   preserved;
2. `providers.yaml` on `config/<name>` — the §9.3 path (staged draft +
   `lernie config`, the only lawful writer).

**models.yaml first, normatively.** A model declared with no role naming it is
inert. A role naming an undeclared model **bricks every step in the
workspace** — the config load fails before the model is ever called. The order
is chosen so the half that can land alone is the harmless one.

**What brazen publishes, and what yog declares.** `bz --list-models --json`
returns `{"models":[{"id":…,"default":…}]}` plus three OPTION-shaped metadata
keys beside them — `context_window`, `max_output_tokens`, `display_name` —
each carried only where that provider's list GET served it (Google serves all
three, Anthropic only `display_name`, OpenAI and Ollama none: brazen's own
empty-set rule). The codex measurement this section once recorded as "and
nothing else" measured a row of the last kind; it was never the shape of the
payload. `models.yaml` requires `capabilities` and `context_window` and neither
serde-defaults, so the picker writes both — but not the same way (bl-848f):

- **`context_window` is the number the provider served**, taken from the
  roster the pick was made from, wherever there was one — §9.4's named constant
  only where there was none;
- **`capabilities: []` is always a declared default** — no provider publishes
  capabilities, so there is nothing to seed it from.

The generated comment above the entry says which of the two each line is, and
names the §9.2 editor to correct it in. The distinction is the point: since
bl-a48b the declared window is the denominator of §5.1 #35's fullness figure,
so a fabricated 200 000 sitting beside a true number brazen served *in the same
request* produced a wrong percentage that read exactly like a considered one — a
default that looks declared is indistinguishable from a judgement. An entry that
already exists is never re-seeded: the operator's own edit wins over any
discovery, and yog's line was only ever the seed.

Rejected, with reasons:

- **The intersection** — offer only models that brazen serves *and*
  `models.yaml` describes, greying the rest with an explanation. This is the
  reported bug restated politely: the operator still hand-edits a second file
  to use a model his provider already offers.
- **A lernie-side default for an undeclared model.** Right shape, wrong repo:
  lernie is an exact registry pin (§16.7) that yog cannot change — and a
  harness that invents a context window has invented it *silently*, where a
  config file that declares one can be read and corrected.
- **Writing `providers.yaml` alone.** `lernie config` validates nothing at
  commit time, so the config lands and every later step dies on
  `LoadError::UnresolvedRef`. The write is not optional; it is the other half.

**yog still declares no YAML dependency (§9.2), and lernie's parser is
private** (its crate exposes only `cmd`). Both writes are therefore **anchored
line edits over the block form lernie's own template authors** — never a
general YAML transform: `roles:` / `models:` at column 0, two-space entry keys,
four-space fields. yog recognizes exactly that shape and **declines loudly**
(ichor, §7.3) on anything else, pointing at the §9.2 / §9.3 raw editors. That
refusal is not a dead end precisely because those raw surfaces already exist —
they are the escape hatch, and the picker is the fast path over the shape lernie
itself writes.

**Scope is stated at the point of change — and it is not what it looks like.**
An existing agent resolves its **governing** config commit, the `config/*`
ancestor its branch forked off (§5.1 #17, "policy frozen at `<short-oid>`"), so
advancing `config/default` does *not* change the conversation on screen. It
changes what the **next** conversation in this workspace forks off. The picker
says exactly that, and the conversation's model line (seated with the rest of
the conversation's settings at the bottom of the surface, beside the composer
— §11) **is** the pair, in the picker's own two dropdowns. The ruling: the
model selection in the conversation window carries both dropdowns, provider and
model, and the whole line becomes `<provider> - <model>` and nothing else
(bl-cd2a, superseding the
`model · <model> · frozen at <oid>` sentence and the `change…` that used to
stand in front of it, bl-a147/bl-9786.)

**What the dropdowns show is what they write.** They carry the config branch
**tip**'s assignment — the pair a pick advances, for the next conversation — not
the governing commit's. A control that displayed the freeze would report the
operator's own write back as a no-op, since the tip moves and the freeze cannot.
The freeze is therefore said on the row's hover, and named outright only when it
has actually parted from the tip:

**A caption is not enough: the model line states the drift itself (bl-9786).**
The picker's scope sentence is read at the moment of the *write*; the surprise
arrives later, at the moment of the *read* — the operator advanced the default
to `gpt-5.6-sol`, came back to a conversation whose model line still said
`gpt-5.4`, and concluded twice that the write had not landed. So the model line
carries **both** facts whenever they differ:

```
[ openai-chatgpt ▾] · [ gpt-5.6-sol ▾]
this conversation is frozen on openai-chatgpt · gpt-5.4 at 1a2b3c4d
```

The clause is the drift, and it appears **only** while the two oids differ: an
undrifted conversation — the ordinary case — is the bare pair and nothing else,
which is what the ruling asked for. Beside the clause are **its two exits**, and
they are the whole of what the sentence is for: one that **keeps** this
conversation — *move this conversation onto the current config* (bl-2d19,
below) — and one that **starts over**, *new conversation uses the current
config*, an affordance that focuses the composer's existing new-conversation
verb (§11) rather than growing a second way to start one. The keeping exit
leads, because discarding a history is the larger act. They are a strip of peers
(§11 rule 8): beside the sentence wherever there is room, wrapped under it where
there is not, never dropped. **Drift is derived at render, never stored:** it is the
inequality of the conversation's governing oid and the workspace's
config-lineage tip, the latter already in the §7 snapshot (`GitTree::commits`),
so the clause appears the moment a config write lands and no field records it.
Drift is the **oids** differing, not the models — a default that moved while
keeping `gpt-5.4` still parted from this conversation, and the line says so with
both oids rather than special-casing the coincidence.

**Rejected: mid-conversation adoption.** Letting the open conversation pick up
the new config is the obvious "fix" and it is the one thing §9.4 forbids — a
lineage whose policy changes under it can no longer be replayed, and every step
already taken was taken under the old policy. The freeze is the invariant; the
model line's job is to stop it reading as a bug.

**The exit that keeps the conversation: `retarget` (bl-2d19).** Until lernie
0.0.8 the only way out was a *new* conversation, and the paragraph above is why:
adoption breaks replay. lernie's `retarget` verb (its ARCH §2.2/§3.4) answers
the same need **without** adopting anything, and the distinction is the whole
justification — it is a **re-fork**, not a mutation. A newly minted dispatch
commit is derived on top of the target config commit, the conversation's own
post-dispatch commits are replayed onto that base, and the branch moves to the
replayed tip. Afterwards `governing_config` — still the same pure ancestry query
— answers the new commit, and every step in the branch was still taken under the
policy its ancestry names. Nothing is stored, nothing adopts mid-lineage, and
the §9.4 freeze holds verbatim: a *different* fork point is not a moving one.

yog's half is the **gesture and its seat**, and both are decided by the sentence
above: the operator reads *this conversation is frozen on …* and the verb that
answers it is the next thing on that row (bl-a0d4's ruling — the weight question
is answered by giving the frozen sentence a verb, not by more ink). The gesture
is the boundary's own `Action::Retarget { workspace, agent }` (§8.5), so the
button, `/retarget` and a deposit are one implementation; the keyboard path is
that line and the §11 focus floor, and the hover names it (§11 rule 3). It
carries **no config name**: lernie defaults to the lineage §9.3 writes, which is
the only one the drift beside it is measured against, so a branch argument would
be a knob with one lawful value.

**What yog does not do here.** It writes no ref and lands nothing: `lernie
retarget` marks `refs/lernie/retarget/<agent>` and the conversation's **own**
executor consumes the mark at its next step boundary (lernie's §2.3
single-writer invariant — the user marks, the executor writes). So the receipt
yog paints says *when* it lands rather than that it has, the drift clause is
still true at the moment of the click, and a conversation that is already on
that config is lernie's own clean no-op rather than a state yog has to model.

**Two seats, one picker — and the second one is the start (bl-824e).** The
operator's request was verbatim *"when starting a new conversation, I should be
able to select the model."* The clean semantic would be a per-conversation pick
that scopes to the conversation being born and leaves the workspace default
alone. **lernie 0.0.3 does not offer it, and this was settled against the crate,
not assumed:** `lernie prompt` takes exactly `<repo>` and `<message>` (`cmd/
prompt.rs`), and `prompt::run` resolves
`ConfigSource::ConfigBranch(workspace::DEFAULT_CONFIG_REF)` — the literal
`"config/default"` — internally (`prompt/resolve.rs`). There is no argument, no
env, and no alternate branch a caller can name; a fresh root *always* forks that
branch's head. A start-time pick is therefore **the same write §9.4 always did,
made one gesture before the start instead of after it** — the workspace default
moves, and every conversation started next is born on it.

So the §11 birth-config block wears the same row (the same two dropdowns over
the same state, minus a drift clause it cannot have) and states that plainly in
one sentence — *"this
moves the `<ws>` workspace default too"* — where the frozen-conversation scope
sentence would have named a conversation the block does not have. The two seats
differ in exactly that sentence and nothing else; `pane` takes the sentence
rather than deriving it, so there is one pane, one write pipeline, and one
authority over the two files. **The rejected alternative is a yog-side
per-conversation config** — forking a throwaway `config/<conv>` branch per
start, or advancing and reverting `config/default` around a spawn. Both make
yog a second authority on a lineage lernie owns (§9.3: `lernie config` is the
only lawful writer), and the second one is a race with any other start in
flight. The honest answer is the one the substrate supports, said out loud; if
lernie ever grows a per-start config argument, this block is where it lands and
the sentence is what changes.

**Roles, not a worker/compactor special case.** `providers.yaml` carries a
`roles:` map; the picker edits one entry of it and lists whatever roles the file
declares — the general path, with the two-role template as its ordinary input.
The conversation's model row shows the `worker` row, the role that talks to
you, because that is the question being asked; the pane's role strip re-scopes
those same two dropdowns onto another role rather than growing a second pair. Writing both roles from one click was
rejected: the template deliberately gives the compactor a cheaper model, and
retargeting a role the operator did not name is the same class of lie as an
unstated scope.

**A dead assignment is marked where it is read (bl-53be).** The candidate set is
a live query, so a dead `models.yaml` entry is never *offered* — but the role
rows name the model each role is **already** on, read from `providers.yaml`, and
that is exactly where a dead entry hides. Each row therefore carries §9.2's
judgement: the model is undeclared in `models.yaml`, or declared on a provider
row brazen's table does not have. A faulted row is glyphed and the reason is
painted in ichor under the selection, so "one usable model out of three" is
visible at the point of change instead of at fire. The two facts this needs —
brazen's rows and the `models.yaml` text — are asked **once per open** and
discarded with the surface (§5.3), on the same terms as the roster: they answer
"is what you already have usable?", a question that exists only while the picker
is on screen.

**Failure renders as itself (INV-2, §7.3).** A `--list-models` that exits
non-zero, times out, or returns an empty roster banners in ichor with the
captured stderr and the exact command to run by hand — the §8.3 fallback
grammar. The query is a *read* and appends no `ops.jsonl` row; the two writes it
leads to do (§4.2), through the surfaces that already log them.

**A credential failure also renders its way out (bl-91f1).** Rendering a
failure as itself is the floor, not the ceiling: forwarding `bz`'s decline
verbatim and stopping there left the operator's only remedies an env var and a
shell command — which is not fixable through the yog interface, and had to be.
So the auth-shaped case ends in a control, exactly as
§8.3 rule 5's step failure does, and on **strictly better** grounds: rule 5
pays a git join to derive which row failed and still admits an `Unrouted`
state, while the picker *is* the row — it named it in the query one frame ago,
so there is nothing to derive and no third state to carry.

- **The classifier is §8.3's**, `login::auth::looks_auth`, not a second list of
  markers. A spawn failure, a 500, or an empty roster routes nowhere: a button
  that sent the operator to a config editor for a dead binary is a guess with a
  control on it.
- **The sentence is the row's own**, `ProviderRow::login_blocked` — the same
  derivation the §8.3 Login rows and the §9.5 config rows render, under the
  row's name. This is a third seat at one wording, never a third wording. It
  matters because `bz`'s own decline is *wrong here*: it is
  `resolved_secret`'s `None` arm, reachable from `StaticSecretAuth` alone, so
  it fires on `api_key`/`bearer` rows and leads them with `BRAZEN_API_KEY` and
  a `--login` those rows can only refuse with exit 78.
- **The destination is a §11 tab**: Login for a row that signs in (the verb
  names it outright), and otherwise the Config tab, where brazen's
  `config.toml` declares the row and its key. One arm for every other
  credential model — `api_key`, `bearer`, keyless, an `auth` spelling this
  build does not know — because the *sentence* already differs per row and the
  destination does not.
- **Additive, never a replacement.** `bz`'s line stays verbatim above it and
  the run-by-hand command stays below it, so rule 5's own clause — *"a wrong
  derivation must never become the only way out"* — holds in this seat too.
  Credentials remain constitutionally untouchable (§8.3): this routes to the
  surfaces that already edit them and reads no secret.

The picker's route out is therefore **one value, not two flags**: the pane
returns the tab it was asked for, which the `add a provider…` entry (bl-bd89)
and this remedy both name. The tab-focus hover that spells its combo (bl-478d
rule 3) is `CenterTab::focus_hover`, one home for the centre strip, the
navigator's entries and this button alike.

### 9.5 The config pane is controls over facts (bl-c225)

The config pane is not blind editing of a config file: every setting the files
declare is internalized to the interface element its kind implies.

Before this section every §9 surface was a `TextEdit` bound to a whole file, so
a setting was typed blind and judged afterwards, at Apply. The pane now presents
each setting the files declare as the control its kind implies, and **judges at
input**. Nothing about what a config MEANS changed; only how it is edited.

**The file remains the single fact.** A control writes into the same RAM draft
the editor already held, through the same anchored line edit §9.4's picker uses;
that draft Applies through the unchanged §9 pipeline (stage → gate → hash-guard
→ atomic rename). There is no second store, no cached copy, and no second
authority on a file's shape — `src/config_edit/form.rs` reads and writes through
`model_pick::grammar`, and the provider judgement is literally the same function
(`grammar::is_unknown_row`) the §9.2 Apply gate and the §9.4 pick gate call.
They cannot disagree.

**Apply stays a button, and that is not a regression from bl-fb6b.** The picker
writes on selection because its gesture *is* the whole pick. Here the unit the
hash guard is taken against is the **draft**, and a pane holds many settings: a
write per keystroke would be N commits and N chances to lose the race with a
concurrent editor. So a control drafts, and one Apply commits — the §9 discipline
this section did not touch.

#### The enumeration (every setting the config surfaces reach)

| Surface | Setting | Control |
|---|---|---|
| brazen `config.toml` (§9.1) | the whole versionless, open-valve schema | **raw TOML** + the `bz` gate — fallback 1 |
| brazen (derived, §5.1 #20–#22) | provider row name / `auth` / credential present | read-only rows (facts, not settings) — the same `row_views` sentences the §8.3 Login rows carry |
| `models.yaml` (§9.2) | `models.<id>.provider` | picker over brazen's live table |
| | `models.<id>.model_id` | scalar field |
| | `models.<id>.capabilities` | list over the inline flow sequence |
| | `models.<id>.context_window` | bounded number (1 … 100 000 000) |
| | *declare a new model id* | id field + provider picker → `declare_model` |
| `workflows/*.yaml` (§9.2) | lernie's workflow DSL | **raw text** — fallback 2 |
| | new workflow name | validated field (`WorkflowNameError`) |
| config branch (§9.3) | which lineage | dropdown + *new lineage…* escape |
| | advance / orphan | radio pair (already typed) |
| | which file in the commit | dropdown over `ls-tree` |
| | `providers.yaml` `roles.<r>.provider` | picker over brazen's live table |
| | `providers.yaml` `roles.<r>.model` | scalar field (the §9.4 picker pairs it) |
| | `providers.yaml` `roles.<r>.tools` | list over the inline flow sequence |
| | every other path (`souls/**`, `descriptions/**`, `workflow.yaml`, `manifest.yaml`, `version`) | **raw body** — fallback 3 |
| task branch (§16.3, re-keyed per agent by the per-agent ruling — bl-e47b) | shared store / stealth / custom branch | radio pair + branch field (already typed) |
| yog `cadence.yaml` (§7.2, bl-3381) | `cadence.watcher.debounce_ms` / `cheap_sweep_ms` / `full_sweep_ms` | bounded numbers, bounds shared with the worker-side parse |
| yog `cadence.yaml` (§7.2, bl-8da1) | `monitor.<workspace>.model` / `provider` / `prompt` | **no control** — the entry is written and removed by the arm/disarm gestures (§8.5), and its policy is prose in the file it names, not a field |

Two absences are deliberate. **No setting in any of these files is a boolean or
a closed enum**, so no checkbox and no value dropdown exists: the two enumerated
things in the Config tab (the branch origin, the task-branch mode) are yog's own
verb selectors, not file values, and already wear radio controls. A control kind with
no member would be mechanism without a setting. And the §4.1 `ui.json` knobs
(transcript density, zoom, panel sizes) are yog's own durable state, edited by
typed controls in the chrome (§11) — they are not config files and do not move
here.

#### The three raw-text fallbacks, each justified

1. **brazen `config.toml`.** §9.1's standing ruling, unchanged and now stronger:
   brazen's schema is versionless, forward-additive and full of open valves
   (top-level passthrough, `body_defaults`); yog declares no TOML dependency, and
   `bz` — *linked*, §16.7 W10 — is the only lawful parser. A form would be a
   second authority that reformats or drops what it cannot model. The provider
   rows beside it are the fix for blindness: the file's *effects* are on screen.
2. **`workflows/*.yaml`.** lernie's workflow DSL, whose parser is private (its
   crate exposes only `cmd`) and which yog has no reader for. A grammar guessed
   at here would be the same second authority.
3. **A config commit's non-`providers.yaml` paths.** `souls/**` and
   `descriptions/**` are prose; `workflow.yaml` / `manifest.yaml` / `version` are
   lernie's own schemas. Prose has no fields, and lernie's schemas are lernie's.

All three are **one editor**, `shell/config_edit/form_ui::raw_editor`: a code
editor as wide as the pane it sits in. egui's stock `TextEdit` is a fixed 280 pt
column, and that is what all three shipped as — every TOML and YAML line wrapped
while the config pane had two to three times that width free (bl-2622). The
fallback exists to show the text a form cannot model, so it must not be the
thing that hides it. One function, so a fourth fallback cannot be born narrow.

#### A new setting is a row, not a rebuild

`src/config_edit/form/schema.rs` is the enumeration: a file is a `Schema` (which
column-0 block holds its entries, what fields an entry carries), a setting is one
`FieldSpec {name, control, help}`. Adding a setting is a row; adding a file is
one `schema_for` arm. A file with **no** schema simply has no typed rows and
keeps its raw editor — the general path with empty input, not a branch on which
file is open. The fleet-cadence settings landed exactly this way (bl-3381):
yog's own `cadence.yaml` (§7.2) is one `Schema` over its `watcher` entry, three
bounded `Number` rows whose bounds are `src/app/cadence.rs`'s own consts — the
control and the worker-side parse cannot disagree — plus one `schema_for` arm
and its own pane section (`shell/config_edit/yog_pane.rs`) on the same
`Editor` + Apply pipeline, touching nothing else.

#### The pane does no work on the frame thread

Per §7.2 (bl-ee0a) the frame renders and captures input. Deriving rows from a
draft is pure RAM string work over a file-sized buffer; everything that could
block is asked at a **gesture**: opening the pane re-reads the drafts (§9's
existing freshness rule), asks brazen once for its effective table and
credential rows, lists the workflows, and reads the workspace's config lineages.
Selecting a lineage reads its tree; **Load** reads one file; Apply/Send write.
Two per-frame reads that predated this section — the `workflows/` readdir and
the config-branch `for-each-ref` — are gone with it.

---

## 10. Portability (Linux + macOS/aarch64)

- **Already portable:** notify (inotify/FSEvents), eframe/glow, libc::kill,
  all git/CLI spawning, the XDG folds (balls/lernie/yog paths are pure-XDG on
  both platforms, matching those tools; brazen's *per-OS* credential/cache
  dirs are reproduced for the read-only displays).
- **The gap:** both probes scan `/proc/<pid>/fd` (Linux-only). The fix:
  - Probe traits return a **tri-state `Probe::{Held, Free, Unknown}`**
    (replacing bool).
  - Linux impls: existing procfs scans, behavior unchanged.
  - macOS impls: parse `lsof -F` output over the inbox dir / response.json
    (writer filter from the fd access-mode field). The **parser is a pure,
    platform-independent function compiled and tested everywhere**
    (recorder-fixture inputs, 100% covered on Linux CI); only the ~20-line
    spawn shim is `#[cfg(target_os = "macos")]`. lsof is slow, so macOS probe
    results carry a 2 s TTL cache (RAM, §5.3), refreshed eagerly on watcher
    events touching the agent, and re-probed only for Live/InFlight agents on
    the sweep (§7.2).
  - `lsof` missing/failing ⇒ `Unknown` ⇒ classification degrades to
    framing-only: closed-with-`end` = quiescent, closed-without = stopped,
    open-file undetectable ⇒ rendered with an explicit **uncertainty badge
    ("live?")**, never a false definite state.
  - **Rejected: flock-acquire probing** (`flock(LOCK_SH|LOCK_NB)` then
    release) — portable and dependency-free but **perturbs the substrate**:
    during yog's transient hold, a `lernie message` writer's probe sees the
    lock taken, concludes a driver exists, and strands the deposit until the
    next scan (writer/driver totality, ARCH §2.11). A probe must never affect
    the observed (I8). Also rejected: the libproc crate (a dependency for one
    probe) and hand-rolled FFI (unsafe, untestable on Linux).
- **Coverage mechanics:** brazen's per-OS creds/cache path folds take
  **`target_os` as a runtime-injected parameter**, so the macOS branch is
  exercised by Linux tarpaulin — no cfg-gated coverage hole. The same rule
  applies to any future per-OS branch.
- **Known upstream limit, documented, not worked around:** `lernie stop` is
  itself /proc-based (Linux-only). On macOS yog surfaces the Stop failure
  verbatim in `ops.jsonl`; fixing stop portability is lernie's ball.
- **CI:** Linux runs the full gate (fmt, clippy -D warnings, tarpaulin 100%
  pinned 0.35.2). macOS (aarch64) job: `cargo build` + `cargo test`, no
  tarpaulin (Linux sees every line because nothing but the lsof spawn shim is
  cfg'd out). Known macOS test issues — `/tmp` vs `/private/tmp`
  canonicalization in probe fixtures, FSEvents timing in fs_watcher tests —
  are already filed as **bl-592b**.

---

## 11. UI structure — three altitudes

Single window; the organizing frame is three information altitudes, each one
click apart. The organizing *unit* on screen is the **conversation** — a root
agent in the focused workspace (§1, STORIES) — and the workspace is a regime
wall (personal / work / client): totally separate blast radius, almost
invisible — nothing in the UI but a small tab bar under the top right.

**Altitude 0 — glance (always visible).**
`TopBottomPanel::top`, left: the **attention strip** — the
§6 total across all workspaces, worded (`⚑ N need attention`, else `⚑ nothing
stirs`). N counts waiting conversations of **both** kinds — wounds and turns
(§6, the bl-2194 ruling) — and the strip never itemizes kinds (amended bl-e266,
below). Immediately beside it the **jump-to-next-attention** control
(`⏭ next`). Legend and control are one unit, in that order: the strip says how
many, the button walks them. It is **disabled when the total is zero** and
carries its job in words on hover — "jump to the next conversation needing
attention" enabled, "nothing needs attention — nothing to jump to" disabled
(glyph doctrine below, bl-e266: `⏭` is not extremely clear and `next` does not
say *next what*, so the seat passes only on the co-visible legend plus the
hover; and a control that clicks but does nothing is worse than one that
visibly cannot). Right: the
**workspace tab bar** — and nothing else. The mark passed through this corner
(bl-b768) and left it (bl-d44e): it states what the *open conversation's* agents
are doing, which is an altitude-1 fact, so it now rides that conversation's own
headline row. Altitude 0 carries only what it is for — the totals and the regime
walls — and the pre-bl-b768 resting wordmark does **not** return to the left
edge: one mark on screen at a time, always the live one, or the same glyph means
two things in two corners. The tab bar is headed at its left edge by the literal
label
**`Workspaces: `** (bl-2d87: a bare row of names never said what the row *was*;
almost-invisible chrome may not cost the operator the word for it), which
carries §3.1's concept in two plain sentences on hover — "A workspace walls off
one sphere of work — personal, an employer, a client. Its conversations, its
settings, its providers, and the balls it claims live inside that wall and
never touch another workspace's." The label is a *name for the row*, not a control: it never
clicks. Then one tab per named workspace (pinned first, in pin
order, then name order), each badged with its attention count; a slim **new**
tab (the deliberate sphere-wall raise, §3.4 — it opens a one-field name form,
§3.1's validation inline, refusing empty and collision; raising a wall is not
the everyday
gesture; the composer is — and the raise **focuses** what it raised, so the tab
that lights up is the one the conversation list and both composers now target);
and an overflow menu (⋯, shown only when needed)
holding foreign and replay workspaces — real but not regimes, so they never
widen the wall row; pinning hoists one into the tabs **without removing it from
the menu** (the menu is where such an entry lives; pinning only changes where it
*also* appears — §11's context-menu doctrine needs that so the menu's ★ is the
visible pin/unpin toggle), and the menu button carries the aggregate attention
of the entries still folded away. Wherever such an entry appears — hoisted tab
or menu row — it says its kind in words after its name (` · ⏮ replay`,
` · foreign`); a named workspace is the ordinary regime and wears no mark
(§11 glyph doctrine, one home in `nav::tabs::Kind::mark`).

`SidePanel::left`: the focused workspace's **conversation list** — headed by a
**new conversation** affordance that clears the agent selection and focuses the
composer, then one row per **visible member of the descent forest** (the unfold
ruling below; with everything collapsed that is one row per root agent, which is
what this list has always been): state badge (aggregated over **that row's own
subtree** — InFlight > Live > the row's agent's settled state, with the
its words on hover, per the glyph doctrine below; the §10 "?" that says the
reading is inferred rather than observed rides right with every other per-row
mark, bl-8257), the agent's **name** with its preview weak beside it
(§3.3's display ladder: the name fact — lernie-stored, legacy goal stamp as
fallback — is the title, the first
payload line is the preview/subtitle, the id when neither exists), the
**live-activity indicator** (below), and — pinned right, in the trailing
metadata group beside the ball badge and the **subagent field** (the unfold
ruling below, which replaces the `(N)` member count) — the **age**
and the **attention flag**: a bare `⚑` in brazen bronze, no number, its words
on hover (§6's badge-only ruling, bl-b9e3; this is the dense repeating seat, so
it hovers rather than states, per the glyph doctrine below). Sort: **last action of any kind,
descending** (the amendment below) over the depth-0 subtrees; within a subtree,
descent order (§2.3, id-sorted siblings). Below the list: a minimal collapsible **balls** section — the
start affordances (▶ Start / ▶ Continue / Assign per §3.5, the new-ball
forms, the empty-project hint — the "internal" toggle that headed the section
is deleted, §5.1 #1) and the
focused workspace's remaining ball rows with join badges; the full per-project
ball views return in the ball-views wave — then the entries that focus the
center's Config and Login tabs (§8.3; tab focuses, never toggled overlays —
the overlay ruling below; the Login surface was the toolchain pane until
§16.7 W13 deleted the phase-1 verdict rows it fronted). Answers "does anything
need me?" and "what's running?" without interaction.

**A send yog has made and the world has not confirmed paints faded** (bl-915e:
a send is shown in-memory in faded colour, brightening when it is actually
locked in as a statement). Two seats wear it, for one reason: §7.2's pending echo — the conversation-list row of a start
whose driver has not written a branch yet, and the inbox-composer's queue row
for a message whose deposit has not been flushed. Both dim the **whole row** to
`theme::tone_solidity(Tone::Weak)` rather than recolouring elements: the row
already wears the colours it will keep, so brightening is that row at full
strength and no seat gains a parallel palette. Neither carries a flag —
"in memory" is the absence of the thing that proves otherwise (`Agent::in_memory`
is no tip oid; `InboxEntry::in_memory` is no filename), which is why a real row
cannot accidentally paint faded and a faded one cannot lie about having landed.

**The name column is a column — conditional marks ride right (bl-b9e3).** The
row is laid out prefix-first and the title's left edge is wherever the prefix
ends, so every **conditional** element painted before the title moves the name
column on exactly the rows that have it: a list where some rows are flagged and
some are not has no readable column of names, which is what the operator's
"*it makes the list not align*" names. A new per-row mark therefore belongs in
the **trailing right-pinned group** (bl-9669: "trailing metadata pinned right,
the title filling what is left"), which grows leftward into slack the title was
going to truncate into anyway, and never moves its left edge. The one lawful
prefix is the **state badge**: it is painted on every row without a condition,
so its seat is a constant rather than a per-row variable.

**The prefix is the state badge and nothing else (bl-8257).** bl-b9e3 left three
conditional elements in the prefix and said so; this is their ruling. All three
ride right, and the rule above now has no outstanding exceptions.

Two of them were never in doubt. The **live-activity chip** and the **alignment
verdict badge** each state a fact about the row that qualifies nothing beside
it, so in the prefix they were pure column drift. The chip is painted *last* in
the trailing group, which seats it immediately right of the title — and the old
"beside the state badge it agrees with" survives as what it always actually
meant: the chip names the class **the title is pulsing in** (bl-9669), so the
two agree and are adjacent. That is a claim about agreement, not about a seat
ahead of the name.

The §10 **`?`** was the one this ruling expected to keep its seat, on the
reading that it is a *suffix* qualifying the badge rather than a mark of its
own — paid for with a **monospace slot allocated on every row**, painted or
blank, so its presence could not move the column. **That was built, measured and
abandoned, and the measurement is why this paragraph does not say what it set
out to say.** The slot costs one character of every title, always, to avoid
movement on a condition §10 makes rare; and the conversation column has no
character to spare. With the slot in, `acceptance::unfold`'s ambiguity guard
reddened: three sibling subagent rows painted the same elided head,
`20260803T04…`, because the titles had lost exactly that much width. Paying on
every row for a rare condition, and buying an ambiguous name column with it, is
the wrong side of the trade.

So the `?` joins the others. It is a fact **about the row** — *this row's state
is inferred from step framing, never observed* — and a row has one state badge,
so it needs no adjacency to say what it is about. A qualifier stranded from what
it qualifies was the objection; one badge per row dissolves it.

Pinned in `shell/acceptance/name_column.rs`, over four rows differing in exactly
one conditional apiece and read through `paint_probe::seen_of` (bl-36c3) so a
mark clipped away by its seat cannot satisfy an assertion about where it rides:
the titles share one edge, each mark is painted on its own row, each sits right
of every title, and the plain row wears none of them.

**Every row is a subtree — the list unfolds (bl-fa82).** The subagent system is
a UX problem, not a data one: the relationship exists and wants representing,
but subagents were hidden. So each agent carries a field on the right of it
naming its number of subagents — two numbers, one direct and one total, each
explained on hover — and clicking that field expands the list: the arrow points
right normally and turns down when expanded. Once expanded, subagents read like
any other agent — recursively indented, with the little chat-reply line.

The reframe that makes this small: **a row is the subtree rooted at its agent**,
and the old root-only list is the all-collapsed case. Every aggregation the row
already carried — the state badge, the attention flag, the flight class, the age
— was always a fold over a subtree; it simply had no subtree but the whole
conversation to fold. Expansion reveals a row's direct children as rows of the
**same anatomy**, recursively. There is no second row kind and no child-rendering
path: with the expanded set empty the list is byte-for-byte what it was.

- **Membership is §5.1 #8's strict descent-id rule** (`git_tree::descent_order` /
  `children_of`), never the loose prefix test the Stop menu's `+children` seat
  uses. An id outside the grammar, or one whose parent ref is absent, is a
  **root** row here exactly as it is there — one tree, one answer.
- **The subagent field** replaces the `(N)` member count in the trailing
  right-pinned group, on every row whose agent has descent children (absent
  otherwise, exactly as `(N)` was): the disclosure arrow — `▶` collapsed, `▼`
  expanded, the crate's one fold vocabulary (`jsonview::GLYPH_COLLAPSED` /
  `GLYPH_EXPANDED`) — and **two numbers, direct then total**: how many agents
  this one dispatched itself, and how many are under it altogether. Total is
  the row's own `members - 1`, so the two facts have one home apiece and neither
  is stored twice. It is an interactive control, so it states both numbers and
  its keys in words on hover (glyph doctrine below; the hover is what says what
  the numbers mean).
- **Child rows indent, and wear the reply elbow `↳`** — the ruling's "little
  chat-reply line". The glyph is `↳` rather than the box-drawing `└`: a
  box-drawing connector only reads as a tree when the continuation strokes above
  it are drawn too, which a flat scrolling list cannot promise, while `↳` is the
  reply idiom and stands alone. It is a **prefix**, which the name-column rule
  above would ordinarily forbid — and does not here, because indentation moves
  the *whole row*: the elbow and the indent are constant for every row at a given
  depth, so each depth has its own title edge and no row's edge depends on a
  per-row condition. The prefix group grows no conditional element (bl-b9e3's
  rule is untouched). The crate had no elbow idiom before this; `theme` is its
  one home, as it is for every other glyph's spelling.
- **The unfold adds length and takes width, and the layout rules already answer
  both.** Length is rule 6's: the list is a bounded viewport, so an opened
  descent scrolls in the room it was left rather than growing the panel or
  painting over what is docked beneath it. Width is rule 1's: the indent narrows
  each depth's title into what its trailing metadata leaves, so a deep row's name
  truncates — the head on screen, the whole of it on hover, never a wider column.
  This is a claim about the *paint*, so the beats read painted **glyphs** rather
  than the galley's input (bl-bc06) and match a title by its head; a head names
  one row only where the frame proves it does, which
  `acceptance::unfold::drive::one_title_each` is the guard for.
- **Expansion state is viewport ephemera** (§5.3, §13.1): a set of agent ids on
  the shell's own RAM, in the mould of the jsonview collapse set — *which data
  you look at*, not data. Deliberately **not** `ui.json`'s `collapsed` array,
  which is for a fixed handful of named sections; an id-per-row key set would
  accrete a stale key for every conversation that ever existed and converge two
  instances onto each other's scroll position.
- **The invariant that dissolves the edge cases: the selection is always on a
  visible row.** Two clauses follow from it and nothing else needs saying. A
  gesture that *selects* a member hidden inside a collapsed subtree — the §6
  jump-to-next-attention, the §8.5 line's address, the §3.4 start adoption, a
  click on an altitude-1 member row — **expands exactly that member's ancestor
  chain**, because landing the operator on a row they cannot see is precisely the
  *why am I here* §6 already forbids of a jump's landing. A gesture that
  *collapses* a subtree holding the selection **carries the selection up to that
  subtree's root**, which is the same move `←` makes on a child. The keyboard
  walk needs neither clause: it only ever steps between rows that are already
  visible, which is why it can be ruled never to reveal anything.
- **Both organizing views unfold.** The grouped-by-ball view (§15 Z9) partitions
  the same visible rows and asserts no order of its own, so it inherits this.
- **Out of scope, deliberately.** The §8.5 `conversations` answer stays
  **root rows**: a machine reader wants the set, not a viewport's fold state,
  and expansion has no durable home to encode. Each root row does gain `direct`
  beside its existing `members` — one more derived integer on a machine surface
  is nearly free, and a reader asking "how wide is this conversation's first
  generation" should not have to re-derive the descent grammar. Drawing the
  descent as an actual **graph** stays bl-5cf8's open question; this is the
  provenance tree seated at altitude 0, not the VISION V1.3 context edge.
- **Altitude 1's compact descent tree (below) is now a second rendering of this
  same membership.** That tension was left unresolved: bl-fa82 fenced it out
  and changed nothing there;
  the decision — keep both, or retire one — is **bl-8905**.

**The list walk (bl-fa82).** Up and down walk the list; left and right collapse
and expand it, including paging back up to the last level when left is pressed
on a child. Going down never expands anything on its own — it skips to the next
thing at the same level.

- **↑ / ↓ walk the visible rows in paint order.** A collapsed subtree is skipped
  whole, so `↓` from a collapsed parent lands on the next row at the same level;
  after `→`, the same `↓` enters the first child. **The walk never expands and
  never collapses anything** — that is the ruling's *never expand just from
  going down*, and it is what makes the visible-row invariant above
  free.
- **→ expands the selected row. ← collapses it**; `←` on a row with nothing to
  collapse — a child, a leaf, an already-collapsed parent — **moves the selection
  to its parent row**, the ruling's *paging back up to the last level*. Pressed
  again there it collapses that parent, so `←` held down walks out of a descent
  the way `↑` walks up a list.
- **This replaces the ↑/↓ roster walk for this seat.** The walk was §6's
  attention-ranked flattened roster across *every* workspace and generation; it
  is now the focused workspace's painted list, in the order it is painted. Two
  consequences, both deliberate: the walk no longer crosses workspace walls
  (crossing a wall is the tab bar's gesture and the jump's, §3.1's blast radius),
  and it no longer reorders itself under the operator when attention lands
  (bl-cad5 already took that out of the list's *sort*; leaving it in the *walk*
  would have been the same surprise one keystroke later). §6's attention rank
  keeps the two seats it still fits: the **jump**-to-next-attention control, and
  the §8.5 decision queue that shares its one roster build.
- **The doctrine (rules 1–3 above) is satisfied, not bent.** `←`/`→` act on the
  row the selection already names, which is rule 1 exactly — the doctrine's
  "which row to expand is a pointer's pick" was written when nothing but a
  pointer could name a row, and a *pick among many* is what rule 2 fences, not a
  verb on the one target already selected. Both keys are paired on the Command
  plane (`Ctrl+←` / `Ctrl+→`) for the same reason `↑`/`↓` are (bl-c21f): every
  selection lands the composer, so the bare plane is surrendered nearly always —
  and lawful under rule 3, since expanding a row repaints a viewport and fires no
  verb. Bare `←`/`→` stay suppressed under a text box, which is also what keeps
  them out of the caret's way.

**"Recent" means recent — the sort is one key (bl-cad5).** A "recent" sort that
does not mean recent makes no sense: the order is by what agent last had any
action. Two things fell to it.

*The order.* The list sorted **attention > running > recency, then root id**
(§6's rank tiers applied here). That is reversed: the sort is now **recency
alone, descending, then root id** for the deterministic tail (I9). Attention
and liveness keep every seat they already had — the state badge, the pulse, the
`✉n` accessory, the strip's count — they simply stop reordering. The old order
was defensible on paper (put what needs you at the top) and read as broken in
the hand: a flagged-but-stale conversation pinned above one that moved a second
ago makes the list look frozen, and the operator scanning for *what just moved*
is the list's actual job. Attention already has a surface built to answer "does
anything need me" — the strip and its jump-to-next-attention walk (§6) — so
paying for that answer a second time in the sort bought nothing and cost the
first one. §6's tiers survive **where they still fit**: the agent roster inside
a conversation and the keyboard walk over it.

*The key.* Recency was the subtree's **newest tip commit timestamp** — the last
*committed step*. Committing is only one of the three ways an agent acts, so a
streaming inference, a running tool, or a just-delivered message moved nothing
and the row aged while the work happened. The key is now the subtree max of
**`Agent::last_action_unix`**: the newest of the tip commit timestamp, the
newest `agents/<id>/messages/` entry mtime (deliveries and results land as
files, §5.1 #12) and the latest step's `steps/<id>/<NNN>/response.json` mtime
(the live tail, §3.5). The tail counts unconditionally rather than only while
in flight — a closed tail's mtime is when that stream *finished*, an action too,
so the in-flight case is the general path with a still-growing file rather than
a branch. Both mtimes are read at **enumerate time**, beside `tip_timestamp`,
never from the render path: the view stays a pure projection of the snapshot
(§3.5), and the one fact drives both the sort and the row's age label. The
grouped-by-ball view needs no change — it partitions the already-sorted rows
and asserts no rank of its own, so it inherits this for free.

**One ball, one row (bl-abbe).** The balls section's rows **partition** the §3.5
join states — ReadyStartable ⇒ the ▶ Start row, Bound ⇒ the ▶ Continue row,
Delivered ⇒ the section's own badged row — and each row carries that ball's
verbs (the §11 ball-row menu seats on the ▶ button, not on a second row). A
bound ball formerly drew *two* half-rows: ▶ Continue above, and, below the
new-ball forms, a bare grey id with no title, no state and no verb (the Bound
badge is `None`, so nothing but the id rendered). The duplicate is deleted
rather than fattened — `AppModel::roster_ball_rows` is the partition's one
home, `ws_balls` staying whole for the workspace pane's §3.2 strip, which wants
every ball the workspace bound.

**The balls section IS the V4 board (VISION §5 V4, bl-9dd4).** The section is
now four folds — **ready / gated / claimed / blocked** — each headed by its own
derived count, each row carrying the facts that column implies. The partition
rule above is unchanged and is what makes the columns possible: one ball still
draws one row, and that row still carries its own verbs (a ready or gated row is
▶ Start + Assign, a claimed row is ▶ Continue, everything else is a read with
the §11 ball-row menu on it). Delivered rows keep their own trailing badged
list — the board is the *live* set, and "delivered" is not a column.

- **The columns are two published predicates crossed, not a second status
  model.** Ready / blocked / claimed are balls' own ladder, read through the
  one `projects::balls::ladder` the §3.5 join and `bl list` both read:
  `claimant ⇒ claimed`, else an unresolved **claim**-blocker ⇒ `blocked`, else
  `ready`. **Gated is balls' other predicate**, `Task::closeable`, whose own
  doc says a close-blocker *"never shows as a status, it only gates the
  finish"* — so a ball that is claimable but not deliverable reads `ready` to
  the ladder while not being what an operator means by ready. `board::column`
  is the two axes crossed and is total; a **claimed** ball that is also gated
  stays in `claimed` (a drone holds it and is working) with its gate rendered
  on the row, so no fact hides in a bucket. Both blocker kinds resolve the same
  way — **the live set is the resolver**, because a resolved ball has no file.
  Nothing new is stored: no status field, no index, no cached column.
- **A gate names what mints it.** A gated row shows the blocking ball's id and
  title; what mints the gate is *that ball's close*, and nothing on this row
  releases it. This is also V4.4's exit-with-handoff, rendered without a
  special case: a drone that commits, unclaims and leaves puts its ball back on
  the board **at its gate**, which is a column, not a dead conversation.
- **A claimed row names its drones, and they are conversation rows.** The
  drones are the §3.3 goal stamps resolved to their roots — *the very set §3.5
  attributes spend by* — so "which conversation is on this ball" and "whose
  spend is this" are one derivation rather than two that agree. The row carries
  the root id, which is the key `Query::Conversations` rows already use, so the
  seat shows the conversation object it already paints instead of a second one.
- **Spend is a column, and the epic rollup is here** — §3.5 recorded exactly
  this as the follow-on and fixed its enumeration source ("a rollup crosses
  workspaces … its enumeration source is the board's join"). A rollup is the
  ball plus its **live** descendants by balls' `parent` pointer, folded across
  every workspace those balls are claimed in, with **one slice per workspace**:
  whole-workspace if any member there attributes workspace-wide (§3.5's
  mid-conversation arm), else the union of the stamped roots. Two
  workspace-wide members in one workspace would otherwise bill it twice.
  A leaf has no rollup (a second copy of its own figure is not a fact) and an
  unbound subtree has none either (no spend to roll up ≠ a rollup of zero).
- **It is a query, not a widget.** `Query::Board` → `Reply::Board`, answered by
  the one `boundary::answer` over the published snapshot; the window's
  `AppModel::board` is that same call. Its three spellings are `/board`,
  `{"op":"board"}` and the GUI's own variant — §8.5's rule, so the board is not
  a GUI-only surface. Every *action* a row offers was already a boundary
  variant (assign / release / move / close / create / update / prepare), which
  is why the board adds no `Action`.
- **The armed loop renders as facts, and only when one is armed (V4.2,
  bl-66fb).** Both of V4's preconditions closed — bl-2b8c's
  mechanical isolated project target (§4.10) and bl-0cea's capability policy
  (§4.11, shipped as bl-fec8 + bl-765d) — so the loop is built, and it is
  **off**. `Board::fleet` is one entry per armed workspace and **empty in every
  world that has not armed one**, which is every world by default: unarmed there
  is no cap chip, no tick line and no reap row, and the section is byte-for-byte
  what it was. That is V4's burden check verbatim (*"unarmed, the board is
  today's balls section"*), and it is mechanical — `src/fleet/facts.rs` derives
  nothing from an empty `fleet:` block, the reply omits the key rather than
  answering an empty list, and the pilot's tick returns before it reads
  anything. There is deliberately **no "unarmed" chip**: a chip announcing the
  absence of a mechanism is the capability theater VISION §5 refuses.
- **Every loop fact is a query.** Cap and project are the `cadence.yaml`
  `fleet:` entry (§4.3 arming); the count is *the board's own claimed rows*
  bound to that workspace, so what the operator is looking at and what the cap
  compares against are one derivation; the last tick is `last_act` over the
  `["yog-fleet",…]` rows (§4.2); and the ceiling is §3.5's own `Ceiling` asked
  over the workspace's already-walked bills — **the gate's policy object, not a
  second opinion**, which is what makes "the ceiling renders where it will bind"
  (V4.3) true rather than parallel. Nothing is stored: no cap field, no count
  field, no tick record.
- **The tick renders as a period, not a countdown, and that is a ruling.** A
  level-triggered loop's tick is not an event — it converges from whatever state
  it finds, so a tick that changed nothing is indistinguishable from one that
  never ran and both are correct. yog therefore states the two things that *are*
  facts: how long ago the loop last changed the world, and the period inside
  which it will look again. Storing a phase to render a countdown from would be
  a second home for a fact the loop does not need, and it would be wrong the
  moment a tick ran late — which §4.3 says is fine ("a missed tick is
  self-healing").

**The live-activity indicator.** The rule: the conversation's name pulses in
the chat list, and something in the active conversation pane pulses with it,
while anything is in flight in that conversation. The three classes — tools,
inference, subagents — look different from one another, and the display
priority is inference > tools > subagents.

So **the name is what pulses** — the title is what the eye is already reading
down the column, and pulsing the state badge (which the row did before this
ruling) animated a fact the operator was not asking about. Beside the name, a
**chip** says *which* of the three classes is in flight; the class is §5.1 #28,
derived per tick and never stored. Both carriers take the class's own hue, so
the pulse itself is one of the three distinctions:

| Class | Glyph | Hue | Why this hue |
|---|---|---|---|
| inference | `◐` | spectral blue | the `InFlight` state's own glyph and hue — a model call streaming |
| tools | `⚙` | hydra green | the hue of `Live`, which is exactly the state an agent is in while its tool runs — so the chip agrees with the state badge, which is the agreement the row states by tinting its **title** in this hue and seating the chip beside that title (bl-8257) |
| subagents | `↳` | brazen bronze | the hue yog already wears for *another agent's* doing (pending mail, attention counts) |

The three differ in **all three carriers at once** — glyph, hue and words —
because a shared carrier would leave two classes told apart by only the one
left over; `theme::flight_badge` is their one home and the mapping is total, so
a fourth class could not ship wordless (the glyph doctrine below). The list row
is the dense repeating seat, so it hovers its words; the altitude-1 pane states
them outright.

**Overlap is the point, not a defect.** A dispatched child that is streaming
satisfies all three predicates at once, and the priority — inference > tools >
subagents — *is* the answer to that, which is why the classes are not carved
into disjoint sets. What the operator wants is the most immediate thing
happening, and a model call is more immediate than the tool it will call or the
child that dispatched it.

**A third seat: the bottom in-flight strip (bl-905f).** The inference status at
the top is right as far as it goes, but an in-flight call's characteristics
belong at the bottom of the screen while it runs, so that an operator looking
down at the chat sees that it is working.

The chat is tail-anchored (bl-5cdb), so the operator's eyes live at the
**bottom** of the pane and the altitude-1 header's chip is out of their field
of view for exactly as long as they are reading the stream. So the same class,
off the same derivation, takes a third seat: **one pulsing line at the
conversation pane's bottom edge** (bl-c038 — inside the pane, like the rest of
the conversation-scoped stack below), the innermost of the §11 bottom
accessories — between the chat tail and
whichever goal box holds the composer's seat. That seat, and not below the
activity accessory: the strip is a fact about the **open conversation** and
belongs beside it, while the accessory under it is world-level ops chrome.
Since bl-929d the goal box is the inbox-composer surface, and the seat is
re-ruled preserved there: the strip sits **above the fold line**, with the
transcript's present, never inside the pending region (§11 inbox-composer).

The strip has a full pane row, so like the altitude-1 header (and unlike the
width-bound list row) it **states the class outright** — glyph, hue and words
from `theme::flight_badge`, nothing restated — and adds the live
characteristics the other two seats have no room for:

| Class | The strip reads | Where the characteristic comes from |
|---|---|---|
| inference | `◐ inference — a model call is streaming · <who> · N chars streamed · <elapsed>` | `Agent::stream.text`'s length, folded at enumerate time and re-folded per frame on the focused conversation (§7.2); `<who>` is the §3.3 ladder over the **member that is streaming**, which in a subtree is not the conversation the header names; `<elapsed>` is now − `Agent::call_start_unix` (§5.1 #28a) |
| tools | `⚙ tools — a tool call is executing · <who> · <tool> · <elapsed>` | `ToolCall::name` — `input.json`'s `name`, read at enumerate time beside the two presence checks that already decide the state (bl-cad5's rule: a fresh stat is gathered where the snapshot is built, never from a render path). A record with no parsable name drops the segment; `toolu_01abc…` names nothing to an operator. `<elapsed>` is now − `ToolCall::start_unix`, off the same record, so name and elapsed are one call's |
| subagents | `↳ subagents — a dispatched child is running · N children running` | the count of running non-root members — a whole-subtree fact has no single agent to name, and **no elapsed** (ruled below) |

Hover says what the strip *is* (bl-68ac), since the class is already stated
inline. Nothing in flight ⇒ **the panel is not created at all**: no line, no
pixel row, no repaint (§7.2).

**Elapsed: the start is in the structure (bl-9dfb — overruling bl-905f's
refusal).** Elapsed is telemetry the world already keeps: when the commit is
made and when lernie/brazen is invoked are both recorded. It is not a file
timestamp as such, but it is definitely in the structure.

bl-905f dropped the elapsed the original ask included, reasoning that yog
observes no start. **Three of its four rejections stand and are kept:**
`response.json`'s mtime is when the *last* token landed, not the first; a birth
time is neither portable (§10) nor free; and a start recorded the moment yog
first *saw* the call would be a fact about yog's own history rather than about
the world — the exact stored-flag shape §5.1 #28 exists to refuse. Its error was
concluding from those three that **no** honest start exists. The world's own
records mark one, and reading a record is not remembering a flag: §5.1 #28a
names the two files, and the pinned lernie's behaviour is what licenses them —
`request.json` and `input.json` are each written **once, immediately before the
call they open, and never rewritten**, at the same instant lernie itself later
records as that call's `started_at`. So elapsed is now − a snapshot field,
recomputed per frame from the wall clock the shell already mints for the §11
list's ages, in that list's own `age_label` spelling. Nothing is stored.

**A fourth candidate was tested and rejected: the branch tip.** It is the
obvious one — `tip_timestamp_unix` is already in the snapshot — and it does not
hold. lernie takes **no pre-call commit from step 2 on** (its §2.10: "the branch
tip already represents what the model reads"), so the tip is whatever the
*previous* step's tool window committed; a branch resumed by `lernie advance`
after a stop issues a fresh model call against a tip hours old, and the strip
would read hours into a five-second call. A start must be tied one-to-one to the
thing in flight, and the tip is not.

**The subagents class carries no elapsed, and that is the doctrine working, not
a gap.** That class is by construction the window in which a child holds a
driver while running *neither* a model call nor a tool — inference and tools
outrank it — so there is no call to time. The child's `dispatch:` commit was the
candidate and it fails the same tie test as the tip: written once per branch,
while the driver over it may be its third run, so "dispatched 40m ago" would
appear in the slot that elsewhere means "running for". A count of three children
has no one dispatch to quote in any case. **A class with no honest structural
start shows none** — partial is lawful, invented is not. The pulse still says
"still alive" there, and the count says how much.

**A tool counts only under a live driver.** `output.json` never lands for a
tool whose driver was killed mid-call, so the record reads in-flight forever;
requiring the member to be running dissolves that with an invariant rather than
an expiry rule. The finer-grained tool chips *inside* a conversation (the
transcript's `⚙ Read — running` row, the descent-tree member's tool chip) keep
their spectral pulse and deliberately do **not** take the tools hue: there the
contrast that matters is running-vs-finished, and hydra green is already the
finished-ok result row two lines down. Same fact, different neighbours,
different seat.

**The panel's width is the operator's, not its content's (bl-9669).** egui
stores a panel's painted rect *as* its width for the next frame, and a widget
laid past the panel edge widens that rect — so a row that overflows makes the
panel wider, every frame, without bound (measured: ~15 pt per frame, and a
splitter drag cannot win against it). Three rules close it — 1 and 2 below, and
the ceiling of rule 5 for whatever escapes them — and they are rules about the
panel, not about any one row:

1. **Nothing in this panel extends past it.** The panel sets
   `wrap_mode = Truncate` at its root, so every label clips at the edge rather
   than reserving space beyond it; and a row with trailing metadata (the
   conversation row's age / subagent field / ball badge, the grouped view's ball
   header badge) pins that metadata **right** and lets the title truncate into
   what is left. Laid the other way round the greedy truncated title consumes
   the full width and the metadata after it lands outside — which is exactly
   the overflow. **A widget that lays its own text `Extend` is outside the
   root's reach** (bl-ac3d): `CollapsingHeader` hard-codes that mode, so the
   balls section's `+ new ball` header ignored the wrap mode and sized the
   column to the project's **absolute path** — ~690 pt of a 1150 pt window with
   a deep scratch project, and at 800×500 a centre of ~110 pt. Such a row
   carries a *name*, not a path: it reads `+ new ball · <label>`, where the
   label is the shortest trailing run of components that tells the project
   apart from the others enumerated (its basename wherever that is already
   unique), elided at 32 characters, with the full path on hover — the path is
   the project's identity (§5.1 #1), so the label may be as short as it likes.
   The fold is keyed by the **path**, not by the header's text, so two labels
   that elide alike still fold independently. One home: `projects::labels`.
   **One call per panel root, not one ambient default** — a `Ui` inherits its
   parent's style, so the rule reaches every row inside without any of them
   restating it, and a seat that must **not** truncate (a wrapped prose block)
   can still say so locally. That exemption is not theoretical: bl-5410 put
   `Truncate` at the *centre* panel's root and the §11 transcript inherited it,
   which turned an open fold into a one-line galley showing 67 of a
   400-character answer (bl-7654, the Altitude-2 Transcript below). The
   transcript states both halves at the row — wrap for the payload a fold opens
   onto, truncate for the one-line chrome — and inherits neither, on the same
   discipline as 1b.
1b. **The control wins the row** (bl-bc06). Rule 1 decides *that* a row
   truncates; this decides *what*. A row that pairs greedy text with a trailing
   control lays the **control first**, at its own natural width, and the text
   truncates into what is left — `shell::row::control_last`, the one home, and
   the only lawful spelling of such a row. Laid the obvious way round the text
   takes the whole width and the control is handed one character: the balls
   board's `assign → <ws>` rendered as `assig…` on one row and as a bare `…` on
   the row whose title was two characters longer, and the Login pane's verb
   vanished on `claude-session-direct` — the longest name brazen's table
   carries, which is precisely the row an operator who is *not* signed in has
   to press. The asymmetry is why the rule has a direction: a truncated
   **label** still names something (its head carries the verb glyph and the id,
   its hover carries the whole value — QUALITY G1), while a truncated
   **control** says neither what it is nor what it does at any length. Nothing
   is laid outside the panel to buy this, so rule 1's own invariant is
   untouched. **The truncation is set by the helper, not inherited from the
   seat**: rule 1 lives on the side panel's root, and these same rows also
   paint where no wrap mode is declared at all (the Login rows render inline in
   the conversation's auth-failed banner, in the centre, where egui's default
   is `Extend`). Pinned right with the text beside it free to extend, the text
   does not run off the edge as it did before — it runs *through* the control,
   an overlap, which is strictly worse than the defect being fixed; bl-9551's
   `acceptance::overlap` walk caught exactly that at 800×500. So rule 1b
   depends on no ambient state. All three halves are pinned in
   `shell/acceptance/elision.rs`, measured
   on the painted **glyphs** — `Galley::text()` reports the string that went in,
   so an assertion read that way is blind to exactly this defect, and
   `paint_probe` reads what the layout actually put on screen instead.
1c. **Rule 1 is a rule about a *panel*, so every panel states it (bl-36c3,
   bl-5410).** It was set at the side panel's root and nowhere else, so in the
   centre, the top bar and the activity panel egui's default `Extend` held: a
   horizontal label there was laid at its natural width and sliced at its
   container's edge — mid-glyph, and with **no ellipsis**, because the galley
   was never truncated and so never had one added. That is QUALITY G1's defect
   in its least recoverable form; the marker that would say "there is more" is
   exactly what is missing, and one measured row was worse still (`auth none…`,
   laid 68 pt and shown 36 — the ellipsis egui *did* add was itself clipped
   off). It is not a minimum-window defect either: `Declare`, a **control**, was
   cut to its first three letters' worth of pixels at 800×500. The rule now has
   one home, `shell::row::bounded`, called at each panel root — the side panel,
   the top bar, the centre and the activity accessory — because a `Ui` inherits
   its parent's style and so one call per panel reaches every row inside it.
   Pinned in `shell/acceptance/legible/`, the disjointness walk's twin: the same
   settled frame and the same four sizes, read through one clip walk
   (`paint_probe::seen_of`, which hands back the glyphs, the rect they were laid
   into and the part of it the clip let through), asserting that **no run's
   shown width is narrower than the width it was laid at**. The assertion is
   one-directional against a list of the cuts the shipped frame paints today,
   each entry naming the ball that owns it — so a new surface that starts
   cutting silently reddens the gate, and a repaired one deletes its line; the
   list is **empty**, which is the state it is kept in. `legible/mod.rs` also
   pins rule 8 through the real window rather than a synthetic strip: a row of
   peers is all of them on the glass or none of them, never four of six.
1d. **Elision has a floor, and rule 1 alone falls through it (bl-5410).** A cut
   marked with `…` is lawful; a run elided until `…` is *all* that is left is
   not, and it is what rule 1 produces when the run is a control in a row with
   no room — the bl-bc06 defect arrived at from the other direction. The two are
   indistinguishable to the operator and opposite under 1c's predicate (one is
   wider than its box, one exactly fits it), so `legible/floor.rs` holds the
   other half: **no run on the glass is a bare ellipsis**, and the composer's
   verbs are painted whole at every size. Which rule repairs a given cut follows
   from what the run is — a label truncates (rule 1), a strip of controls wraps
   (rule 8, `row::peers`, which is what the composer's verb row does), and
   **prose in a trailing slot is neither**: a sentence pinned at its natural
   width is not a control, and where the §8.3 provider rows pinned their blocked
   reason there, the row was allocated rightwards from the pane's edge until it
   began *left of the pane's own left edge* and every run on it was clipped. A
   reason like that takes the line beneath its row and **wraps**. Preformatted
   text is the third case and rule 6 answers it: the §8.5 line's JSON reply
   scrolls on both axes, since truncating it leaves `{…` and wrapping it destroys
   the structure that was the value.
2. **The floor is a sliver, not egui's 96 pt.** `min_width` is
   `MIN_SIDE_PANEL_WIDTH` (24 pt) so the roster can be dragged out of the way;
   what it settles at (~89 pt, measured) is the width of its own controls, and
   that is an honest floor rather than a stored default.

The window's own `min_inner_size` is the same question one level up: 420×320,
lowered from 700×500 — yog must fit a tiled quarter-screen, not dictate one.
The regression is pinned in `shell/acceptance/geometry.rs`: a long title cannot
widen the panel, the width does not creep across frames, and a drag to nothing
settles below egui's stock floor.

3. **Which boundaries drag, and why the others do not** (bl-9ad4). Three:
   the conversation column, the expanded activity trail, and the start-goal
   composer — every panel that holds more than it can show. The top bar and
   the inbox-composer size themselves to their content: their height *is*
   their content (bl-929d: the queue's rows plus the draft's wrapped height,
   capped at half the pane — the fold line is derived, so a dragged boundary
   would be a second, stored answer to the same question), so a drag would
   reveal nothing true and no boundary is offered. The
   center takes the remainder by construction — it grows by whatever the
   others give up, which is what makes three draggable boundaries enough.
   Sizes persist in `ui.json.panels` (§4.1).
4. **A sized panel pins its content to its own rect.** egui re-opens a panel
   at the rect its *content* last occupied, so the same mechanism that let a
   long row ratchet the column wider (rule 1) lets short content collapse a
   pane the operator sized: a 200 pt trail holding one row settles at its
   48 pt floor on the very next frame, measured. Each sized panel therefore
   sets its content's minimum to the panel's own extent (`shell::pin_to_panel`),
   and each scrolling body caps at the room actually left — so neither
   direction of content can move a boundary the operator set. The activity
   accessory is **two panels** for the same reason: a collapsed chip is sized
   by its content and an expanded trail by the operator, and egui keys panel
   geometry by id, so under one id the trail would open at the chip's stored
   height with no way to seed it back. Two ids need no transition detection.
5. **No panel may take more than half the window** (bl-ac3d). Rule 1 is a rule
   about rows and can only ever be as complete as the widgets audited under it;
   this is the invariant that does not depend on the audit. Every panel carries
   a ceiling of `0.5 × window` along its own axis — width for the side panel,
   height for the two bottom ones — and egui re-applies it to the stored rect
   on every frame, so whatever put a runaway size there, the frame after it is
   already back under the ceiling and the centre is never squeezed to nothing.
   A **share**, not a point count, because the defect is a ratio: 690 pt of a
   1150 pt window is a wide roster and of an 800 pt window an unusable centre —
   the mirror of the floor, which is in points because a grabbable sliver is a
   physical size. The clamp has **one home**, `Panel::clamp`, and everything
   folds through it: the size a panel opens at, the size a released boundary
   stores, and the widget's own `width_range`/`height_range`. It bounds the
   settle too — the measured size a settle reports is the panel's *content*
   rect, so the worst a row that escapes rule 1 can now write to `ui.json` is
   the ceiling. Pinned in `shell/acceptance/geometry.rs` and
   `app/tests/panels.rs`.

   **The ceiling is a budget over the stack, not a cap on each member
   (amended bl-9551).** Written per-panel the rule is not an invariant at all:
   the conversation pane docks a composer, the settings rows, the in-flight
   strip and — while a start is pending — a goal box, and four halves are
   200% of the pane. Measured at the documented 420×320 minimum, the composer
   and the settings rows took 107 pt of a 138 pt pane, the conversation was
   left 26 pt, and every run in it painted on top of every other one — the
   QUALITY G4 defect. So a container keeps half of **itself**, read once
   before its first accessory is created, and every accessory draws from the
   other half **in creation order**: each one's ceiling is what the budget
   still holds (`layout::share`, the one home — `Panel::max_size` divides the
   same number). Because the ceiling is measured against what is still
   unallocated, the reserve survives a fifth accessory without the rule being
   re-derived, which is exactly what the per-panel spelling could not promise.

   **An accessory the container cannot pay does not paint.** egui sizes a
   panel by its *content*, so one handed a ceiling below its content's height
   does not shrink to it — it lays out at its natural size wherever it was
   seated, which is the overlap again one level down (measured: a panel given
   a 0 pt ceiling still painted 43 pt, straight through the composer). There
   is therefore no honest very-small accessory: either the budget seats a row
   (`layout::ROW`) or the accessory is off screen this frame. This is the
   in-flight strip's own rule — *"the panel itself is conditional, not its
   content"* — generalized to every accessory, the composer included, with no
   special case: a pane that cannot seat a row of composer has nothing to type
   into either.

6. **What the remainder cannot show, it scrolls (bl-9551).** The budget
   guarantees the centre is at least half its pane; it cannot guarantee the
   surface *fits* in half a pane, and at 420×320 nothing does. A pane whose
   column is a free flow paints its overflow straight over the accessories
   docked beneath it — egui shrinks an inner panel's parent `max_rect` but not
   its clip rect, so nothing stops it. The conversation column is therefore a
   **bounded viewport**: one `ScrollArea` around the centre's tab strip and
   whichever tab it heads, clipped to exactly the room it was left (rule 1 on
   the vertical axis — a scroll body's own clip rect is its viewport grown by
   egui's `clip_rect_margin`, and against an accessory docked hard beneath it
   those few points are the bottom of a banner glyph painted over the
   composer's first row). What does not fit is reached by scrolling, which is
   the one answer that stays true at every window size. Pinned in
   `shell/acceptance/overlap.rs`: the whole window rendered at 420×320,
   800×500, 1150×760 and 2560×1700, with the accessory stack at rest and
   fully subscribed, and **no two painted galleys sharing pixels** — the
   audit's own `crop-s6-overlap.png` finding stated as arithmetic. That file
   also proves the walk bites, on a frame that really does stack two runs.


7. **A control's width is a share of its row, never a constant (bl-76f8).**
   The same reframe as rule 5, on the other axis. egui's default value width is
   `Style::spacing::text_edit_width`, a fixed 280 pt column: measured at a
   maximized 2560 pt window the §9.5 `capabilities` row read
   `tool_use_native, prompt_caching, streaming, stop_` — cut mid-token, no
   ellipsis, with ~1700 pt of pane unused immediately to its right. G1 and G4 in
   one row, the space that would un-cut it right there and unspent; and at
   420x320 the same constant is too *wide*. A constant cannot be right at two
   window sizes, so the width is derived at both ends: `layout::value_width` is
   the row's remaining width less whatever is pinned trailing (the fault glyph's
   seat), floored at a legible field. The floor is in points and the ceiling is
   the row, for the same reason rule 5's floor and ceiling divide that way — a
   legible field is a physical size, a share of the pane is not. A **number** is
   exempt and stays as wide as its digits: stretching a bounded integer across
   2300 pt is the dead field G4 names, not the cure for it. Pinned in
   `shell/config_edit/form_ui.rs`, which measures the value's *visible* run —
   a `TextEdit` lays its text out unwrapped whatever its box width, so the
   galley's own size says nothing about the seat and only the clipped
   intersection is what the operator can read.


8. **A strip of peers wraps; none of them is dropped (bl-b531).** Rules 1/1b
   are about *elision* — a label losing its tail. A row of peer controls fails
   a different way: egui does not truncate a control that does not fit, it
   never lays it out. Measured on the altitude-2 inspector strip in the 202 pt
   centre a 420x320 window leaves — yog's own documented `min_inner_size` — the
   row painted `Transcript Steps Inbox Files` and **`Config` and `Work` did not
   exist**: no seat to hover, no rect to click, and no ellipsis to say they
   were gone. That is G1's *"rendered off-screen … the full value is
   reachable"* in its least recoverable form, and F1 does not excuse it (the
   bare digits still reach every tab) because a control the pointer cannot find
   is not a control. So a peer strip wraps to a second line: it costs a row at
   the minimum size and nothing at any other. Rule 1b is the same question with
   the other answer, and the two do not conflict — there the row pairs *greedy
   text* with a control, so the control is pinned and the text truncates into
   what is left; here every member is a control of its own natural width and
   none may be dropped. One home, `shell::row::peers`, which the centre's tab
   strip (bl-1ca2, which reached this answer first and stated it inline) now
   also reads. Pinned in `shell/row.rs`, in **both** directions: the wrapped
   strip keeps all six, and the one-line strip must still drop peers, or the
   assertion proves nothing.

**Altitude 1 — the selected conversation (center).**
The center renders the selected conversation, **transcript first**: a header
(the conversation's display name — §3.3's ladder — with **when it started**
weak beside it: the id is
the identifier, the name is the title, and — on that same line — the
**live-activity indicator** for this conversation: the same class off the same
derivation as the list row's, in the seat where the eye already rests. This
seat has the room, so per the badge-seat pattern it states the class outright
(`◐ inference — a model call is streaming`) rather than hovering it. **The live
mark holds that row's right edge** (bl-d44e): one circle per agent in this
conversation's subtree, hue = what each is doing (§5.1 #28b, the live mark
below). The two are not one fact twice — the badge states the subtree's *one*
class in words, the mark shows *every* agent's own state at a glance — and the
mark is bare here, no "yog" beside it, because that word brands a window and
says nothing inside a conversation. The aggregated state badge and the age
ride the same identity row.) **The header is the identity line and nothing
else (bl-2e18): every setting for a conversation moves to the bottom of the
surface instead of the top.** The conversation's
config-shaped rows dock at the **bottom** of the surface, in the
conversation-scoped stack — below the composer since the band-order ruling (the
bottom accessories, below) — so what the conversation runs on is out of the way
of what the operator is saying and the transcript leads uninterrupted. **Identity includes what a conversation is
working for, not what that has cost** (bl-2e18, deciding what the ruling left
open): the conversation's own start-flow ball and the workspace's bound balls
(§3.2/§3.3) stay on the header, because a binding is *who this conversation
is*; the figures those same balls have spent go down with the settings, because
a figure is a reading. Two kinds of row sit there:

- the whole-tree **budget-spent figures** — **and, when the §4.1 price table
  is present, what they cost**: one figure line per bound workspace ball plus
  the open conversation's own, each carrying its §3.5 attribution clause when
  the sum is wider than the seat claims (a mid-conversation pickup reads
  workspace-wide, and says so on hover). With no table the lines are the token
  figures they have always been **plus that clause** — the clause is a fact
  about what the sum ranges over, not about pricing, so severability deletes the
  cost column and never the sentence that keeps the number honest. It rode
  behind the cost seat until bl-1765, which made the default install — no price
  table — the one configuration that showed a ball a workspace-wide total with
  nothing marking it as one;
- the **model row**: two dropdowns, `<provider> · <model>` and nothing else on
  the line (bl-cd2a), showing and writing the pair the workspace default
  assigns, §9.4/§5.1 #27 — and, when this conversation has parted from that
  default, the clause naming what it is frozen on, with the freeze
  explained on hover and the *new conversation uses the current config* exit
  beside it (§9.4, bl-9786). The rest of the picker — the role strip, a custom
  id, a provider to add — is `m`, which expands under the row it re-scopes.

Then the **marks the focused agent wears** — the §6 durable facts (notified,
budget-exhausted, declined-transfer, abandoned), each said outright as a full
sentence because this seat has the room and because it is where a
jump-to-attention lands.

**The centre renders no membership list (bl-8905).** It carried one until this
ball: a compact descent tree, one selectable row per member, rendered whenever
the open conversation had children. bl-fa82 made that a **second rendering of
one fact on one screen** — the conversation list's unfolded rows are the same
descent-id membership (§5.1 #8), in the same order, selectable by the same
gesture, and the list's visible-selection invariant guarantees the member you
pick in one is revealed in the other. Two surfaces, one set, both on screen at
once, is the drift the single-source rule names; it was already costing the
epic's own acceptance beats, which filter galleys to the conversation column and
run a deliberately unfocused fixture *solely* because the member names painted
twice (`shell/acceptance/unfold/drive.rs`).

Nothing the tree uniquely carried is lost, and no seat was invented to catch it
— each fact lands where this doc's own altitudes already put it:

- **Membership at a glance** stays in the centre, on the header's **live mark**:
  one circle per agent in the subtree, hue = what each is doing, the roster
  named in words on hover. That is the centre's membership reading, and it is
  the one it was already documented to be.
- **A member that needs you** raises `⚑` on its **own list row** — a row's
  attention is the fold over that row's subtree, so at depth the flag is the
  member's own. Undriven mail is §6 rule 5 and is deliberately not seen-gated,
  so the one thing the tree's `✉n` made actionable is what the flag already
  states.
- **The `✉n` count itself** is altitude 2's Inbox tab, which owns it (S7-T4).
  **A tool call's identity** is altitude 2's transcript, where the call is; the
  list row's flight chip and the mark's roster both already say *that* a tool is
  running. **A non-focused member's durable marks** are one selection away, in
  the sentences directly above — which is where a jump-to-attention lands.
- **A member's raw id** kept the hover seat §3.3 gives it, which moved onto the
  list row rather than being dropped: the title is the display ladder and never
  the id, and the id — the branch name and the on-disk key — rides the tooltip.
  Every row here is the subtree of an agent, so the rule needs no depth clause:
  a row hovers the id of the agent it is, at depth 0 exactly as at depth 3. This
  one was **not** foreseen when this ruling was drafted; `acceptance::naming`
  caught it, which is the argument for a beat that reads paint rather than a
  survey of what a surface seemed to carry.

**What this costs, stated rather than hidden.** Opening a conversation no longer
shows its members without a gesture: the list row must be unfolded (`→`, or the
field's arrow). That is accepted, not overlooked — the operator's ruling asked
for exactly that disclosure (*"right now, subagents are hidden … what I want to
do is make each agent have a field on the right of it … which if clicked,
expands the list"*), and an always-open tree in the centre is the surface it
replaced. **No auto-expand-on-focus was added**: the walk never expands (the
second ruling), and expanding on selection would be the same surprise one
gesture later. The centre reclaims the vertical space, which the accessory
budget (rule 5) and the bounded viewport (rule 6) make worth more to the
transcript than to a second list.

Then the
Altitude-2 inspector for the selected member, Transcript
tab by default with the live streaming tail appended and visually distinct
(§5.1 #10/#12). A conversation whose **latest step is an auth-shaped failure**
(the §8.3/Z8 detection over the derived step facts) banners in ichor red and
renders the **Login** affordance inline — the same streamed `bz --login`
machinery as the Login pane, one click away where the wound is. A
conversation whose latest step is the §7.3 **no-response wound** — its driver
died before the model said anything — banners in ichor red the same way,
**stating the cause rather than pointing at it** (bl-55d8): the sentence carries
the tail of that step's own `stderr.log`, the model adapter's last words, and
says outright when there are none. It does not offer to re-run the prompt
(§14), and it no longer sends the operator to the activity accessory, which for
a `lernie message` turn holds nothing (§7.3's row). The whole sentence has one
home (`steps_view::wound`, `Wound::banner`), so the alarm cannot drift from what
the derivation found — the §11 badge-seat rule applied to a banner. That one
banner paints through a **grace window** (§7.3's own row, bl-90bf): the wound
must still read true after `WOUND_GRACE` has passed, so a healthy send whose
driver the snapshot has not yet seen holding its lock never flashes red. The **model
picker** (§9.4) opens here too, at the conversation's own model line in the
bottom settings rows rather than in the Config tab: the question is asked
while looking at a conversation, so the answer is answered there. Opening it fires `bz --list-models --provider <row>` and paints
the flight (spectral, the shared §11 pulse); it lists one row per role
`providers.yaml` declares, states that the change advances `config/<name>` for
the **workspace** and takes effect for the *next* conversation while this one
stays frozen at its oid, and renders a failed or empty roster in ichor with the
run-by-hand command (§7.3). Both banners sit above the
inspector, so the cause is on the conversation surface whichever
tab is open — including the default Transcript, which for a step that produced
nothing has by construction nothing to show. The picker no longer shares that
seat: it opens where its line is, in the bottom settings rows (bl-2e18), which
is the same property said of a different edge — the surface is the
conversation's, not any one tab's. yog's own plumbing never reads as
conversation content: the ops trail lives in the bottom activity accessory
(below), never as inline rows between conversation content.

**The when-seat reads the id, it does not restate it (bl-16da).** The timestamp
at the top of the chat was unconsumable — `20260801T225418Z-2286254c` — and had
to stay ISO 8601 while being built for a reader rather than a machine. That
string is lernie's agent id — ISO 8601 **basic** form (no separators, for a
filename and a branch name) plus a discriminator — and it was being shown raw
in the seat beside the title. The seat now paints the **extended** form,
`2026-08-01 22:54:18Z`: still ISO 8601, still UTC, no locale prose, and the
hash suffix gone, because a hash is not a timestamp. It is **derived at
render** from the id and stored nowhere — the id IS that storage, exactly as
lernie's `name` blob is the name's (§3.3) — and the full raw id **hovers** the seat,
named as what it is ("the conversation's id — its branch name and on-disk
key"), so the key stays one gesture away for anyone reading `git branch` or a
worktree path. An id the stamp grammar does not recognize is **its own label**,
the same last-rung rule §3.3's ladder ends on: a foreign or hand-made branch
renders itself rather than a guessed date.

**Altitude 1, nothing selected — the birth-config block (bl-824e).** An empty
selection is not an empty seat: the center then holds *the parameters the next
conversation will be born with*, in the very seat the conversation's settings
rows occupy — at the foot of the surface, below the composer, where the
settings-seat and band-order rulings together put every config-shaped row (bl-2e18
re-seats the block with the settings it mirrors; bl-58e4 moves both to the far
side of the input box). It is the general path with an empty input, not
a second surface — the settings rows answer "what is this conversation running
on", and with no conversation the same question is "what would one started now
run on". The block's rows are config-shaped facts and nothing else; the
composer beside them stays one box and one Enter (§8.2). Two rows today:

- the **work directory** (§3.4 path rung) — an editable box pre-filled with the
  bare rung's own resolution, the single carrier of *where the next start runs*
  (bl-7927; it used to ride the composer as `dir (optional)`, which put a birth
  parameter in the drafting seat and made "blank" mean "home" without saying so),
  validated inline like the §3.1 name field: a path that is not an existing
  directory paints its refusal in ichor beside the box and disarms Enter
  (bl-6191, §3.4);
- the **model row** — the worker pair the config branch's head assigns, in the
  same two dropdowns the conversation row wears (the **one** §9.4 picker, never
  a second implementation), with the short oid a start would fork on their
  hover. A workspace whose snapshot carries no config lineage yet paints no row
  rather than a row about nothing; the
  directory row is unconditional, since it depends on no lineage.

**Altitude 2 — the inspector (per selected agent, tabbed; every tab that
parses a file has a Raw toggle showing that file's verbatim bytes).**

**The step spine runs through the chat, not beside it (bl-98da, re-seated by
bl-1802; VISION V1).** One notch per step, each wearing the read-state commit
that step's model call was assembled against (§5.1 #30) — and each **is** the
faint boundary rule the Transcript tab already draws at that commit (§5.1 #29).
Those were one fact drawn twice, from one `meta.json`, in two places; the
operator's ruling collapsed them: *"every operable commit should be a horizontal
rule across the chat, instead of a window on the side. the fork overlay should
show up when you click on one."* The `SidePanel` gutter is gone. An **operable
commit** is a notch with a `meta.json` commit and a seat in the chat, and its
rule is where that call's reading began — the run of entries (its predecessor's
tool results, then whatever the boundary drain delivered) that it read and
nobody before it had. Clicking a rule **pins** every tab to that commit (§5.1
#31); the pinned rule burns brazen so the mark is visible where it was made.

**One gesture, both directions, and the release reaches every pinnable tab.**
Clicking the pinned rule releases it — no second "unpin" control in the chat —
and because the rules live in the Transcript tab while the pin reaches all four
pinnable tabs, the **pin banner is itself the release**: it already paints above
whichever tab is open and already names the commit and the spend as of it, so
clicking it comes back. That is one existing gesture given the seat it needed,
not a second control; it adds no verb and no tab routing. The banner's former
sentence, "Pick the same mark again to come back", stopped being true the moment
the mark lived in another tab, and is replaced.

Under a rule hang its **child cards**: the child's name, its fork-point label
(`from here` / `from config/<name>` / `from <Name>@<oid>`), its state chip, its
own `steps/<id>` spend, and a **streaming tail** — the last line or two of its
in-flight inference text, off the same fold the bottom in-flight strip reads
(§5.1 #10), pointed at the child's id. Moving text means active; still text
means tool-wait or quiescent. Clicking a card is the ordinary selection
gesture, the same one the descent rows above spend. **The two edges are still
two facts and are no longer two strokes** (VISION V1.3, amended by bl-1802): a
gutter had a column to draw a solid ancestry line and a dashed provenance line
in, and a rule across a chat does not — but the card's fork label already says
which it has, in words, so the strokes were a second home for a fact the label
states and they are deleted along with the `context_notch` index only they read.
Drawing the descent as a graph needs a seat that is not the chat; that is
bl-5cf8, not a stroke smuggled into a chat rule.

**The burden check needs no gate.** The rules the chat carries are the ones
bl-929d already shipped — one faint line per commit boundary — so an operator
who never clicks one sees exactly today's transcript, and the old "the rail
paints only when it has more than one notch" gate is retired with the gutter
that needed it. No dispatches, no cards.
**The fan anchor is this seat, and the fan has landed on it** (VISION V1.7 →
V2.3, bl-dc0c). The children born at one notch render as a **cohort**: a
`×N` chip and the ancestry they share stated once, then a column per candidate
carrying its state chip, its own spend and its terminal response, so the four
facts an operator judges by sit side by side. Candidates that forked off
different refs share nothing to hoist, so the group says so and every column
states its own ancestry. **A cohort of one wears no header and is exactly the
card V1 drew** — the count is a `Vec`'s length here and nowhere a branch.
Nothing records the group (§5.1 #33).

**A pinned notch offers Fork from here** (VISION V2.1): the composer sits
beside the pin banner, seeded empty, and dies with the pin — which is V2's
burden check, *"the composer is reachable only from a pinned notch"*, made
structural, and which is why clicking a rule *is* the "fork overlay" the
operator asked for: the composer rises on the pin the click raises, with no
mechanism of its own. Its three fire-time controls are three parameters of the one
`lernie dispatch` argv (§8.2) and never a fourth thing yog owns: the **fork
point** (`here`, or a config branch — one control, two kinds of value), the
**role**, which *is* the model because lernie binds provider and model to a
role in the governing config's `providers.yaml` and the picker names both
(§5.1 #34), and the **skills**, pinned from the world's own pool. **×N is the
gesture repeated, not a gesture of its own**: Fire crosses the boundary once
per candidate. A workspace whose config declares no role anywhere paints no
composer at all — a button that cannot work is not offered.

**The toggle's scope is a rule, not a list** (bl-1ff1): Raw is the escape from
a *parse*, so it rides Transcript, Steps and Inbox — the three tabs that
project a file's bytes through a parser — and not Files (whose per-file preview
already *is* the bytes) nor Config (which parses no file: it names the commit
policy is frozen at and lists its tree). One `InspectorState::raw` flag behind
all three, one label, and each tab's own render honours it; STORIES §S7 point 3
carries the reasoning and the cost it accepts.

**A stream is read from its tail, and the tail is where the view sits**
(bl-5cdb). The newest text sits at the bottom and pushes older text up and out
of view: following the live tail is the default that costs nothing, and
scrolling up is the deliberate act of reviewing history.

Every scrolling body whose bottom row *is* its newest content
therefore sits on that bottom, **and an underfull body sits at the bottom** —
one rule, two halves, and a surface takes both or neither. Overfull is
`ScrollArea::stick_to_bottom` (egui's own release-on-scroll-up /
re-engage-on-return semantics — no yog state, so this stays viewport ephemera
under §13.1 and nothing new is stored). **The release is the operator's right
of way**: a scroll-up leaves the tail at once, growth never recaptures a
released view, and the anchor re-engages only when the operator returns to the
bottom — pinned by the transcript's own wheel-driven interaction test
(bl-e90a; the "sticky" scroll the operator reported was §7.2 frame-time cost,
not this anchor, and the test is what keeps an egui upgrade from making it
this anchor). Underfull is a top pad of `viewport −
body`, because egui draws a short body at the top and leaves the space beneath
it. The surfaces: the **Transcript** (rows in filename order, live tail last),
the **Inbox** (oldest-first, §2.11), and the **activity trail** (`ops.jsonl`,
chronological — §4.2). The condition is the rule, not a list of tabs, and it is
what excludes the rest: **Files** and **Config** list a tree, whose end is
alphabetical and not new, and **Steps** takes the idiom *only while the table is
the whole body* — a drill-in hangs below it (§11 Steps), so with one open the
body's last pixel is the end of that detail; riding it would carry the step rows
off the top the instant the operator picked one, and padding a short table down
onto it pushes those same rows out of the way for the same wrong reason.

Both halves live in **one home** (`src/tail.rs`), never restated per view. The
second half is bl-8c13 overruling bl-5cdb's original ruling — "underfull content
is left alone: an anchor that has nothing to hide moves nothing" — which the
operator rejected on sight: *"we're still starting at the top and going down.
The anchor is working fine, but we should start at the bottom."* An anchor that
only engages once the screen is full is not terminal semantics; new text must
appear at the bottom edge from its very first line and grow upward into the
empty space. The pad reads the body height the previous frame measured (a scroll
body learns its extent only while painting), so it carries the same one-frame
settle the anchor does; an unmeasured body is assumed full, so the unsettled
frame paints exactly what a bare `stick_to_bottom` would and the settle can only
move content down.
- **Transcript** — `messages/NNN-*` in filename order; `.md` deliveries with
  origin header; model `.json` as content blocks (text/thinking/tool_use);
  `NNN-tool.json` results; committed tool_use without tool_result = "tool in
  progress"; live tail from the open response.json appended, visually
  distinct — **as up to two rows, `thinking:` then `live:`** (§7.2 the thinking
  ruling, bl-54f7), the same two a committed model turn has, so what is on
  screen does not change shape when the step commits. Both wear `Tone::Live`
  and are therefore auto-expanded while the step is happening; both update at
  frame cadence off §7.2's follower, not at the derivation's.
  **That parity is now asserted, not merely intended** (bl-7654,
  `transcript/tests/parity.rs`): for one payload the live rows and their
  committed counterparts agree on `RowClass`, on fold availability, on the
  preview/body split, and — read off the glass — on the glyphs, the ink and
  being whole. The **one** thing they may differ on is stated and pinned in the
  same file: with the knobs shut, `Tone::Live` still auto-expands the streaming
  rows while the committed ones fold, because the live answer is the show
  (bl-1f21). So parity is asked with the knobs open, which is the state the two
  are supposed to share.

  **Density: one line per thing, folding open.** The chat was very inefficient
  in vertical space. Each message is one line, each tool use is one line, and
  each expands or contracts; actual responses to the operator auto-expand
  fully, everything else auto-contracts, and both automatics are config knobs.

  So the row is a **block, not a file**: a delivered message, one model text
  block, one thinking block, one tool call, one tool result, the live tail —
  each is exactly one line (a `prefix` label plus the payload's clipped first
  line) that folds open to its whole payload. A model message that says
  something and then calls two tools is three rows, because it is three
  things. A payload that already fits its line has nothing to fold and shows
  no toggle (the empty body *is* that fact — no separate "foldable" flag). The
  fold affordance is jsonview's `▶`/`▼`, imported from that one home rather
  than restated; the disclosure triangle passes the glyph doctrine on
  convention, and the tool-call and tool-result rows carry words beside their
  glyphs (`⚙ Read — running`, `✔ tool result — ok`, `✖ tool result — error`,
  the latter two from `theme::tool_result_badge`) so hue and
  glyph are never the only carriers. A tool-result row that folds also **sizes
  its fold** (`✔ tool result — ok · 4213 chars`, bl-1f75; operator: *"I'd like
  tool result collapses to show me the number of characters in the output"*) —
  contracted, the row said nothing about whether the `▶` opened onto four
  characters or forty thousand, which is the one decision it exists to inform.
  The count rides the prefix because that is the seat visible while contracted,
  it is the **body's** length so a row with nothing to fold stays bare (the same
  fact the missing toggle is), and it is `char`s, not bytes, which would
  over-state any non-ASCII payload. The **live tail carries none**: it is
  in-flight, so it is already expanded, and how much of the answer has landed is
  the in-flight strip's `N chars streamed` (§5.1 #28a) rather than a second
  per-frame spelling of a growing number. The per-message model-id line and the raw
  `tool_use_id`s are dropped from the parsed view — the model is the
  conversation's model line's fact (§9.4, the §11 bottom settings rows) and
  the ids are plumbing the Raw toggle still shows verbatim.

  **Anything hidden is hidden behind a triangle (bl-7654).** Model output was
  being elided rather than shown, which is two defects at once: the content was
  unreachable, and what *was* hidden carried no affordance to reveal it.
  Anything hidden is hidden behind a triangle to turn. Output with no fold
  available — too small to have one — was faded as though it were folded, which
  is exactly inverted; and an expanded fold still did not show the model output
  in its entirety.

  So: **nothing on this surface is cut, capped, clipped or elided without a
  disclosure control that reveals it**, and the row already knows which case it
  is in. `body.is_empty()` — the projection's own spelling of *"is there
  anything behind this?"*, and already the fact the missing toggle and the
  absent size hint both are — decides **three** things at once, which is why
  they can never disagree:

  - **The expanded body wraps.** It is a *wrapped prose block*, the exemption
    rule 1 names in its own body, taken locally at the row and never as an
    ambient default. It had to be taken because rule 1 *reached* this surface:
    bl-5410 put `Truncate` at the centre panel's root, a `Ui` inherits its
    parent's style, and an open fold was therefore a one-line galley ending in
    `…` — **67 of a 400-character answer at 420x320, 133 at 800x500**. A
    triangle that reveals a still-cut payload is worse than no triangle. The Raw
    view wraps for the same reason and a stronger one: it carries no fold at
    all, so bytes it cuts are unreachable.
  - **The contracted preview truncates**, and states rule 1 itself rather than
    inheriting it — the same discipline as rule 1b, and for the same reason: in
    a seat that declares no wrap mode egui's horizontal default is `Extend`, and
    the pane's clip rect slices the run mid-glyph **with no ellipsis at all**
    (measured: 940 pt laid, 267 pt shown at 420x320). Truncating is lawful here
    precisely because the `▶` beside it is the affordance the `…` promises.
  - **Only an abridged preview fades.** `ui.weak` was unconditional, so a
    payload shown *whole* — the complete content — read exactly as abridged as
    a one-line stand-in for hidden text, which is the surface saying the
    opposite of the truth where it tells the whole truth. Inverted: complete
    content reads complete. The fade is **solidity, not a tone** — one number
    from `theme::tone_solidity` (bl-915e's "a statement that is not yet a
    statement"), the same 0.55 the §7.2 pending echo spends, dimming the seat
    rather than repainting it into a parallel palette. `Tone` could not carry
    it: a tone is the projection's claim about what kind of thing a row *is*,
    while abridgement flips when the operator turns the triangle, and one
    payload may not wear two tones.

  The **160-character preview cap** is unchanged and is not in tension with any
  of this: it is what *creates* the body a row folds onto, so a capped payload
  always has a `▶`. The **fold defaults were already right** —
  `transcript_expand_responses` is `true` and no persisted knob outranked it;
  what was never asserted is the *delivered* behaviour through the turn rollup,
  which swallows a segment's machinery into one shut aggregate and must never
  swallow the answer with it (the answer is the segment's last row, by
  construction). The operator's "should not be folded by default" and the fade
  were one complaint: faded read as folded.

  All of it is pinned on the **laid galley** (`transcript/tests/legible.rs`,
  `transcript/tests/parity.rs`) at both window sizes this surface has broken at
  and in **both seats** — with the centre's ambient `Truncate` and with none —
  because a row that depended on ambient state would be correct in one and
  broken in the other. `Row::preview`/`Row::body` are never asserted against:
  the standing hazard here is a probe that reads the projection and so cannot
  see elision at all (bl-bc06).

  **The sender label is the agent, not the model (bl-2335).** A model response
  is labelled with its agent name, not the model. A model turn used to render `· gpt-5.4: Yes`, which puts a **config
  fact in the speaker's seat** — every conversation in a workspace would then
  introduce itself with the same name, and the one thing the seat exists to
  answer (*who is talking*) went unanswered. The label is therefore the
  conversation's §3.3 display name (`shudder-storeroom:`), derived through the
  **same one function** the composer's `→ message <x>` target line reads
  (`root_of` → `display_name_of`) — the ladder is the single source, and the
  transcript never spells a name a second way. A descent child is labelled by
  its conversation, on the same rule and for the same reason the composer is.
  The model id keeps its home on the row's **hover** ("ran on gpt-5.4 — the
  model is config (§9.4), not the speaker"), which is also where a turn taken
  on a *different* model than the header now names stays legible: the hover is
  read off the entry lernie wrote, so it is that turn's truth, not the current
  assignment's. Only model turns carry it — a thinking or tool row labels what
  it *is*, and has no speaker to explain.

  **A delivered `.md` is a deposit file, envelope and all** (bl-6ec6).
  Delivery is a literal `rename(2)` and "the file's frontmatter travels
  untouched" (ARCH §2.11), so `messages/NNN-<sender>.md` opens with
  `---\nfrom: … \n---\n`, not with the message. Yog parses it with the one
  envelope reader it has, `inboxview::parse_deposit` — a second copy in
  `transcript` would be a second truth about one format. The envelope's
  asserted fields are dropped from the parsed view exactly as the model-id
  line and the `tool_use_id`s are; the Raw toggle still shows them verbatim.
  **`epitaph:` is the one field kept, and the rule's own reason says why**
  (bl-71e8): what is dropped is re-asserted elsewhere — the sender by the
  filename, the timestamp by the file order — but nothing else in yog carries
  how a child ended, and on a body-less result deposit it is the entire
  message. It rides the row's prefix seat (below).
  Reading the file as its own body was the bug behind the operator's
  `▶ user:---`: the fence was being previewed as the user's words while the
  words themselves sat folded away behind it.

  **A committed `.json` message is a bare array of canonical blocks**
  (bl-47ec). Lernie writes `[{"type":"tool_use",…}]` and
  `[{"type":"tool_result",…}]` alike; an API-shaped object wrapping them in
  `content` is also accepted. Both parsers ask **one** function where the
  blocks live — two answers was the bug behind the operator's stuck
  `⚙ bash — running`: the model parser took the bare array, the `tool` parser
  demanded an object, so every real `NNN-tool.json` fell to the Raw bucket and
  rendered as a bare `▶ 021-tool.json`. A result that is not classified as a
  result names no `tool_use_id`, and "tool in progress" is precisely *no
  committed result names this id* — so the pulse could never retire. Nothing
  stores "running" for something to flip; fixing the classification fixed the
  badge. The id itself is **opaque**: `call_…` (OpenAI) and `toolu_…`
  (Anthropic) pair by byte equality, and no parser inspects their shape.

  **Expansion is derived, never stored per row.** A row's state is a pure
  function of its class and two durable knobs: `expanded = auto XOR
  overridden`. The classes split **conversation from machinery**, not model
  from everyone: **Response** is anyone talking (a delivered message, a model
  text block, the live tail) and **Other** is the machinery around it
  (thinking, tool calls, tool results, raw bytes). A message delivered *to*
  the agent is the other half of the exchange the operator came to read, so it
  arrives expanded beside the reply it provoked — a folded user turn is the
  operator's own words hidden from them (bl-6ec6). Collapse stays the opt-in
  it always was: the `▶` toggle, or the knob. A delivered message whose body
  is empty once the envelope is off — a result message that asserts an epitaph
  and nothing else (§2.6) — previews `(no message body)`, the same arm the
  no-content-blocks model message already takes.

  **A result deposit says how the child ended, in the prefix seat.** An
  `epitaph:` in the envelope is what distinguishes a *result deposit* — a
  child's terminal, arriving because this agent dispatched it — from a message
  someone chose to send, so the seat reads `<sender> ended: <epitaph>` instead
  of `<sender>:`, and the wording comes from the one mapping that owns it
  (`inboxview::Epitaph::label`, also the Inbox tab's) rather than being
  invented at the row. This is the §11 glyph doctrine applied to words: the
  fact is in the always-visible slot, not behind a hover or a Raw toggle.
  Before it, a stopped child's deposit rendered as `(no message body)` under a
  hundred-character descent id — a blank line from a stranger — and the reply
  it provoked from the parent had no visible cause on screen (bl-71e8,
  diagnosed from the operator's `energize` transcript). A message with no
  epitaph is unchanged in every respect. The knobs
  `transcript_expand_responses` (default `true`) and
  `transcript_expand_others` (default `false`) are the operator's two
  automatics, and they live in `ui.json` (§4.1) because that is yog's only
  durable UI-state artifact — no yog config file exists and none is invented
  here. The RAM set (§5.3) holds **explicit overrides only** — the
  `ui.json.collapsed` discipline applied to viewport ephemera — keyed
  `tx/<entry filename>#<block index>`. This dissolves "state on arrival" as a
  special case: there is no arrival event to hook, and a row that appears
  mid-frame is already in its auto-state without anyone noticing it appeared.

  **Every speaking row wears its role on a left-edge stripe (bl-3acb).**
  Operator input, model response, and the other forms of inbox item must be
  easier to tell apart at a glance. The answer is ONE
  mechanism, applied uniformly — a thin vertical stripe at the row's left
  edge, before the fold toggle — never a different trick per kind, and the
  **role is what the committed bytes assert, never a content inference**: in
  `.md` sender space the token `user` is reserved for the operator (ARCH
  §2.11), an `epitaph:` marks a result deposit (§2.6), any other sender is a
  peer, and model output is the `.json` model-id origin plus the live tail.
  Four roles, no minted hue (the single colour authority below): **user** =
  gate violet (yog's own — the operator's hand at the gate), **model** =
  spectral blue (already "a model call producing text": the live tail, the
  `InFlight` badge), **peer** = brazen bronze (already "another agent's
  doing": pending mail, the subagent badge), **result deposit** = tarnished
  brazen (already "finished for now") — so user and model separate instantly
  and the third-party kinds read as one bronze family, split within it only
  by the epitaph the parse genuinely distinguishes. Machinery rows (thinking,
  tool calls, tool results, raw bytes, the turn aggregate) wear an **empty
  seat of the same width** — nobody is speaking, and the toggles stay
  aligned. `theme::role` is the one home (`Role`, `message_role`,
  `role_badge`, `role_stripe`): the transcript rows and the inbox-composer's
  pending queue both paint through it, so a message looks the same pending as
  it does delivered — one spelling of the mapping, two seats reading it. The
  stripe paints no glyph, so per the glyph doctrine the words ride the
  mapping and every stripe hovers them (the discoverability invariant). It is
  styling only: no reordering, no relabeling, and the crossing rules and turn
  rollups keep their own seats untouched.

  **A step in flight expands itself (bl-1f21).** The surface shows a lot
  happening while it is happening — streamed thinking directly visible, for
  one — and when it is done and the agent is responding, just one line before
  the response.

  So a row that *is* happening — the live streaming tail, and a tool call no
  result has retired — auto-expands whatever its class knob says, and
  completion returns it to that class auto-state. In-flightness was already a
  **query** (the virtual tail entry; a committed `tool_use` no committed
  `tool_result` names), so this is one more input to the same pure function,
  not a state to store and not an arrival to notice: `expanded = (in flight OR
  class knob) XOR overridden`. The operator's own flip still wins, in both
  directions.

  **A finished turn's machinery rolls up to one aggregate row (bl-1f21).** A
  **turn** is derived, never stored. Delivered messages delimit the row
  sequence — a message *to* the agent is the other half of the exchange, never
  a step inside a turn — and within a segment the turn's **answer** is its last
  row, when the model ended by talking. Everything before that answer is the
  turn's machinery (thinking, tool calls, tool results, and the model's own
  intermediate remarks), and it collapses to ONE row:
  `⚙ 9 inference calls · 14 tool calls · 3 thinking blocks`. Opening it puts
  those very step rows back into the projection, each still folding on its own
  — folds all the way down — and a shut turn leaves them out of it entirely, so
  vertical space is the measure and nothing is painted merely to be hidden.
  Three conditions gate the rollup, each read off rows the projection already
  built: the turn **ended by talking** (an unfinished turn keeps its steps on
  screen — that is the work the operator came to watch); **nothing in it is in
  flight** (the live turn is the show, above); and it holds **at least one
  inference call** (a run of stray entries is not a turn — which is also why
  the aggregate line can never come out empty). The aggregate row is machinery
  like any other row of machinery, so `transcript_expand_others` rules it and
  **no new knob is minted** (§4.1 severability): that knob ON opens every
  finished turn and every step inside it. Its key is
  `tx/<turn's first entry>#turn` — reproduced exactly by the stateless
  re-read, and unable to collide with a `#<block index>`, which is a number.

  **The aggregate says only what the committed bytes carry (bl-1f21).** The
  line counts what the entries *are*: distinct model entries (the inference
  calls), `tool_use` blocks, thinking blocks; a term that counts zero goes
  unsaid. A badge claiming more than its data knows is a filed bug class
  (bl-8433). **Token counts joined the line the moment the bytes carried them
  (bl-8b3c):** lernie ≥0.0.4 seals the provider's `usage` report beside
  `content` (`{"content":[…],"usage":{"input_tokens":5,…}}`), so the aggregate
  sums each committed counter across exactly the entries its inference count
  covers and states each nonzero sum verbatim under the counter's own name —
  `3150 output tokens` — never estimated, never derived (the operator's
  example asked for "thinking tokens"; no committed counter says that, so no
  line does). A legacy bare-array entry contributes nothing, and its absence
  of usage is the general path: a turn with no reports states counts only,
  today's line. **A mixed turn** — some counted entries usage-bearing, some
  legacy — suffixes each token sum with `+` (`2000+ output tokens`): *at
  least this many*, because a partial sum must never pose as the total.

  **Every crossing leaves its line, and the line is the notch (bl-929d,
  bl-1802).** The operator's ruling (quoted in full at the inbox-composer
  below) ends: *"since that's a commit boundary, that line, such as it is,
  should persist, these lines (faint, gray, with a commit id on the right side)
  should persist through the entire chat."* So the transcript renders one
  **boundary rule** — faint, full row width, commit id right-aligned in
  short-oid form — per **operable commit**: one per step, at the row where that
  call's reading began. The **id is that step's `meta.json` `commit`**, the
  branch tip at step-start (§5.1 #29), which is bl-98da's exact spine read,
  reused not re-derived — and since bl-1802 the rule carries that step's whole
  gesture: click to pin, click again to release (the spine above). Two drains
  with no call between them are **one** crossing under one line: both batches
  entered the same prompt, and a line apiece would claim a boundary no model
  call observed. **Absence of a commit = no line**: a step with no `meta.json`
  yet (the call is in flight, or died before meta) draws nothing — the in-flight
  strip owns that interval, and the line materializes when the commit does, on
  the ordinary §7.2 re-derivation. Nothing is animated and nothing is stored;
  the line is as reproducible as the rows around it. Hover explains the seat
  (bl-68ac): what the id is and what clicking does, never how it was derived.

  **What bl-1802 corrected, because it had been wrong since bl-929d.** The
  first version paired *the i-th maximal run of delivered `.md` entries* with
  *the i-th step*, arguing that drains and steps serialize under lernie's
  executor lock. Serialization gives order, not a bijection: a lernie step is
  **one model call**, and a tool loop runs many steps behind one delivered run
  (lernie ARCH §2.3 step 3 — the boundary drain lands delivery commits only
  when the inbox holds something). So from the second tool-using turn onward
  every rule wore the read state of a call several messages earlier, and
  `rail::pin`'s cut — which counted runs the same way — cut the pinned
  transcript in the wrong place. What pairs exactly is **one sealed
  model-output entry per completed step**: a call that reaches `Finish` commits
  `messages/NNN-<model-id>.json` (lernie ARCH §2.3 *The transcript writer*) and
  one that errors, is killed or is still open commits nothing, so the
  transcript's model entries are the `Framing::Complete` steps in step order,
  one for one. `rail::place` is that walk, and it yields both halves at once —
  the row each rule paints above and the entry count its pin reads to — so the
  line in the chat and the fold behind it cannot disagree. A call that sealed
  nothing takes a seat only when it is the **last** step (its read state is the
  tail of the chat, which is where a child dispatched right now hangs its card);
  a superseded one produced no output to sit above and takes none.
- **Steps** — `steps/NNN` **headed table**: framing status **said in words beside
  the glyph** per the glyph doctrine below — `✔ complete` / `✖ failed` / `■ no
  clean end`. Three, not four: §4.4 framing cannot separate a kill, a crash and a
  call in flight (indistinguishable on disk, §2.9), so the row claims only that
  the step never ended clean (bl-b88e corrected this line, which had promised an
  `in-flight` word the enum has no variant for). Plus the §7.3 no-response
  wound, which outranks the framing read and takes the badge's seat with the
  ichor ✖ and its own sentence.

  **Every column is named, and the name is the only route to the value**
  (bl-3ffc). The list used to paint seven bare values in a row — a badge, three
  figures, a sha and two timestamps — with nothing on screen saying which was
  which; it is now a grid whose header row is the column table itself
  (`steps_view/columns.rs`), header + hover explanation + cell in **one home**,
  for the same reason glyph/hue/words share one: two carriers of one fact drift,
  and a field reachable without its name will eventually ship without it. The
  seven, left to right: **Outcome** (the badge), **Step** (the sequence number,
  ▶ marking the drilled-in one), **Attempts** (finished tries inside the step —
  the §4.4 segment count), **Tokens** (all four counters summed, the same figure
  the budget line spends), **Commit** (branch tip at step-start, short),
  **Started**, **Ended**. Figures are bare under their heading rather than
  self-suffixed (`15`, not `15 tok` — the heading is the label, and a value that
  restates it is the same drift in miniature). An absent value still takes its
  cell, so the column below stays under its own heading. Each heading carries a
  plain-sentence explanation on hover — the `Workspaces:` label's idiom (bl-2d87,
  §3.1): what the column *is*, for an operator meeting the word cold, never how
  it is computed. Drill-in — the same treatment, since the five names are the
  on-disk file names (§2.3) and say nothing to a reader who has not met the spec:
  a **`Records:`**-labelled picker over
  meta.json, request.json, response.json event list, staging.json, per-tool
  input/output (each tool call's opaque id labelled `call` and explained on
  hover) — all rendered through **jsonview**, a small hand-rolled pure
  collapsible `serde_json::Value → row tree` widget (zero-dep, uniformly used,
  fully testable): every byte inspectable. A record is one of three derived
  facts, never a stored one: parsed, **absent** ("(absent)"), or **unparseable**
  — and the last renders the **error row** (`unparseable JSON — bytes verbatim
  below`, ichor, the §7.3 wound's grammar) *above* its bytes. Both halves of the
  promise: nothing is summarized away, and the reader is told the file is broken
  rather than left to mistake it for prose (bl-307f). **Raw** flips those trees
  to the record file's own bytes (bl-1ff1) — a `serde_json::Value` loses key
  order, spacing and number spelling, so the tree can never answer "what does
  the file *say*", and `Doc::Json` therefore keeps the bytes it parsed from
  beside the value. Under Raw the [`UNPARSED`] framing drops away, since that
  row is the parsed view's word *about* the file and Raw is the file; a record
  with no bytes still says "(absent)" rather than painting a blank.
- **Inbox** — deposits with from/deposited_at/epitaph; explains `✉n`; Flush =
  `lernie scan`. **Raw** shows each deposit file's name and its bytes envelope
  and all — the same `---\nfrom: …\n---\n` frontmatter the parsed view turns
  into the `✉ from · at` header, which is exactly what makes the toggle worth
  having here (bl-1ff1). The listing therefore carries each file's name and
  bytes beside its parse (`inboxview::InboxEntry`), the transcript entry's own
  shape.
- **Files** — the agent worktree read-only, bounded previews: goal.md,
  soul.md, summary/, skills/, descriptions/, work products. **No Raw toggle,
  because the preview is already it**: the bytes as they sit (bounded at
  `PREVIEW_CAP` = 64 KiB with the cap said outright, a NUL-bearing file
  declared binary rather than mangled). There is no parsed projection here to
  escape from.
- **Config** — governing config files with "policy frozen at `<short-oid>`";
  links to the workspace config editor (§9.3). **No Raw toggle, because the tab
  parses no file**: it derives a commit from git ancestry and lists that
  commit's tree. Its contents stay unreachable from this tab until the history
  rung pins the whole inspector to a commit (VISION V1) — a per-frame `git
  show` per listed path behind a checkbox would re-take the per-frame git read
  bl-ee0a removed from §9.3, at N× the cost.
- **Work** — **what the agent actually changed** in the project it works in
  (§5.1 #32, VISION §4.10, bl-3746): per ball this workspace holds, the exact
  range `target..source`, both commits at short-oid width, and one selectable
  row per changed file with its added/removed counts (a binary file says
  *binary*, never zero lines). Picking a row reads that file's patch below,
  bounded exactly as a file preview is. **No Raw toggle** — the patch already
  *is* the bytes git wrote. **The rail's pin does not reach this tab, and the
  pinned banner does not claim it**: a pin is a commit of the *conversation's*
  repo, and the project commit each step read is VISION §4.10 item 4's
  per-step OID, which nothing writes yet. It is the only tab whose subject is
  a repo other than the conversation's, which is why it sits last and why it
  is keyed on the workspace rather than on the selected agent — the
  conversation-to-directory binding is lernie's working-directory mark, which
  yog now seeds typed at creation (bl-6654) — but only per *conversation root*,
  and an agent's own `cd` may move it after, so a per-conversation answer read
  off this tab would still be a guess. **Nothing here mutates**: it is a git read, and the range it prints
  is the command an operator can run in a shell.

**Bottom accessories (`TopBottomPanel::bottom`, stacked — innermost first
below, which is the reverse of the order they are shown in). Three of them are
conversation-scoped and one is not, and since bl-c038 the seat says so
(the ruling: the conversation's chat line belongs in the
conversation box, rather than across the entire bottom): the in-flight strip,
the goal box — the inbox-composer (bl-929d), or the pending start draft holding
its seat — and the conversation's **settings rows** (the model row, the context
line and the spend figures, moved off the header by bl-2e18 —
altitude 1 above) — dock *inside* the conversation pane (`show_inside` on the
center, same outermost-first stacking), so the input sits at the bottom of the
conversation it feeds and the navigator column runs full height beside it. Only
the activity accessory, world-level ops chrome, remains a window-spanning bottom
panel.**

**Top to bottom on screen, that is: transcript, in-flight strip, goal box,
settings rows** — and the last of those is the band-order ruling, which
supersedes bl-2e18's ordering clause (*"between the goal box and the in-flight
strip, so what the conversation runs on reads beside where the operator talks
to it"*). The ruling: the work directory, the budget, the context and the model
selection must not sit between the input bar and the chat — those four elements
belong below the input box, not above it.

The four elements named are exactly the settings band, so exactly one band
moves. What survives untouched: bl-905f's strip, which is not among the four and
whose own ruling is the reason that seat exists; bl-c038, so the whole stack
stays inside the pane; the outbox, named as the exception and already riding
*inside* the goal box's panel above the draft rather than as a band beside it
(the inbox-composer, below); and bl-2e18's other ruling — settings belong at the
bottom of the conversation rather than on its header — along with the internal
order of the rows. **The budget follows the order** (§11 rule 5,
`crate::layout::share`): claim order is creation order, so the settings band now
asks first, and its ceiling holds back the goal box's own floor — a pane that
cannot pay both stops paying the rows rather than squeezing the box. Measured at
420x320 with the activity trail open, an unreserved budget left the composer
under 30 pt and painted its target line across its own draft. The bands, then:
- The **in-flight strip** (bl-905f) — one pulsing line hard against the chat
  tail, innermost of the stack, present only while something is in flight in the
  open conversation. **The band-order ruling does not reach it**: it is not one
  of the four elements named, and its seat is itself a ruling (stated in full
  at the live-activity class above) whose whole point is that the line
  is where the eyes already are.
  The live-activity indicator's third seat; its content, its wording and the
  elapsed each class either derives from a structural start or deliberately
  omits are ruled above.
- The **settings rows** (bl-2e18, re-seated by bl-58e4) — the conversation's
  config-shaped rows, seated at the **pane's bottom edge, below the goal box**:
  the §3.5 spend
  figures, then — directly under them — the **context-fullness line** (bl-a48b,
  §5.1 #35: `context N%` with the prompt read, the declared window and the model
  weak beside it), then the §9.4 model row — the two dropdowns themselves, with the
  picker's remaining extras expanding under them on `m`. The order is the
  ruling's own — the figures state what has been spent, the model row states
  what is spending it and is the one row that is itself a control, so it sits
  nearest the box the gesture is aimed from. The context line takes the seat
  **directly under the spend it is not**: adjacency is what teaches the two
  apart, since "18 000 tok" and "24%" are otherwise read as one figure said
  twice (spend is the whole descent's cumulative burn; fullness is this
  conversation's one current prompt). It is **absent, not zeroed**, whenever
  nothing measured can be said — no step taken, no model on the step, or no
  declared window — so a row that is present is always a row that means
  something (§5.1 #35). **An
  empty selection is the same seat, not an empty one**: with no conversation
  the rows are the birth-config block (bl-824e, re-seated here), one branch on
  the selection rather than a second surface. The region is **bounded at half
  the pane and scrolls past it** — the fold line's own ceiling, reused rather
  than restated, because an accessory that can expand a picker inline is
  exactly the one that eats its own pane (QUALITY G4; the over-subscribed
  bottom stack is bl-9551's subject). Two mechanics follow from that seat and
  are load-bearing, not incidental: the panel is sized by its content while a
  scrolling region is sized by what is *available*, so the region is handed
  the cap outright — left to read the panel's own last-frame height the two
  lock each other at whatever the first frame was, and an opened picker paints
  into a clip rect two rows tall.
- The **inbox-composer** (bl-929d, overhauling the bl-c038 composer's
  surface but not its seat, verbs, or drafts), docked at the conversation
  pane's bottom whenever a non-replay workspace is focused **and no start
  draft is pending**. The ruling: the operator's input *is* the inbox. Typing
  types into the inbox; items arriving land in that inbox above the text box.
  A horizontal rule rises as more items pile in, or as the operator types
  enough to push it up — everything but the input carries a fold arrow — and
  when the prompt goes out the rule snaps back down, making it plain that the
  input was pushed "over the line" into the prompt. That rule is a commit
  boundary, so it persists: faint, grey, with a commit id on its right side,
  through the entire chat.

  **The surface is a queue with the draft as its last item.** Below the fold
  line (the horizontal rule), oldest first: every pending deposit in the
  target agent's inbox (`inbox/<id>/*.md`, §5.1 #11 — the same derivation
  the `✉n` accessory and the Inbox tab read; a third seat, never a second
  representation), then the text box. Each pending item is one line — the
  transcript's density idiom, `✉ from · at` plus the clipped first line —
  with the jsonview `▶` fold arrow; only the input has no arrow, exactly as
  ruled. Each pending row also wears the §11 **role stripe** (bl-3acb, ruled
  with the transcript above) at its left edge, from the same `theme::role`
  mapping over the deposit's own asserted sender and epitaph — the row's role
  identity is decided before delivery and unchanged by it. The rendering says
  what the substrate does: everything below the line enters the *next* prompt
  together, your words and the waiting mail alike.

  **The typed draft is a yog draft, and becomes a deposit only at send.**
  The ruling: deposit at send. Typing in a box does not hit disk, and it does
  not auto-trigger the agent on turn. That is the one lie of this system —
  what is typed is not *actually* there until Enter. The lie is the honest, intended semantics, not a compromise: the
  input renders in the queue as if it were an item, and deliberately is not
  one until Enter. The loser — typing writes a live lernie deposit — lost
  three ways, each structural. (1) **A deposit is a send, not a draft**:
  lernie's deposit is create-only and its delivery drain is unconditional
  (ARCH §2.11) — a live driver would rename a half-typed file into the
  transcript at its next step boundary, converting crash-*survival* into
  mid-typing *missend*; there is no held deposit, and inventing one is a
  second inbox representation that drifts. (2) **Half the targets have no
  inbox**: the box's other target is `new-conversation-in-<workspace>` — no
  agent id, no `inbox/<id>/` — so the fact would need a RAM home there
  anyway, two homes by construction. (3) **I2**: yog never hand-writes
  workspace internals; deposits go through lernie verbs, and the verbs offer
  no mutable draft because the §5.3 carve-out already homes one. So the
  draft stays the existing per-target RAM fact ("text typed in a box can
  live in RAM until sent" — crash behavior unchanged, ruled at §5.3/§13.1),
  and Enter gives it its one durable home through the unchanged verb:
  `lernie message` (deposit + probe/launch) or `lernie prompt` (§8.2).

  **The fold line is derived, never stored.** Its position *is* the region's
  content height: pending items (one line each, folded) plus the draft's
  wrapped height, above a floor of the bare input row and below a cap of
  half the pane (past the cap the queue scrolls, tail-anchored like every
  end-is-new surface — `src/tail.rs`, one home). An item landing pushes the
  line up one row; typing wraps and pushes it the same way. The **empty
  inbox is the general path**: zero items plus an empty draft renders the
  same rule at its floor — the input's top border — not a branch; there is
  no "has pending" flag anywhere, only a queue whose length may be zero.
  Arrivals surface exactly as every fact does — the deriver's off-thread
  enumerate over the watched `inbox/` root (§7.1) publishing a snapshot;
  the frame renders it, adds no IO, and notices no arrival event (the
  state-on-arrival dissolution above applies verbatim: an item present in
  the snapshot is already in its auto-state).

  **The snap, and what it claims.** On send the rule animates down to its
  floor: the items and the input visibly cross into the transcript's side of
  the line. The animation is render-layer viewport ephemera (§13.1) —
  easing between two snapshot states, nothing stored, nothing claimed
  durably — and its **trigger is structural, not gestural**: the pending
  count dropping because delivery commits landed. One path therefore covers
  every drain — your Enter, a live driver's own step-boundary drain, a
  `lernie scan` flush, another instance's send — and the snap can never
  show a crossing the substrate didn't make. Crossed items leave the queue
  and re-enter as delivered transcript rows in their auto-state; their
  pending fold overrides die with the pending row (a deposit and a
  delivered entry are different facts under different keys, §5.3).

  **The cut is the commit, and yog never defines it.** What crossed into a
  prompt is exactly the set of files lernie's drain renamed and committed
  (the delivery commits, ARCH §2.11); what is pending is exactly what still
  sits in `inbox/<id>/`. An item landing while the prompt fires joins this
  crossing if the drain caught it and the next pending set if it didn't —
  yog holds no membership claim to get wrong, it renders delivery. The
  snapped-down rule and the persistent gray line are the same seam seen
  twice: the live rule at its floor is the composer's, and the transcript's
  boundary rule (ruled above, with the ensuing step's `meta.json` commit as
  its id) takes over the moment the commit exists. Between the two — sent,
  not yet stepped — no line overclaims; the in-flight strip is the
  interval's voice.

  **The in-flight strip's seat is preserved, explicitly** (bl-905f,
  re-ruled here): **hard against the chat tail**, innermost of the whole
  bottom stack — above the fold line, with the transcript's present, never
  inside the pending region. The strip is telemetry about the turn in flight;
  the region below the line is the future, and a pulse filed among pending mail
  would read as an item. Neither re-seating of the settings rows has moved it —
  bl-2e18 put them between the strip and the goal box, bl-58e4 put them below
  the goal box instead, and through both the strip kept the *chat's* edge,
  which is the only edge its ruling was ever about.

  **No new control crosses the boundary** (bl-8aab): Enter fires the same
  targeted verb, Flush stays `lernie scan`, and the only new interactivity
  is the pending rows' fold toggles — the existing jsonview idiom over
  render-local overrides, no `Action` variant, no new verb, no new knob.
  Send is Enter; Shift+Enter inserts a newline (the key contract; filed and
  implemented independently of this ruling).

  **A draft that starts with `/` is a command, not a message** (§8.5's line,
  bl-ec8f) — and that is still no new control: Enter runs the drafted
  gesture, the one Send button re-labels to Run, and the answer (the reply's
  own JSON, or the refusal) renders under the box until the next line or the
  next keystroke retires it. `//` says a slash and means it. The rule holds
  because a line *is* the boundary crossing, not a widget beside it: nothing
  is typable here that a click cannot already fire.

  The rules the overhaul does *not* touch, restated as still binding: the
  pending §8.1 goal draft is a goal box too, so it **takes this seat** rather
  than stacking above it (bl-6ad8) — two live boxes, each with its own greyed
  name preview, gave Enter no unambiguous target, and one box with one Enter
  is the S0 rule this whole surface is built on. Cancel/Escape hands the seat
  back, with the composer's own text intact: it lives in the per-target draft
  map below, which the start pane never reaches into. A start whose Send fails
  banners where the start was offered — the roster's balls section (§7.3), the
  surface the ▶ Start row itself is on — not under a box that has taken the
  composer's place. **That is now enforced, not merely stated** (bl-48f8): the
  ops row carries an `origin` (§4.2) and this composer paints only
  `conversation` rows, so a ball-rung failure cannot reach it however the
  seating falls. The composer's own banner is likewise its own — a failed
  `bl close` from the ball-action row beside it is a *ball* op and banners in
  the balls section, one seat per subject (§7.3). The target follows the
  selection — a selected
  agent ⇒ **message** (the resume gesture); none ⇒ **new conversation** (a
  detached prompt into the focused workspace; §3.4's rungs, founding `home`
  only when
  no workspace exists, §3.1). **The composer names its message target by the
  conversation's display name** (bl-2f30): the `→ message <display name>`
  target line and the box's hint are two spellings of one fact, both from the
  §3.3 one function — never the raw agent id alone (the id survives weak in
  the center header). **Enter fires the targeted verb** — the S0/S1 gesture,
  one box, one Enter — with the greyed name prediction above the box when the
  target is a new conversation (§3.3; a message to an existing agent predicts
  nothing — its name was minted at its start). **Shift+Enter inserts a newline
  instead of firing** (bl-4515): the box is multiline and grows with the
  draft, and any other modified Enter is inert — the combo plane stays free
  for bl-a33d's send-and-interrupt.

  **↑ at the box's top row recalls what you already said here; ↓ at its bottom
  row comes forward, and past the newest hands your draft back** (bl-f908) —
  the gesture codex and Claude Code both bind, in yog's terms. It is the box's
  own pair of keys, exactly as Enter is (the bullet below): the bare arrows are
  already suppressed while a text box holds the keyboard, so the table is not
  touched and no plane is spent. **The history is derived, never stored**: the
  target's own turns, read from the two seats this composer already reads — the
  pending deposits above the box (§5.1 #11) ahead of the delivered transcript
  (§5.1 #12, through the inspector's per-snapshot memo, never a second
  `messages/` read) — each filtered through the one §11 role derivation
  (`theme::message_role`), newest first. No session log, no `ui.json` key: a
  restart pages the same history, because *the history is the conversation*. A
  new conversation has neither seat, so it recalls nothing — the same
  derivation at zero items, not a case.

  **The caret gate is the whole rule, and it holds on entry and continuation
  alike**, so there is no browse mode and no mode flag. A recall parks the
  caret at the end of what it brought back, so a one-row prompt sits on the top
  row *and* the bottom row (one key per step) while a multi-row prompt walks
  the caret up through itself first and pages only once it is at the top —
  readline's own behaviour, and what keeps the arrows usable *inside* a
  recalled prompt. "Top row" is the **visual** row, so a long wrapped draft
  does not swallow the gesture. **Leaving the recall is a derivation too**: when
  the draft is no longer the entry the recall put there, the operator edited it.
  That one check is every exit at once — typing over a recalled prompt (it
  becomes the draft, and the displaced one is rightly forgotten), sending,
  switching targets, and the history growing under a landed send — so no call
  site resets anything and the recall holds two fields (how far back, and the
  draft it displaced), both §5.3 RAM.

  **One box, one draft *per target*** (bl-a69a): the buffer is keyed by what
  the box is pointed at — `new-conversation-in-<workspace>` (the empty world's
  bootstrap box being that same key with no workspace yet) or
  `message-to-<agent>` — so selecting a conversation shows *its* draft and
  selecting back restores the one you left. The bug this closes is the verb and
  the buffer disagreeing: a goal typed for a new conversation stayed in the box
  while the label became `→ message <name>`, and Enter would have deposited a
  fresh start's text on an unrelated agent. A verb that re-labels itself with
  the selection must **re-key** with it; the rule is the industry norm (every
  chat app keeps a draft per conversation) and it is the same shape the §8.1
  new-ball form already uses, keyed by project. The start pane's editable goal
  is not this box at all — it is the pending start's own field (§3.4), separate
  state that a selection cannot reach. Still RAM until sent (§5.3): the fix is
  the key, not persistence.
  The dir (path-rung) field, Stop (+children checkbox), Scan, and the ball
  actions (Close/Release/Move, §8.2) ride beside it unchanged.
- The **activity accessory** — the demoted ops pane: one collapsed chip
  (`activity · N ops · M failed ⚠`, the **live**-failure count in ichor when
  M > 0 — §6's retirement rule, so a failure a later clean run of the same verb
  superseded is not counted, nor is one the operator has acknowledged: `M` and
  the `· K drift` count both read §4.2's ack watermark, while `N ops` never does,
  because it names the rows the expansion lists) expanding on demand to the
  `ops.jsonl` tail; a row
  expands to the full entry — argv, cwd, exit, stderr — because a trail that
  hides *why* is not a trail (§7.3 failed-action row); a retired failure still
  renders its ⚠ row, ash instead of ichor. Every row's outcome — clean, failed,
  retired, detached (bl-8433: a handed-off spawn nobody has observed the exit
  of, its own outcome rather than a false "clean") — comes from the one badge
  mapping (`theme::op_badge`, glyph + hue + words) and is said on hover; the
  chip says it outright (bl-51cb). A row's
  leading column is `OpRow::when()` (bl-61db), not the raw `ts` field — `ts` is
  unix seconds as a decimal string (`ui_state::Clock::stamp`'s convention), and
  read raw it was unreadable (`1785630266`). `when()` renders it through
  `ui_state::iso8601_extended`, the same ISO 8601 extended spelling
  (`YYYY-MM-DD HH:MM:SSZ`) and the same `format_iso8601` assembly call the chat
  header's when-seat uses for its id-derived stamp (bl-16da) — one
  human-timestamp grammar for the crate, not two. The epoch is a lossless
  round-trip of what `when()` already shows, so it earns no seat in the
  expanded row either. Default collapsed; per-instance viewport state (§13.0).

  **Two verbs over the trail itself, both inside the expansion (bl-c417).**
  **Dismiss** — the §4.2 ack; offered only while something is actually alarming
  (`AppModel::has_alarms`), since a control that would write a line and change
  nothing should not be on screen. It is the same control, the same words and
  the same hover as the one on the §7.3 banner, read from one home
  (`opslog::operator`), because an operator who meets a control twice must not
  meet two spellings of it. **Clear trail** — the §4.2 clear. Neither rides the
  collapsed chip: the chip sits at the bottom of every frame, and the guard on
  the destructive verb is precisely that reaching it costs opening the pane
  first. Each explains itself on hover (bl-68ac), and the clear's hover names
  the destruction outright — that the rows are discarded and kept nowhere else.
- The **Config tab** (focused by the left-panel entry) holds the brazen /
  lernie-global / config-branch surfaces (§9) — **controls over facts** since
  §9.5: one typed row per setting, raw text only where §9.5 justifies it.
  **It is a tab focus, not a toggled overlay (bl-1ca2).** Several surfaces —
  config among them — were interface overlays toggled on rather than tabs, and
  since they cover everything they should simply be tab focuses. The rule is
  general: **a surface that takes the whole center
  is a tab — a named peer the operator focuses and leaves by ordinary
  navigation — never a mode toggled on over everything.** Config is the named
  case; the Login pane (§8.3) and the world-search results (§8.5) are its
  kin, reseated the same way (bl-1ca2 seats all three). The two modals
  (§3.1's name form, §3.6's confirmation) are not of this kind: a modal owns
  the frame for one small form, and the modal invariant above is untouched.
  Opening the tab is the gesture that reads everything the pane renders
  (§9.5), so no frame pays for a file, a git call or a brazen query.

  **The seating, landed (bl-1ca2).** The center is headed by a **strip of tab
  focuses** — `Conversation`, `Config`, `Login`, and `Search` while there is an
  answer — of which the center shows exactly one. One enum carries which
  (`keymap::CenterTab`, RAM per §13.1); the three surfaces used to hold a
  `bool` apiece, none aware of the others, which is precisely how a mode came
  to paint over the conversation. What each reseat cost:

  - **Config** swapped the whole `CentralPanel` behind `ConfigState::active`.
    That flag is gone; the tab focus is the one gesture, and it still carries
    §9's freshness re-read (`shell/center::focus` is its only home, so a second
    carrier cannot ship a stale pane).
  - **Login** was a `ui.collapsing` *inside* the left panel — unfolding it put
    ten provider rows and a live command stream into a column sized for
    conversation titles. It is a center tab; the auth-failed banner still
    renders the same section inline where the wound is (above), which is one
    machinery in two seats, not two surfaces.
  - **Search** grew out of the composer, pushing the conversation off its own
    pane — the same defect in a smaller frame. It is a tab **offered**, not a
    permanent peer: the results are a view of the published answer (§8.5), so
    the tab appears with an answer, asking focuses it (the ask is the
    operator's one gesture — the answer must not need a second), and a
    `/search` with no text clears the answer, retires the tab and drops the
    center home. The vanishing *is* the dismissal; there is still no "search
    mode" to enter or leave.
    **Offered on the question, never on the hits** (bl-648a): the answer
    carries the needle it answers (`Found::needle`, set by `search::run`), and
    the tab is offered while there is one (`Found::asked`). A search that
    matched nothing is an answer like any other — it paints
    `no matches for <needle>` and the paved path — because reading intent off
    emptiness made a zero-hit search retire its own tab and reseat the
    operator mid-read, leaving a frame byte-identical to never having
    searched (QUALITY H2). An empty query still yields an empty needle, so
    the clearing rule above holds unchanged and with no case of its own.

  Three properties hold of every tab, and they are what "ordinary tab
  semantics" means here. **Reachable**: the strip is co-visible with whatever
  it heads, so every peer is one click away from every other — the whole
  difference between a tab and a mode. **Keyboard-addressable**:
  Command+Shift+1–4 (the table below). **Dismissable**: Escape comes home
  (QUALITY F3), and it loses nothing typed — a config draft is the editor's, a
  message draft is its target's (bl-a69a), and neither is on the surface being
  left. The **conversation's own accessories** — the composer, the settings
  rows (bl-2e18), the in-flight strip — paint on the Conversation tab and
  nowhere else: they are accessories of a conversation, not of the window, so
  under Config there is no composer *buried*, there is simply none.

**Keyboard bindings.** Every gesture of the three altitudes above is reachable
from the keyboard; the bindings are a pure key → intent table (`src/keymap`,
tested), and the `egui::Key` lift and the dispatch that calls the same effect a
click calls are thin shell glue (`src/shell/keys.rs`), excluded like the rest of
the tree.

Three rules decide what gets a key, so the table never grows by taste:

1. **A key fires a verb on the current target.** The target is the one the
   selection already names — the focused conversation (↑/↓), its bound ball,
   the focused workspace. Nothing needs a second cursor.
2. **A pick may ride the pointer; the pointer is never the only path** (the
   everything-is-keyboard-operable ruling, resolving this rule's old carve-out
   against QUALITY F1, which stands as written).** A
   pick is what a pointer is *good* for — Move's destination workspace, the
   overflow menu's entries, which descent-tree member or ops row to open, which
   step record to drill into — and where the thing picked has an *address*,
   §8.5's line names it outright (`/assign <id>`, `/move <id> <to>`), so a pick
   at the pointer and a name at the line are one gesture with two spellings.
   What this rule refuses is a second *cursor* in this table, not a keyboard
   spelling: every pick must have one — a line address, or a keyboard walk over
   the candidates — and a pick whose only carrier is the pointer is a violation
   (QUALITY F1), swept and closed by bl-478d. The rule bounds **this table**,
   not the surface. **The walk it names is the frame's own** (the focus floor
   below): Tab steps the focus control by control and Space presses what it
   reaches, so a fold, a toggle, a drill-in, a file pick and the notch pin are
   each operable from the keyboard without a binding apiece — which is how one
   invariant answers a whole class instead of growing this table by taste.
3. **A combo may repaint or create; it may never fire a verb at the
   selection.** The combo plane exists to keep working *while a text box holds
   the keyboard* (the suppression rule below), so a combo can land while the
   operator's attention is on a draft rather than on what is selected. Only
   gestures whose worst case mid-typing is a repaint or a new empty thing are
   admitted. Every verb that acts on the existing selection — Start, Stop,
   Flush, Close, Release — keeps its bare letter and takes **no** combo. The
   absence is the safety property, not a gap in the table.

| key | combo | intent |
|---|---|---|
| ↑ / ↓ | Ctrl+↑ / Ctrl+↓ | step the selection through the **visible rows of the focused workspace's conversation list, in paint order** (the unfold ruling above, bl-fa82: a collapsed subtree is skipped whole and the walk never expands), landing via the seen-acknowledgement path (§6). It walked §6's attention-ranked roster across every workspace until that ruling; the cross-wall walk is now the jump's alone. The bare key is suppressed while a text box holds the keyboard, like every bare key — which is where the composer's own prompt recall lives (bl-f908, the inbox-composer above), the box's keys rather than the table's. Since a selection lands the composer (focus discipline below, bl-c21f), a bare step spends its own plane, so the **combo is the walk's continuation** (bl-c21f): the only pairing in this table that exists because its bare twin surrenders the keyboard, and lawful under rule 3 because stepping a selection repaints and fires no verb. The recall's own ↑/↓ take modifier-free presses only, so the two never collide |
| ← / → | Ctrl+← / Ctrl+→ | unfold the selected conversation row (the unfold ruling above, bl-fa82): → expands it, ← collapses it, and ← on a child or a leaf pages the selection up to its parent row. A verb on the target the selection already names (rule 1), paired on the Command plane for the same reason ↑/↓ are — a selection lands the composer, so the bare plane is surrendered nearly always — and lawful under rule 3, since expanding a row repaints a viewport and fires nothing. Bare ←/→ are suppressed under a text box, which is what keeps them out of the caret's way |
| 1–6 | Ctrl+1 – Ctrl+6 | select an Altitude-2 inspector tab (RAM, §5.3) |
| — (rule 3: the bare digits are the inspector's) | Ctrl+Shift+1 – Ctrl+Shift+4 | focus a **center tab** — 1 Conversation, 2 Config, 3 Login, 4 Search (bl-1ca2). Combo-only and on the *shifted* plane: Ctrl+digit already carries the altitude-2 strip, and this is the other one — the same "the other X takes the shifted spelling" the workspace raise rides. Lawful under rule 3, since focusing a tab repaints and fires no verb, which is exactly why it must be a combo: every selection lands the composer, so the bare plane is surrendered nearly always and a bare spelling would be unreachable in practice. Escape is the way back (`Cancel` below) |
| `i` | Ctrl+I | hand the keyboard to the composer — whichever one is painted (pane-docked composer or the empty-world bootstrap box), target unchanged |
| `n` | Ctrl+N | new conversation: clear the agent selection, then `i` |
| `w` | Ctrl+Shift+N | new workspace: the deliberate sphere-wall raise, name typed by the operator (§3.1, §3.4) |
| `s` | — (rule 3) | ▶ Start the balls section's **first ready row** — the row it paints at its top (rule 2 keeps the pick among many a click; the top row is the one the keyboard can name) |
| `x` | — (rule 3) | Stop the selected conversation (+children per the row's checkbox, §8.2) |
| `f` | — (rule 3) | Flush the inbox — `lernie scan` on the focused workspace (§8.2) |
| — (rule 3: the bare plane is Flush) | Ctrl+F | **search** — put the composer on a `/search ` line (§8.5). Combo-only: a query spends nothing, so it is safe mid-typing, and the letter's two planes carry its two meanings |
| `c` | — (rule 3) | Close the focused conversation's bound ball (§8.2; refused exactly where the button is disabled) |
| `r` | — (rule 3) | Release (unclaim) that same ball (§8.2) — Move stays a click, because its destination is a pick |
| `b` | Ctrl+B | fold / unfold the balls section (the persisted §4.1 collapse) |
| `m` | — (rule 3) | open / close the model picker for the focused workspace (§9.4) — a verb on the target the selection already names, so no combo; *which* model then gets picked stays a pointer gesture, per rule 2 |
| `g` | Ctrl+G | organizing view: recent ⇄ by ball (§15 Z9, RAM) |
| `a` | Ctrl+J | activity accessory: collapsed ⇄ expanded (§13.0 viewport state) |
| — (rule 3 inverted: typing owns bare `+`/`-`/`0`) | Ctrl+`+` / Ctrl+`=` / Ctrl+`-` / Ctrl+`0` | whole-UI **text size**: one 0.1 step either way, or reset to 1.0 — written through to the durable §4.1 `zoom`, which the frame then derives egui's live factor from |
| Enter | — | fire the pending start goal — the editable composer's Send (detached prompt, §8.1) |
| Escape | — | the one **put-it-down** key, aimed at whatever is up, nearest first: dismiss the modal that owns the frame, dropping its draft (below); else drop the pending start goal; else return a focused center tab to the conversation (bl-1ca2 — QUALITY F3's "Escape dismisses", kept in tab form). With none of the three up it is the composer's own release to the bare plane, and must not re-grab the box |

**Suppression: bare keys are suppressed, combos are not.** A bare key is
skipped while a text box holds the keyboard (egui's `wants_keyboard_input`), so
typing never steals it and a bare letter is a lawful key: the release gesture is
Escape, which egui spends on surrendering that focus before the table ever sees
it. `Escape` then `i` is therefore the one deterministic "put the cursor in the
composer" idiom from any state — and the drive harness (`scripts/drive/`) steers
by exactly these keys wherever its subject is the window, because a coordinate
regresses on every layout change and a key does not. Where its subject is not
the window it steers by §8.5's line instead; a coordinate survives there only
for a **view**, which has no spelling anywhere by design (STORIES.md, "A gesture
must not ride on pixels"). A **combo runs regardless of text focus**, because Ctrl+N while
composing is precisely when an operator wants it: a combo that stopped at the
composer would be a second spelling of a key that already works, and the combo
plane would have no reason to exist. Rule 3 is what makes that safe, and the two
halves are one decision — the plane that survives text focus is exactly the plane
that cannot hurt you there. `src/keymap` owns both: the suppression predicate is
part of the pure table (`keymap(key, mods, held)`), not shell judgement.

**The focus floor: this table is an accelerator over the frame's own walk
(bl-478d).** The ruling is one sentence: everything is keyboard-operable.
egui traverses its own controls — every `Sense::click`
seat is focusable — with **Tab / Shift+Tab**, and presses the focused one with
**Space** (or Enter). Nothing in `src/shell/keys.rs` lifts Tab or Space, so the
floor is untouched by the table above and the two planes never argue. Three
consequences, and they are the whole of it:

- **Rule 2 holds for every control, not per gesture.** A pick with no line
  address and no letter — which notch to pin, which step to drill into, which
  file to preview, which provider to sign into, a fold, a Raw toggle — is
  reached by walking to it. The table stays what it was: the *accelerators*,
  one press for the gestures worth one, chosen by the three rules above rather
  than by enumerating the surface.
- **Suppression already agrees.** egui's `wants_keyboard_input` is *any* widget
  holding focus, not only a text box, so a bare letter is suppressed while the
  walk is out — which is exactly right: Space and the letters must not both
  fire. Escape drops the focus and returns the bare plane, the same release it
  already was.
- **It is driven, not assumed** (`shell/acceptance/floor.rs`): Tab moves the
  frame's focus control by control, and Space presses what it reached — the
  balls fold, asserted through the durable §4.1 collapse the model answers for,
  so a walk that never walked cannot pass.

**A modal owns the frame while it is up (bl-d921).** yog has two — §3.1's `new
workspace` name form and §3.6's delete confirmation — and while either stands,
*nothing beneath it is reachable*, by pointer or by key. This is one invariant
with one predicate behind it (`shell::modal::open`), spent twice:

- **Pointer.** A screen-sized, non-interactable backdrop is shown between the
  panels and the dialogs. egui's hit test picks the topmost *layer* under the
  pointer and discards every widget below it, so a click at the left panel's
  Config entry lands on the backdrop's layer and reaches nothing. It is
  deliberately not a click target: it neither closes the modal (a stray click
  must not destroy a half-typed name) nor rises above the dialog it sits under,
  which is what a click-sensing backdrop in egui 0.29 would do to itself. The
  scrim it paints (`theme::SCRIM`) is the visible half of the same fact.
- **Keyboard.** `Held::Modal` is a *third* plane in the pure table, not "text
  focus plus something": the bare plane collapses to the single gesture aimed at
  the modal itself. **Escape dismisses it on the first press** — the box the
  modal holds is inside the thing being dismissed, so suppression has nothing
  left to protect, and every other bare key is inert for the same reason the
  pointer is. Combos are untouched, so Ctrl+I still reaches the composer from
  inside the name form.
- **Enter belongs to the box, not the table.** A one-field dialog submits on
  Return, and that Return never reaches the keymap at all — it is the text box's
  own `lost_focus() && Enter`; the composer's Send is the same ownership on its
  multiline box, whose return key is Shift+Enter (bl-4515), so the plain press
  is read as the send while the box keeps focus. **↑/↓ belong to the composer's
  box the same way** (bl-f908): the box takes them off the frame's input before the
  widget is added — so a recall never also moves the caret — and hands them
  straight back when the caret is not at that edge or the history has nothing
  that way, which is what the arrows did here before. The
  §3.1 name form had it and it did not fire, because the form re-claimed the
  keyboard *before* reading `lost_focus` and so handed focus back on the very
  frame Enter surrendered it (bl-d921). A submit egui refuses (a name §3.1 will
  not take) re-claims the box instead, so a refusal leaves the operator typing
  rather than hunting.

Dismissal drops the draft (§5.3 — unsubmitted input is RAM precisely so it can
die with its surface) and hands the keyboard back to the composer, which is the
focus rule below, not a second one. The ✕ and Escape spend the same verb.

**Focus discipline: the keyboard ends up in the composer.** Suppression says
what a bare key does *while* a text box holds the keyboard; this says how it
comes to hold it. yog is a thing you talk to, so the resting state of the
keyboard is the message box: an operator who has just launched, opened a
conversation, switched workspace, sent a message, or dismissed a modal should
type, not hunt for the box first. There is **one** mechanism (`src/shell/focus.rs`)
— a deferred request bit that whichever composer paints next frame consumes,
the bottom one or the empty-world bootstrap. egui focus is per-frame and a
gesture is handled before any widget exists, so "next frame" is not a delay
bolted on: it is the only frame that has a box to hand the focus to. No other
`request_focus` on a composer exists in the tree.

Two rules decide who asks, and there is no third:

1. **A pointer gesture hands the keyboard back.** Opening a conversation (the
   list row, a descent-tree member, the strip's ⏭ jump), switching workspace
   (a tab, an overflow entry), launching, sending, and dismissing a modal all
   end with the cursor in the box. The mouse said *where*; the keyboard's only
   remaining job is *what to say*. Launch is not a special case — the request
   simply stands from `ShellState::new`, which is what let the bootstrap
   composer's once-only `egui::Id` memory flag be deleted rather than joined.
2. **A selection lands the composer no matter the plane it rode (bl-c21f).**
   Opening the app focuses the chat prompt; selecting an agent focuses the chat
   prompt. The rule this supersedes — *a keyboard gesture leaves the keyboard plane
   alone* — protected the roster walk: a ↓ that grabbed the box would spend
   the plane it was pressed on, and a second ↓ would do nothing. The ruling
   decides that trade the other way: a selection is a selection however it was
   made, and the operator who just selected is about to type. The walk's cost
   is real and accepted — a bare step now surrenders its own plane to the box
   it focuses — so the walk's continuation is spelled on the plane that
   survives text focus: **Ctrl+↑ / Ctrl+↓** (bl-c21f; rule 3 above admits it,
   since stepping a selection repaints and fires no verb). Every selecting
   gesture asks on the one bit — the list rows and descent-tree members, the
   strip's ⏭ jump, a followed inspector card, and the walk on either plane —
   and all but the jump reach it by selecting *through* `shell/focus.rs` rather
   than through the model directly, which is what keeps "a selection" one thing
   here instead of a list of them. `i` / Ctrl+I stays the explicit
   request from anywhere and Escape the release — and Escape with nothing
   pending still must not re-grab: with every selection now landing the
   composer, Escape is the one door back to the bare plane.

**The ask survives one extra frame when it rode an arrow** (bl-58e4), and this
is the bit's delivery rather than a second mechanism. egui walks its own focus
floor on a bare arrow key, and a widget that *gains* focus during a frame has
not yet installed the event filter that would claim the arrow for itself — so
the very ↑/↓ that made the selection is also read as "step the floor one control
on", and the keyboard lands on the control **under** the box instead of in it.
Rule 2 was therefore only ever nearly true: `wants_keyboard_input` answered yes
while the cursor sat on Send. Asking again on the next frame — which no longer
carries the key — makes it true outright, and a second ask on a box that already
holds the keyboard is a no-op. Escape cancels a carried ask exactly as it
outranks a standing one; an Escape frame that *also* asks is a dismissed modal
handing the keyboard back, which is rule 1 and is honoured. What made "nearly"
stop being good enough is the band reorder: the control under the box
is now the settings band, whose height settles a frame late, so the control the
floor stepped onto could vanish mid-settle and take the keyboard down with it.

Two consequences worth stating, because both are the rules meeting an existing
one rather than an exception to them. A **send** asks on the attempt, not the
outcome: a click never had focus (the multiline box holds it through Enter,
bl-4515), so a refused send would otherwise strand the draft in a box nobody
is typing in.
And the `new workspace` form **claims** the keyboard on open rather than
holding it — it asks only while the keyboard is unclaimed, because an
unconditional per-frame `request_focus` made that form the one place Ctrl+I
could not reach, out-shouting the request forever.

Ctrl here is egui's `Modifiers::command` — ⌘ on macOS, Ctrl elsewhere — so the
platform convention rides for free on both §10 targets, and no binding is
spelled twice.

**Why these combos.** `Ctrl+N` = new is the one exact convention available, and
it goes to the **conversation**, not the workspace: the conversation is the
everyday new (raising a sphere wall is not, §3.4/Altitude 0), so the workspace
takes the standard "the other new" — `Ctrl+Shift+N`, new window / new incognito
window / new folder. `Ctrl+1`–`Ctrl+5` is the browser and editor tab-select
convention, matching the digits exactly — and `Ctrl+Shift+1`–`Ctrl+Shift+4` is
that same convention applied to the *other* tab strip (bl-1ca2), the shifted
plane doing here exactly what it does for the "other new". The window has two
strips because it has two altitudes, and the two spellings differ by the one
modifier that already means "the other one". `Ctrl+B` is "toggle the side panel"
(editors), and the balls section *is* a fold in the left panel. `Ctrl+J` is
"toggle the bottom panel" (editors) and the downloads shelf (browsers) — the
activity accessory is the bottom panel; the letter parts from `a` because
`Ctrl+A` is select-all and belongs to whatever text box has focus. `Ctrl+F` is
find, spelled exactly, and since bl-3c28 there **is** a find to spell (§8.5) —
it puts the composer on a `/search ` line. It was "deliberately unbound" below
for exactly as long as the concept was absent, which is the rule that list
states rather than an exception to it. `Ctrl+I`
carries no meaning in a plain-text app (italics needs rich text yog does not
have), and `Ctrl+G`'s meaning — find-**next** — is a cursor advanced through
results, which yog still does not have (its results are a list you click, not a
position you step); both are **invented**, chosen for letter parity, and both
are safe by rule 3.
`Ctrl+`+`/`-`/`0`` is the browser's zoom, spelled exactly; yog binds it rather
than leaving it to egui's built-in handler (switched off in `theme::apply`)
because the size has to *persist*, and a binding egui owns writes only to RAM
(bl-42e7). It has no bare plane at all — bare `+`/`-`/`0` are characters an
operator is typing — which is rule 3's logic run in reverse: the gesture is a
repaint, so the combo plane is where it is safe, and the bare plane is where it
would be harmful.

**Deliberately unbound, and why.** Each of these is a decision, not an
omission — and the list was re-audited against the
everything-is-keyboard-operable ruling and survives it: every entry
refuses a *combo* for a concept yog does not have, never a keyboard path for a
gesture that exists, so nothing operable is stranded by it (the moment an
entry's concept becomes real, it binds — the Ctrl+F precedent above). What the
ruling does sweep is pointer-only picks, which are keyboard rule 2's business
(bl-478d), not this list's:

- **`Ctrl+R`** — yog has no reload. §7.2's inotify + 100 ms debounce + 2 s cheap
  sweep + 15 s full sweep means every surface is already re-derived from disk
  ("a dropped inotify event costs ≤15 s of latency, never divergence"); there is
  no cached view a refresh could correct. The concept being absent, the only
  question left is whether a reflexive press is harmless — and `r` is
  **Release**, which unclaims a ball. A destructive verb behind a refresh reflex
  is the worst outcome available, so `Ctrl+R` binds nothing, and no "force
  refresh" verb is invented to give it something to do.
- **`Ctrl+S`** — nothing to save: a draft is RAM until sent (§5.3) and config
  edits land through an explicit Apply (§9). `s` is ▶ Start, which spends a
  model call. Same reasoning, same answer.
- **`Ctrl+W`** — close, in every browser and editor alive. yog has no closable
  view (a conversation is never dismissed, only unselected), and its only
  "close" is `bl close` — a *delivery*. Binding the close reflex to that is
  strictly worse than binding nothing, and letter-matching it to new *workspace*
  would be worse still.
- **`Ctrl+A` / `Ctrl+Z` / `Ctrl+Y`** — owned by the focused text box. Since
  combos are not suppressed, binding one would double-fire: select-all *and* the
  verb. The text box's combos are permanently out of the table.
- **`Ctrl+C` / `Ctrl+X` / `Ctrl+V`** — these never reach the key table at all:
  egui-winit turns them into `Event::Copy` / `Cut` / `Paste` before any
  `Key` event exists (verified empirically on X11/eframe 0.29). Worth recording
  because it closes the one case where convention *did* match a yog verb —
  Ctrl+C ≈ interrupt ≈ **Stop** — on platform grounds rather than taste.
- **↑/↓, Enter, Esc** — not alphanumeric, so outside the pairing; Enter and Esc
  are spent by the composer by design (above), which is the whole reason `Esc`
  then `i` works.

Widget split discipline (unchanged religion): pure view-model modules (no
egui) + pure render functions (headless shape-walk-tested) + interaction glue
(`.clicked()` branches) confined to `src/shell/*`, coverage-excluded alongside
`main.rs`: shell-level clicks are unreachable in the headless harness, so the
split keeps everything a click *calls* covered. The exception, proven by Y13
(jsonview's tested collapse toggle): a self-contained widget whose interaction
is *intrinsic* may own a tested click, exercised under a simulated-pointer
render test — the exclusion is for the shell tree, not a claim that no click
can be driven headlessly.

**Visual identity — the congeries palette (`src/theme`).** Yog-Sothoth
manifests as *a congeries of iridescent globes*; the UI's identity is exactly
that — luminous sphere-hues against a violet-black void. `src/theme` is the
**single colour authority**: every hue the UI paints is a lore-named constant
there (hydra green = liveness/ok, spectral blue = in-flight/streaming, brazen
bronze = pending/warn, ichor red = error, ash = stopped, sigil magenta = the
uncertainty "?", gate violet = yog's own selection/wordmark hue), renderers
import the name and never restate an RGB triple. The module also derives the
whole-app `egui::Visuals` (installed once at eframe bring-up), owns the one
shared in-flight pulse (every pulsing indicator beats in step), maps each
driven integration to its hue (`lernie`→hydra, `bz`→brazen, `bl`→gate — used
by the config-editor headings), and renders both mark seats — the
circuit-triskele mark itself, painted at a medium ~28 pt, + "yog". No PNG is
decoded and nothing is read from an install path: the mark is computed.

**The mark is live — one circle per agent (`src/theme/mark.rs`, bl-b768; seated
by bl-d44e).** On the open conversation's headline row (altitude 1) it is not
decoration but that conversation's telemetry: the **eye**
is its root agent, the **nine node circles** are its
subagents in §2.3 descent order, and each circle's hue is that agent's §5.1 #28b
`Doing`. The coding is fixed: green is nothing, blue is inference, red is tool
calls, purple is thinking, and orange is waiting on the API with nothing back
yet — hydra, spectre, ichor, sigil, brazen, mapped once in
`theme::doing_badge`. Purple is **sigil magenta, not gate violet** (the ruling
below).

Four things make it cost nothing to reason about:

- **Rest is not a case.** Idle is hydra green, which is the hue the mark is
  built from, and an *unfilled* seat is the same green — an absent agent and an
  idle one both say "nothing happening". So the logo is the empty reading of the
  telemetry rather than a second picture, and the empty-workspace wordmark is
  the same walk with its default argument.
- **Hues, never colours.** Every seat is driven through `icon::deep` at the
  phosphor value the mark has always used, so the five states read as one family
  and rest is byte-identical to the shipped icon.
- **The hue never carries the fact alone** (glyph doctrine): the mark hovers a
  worded roster — every seat named by the §3.3 ladder, what it is doing in
  `doing_badge`'s words, and, when a conversation has more subagents than the
  mark has circles, *how many are not shown*. A cap that stays quiet reads as
  full coverage.
- **Nothing animates and nothing polls.** The hues change exactly when the
  snapshot does, and a new snapshot already brings a frame (§7.2); an idle
  window paints once and sleeps.
- **It is seated where its scope is.** The mark's seats are one conversation's
  subtree, so it lives on that conversation's row, not in window chrome
  (bl-d44e, overruling bl-b768's corner). It therefore paints only when one is
  open — the pane returns before its header when nothing is selected — and
  identity, for a window with no conversation in it, is the placeholder's
  resting [`wordmark`] plus the desktop entry's icon.

**The five are chosen for legibility at 3 px, and the choice is measured**
(bl-c16f, amending bl-b768's set). A node circle
is about three pixels across, so hue angle and brightness are the only channels
that survive; the set is picked to be maximally separable *against each other*,
and what a hue is called elsewhere loses to that.

Two are therefore **borrowed on this device**: ichor is the wound hue everywhere
else, sigil the §10 uncertainty mark. Nothing on the mark is ever an error or
an uncertainty, so neither reuse can be ambiguous at this seat, and minting two
more palette entries to dodge a collision that cannot occur would cost the
palette its one-hue-per-fact discipline instead.

The ruling moved **thinking off gate violet onto sigil magenta**. Violet is the
dimmest hue in the palette (luminance 67 against a void of 17) and it is the
wordmark's own hue, painted two pixels to the mark's right — so the state the
operator most wanted to see was simultaneously the hardest to see and
indistinguishable from brand furniture. Sigil is the same perceptual family
(it still reads "purple"), 45% brighter, and its stated job is already to never
blend into a definite state's hue.

**Tools then kept ichor rather than taking the freed violet**, which is the
non-obvious half. Hue-wheel separation does not decide it — both candidate sets
have the same 43° minimum gap. Perceptual distance over the whole five-set does:
min ΔE **65** with red, **49** with violet. The tight pair is what differs.
Red↔orange are close in hue but 79 against 175 in luminance, and that 2.2×
carries them apart where hue cannot; violet↔magenta are close in hue *and* in
luminance (67 against 97), so nothing rescues them. The measure (ΔE76)
overstates differences in the blue/violet region — it is biased *toward* the
rejected set, which loses anyway. And the degrading pair is the wrong one: with
violet it is **thinking↔tools**, the two states an operator most needs apart
(one means "pondering", the other "touching the repo right now"); with red it is
waiting↔tools, both "busy, be patient", where a mix-up costs nothing.

Also **rejected: idle → ash grey** to free hydra green for tools, which would
have made the mark agree with `flight_badge`'s green `⚙`. Rest would stop being
the logo: a grey mark whenever nothing is running reads as broken, and the
empty state *being* the shipped icon, byte-identical, is the best structural
property the design has. The strip keeps its green tools and the mark its red —
a declared divergence, not a drift, and the strip can afford it because it
paints the class in words beside the glyph while the mark has only hover.

**Three emissions, one walk (`src/theme/icon*.rs`).** The mark is walked into a
list of flat-filled primitives once and emitted three ways: rasterized for the
window icon, written out as the checked-in `assets/yog.svg`, and painted live on
an egui layer. All three walk the same list in the same order, so they are one
picture rather than three approximations of one. A **trace** is therefore named
by its centreline and its width rather than by its edges (bl-b768): that is what
it *is*, and it is what lets the two edge-walking emissions ask for ribs while
the painter hands egui the path and lets its own stroker do the joins and the
antialiasing. The painter is the live path because the rasterizer is a
shapes×pixels sweep — tens of milliseconds per image, which §7.2 will not spend
on the render thread every time an agent changes what it is doing.

**The application icon — pinned ends, one free knob (`src/theme/icon*.rs`).**
The mark is a **circuit triskele**. Three circles sit *tangent* to a central
one, 120° apart with one at bottom dead centre; from each, an arm runs 60° of
arc to a further circle, the circles between riding that arc, joined by a trace
of dim casing under a bright phosphor conductor. The centre carries a slit pupil
in the void's own black. Two hues, both derived from one palette entry, on
transparency.

**Both ends of an arm are fixed; the swell is the only free parameter.** A
start tangent to the middle circle, an end `SWEEP` 60° away at `END_R` — and
that pair is what puts the top-right arm's last circle **directly over the main
circle**, since its base sits at −30° and 60° counter-clockwise of that is
straight up. Between them lies an arc whose sagitta is `SWELL`, a fraction of
the chord, and everything else follows from it. Two things come free:

- **Equal legs.** Even spacing along a circular arc gives equal chords, so the
  legs between circles are equal without being constrained to be.
- **A curve that fits the circles.** The seats are *sampled from* the arc rather
  than placed against it, so the drawn trace passes through every centre
  exactly — by construction, not by agreement. Both the straight-leg and the
  curved reading of the same figure are therefore the same data.

**`SWELL` 0.448 is a near-semicircle, not a semicircle.** At that sagitta the
arm leaves its tangent circle at exactly 45° off the radial — which was the ask
— and the arc sweeps **167.4°**. An arc is a semicircle exactly when its
sagitta equals the **half** chord, which is `SWELL` 0.5; that sweeps a true 180°
but drops the departure to 41.8° and costs frame margin (ink to 0.477 against
0.465). The two properties are mutually exclusive, and the swap between them is
this one constant.

**What this mark is not.** An earlier one wore purple, gold and green — the
carnival tricolour — and read as a jester's hat: bulging lobes ending fat at the
rim, twelve beads scattered there like bells. Three fixes, each load bearing:
**two hues** (a third saturated colour was what paid the carnival tax), **arms
that terminate rather than bulge**, and **few, large elements**, because 64 px
is where a taskbar icon lives and anything smaller than a node is noise there.
One consequence to know: at 64 px the conductor is a single pixel wide and never
resolves to pure phosphor — it reads as a bright core in a dim sleeve, and the
test asserts that rather than a colour it cannot reach.

**Everything is compass work (`icon/arc.rs`).** No spline, no easing, no tuned
curve: an arc is named by two endpoints and a **sagitta**, the height of its
apex above the chord, which fixes one circle exactly — radius `(h² + s²)/2s`,
centre `radius − s` back from the chord's midpoint, swept angle
`2·atan2(h, radius − s)`, needing no case for the major arc because `radius − s`
simply goes negative there. A `stroke` is a constant width run along such an
arc; a **lune** is the sliver between *two* arcs on the *same* endpoints,
pointed at both ends because that is where they meet. The pupil is a lune, so
its points are a property of the construction rather than a taper anyone tuned.

**The pupil is filled, not punched.** Cut as negative space it showed what lay
beneath — and the arms converge under the eye, so what it showed was them.
Filling it with `VOID_DEEP` also spares both renderers a coverage-subtraction
primitive neither has: one more flat shape, no new mechanism.

**Order is by layer, not by arm** — every casing, then every conductor, then
every circle, then the eye. Drawing each arm complete in turn looks equivalent
and is not: the arms overlap near the middle, so a later one's casing paints
over an earlier one's conductor.

**Flat, and that is structural.** Every shape is one flat fill, which is what
lets `rgba()` and `svg()` be *the same picture* rather than two approximations
of one: both walk the same `Shape` list (a `Band` of ribs, or a `Disc`) in the
same order, with no light model and no gradient in between to drift.

**Hues are derived, not restated.** `deep` strips a palette hue's white
component and scales its value — past 1.0 it drives the hue beyond its own peak,
which is how hydra green becomes phosphor. The whole mark is `deep(HYDRA, 1.15)`
and `deep(HYDRA, 0.45)`: one palette entry, two ways, plus the void.

**Three channels carry an icon, and which one is load bearing depends on the
display server.** This is the part that is not obvious, and getting it wrong
costs a day:

| channel | what it is | who reads it |
| --- | --- | --- |
| desktop entry + icon theme | `yog.desktop` says `Icon=yog`; the shell resolves that name under `hicolor/` | launchers, docks, app grids — **and the Wayland compositor** |
| window icon | pixels handed to the display server by `with_icon` (`icon_data()`) | X11 window managers and alt-tab, macOS, Windows |
| `app_id` | the string the toplevel advertises, matched to a desktop entry of that name | Wayland, and *only* Wayland |

Wayland has no protocol for a client to set its own window icon, so the middle
row does not exist there — `icon_data()` is computed and discarded, and the
`app_id` line is the whole binding (see the `egui-winit` feature note below).
On X11 and macOS the middle row *is* the binding, which is why the rasterizer
stays: **yog targets both.**

**`icon_data()` computes rather than embeds, deliberately.** A pre-rendered
buffer baked in with `include_bytes!` would spare the start-up arithmetic — but
the binary can compute those pixels from the same `Shape` list it already
carries, so a stored copy is storing what you can compute, and the arithmetic
is microseconds. What *does* need baking is the set of files an external
consumer reads, and that is what `assets/` holds.

**The generator emits the whole set (`examples/icon.rs`).** `make icon` writes
the scalable SVG and one PNG per size in `PNG_SIZES` — 16 through 256. The
small end is what a fixed-size icon theme installs (`make install` lays each
into `hicolor/<n>x<n>/apps/` beside the scalable SVG); GNOME is content with the
SVG alone, but plenty of X11 shells and panels still want the fixed sizes, and
an OL9 desktop is exactly that audience. The large end exists because a mark
gets dropped into READMEs and pages where an SVG is not always welcome.

**The PNG encoder is a dev-dependency, and that is the point.** The property
worth keeping was never "no encoder anywhere" — it is **no codec in the shipped
binary**, and encoding is a *build-time* concern. `image` sits in
`[dev-dependencies]`, where the generator and the test that holds its output can
both reach it and where the binary cannot; it is already in the graph at exactly
the features eframe asks for, so it costs no crate. An earlier pass wrote a
codec-free encoder out of stored deflate blocks — correct, and preserving a
property nobody had asked for, at the price of uncompressed output. Real
compression takes the whole checked-in set from 60 KB across five sizes to 44 KB
across six, the 64 px alone from 16.5 KB to 1.8.

**The PNG test compares the decoded image, not the encoded bytes.** Compression
level, filter choice, even the encoder itself may change freely; the contract is
that each file decodes to exactly the pixels `rgba()` produces. Testing the
bytes would have pinned an encoder nobody promised.

Every file in `assets/` *is* an emitted form — checked in so it can be
inspected and diffed, never hand-edited; **`make icon` is the only sanctioned
way to move any of them**, and a test asserts each still equals what the
generator produces.

`make install` lays the SVG into `share/icons/hicolor/scalable/apps/`
beside `assets/yog.desktop` in `share/applications/`; the entry's `Icon=yog`
resolves by name through hicolor.

**The installed seats are a cache of the mark, and staleness there was once
invisible (bl-121d).** `make run` showed a superseded mark long after the orb
table moved. The binary was never the stale surface: on Wayland the first
channel is the only one, so the window's icon is whatever the *installed*
hicolor seats say — and those seats sat behind three traps. Two are mtime
games: GTK judges an `icon-theme.cache` at the theme root **valid while its
mtime is at or above the toplevel dir's**, and laying files into existing
`<n>x<n>/apps` subdirs never bumps that toplevel — so a third-party
`gtk-update-icon-cache` run (a Steam/Chrome shortcut installer, here) freezes
the index forever; a **running shell** likewise rescans its theme only when
that same toplevel mtime moves, so even byte-fresh PNGs stay unseen for the
session. The third is orphaning: a size dropped from `PNG_SIZES` leaves its
installed seat unowned — never overwritten, never uninstalled (a 24 px seat
outlived two marks). The answer keeps the one home (the orb table) and makes
seat-laying **total and loud**: `make icon-seats` — a prerequisite of both
`make install` and `make run`, so the launch verb refreshes exactly what its
window will resolve through — sweeps *every* sized `yog.png` seat, lays the
current set, and rebuilds the theme cache (`gtk-update-icon-cache -f -t`,
falling back to touching the theme root), which is also the toplevel mtime
bump a live shell needs in order to notice at all.

**How the window binds to it, and the trap in the middle.** On X11 the
viewport's `with_icon` ships the raster straight to the server and
`StartupWMClass=yog` matches `WM_CLASS`. **On Wayland neither of those is the
mechanism.** There is no protocol for a client to set its own window icon, so
`with_icon` — and therefore all of `theme::icon::rgba` on that path — is a
**no-op**; the compositor can find an icon only by matching the toplevel's
`app_id` to a desktop entry of the same name. That makes `with_app_id("yog")`
the single load-bearing line, and it is easy to lose: `egui-winit` forwards it
to Wayland **only under its own `wayland` feature**, which eframe's default
feature set would have carried but this crate's trimmed one
(`default-features = false`) does not. yog therefore depends on `egui-winit`
directly — for no code, purely to turn that feature on. Taking the feature on
`eframe` instead pulls the entire wgpu stack in behind `egui-wgpu?/wayland`
(~50 crates, for a glow-only app); via `egui-winit` it adds none, because both
edges of the feature (`winit/wayland`, `bytemuck`) are already in the graph.

Symptom when it is missing: a generic icon in the shell and the alt-tab
switcher, with everything else — asset, entry, icon cache, `icon_data()` —
provably correct. Confirm with `WAYLAND_DEBUG=1 yog 2>&1 | grep set_app_id`,
which must show `xdg_toplevel#N.set_app_id("yog")`.

**Glyph coverage — one font set, both families.** The badge vocabulary
(`● ◐ ○ ■ ◈ ⋯ ▼ → ✔ ✖ ⚑ ✉ …`) is only a vocabulary if it *renders*. egui's
stock font families are asymmetric — Monospace leads with Hack, Proportional
has no Hack at all — so every glyph only Hack covers (`●◐◈⋯▼→⇒`, including the
Live and InFlight state badges on every conversation row) painted as a tofu box
in proportional seats while rendering fine in mono. `theme::fonts` folds the
Monospace font list into Proportional as a fallback tail: the same font *set*
in both families, differing only in priority. **Coverage is then identical by
construction** — no seat has to know which family paints its glyph, and no font
is shipped (Hack is already in the binary via `epaint_default_fonts`, already
licence-cleared in `deny.toml`), so the fix costs zero bytes and zero
dependencies. Two glyphs are missing from *every* bundled font and are simply
not available: `✓` U+2713 and `✗` U+2717 — the good/error marks are the heavy
variants **`✔` U+2714** and **`✖` U+2716**, which the bundled fonts do cover.
The invariant is machine-checked: `tests/integration/glyph_coverage.rs` lexes every `src`
string and char literal and asserts `epaint::Fonts::has_glyph` for each
non-ASCII character in both families. Adding an uncovered glyph fails the
build; that guard, not the font list, is what keeps this closed.

**Glyph doctrine — glance-clarity over already-clear text.** Coverage (above)
is whether a glyph *renders*; this is whether it should be doing the work it is
doing — a glyph can render perfectly and still be a legibility bug. The ruling:

> Glyphs are welcome, used the way colour is used in accessible UI: as visual
> organizing data for interface elements that are otherwise already clear. A
> glyph is never added so the state need not be said; it is added so the state
> is glance-wise clear. Use as many glyphs as are appropriate — just never
> assume one is clear unless it is *extremely* clear.

The colour analogy is the whole rule: in accessible UI, colour is never the only
carrier of meaning — remove it and the interface still reads. Glyphs get the
same treatment. **The test for any glyph seat: is the state still legible with
the glyph deleted?** If not, the glyph is carrying meaning it must not carry —
say the state (a hover text at minimum; outright words where the seat has room),
then put the glyph back on top for the glance. Volume is not the constraint —
the vocabulary above is wanted; the *load* on each glyph is what is capped. And
"extremely clear" is a high bar the author's own judgement cannot meet (the
author chose the glyph; `●` for Live is legible *once you know*, which is not
the same thing): assume a first-time reader with no legend. Two ways a glyph
lawfully passes without adjacent words — a platform-universal convention (`⋯`
overflow, `▶`/`▼` disclosure folds, `★` pin, `✉` mail) clears the bar on
convention alone; and a glyph whose meaning is stated in words *co-visibly on
the same screen* inherits that legend (the `⚑N` workspace tab badges and the
conversation row's bare `⚑`, under the attention strip's own worded
`⚑ N need attention` — the row's flag also hovers its own sentence, bl-b9e3).

**The audit table is frozen (bl-43cd).** The audit campaign swept
every glyph seat against this rule and closed with every seat passing (the
seats that failed were fixed at bl-ae05, bl-b88e, bl-4305, bl-51cb, bl-9a01,
bl-efa2); the per-seat table is retired to git history. A new glyph seat is
governed by the doctrine above, the badge-seat pattern below, and the machine
checks — `tests/integration/glyph_coverage.rs` for rendering, the exhaustive badge-mapping
tests for wording — never by appending an audit row.

**Word doctrine — say the thing, never cite the section (bl-cdd2).** The glyph
rule's twin, for text. A `§9.5` in a rendered string is a coordinate into a
document the operator does not have and cannot open; it reads as noise at best
and as a promise of an explanation at worst. Nor is spec voice operator voice:
the pane once headed its raw TOML *"bz is the only lawful parser (§9.1)"*, which
states this document's ruling rather than what the box does — the operator needs
*"raw config.toml — validated by bz before it lands"*. The rule is mechanical
and has no allowlist: **a `§` belongs in a comment and never in a string
literal**, outside test code, whose assertion messages address the author.
`tests/design_citations.rs::the_operator_never_reads_a_section_number` lexes
every `src` string and enforces it, so the sweep is an invariant rather than a
cleanup that regrows. Citations in doc comments are untouched and remain
correct — they are how the code stays anchored to this file, and the guard in
the other direction (every `§` resolves to a heading) still holds over them.

**The badge-seat pattern — how a failing seat is fixed (bl-ae05).** Two
moves, in order:

1. **The words live with the glyph, in the mapping function.** `theme::state_badge`
   returns `(glyph, hue, phrase)` — one home for all three carriers of one fact,
   so a new state cannot ship glyph-only (the match is exhaustive) and no
   renderer invents its own wording, exactly as no renderer restates an RGB
   triple. A sigil with no enum behind it gets a named const instead
   (`theme::STATE_UNCERTAIN` for the §10 "?").
2. **The seat chooses how it says them.** Outright words where the seat has room
   (a framing badge, a header, a banner); `on_hover_text` on the glyph where the
   seat is a dense repeating row — the conversation list is width-bound by
   construction (bl-9669: an overflowing row ratchets the panel wider every
   frame), and the descent tree is compact by design, so both state-badge seats
   hover. Hover is the doctrine's stated *minimum*, not a licence to skip step 1:
   the phrase must exist in the mapping either way, so promoting a seat to
   inline words is a render-site change and never a re-wording. **One fact's two
   seats need not resolve the same way** — bl-51cb's `theme::op_badge` is worn
   inline by the activity chip (`· 2 failed ⚠`: a count already reads as a
   phrase, and the chip has the room) and on hover by the ops rows beneath it (a
   dense repeating row whose argv is already long, where an inline "ran clean"
   on every row would bury the one that failed). A phrase that has an inline
   seat is kept short — it is the state's *name*, not its explanation.

The **tool-result seats take the inline branch of step 2 (bl-4305)**, and the
reasoning is the model for judging a seat. `theme::tool_result_badge(is_error)`
returns `("✔", HYDRA, "tool result — ok")` / `("✖", ICHOR, "tool result —
error")` — one home for the pair the transcript row and the Steps drill-in had
each been deciding for itself. The transcript's result row is dense and
repeating, which argues hover, but two facts overrule it: the row's `prefix` is
its always-visible **identity slot**, already worded for every other row class
(`user:`, `opus:`, `⚙ Read — running`), so hovering the outcome would make the
tool result the one row that will not say what it is; and the width pressure
that forced the conversation list to hover (bl-9669) is the left column's, while
the transcript is the centre pane whose payload preview is clipped to one line
anyway. The Steps `tools` seat is not dense at all — `✔ <tool_id>` heads that
tool's input/output trees on a line of its own — so it simply has the room.

The tuple is the *shape* the pattern usually takes, not the rule. A seat with no
hue that paints one label collapses it: `nav::tabs::Kind::mark` returns the mark
itself (`Some("⏮ replay")`, `Some("foreign")`, `None` for the ordinary named
regime — an unmarked tab *is* a named workspace), which is the same requirement
with one product instead of three — glyph and words in one home, exhaustive over
the kinds, and never glyph-only. The test that keeps it honest generalises the
siblings' "phrases non-empty and pairwise distinct": **every mark carries at
least one letter and no two kinds share one**, since a mark of glyphs alone puts
the load straight back on the glyph and a shared mark says nothing about *which*
kind. The workspace tab takes the words outright rather than on hover: only a
*pinned* foreign/replay entry is ever marked (an operator's deliberate hoist, not
a dense repeating row), and a top-bar tab you must hover to tell a read-only
regime from a live one is the same loss the ⋯-behind word already was — so the
mark is kept short instead, which is the width the tab row actually cannot spare.

**Discoverability invariant — every control says what pressing it does
(bl-68ac).** A shipped window prompted the question *what does the scan button
mean?* The answer — it runs `lernie scan`, which writes a died epitaph
for every driver that crashed without one and delivers inbox deposits still
queued (§8.2, §10) — existed only in this document. Fixing that one button
would have left the next control equally mute, so the rule is general:

> **Every interactive control carries `on_hover_text` stating what pressing it
> DOES, in the operator's terms, in one or two sentences.** Buttons, editable
> fields, checkboxes, dropdowns and their entries, selectable rows, disclosure
> toggles — the whole surface, with no exception and no roster of exempt
> controls.

It is the glyph doctrine's rule one level up, and its test is the same shape.
The glyph test is *is the state still legible with the glyph deleted?*; this
one is **can a first-time operator tell what will happen before pressing it?**
A label is a *name*, not an answer: `Scan`, `Flush`, `Close`, `Release` and
`advance` are all names of things the operator has never met, and a name that
must be looked up is the same failure a glyph with no legend is.

Four rules keep the wording honest, and they are what stops a hover degenerating
into a restatement of the label:

1. **Say the consequence, not the category.** Name the substrate verb where
   there is one (`lernie scan`, `bl close`, `bz --login`) *and* what it leaves
   behind: what is written, what is killed, what survives, what is
   irrecoverable. "Closes the ball" is the label again; "runs the project's
   pre-commit gate, squashes the work onto the target branch, and removes the
   worktree — a failing gate aborts and leaves the ball claimed" is the answer.
2. **A disabled control says why it cannot act**, through
   `on_disabled_hover_text` — the other half of bl-e266's ruling that a control
   which clicks but does nothing is worse than one that visibly cannot. Greyed
   and silent is still a mystery.
3. **The words derive from this document, and the hover names the combo**
   (bl-478d: every button's mouseover states the combo that fires it).** A hover is a §-backed sentence in
   plain language, not invented at the render site — and every control's
   keyboard spelling rides in the sentence, in one of exactly three
   vocabularies: a **key** the binding table names (`(f)`, `(x)`, `Ctrl+N`), a
   **§8.5 line** where the gesture's address is a line (`/move [id] <to>`,
   `/config <destination> <text…>`), or **the focus floor** where the control
   has neither and needs neither ("No key of its own: Tab reaches it, Space
   presses it"). The first two are read from their authorities rather than
   restated — `keymap::spell` sweeps the table itself and `boundary::help` is
   the verb roster — so a rebinding rewrites what the scan accepts. A control
   whose gesture has no spelling at all is a keyboard gap (keyboard rule 2 as
   amended), never a hover exemption.
4. **The seat is the home.** Unlike a badge — one fact worn at many seats,
   hence one mapping function — a control is one fact with **one** seat, so its
   sentence lives at the render site. A phrase genuinely worn twice (a field and
   its label; `move to:` and each destination) is a named `const` in that
   module, never two spellings. Where the seats are a *set* — the five inspector
   tabs, the five step drill-ins — the sentences are one exhaustive match over
   the enum, so a new variant cannot ship wordless.

The invariant is machine-held in `src/shell/acceptance/hover/`, in three halves
that catch different failures. `every_interactive_control_carries_a_hover` reads
the tree's own source (`hover/scan.rs` reduces a file to its call structure,
then walks the method chain off each widget constructor), so a control shipped
mute fails **even where no fixture can reach its seat** — a Move destination, a
provider Login row, a workflow-file button.
`every_control_hover_names_its_keyboard_spelling` (bl-478d) is that same reading
held to rule 3: the skeleton keeps each literal's **words**, so the scan reads
what a control actually says and fails it for naming no spelling — following a
hover the seat delegates (`RELOAD_HINT`, `tab.hint()`) to the `const` or `fn`
that holds the sentence, since rule 4 puts a phrase worn twice in a named home.
`the_hovers_reach_the_paint_layer`
is the paint-layer proof in bl-2d87's idiom: egui's `everything_is_visible`
forces every tooltip to paint, so a hover hung on the label *beside* a button
rather than on the button never reaches the galleys and is caught. Neither is a
review promise, and the scan fails if it enumerates nothing — the same
two-direction discipline `make rules-audit` keeps.

The invariant governs **controls**; data columns answer to their own home. A
read-only heading, a badge or a row of figures is the glyph doctrine's business
and the column table's (`steps_view/columns.rs`, bl-3ffc) — and where a control
picks among the *same* words a data surface already names, it reads them from
that home rather than spelling them twice: the shell's step-record picker takes
its five sentences from `steps_view::RECORDS`, exactly as a badge seat takes its
phrase from the mapping. Rule 4's "the seat is the home" is about a control
whose words nothing else owns; where something else already owns them, that
owner wins.

**Context-menu doctrine — accelerators over visible verbs (bl-ef89).** egui
gives every widget a native secondary-click menu, and yog already uses one
(the pinned tab's unpin, below). The rule is the glyph doctrine's, one seat
over: the glyph test is *the state must survive glyph deletion*; the
context-menu test is **every verb must survive context-menu deletion** —
delete every context menu and the UI loses clicks, never capabilities. A
context menu is an *accelerator surface*: it carries object-scoped verbs that
already exist on a visible affordance or a key, retargeted at the row under
the pointer. It is never the sole carrier of any verb, critical or not —
right-click is invisible chrome (nothing on screen says it exists) with no
keyboard path, so a menu-only verb fails exactly the test a glyph-only state
badge fails. The one thing convention clears is the menu's *existence*:
"right-click an object for its verbs" is platform-universal, which is why the
menus are worth having at all — but convention makes the gesture
discoverable, never any particular verb completable.

Two properties every context menu keeps:

- **Right-click is not the §6 gesture.** Opening a row's menu neither focuses
  nor acknowledges the row — it is a verb carrier, not a selection. That is
  the accelerator's value beyond the click it saves: acting on a conversation
  *without* leaving the one being read, which no visible affordance offers
  (they all act on the selection).
- **A destructive verb reached through a context menu opens the §3.6
  confirmation** exactly as its visible carrier does — the menu accelerates
  reaching the dialog, never past it.

The v1 seats, a closed roster (extension is governed by the rule above, not
by taste):

| object | context menu carries | visible carrier of the same verbs |
|---|---|---|
| workspace tab | Delete workspace… (§3.6, named workspaces only); unpin (pinned hoists) | delete: a worded, ichor `delete this workspace…` row on the Config tab's per-workspace surface (§9.3's entry for the focused workspace — the settings-danger-zone convention); unpin: the overflow menu's ★ toggle (fix below) |
| conversation row | Stop (+children), Flush, Delete… (§3.6 one-conversation delete, named workspaces only) | Stop/Flush: the composer-side Stop/Scan affordances and the `x` / `f` keys (§8.2) — selection-targeted where the menu is pointer-targeted; delete: a worded, ichor `delete this conversation…` row at the foot of the inspector's Config tab (bl-f17a — the per-conversation settings surface, mirroring the workspace verb's danger row), opening the confirmation and never past it |
| ball row | Assign / Move (destination submenu) / Release / Close | the ball actions beside the composer and the `c` / `r` keys (§8.2), plus the ready row's `assign → <workspace>` button (§8.1); Move's destination is a pick, and a context submenu is a pointer surface — keyboard rule 2's carve-out, satisfied in place |

Each seat's entries are offered exactly where its visible carrier is enabled,
because both read the same `crate::actions` predicate — `stop_enabled`,
`assign_enabled`, `move_enabled`, `unclaim_enabled`, `close_enabled`. A menu
that offered a verb the button refuses would be a second rule; there is one.
Where every entry is withheld the object has **no menu at all** — an empty popup
is a mystery no-op (bl-e266), so nothing is attached.

**Unpin's sole-carrier violation is closed by the roster rule, not more
chrome (bl-7e32):** the overflow menu keeps listing a pinned foreign/replay
entry with its ★ lit (pinning changes where an entry *also* appears, never
where it lives), so ★ is the visible pin/unpin toggle and the tab menu's
unpin is an accelerator. The overflow *badge* still counts only the entries
actually folded away, so a pinned entry's attention is not tallied twice
against its own tab.

**Where the roster lives (bl-0ccc, extended bl-7e32).** The seats are a *table*,
not scattered `.context_menu()` calls: `src/nav/menu.rs` maps a `Seat` (the
object, carrying the facts its entries are gated on) to its `Entry` list — a
worded label, the **visible carrier** that entry claims, and an `Action` that is
either `Fire(Verb)` or a `Submenu` of the same rows (Move's destination pick;
one representation of the one fact, so a row is never both). `src/shell/menus.rs`
is the one place a menu is attached and the one dispatch from a verb to its
effect, against a `Target` the **attach site** resolved from the row under the
pointer — never re-derived from the focus, which is what keeps right-click off
the §6 gesture (focusing acknowledges, so a focusing right-click would silently
clear the row's attention). Each `fire` arm calls the same function the visible
carrier calls. Adding a seat is three edits in those two files, and the
doctrine's rule is a unit test sweeping the whole table — every entry, submenu
rows included, names a carrier — rather than a review promise.

---

## 12. Module map and line budgets

Dependency doctrine: serde_json covers every machine contract; small pure
functions (percent-decode, XDG folds, jsonview, lsof parsing) beat
dependencies; and the existing trait-injection pattern (LockProbe/WriterProbe)
is **the template for every new effect** — lsof runner, bl runner, bz runner,
editor shim, clock. The three embedded substrates are exact-pinned crates.io
releases; **the pin authority is `Cargo.toml`/`Cargo.lock`, never this doc**
(§16.5, §16.7). brazen brings the rustls/`ureq` TLS stack, governed by
AGENTS.md rule 6.

tarpaulin excludes: `src/main.rs`, `src/shell/*`. Budgets include inline
tests; **anything projected ≥200 is pre-split at design time, not at the
cap** (bl-52f8 brought the tree to that aspiration; the hard cap stays 300).
A module's test corpus is its own seam — `X.rs` pairs with `X/tests.rs`, a
subsystem with `<sub>/tests/*.rs` — so **a test module is covered by its
production module's row** and never earns one of its own; the budget below is
the production file's.
**Rows stay sorted by module path** — inserts distribute instead of stacking
at subsystem boundaries — and a responsibility cell is one clause plus the
§ citations that own the doctrine; doctrine lives in those sections, never in
this table. **The table is machine-checked**: `tests/design_module_map.rs`
holds both directions — a production file under `src/` with no row fails, and
a row naming a path that does not exist fails — so the paths here are a
contract, not typography, and a brace spelling the guard cannot expand reads
as a missing row.
**A row stays short enough to merge** (bl-0012): git merges by line, so a row
that grows to hold a whole subsystem becomes one physical line every
concurrent module addition collides on, unconditionally and unresolvably. The
`src/shell/*` glue was exactly that — one ~16,000-character line — and is now
one row per file; keep it that way, and split any other row that starts
accumulating a subsystem. The whole `src/shell/*` family is **interaction glue
only**: each file the thin egui face of a §-owned surface, tarpaulin-excluded
beside `main.rs`.

| Module | Est. lines | Responsibility |
|---|---|---|
| `src/actions/mod.rs` | 230 | the action root (ARCH §3.4/§3.5): every **enablement predicate**, pure and egui-free — whether a verb is offered for the current selection; the verbs themselves live in `verbs` (§8.2) |
| `src/actions/drafts.rs` | 85 | the composer's draft store, keyed by target (§11, §5.3): one draft per new-conversation-in-workspace / message-to-agent |
| `src/actions/verbs{,/balls,/bound}.rs` | 200+186+82 | the §8.2 verb dispatchers + opslog wiring, cut on the table's own seam when the V2 attempt joined it (bl-dc0c): `verbs` the lernie family — message, the attempt's `dispatch`, stop, scan, and the §9.4 `retarget` exit (bl-2d19) — acting on a conversation in a workspace; `balls` the `bl` family, acting on a ball in a project and stamped `--as` its §3.2 claimant to a verb; `bound` the workspace-bound spawn seam every lernie verb takes (§8.2's workspace-bound rider, bl-bf79) — one fold laying the wall and the name, so the family owes no per-verb decision |
| `src/actions/verbs/dispatch.rs` | 197 | the dispatch + `ops.jsonl` logging core beneath the short verbs (§8.2, §4.2 as amended): every attempted action leaves a durable ops line, a spawn failure logging a synthetic one, so no error class is un-logged (§7.3) |
| `src/app/{mod,roots,knobs,view,ops}.rs` | 268+64+62+288+57 | AppModel — what a *frame* owns (§7.2): the held snapshot, ui-state integration, the per-frame refresh; `roots` the boot-time fold of the composed world and the four derived paths every root read goes through (bl-3f46); the §11 transcript-density knobs and the whole-UI zoom (§4.1); the per-conversation view-model assembly the shell paints, plus the snapshot's staleness and live-cadence reads; `ops` the frame's two *writes* to the trail — the operator's ack and the clear verb (§4.2 as amended, bl-c417), here rather than in the excluded shell so the gestures a click makes are covered |
| `src/app/balls{,/starts,/targets,/convball}.rs` | 253+238+51+107 | the frame's read of the live `bl` projection — the §3.5 join, ops tail, and the two post-verb dirty marks (§5.1, §7.2); the §3.4/§8.1 start hand-off; the §8.2 stamp-target rule |
| `src/app/line.rs` | 45 | the §8.5 line context: the seat's focus read as what a slash command elides — the §3.2 stamp a `bl` verb carries (the focused ball's claimant, else the workspace's own name), derived here so a typed verb and a clicked one cannot aim differently |
| `src/app/cadence.rs` | 204 | the clock's periods (§7.2, bl-3381): the `Cadence` value, its `cadence.yaml` grammar (total parse, shared bounds), and the derived periods (wound grace, late pass, staleness) |
| `src/app/deletes.rs` | 140 | the §3.6 hand-off, both altitudes: confirmation derivations, fire-time re-gates, post-delete convergence (bl-f17a: the agent delete's focus-off-the-dead-subtree move) |
| `src/app/derive{,/route,/sweeps,/worker}.rs` | 273+127+279+71 | the derivation worker (§7.2): its state and one pass; the §7.1 dirty-root routing table; the two sweeps, reconcile, the fetch cadence and re-deriving one root — the work every sweep ends in, moved beside them at the budget (bl-4b28); the thread that drives the pass |
| `src/app/dirty.rs` | 215 | Change→dirty-root mapping, debounce/sweep scheduling over the live `Cadence`, `watch::Mark` provenance (§7.2) |
| `src/app/grace.rs` | 165 | the §7.3 wound banner's grace window (bl-90bf): the render-layer age gate over the same injected clock, so a wound that heals inside the snapshot's own catch-up bound never flashes |
| `src/app/drift{,/tests}.rs` | 186+130 | the four drift kinds and their `ops.jsonl` fold, the late-pass and stale-snapshot thresholds, and the edge test that makes a permanently-late derivation one event rather than one row a sweep (§7.2, bl-4b28) |
| `src/app/echo.rs` | 195 | the pending echo (§7.2, §3.4, bl-915e): the §3.4 start claim's value, the landed-message reconciliation, the expiry bound, and the **one** fold of the derivation + every non-derived fact into the snapshot the frame paints (the echo, and the §7.2 live tail) |
| `src/app/live{,/follow}.rs` | 130+185 | the §7.2 **live tail** (bl-54f7) under the in-memory carve-out: the value, its fold onto the painted snapshot, the model's side of the hand-off — and, in `follow`, the append-following read of the focused conversation's open `response.json` and the thread that drives it. The split is *what the tail promises* vs *how the bytes are gathered* |
| `src/app/memo.rs` | 162 | the per-snapshot memo for frame-side view-model builds (§7.2): the altitude-2 view-models are functions of disk *and* of frame-owned selection (§5.3), so the worker cannot build them ahead |
| `src/app/focus.rs` | 255 | the §6/§11 **selection**: the roster ladder (↑/↓, jump-to-attention), the pin/collapse writes, the seen-acknowledgement, and the §3.4 start claim a fire leaves for the first roster that carries its root. Not `shell/focus.rs`, which owns the keyboard |
| `src/app/search.rs` | 80 | the frame's §8.5 search seat: the ask handed to this instance's `Searcher` (the frame runs no search), the landed answer it renders, and `open` — a hit routed into the existing selection, never a new navigation path |
| `src/app/snapshot.rs` | 165 | the published derivation the frame renders, its age, the per-conversation branch-growth diff (§7.2), the per-workspace `steps/` fold every spend figure filters (§3.5, bl-9dd4), and the `models.yaml` context windows every fullness figure divides by (§5.1 #35, bl-a48b) |
| `src/app/spend.rs` | 85 | the §3.5 spend queries the frame (or a headless caller) asks — the price table off `ui.json`, the snapshot's bills, the per-conversation and per-ball figures, the §5.1 #35 per-conversation context figure beside them, and the frame's delegation to the one `board::build` |
| `src/attention/{mod,roster}.rs` | 210+150 | the §7.3 attention flag: the ack state machine (incl. `evidence` — the **one** definition of what an acknowledgement writes, read by the window's focus tick and by the §8.5 `seen` action, and naming neither of the two signals no watermark may answer: mail, and the §8.6 park) and `AttentionKind::says`, the one home for each rule **in words** (bl-e160's desktop alert states it where the badges glyph it); the per-conversation roster it is raised against |
| `src/alert/{mod,send,tests}.rs` | 150+145+180 | §6 as amended (bl-e160) — the strip escalated to the desktop. `mod` is the whole decision, pure: a §8.5 queue row projected to the sentence a notification shows, the per-window baseline of what has already been said, and the two gates (focus, the §4.1 knob) that silence the announcing while the baseline advances regardless. `send` is the one spawn — libnotify's `notify-send` through the bare `git_env::command` constructor (not `Cli`: the desktop is not substrate and must not take the §16.2 world fold), synchronous so a test can drive it, with the window running it off-thread and every failure silent |
| `src/binding/mod.rs` | 150 | names-root enumeration (§3.1), claimant join (§3.2), worktree formula, workspace classification |
| `src/board/{mod,rows,rollup}.rs` | 215+125+125 | the V4 board (§11, VISION §5 V4): the four columns as balls' ladder crossed with its close-gate predicate, and the whole board built pure over one snapshot — its rows, and (bl-66fb) the facts of any §4.3 loop armed over them, empty in every unarmed world; one row's gate, drones and figure; the epic rollup that crosses workspaces, one slice apiece |
| `src/boundary/{mod,project,query,codec,codec/start,codec/config,codec/query,codec/monitor,codec/fleet,codec/control,codec/fork,codec/fields}.rs` | 291+50+106+274+166+141+156+53+51+60+60+48 | the §8.5 typed surface: the Action and Query enums both frontends construct — `query` the populating-read roster in its own file at the cap (bl-765d), cut on §8.5's own taxonomy, the seam the help table is already cut along — with `project` — which project a gesture mutates, a *query on* the enum rather than a part of it — split off at §12's cap (bl-dc0c); the headless JSON envelope, exhaustive both directions (the VISION §4.8 compile gate), cut per family — `codec/config` the §9 destination, the §16.3 mode and the §9.3 origin (bl-3f46), and `codec/query` every populating read's spelling, so the top-level match is the action roster and chains to each family's own reader before it refuses an unknown op (bl-3746; `config`/`marks` read there too since bl-0164, recognized only in their fieldless shape before falling through to the write); `codec/monitor` the VISION §4.9 family (bl-8da1); `codec/fleet` the VISION §4.3 armed loop's two, total where the line is terse — the envelope has no seat, so it names the workspace, the project *and* the cap yog will not guess (bl-66fb); `codec/control` the VISION §4.11 hold answer, whose one field is a verdict and whose `tool_use` id is deliberately not on the wire (bl-765d); `codec/fork` the V2 attempt, whose `skills` list needs the one strict array reader the scalar fields do not (bl-dc0c); and `codec/fields` the total field readers every family imports, split off at §12's cap on the seam `line/args` already draws one serialization over — the verb roster is one thing, what a field is read as is another, and strictness lives there (bl-66fb) |
| `src/boundary/{answer,answer/queue,dispatch,dispatch/deps,dispatch/delete_exec,reply,reply/rows,reply/board,reply/search,reply/queue,ceiling,monitor,fleet,control,control/floor}.rs` | 266+137+297+60+85+279+146+116+35+47+147+106+80+165+82 | the two §8.5 chokepoints, symmetric in shape (`answer(query, deps, ui, now) -> Result<Reply, String>` beside `dispatch(deps, ui, ts, action) -> Result<Reply, String>`, bl-0164): queries mostly pure snapshot derivations the frame's view-models delegate to, the §9 config family's three read from `Deps`'s world exactly as their writes do and so can refuse as they do; actions routed to their §8 executors; the typed replies and which encoder each spends, `reply/rows`/`reply/board`/`reply/search`/`reply/queue` each cut off at the §12 budget on the seam of one reply whose row carries derived sub-objects, its own address shape, or a derived list; and the §3.5 spend gate's one seat inside the `Prompt` door, whose refusal is a §4.2 `yog-step` row before it rides back (bl-56d5). `answer/queue` is the §6 decision queue (VISION §5 V5.2, bl-f6fe): the flattened roster the ↓ key and the queue share, the queue itself, and the acknowledgement that answers one row; `dispatch/deps` the environment a gesture executes in, split out at the cap, and `dispatch/delete_exec` the §3.6 unmaking's two executors beside it (bl-765d) — the seam being that every other arm *routes* while those two **gate**, re-deriving their confirmation at fire time and refusing fail-closed; `monitor` the VISION §4.9 arm/disarm/flag executors — a `cadence.yaml` entry write and one trail row, bodied out of the chokepoint's table (bl-8da1); `fleet` the VISION §4.3 arm/disarm pair, the same shape one block over — no policy file to seed and no first spawn, which belongs to the loop's next tick (bl-66fb); `control` the VISION §4.11 capability family — the hold answer's `["yog-control","answer",…]` row, the detached `lernie advance` that releases it, and the confinement-required birth gate both drone doors run (bl-765d) — with `control/floor` the family's other writer beside it (bl-94b4), VISION §4.9's fifth rung as the `["yog-control","floor",…]` row the same fold reads back; the seam is real and not a line budget, since the answer resolves **one invocation** off a live mark and drives the branch on, while the floor writes **standing policy** over a whole descent and launches nothing |
| `src/boundary/config{,/write,/read}.rs` | 272+138+73 | the §9 family's six executors — the `ConfigFile` destination and the apply/marks/pick gestures (bl-3f46), and the read/read_marks/providers queries beside them (bl-0164) — cut from the pipelines each destination runs (the §9.1 `bz` gate, the §9.2 provider gate, the §9.3 staged `lernie config` commit, the two verdict folds, and `read`'s own `load_snapshot`-minus-the-hash twin) |
| `src/boundary/help{,/table,/table/standing,/table/queries}.rs` | 82+160+177+132 | the §8.5 verb table — every gesture's usage, one-liner and page — and the one text rendering every seat prints; the single source for refusals, `/help`, the codec's verb check and the parity tests. The roster outgrew one file at the cap (bl-dc0c) and is cut along §8.5's own taxonomy — `table` the acts on a conversation or a ball (the §9.4 `retarget` exit among them, bl-2d19), `table/standing` the verbs whose subject is a setting, a standing policy or a record (the §4.9 monitor, the §4.3 loop, the §9 config family, the §4.11 capability answers, the trail's own two — split off at the cap when the exit landed, on the seam where the subject changes), `table/queries` the populating reads (`providers` among them, bl-0164) — then read back as one by `help::table()`, a function rather than a const because const slices cannot be concatenated in a const: each split is a line budget and must never become a list an operator can read half of |
| `src/boundary/line{,/args,/parse,/spell,/config,/verbs,/fork}.rs` | 96+141+231+174+186+264+99 | the §8.5 **line**: the slash spelling of the boundary — the `/`-marker and its `//` escape, and the higher-order help rule read once above the verb match; the argument grammar (context reads that refuse by name, the `--flag value…` split); the reader — the mutating verbs, then `queries` (split at the 100-line clippy function cap) for the populating half, bl-0164; and the writer whose exhaustive match is the compile gate; `config` the §9 family's own grammar, reader and writer in one home, whose destination is words and whose tail is the file verbatim (bl-3f46) — and (bl-0164) whose *absent* tail, on a destination other than a lineage, is `/config`'s and `/marks`' own read; `verbs` (bl-3746) the per-verb argument builders the reader calls into — the §4.9 monitor's three and the §4.3 loop's two among them (bl-66fb); `fork` the V2 attempt's, whose flags lead so the goal can be the verbatim tail (bl-dc0c) |
| `src/boundary/{deposit,consume,consumer,sugar}.rs` | 170+77+114+112 | the §8.5 deposit transport: the gestures-inbox protocol (claim-by-rename, reply files, and `mint` — the id won by an exclusive reply-slot reservation rather than guessed from a clock and a pid, bl-aa9f); one consumption pass; its thread (both faces run it); the `yog gesture` deposit-and-wait sugar (`sugar/argv.rs`: its one payload — envelope or line — and the context flags a line reads its targets from, `--prepared` among them, which is how the start flow's two steps compose across two processes, bl-44d8) |
| `src/boundary/sugar/argv.rs` | 94 | `yog gesture`'s own argv (§8.5): the one payload — a JSON envelope **or** a slash line — and the context flags a terminal line states its elided targets with, the terminal holding no selection to read them off |
| `src/budgets/{mod,bills}.rs` | 180+150 | the Usage-event vocabulary, parsed in one place (§5.1 #16) — the fold of every attempt segment beside `last_usage`, the final segment alone (§5.1 #35); the one `steps/` walk every figure shares — per-step fold beside the model that billed it, the conv that owns it and its own `seq`, so a `Scope` applies after the walk and "which step is the latest" is asked in memory (§3.5, bl-9dd4; bl-a48b) |
| `src/context/{mod,render,tests}.rs` | 105+90+110 | §5.1 #35 — the context-fullness query (the root's latest step's prompt against the window `models.yaml` declares, `None` wherever nothing measured can be said) and the one line §11's settings rows paint from it. Pure over `Snapshot::bills`; the prompt reading's two-provider rule lives in its header and nowhere else |
| `src/bz_host.rs` (+ tests) | 150+165 | the entry to the linked brazen (§16.7): the wall fold, the snapshot brazen reads, the route decision, and the no-wall refusal (§16.2) |
| `src/bz_host/routes.rs` | 165 | the six `bz` routes and their seam bundles — `bz`'s own `main.rs`, one layer in |
| `src/bz_host/store.rs` (+ tests) | 140+150 | yog's wall-rooted `CredStore`/`ModelCache` (§16.2): brazen's shim, rooted at a path instead of the process env |
| `src/cli_outbound/{mod,detach,chunk}.rs` | 205+120+80 | parametric binary resolution + the three spawn classes (§8); the detached spawn and its stderr sink (§8.1); the stream-chunk framing |
| `src/cli_outbound/exec.rs` | 47 | the `yog exec` escape hatch's spawn shape (§8.4, §16.4): a blocking wait with **inherited** stdio, split from `mod` so the piped `run` family keeps the cap |
| `src/cli_outbound/resolve.rs` | 203 | binary resolution (§16.7 W12, the self-multiplex spine): which executable a `Cli` execs and under what leading argv — the one switch point, a per-namespace `self_multiplexed` const |
| `src/cli_outbound/stream.rs` | 130 | the live-handle half: the running-subprocess handle whose iteration yields chunks (final item always the exit) and whose drop terminates the child — SIGTERM, then SIGKILL after a short grace (§2.9) |
| `src/cli_outbound/streamed.rs` | 203 | the **streamed-piped** spawn class (§8, §8.3): both output streams line-buffered off a running child, each line tagged with the stream it came from, plus the terminal exit — the shape `bz --login` renders through (§5.3) |
| `src/cli_outbound/sys.rs` | 19 | the crate's single confined `unsafe` — `libc::kill` for the drop's best-effort SIGTERM, the one audited home `rules/unsafe-outside-sys.yml` leaves it (AGENTS.md rule 3) |
| `src/composer/{mod,tests}.rs` | 185+160 | the §11 inbox-composer's derivations (bl-929d): the pending-queue row projection over the snapshot's §5.1 #11 listing (fold key = the deposit's inbox path, §5.3; each row's §11 role from the one `theme::role` mapping, bl-3acb), and `SnapState` — the derived fold-line height (settled content, capped at half the pane) plus the snap-down ease whose trigger is the pending count dropping, per target |
| `src/composer/recall{.rs,/tests.rs}` | 155+220 | the §11 prompt recall (bl-f908): `prompts` — the operator's own turns in this conversation, newest first, folded from the pending listing (§5.1 #11) and the delivered transcript (§5.1 #12) through the one `theme::message_role` derivation and stored nowhere — plus `Caret`, the visual-row gate that decides whether ↑/↓ are the box's or the caret's, and `Recall`, the two-field walk whose *exit* is a derivation (the draft is no longer the entry we put there) rather than a reset call at each site |
| `src/control/{mod,wire,classify,bash,lex,rules,policy,hold,root,judge,author}.rs` | 195+98+225+205+193+259+210+75+205+165+153 | the §8.6 capability control (VISION §4.11): the consult a `world/tools/` shim runs, and the one sentence a park hands the operator — tool, bounded input summary, class, evidence; lernie's two wire shapes; the effect vocabulary and the built-in intrinsic map; the bash ruleset over every program a command runs; the shell lexer that finds them; the shipped ruleset as data; `policy` the per-workspace override that ruleset is the default of — `capability.yaml` at the live config tip, four keys, absence *is* the defaults (bl-765d); `hold` lernie's valued hold mark, read one agent at a time by the answer gesture and whole-namespace by the snapshot tick; the writable root and its lexical containment; the class→verdict table folded with the trail's answers and floors; the `config/default` authoring that makes a workspace born adjudicated |
| `src/config_edit/mod.rs` | 40 | §9's root: load → edit a RAM draft → Apply = stage → validate → hash-guard → atomic rename, one discipline across every file-editing surface |
| `src/config_edit/apply.rs` | 100 | the `--editor-apply` copy — drafted files only (§9.3) |
| `src/config_edit/branch.rs` | 215 | config-ref browse, governing-config derivation, edit plan (§9.3) |
| `src/config_edit/branch/edit.rs` | 260 | the §9.3 edit half — the scripted-`$EDITOR` drive of `lernie config`, the only lawful writer of `config/*` (ARCH §2.2), re-entering the yog binary at `config_edit::apply` |
| `src/config_edit/brazen/{mod,effects,providers}.rs` | 274+89+164 | the §9.1 editor (staged validation, hash guard) and the wall's `BrazenPaths` layout; the real BzRunner; the provider-row projection (§8.3) |
| `src/config_edit/draft.rs` | 120 | the ONE staged-edit `Draft` both §9.1/§9.2 editors are built from — dirty tracking, revert, the hash guard |
| `src/config_edit/effects.rs` | 105 | the production `FileIo` — the thin `std::fs` shell behind every editor's pure view-model, covered against a real tempdir with no fakes |
| `src/config_edit/form{,/schema}.rs` | 165+170 | the §9.5 typed pane: a setting read from and written back into the draft through the §9.4 grammar, with the shared provider judgement; and the enumeration itself — which settings exist, and the control each gets |
| `src/config_edit/lernie_global/mod.rs` | 230 | the §9.2 editors and provider gate |
| `src/config_edit/pipeline.rs` | 230 | the write pipeline every §9 editor shares: the one home for how a draft reaches disk without a torn write or a silent last-writer-wins over a concurrent edit |
| `src/delete/{mod,exec}.rs` | 150+90 | the §3.6 unmake: pure confirmation + plan; the logged runner |
| `src/delete/agent.rs` | 160 | the §3.6 one-conversation delete (bl-f17a): the member-scoped gate, the blast-radius arming, the `DeleteReport` census parse, the dry-run and removal spawns |
| `src/elide.rs` | 121 | **where to cut a string that will not fit** (QUALITY G1, L4; bl-3aa1) — one rule, *cut where the information is not*. Prose is written front-first and keeps its head, so the eight prose sites (previews, reasons, titles) are correct as they stand and are deliberately NOT routed here — a module claiming every cut while they kept their own would be a false claim. A **machine string** (absolute path, spawned `argv`, ancestry chain) is invariant at the front and distinguishing at the back, and that is `middle`'s case and the only one: the activity rows all opened with the same `/home/<user>/.cache/…/data/yog/` run, over half the row, while the workspace leaf and agent id that told two operations apart were exactly what the old head-keeping cut discarded. Carries a legibility FLOOR a tighter cap is raised to, since `…e` names nothing. The other half of L4 — an **id**, whose distinguishing end is a whole terminal segment rather than a character count — is a floor and not a cut, and keeps its one home in `nav::convs::id_floor` (bl-63a1) |
| `src/engine.rs` | 160 | **the one assembly both faces boot** (VISION §5 V5, bl-f6fe): the §5.2 startup sweep, the §7.1 roots, the model's first synchronous derivation, and the derivation worker + watch bridge + gesture consumer + VISION §4.9 monitor sentry + VISION §4.3 fleet pilot + §7.2 live-tail follower spawned beside it (bl-8da1, bl-66fb, bl-54f7 — each is a fact of the world, so none rides a face). The window and `yog headless` are two calls to it with different repaint hooks — **shared**, since worker and follower both wake the face — it left `main.rs` precisely because that file is coverage-excluded and carried the assembly twice |
| `src/files_view/{mod,render,tests}.rs` | 205+230+185 | agent-worktree bounded walk + file preview (§11 Files); `classify` is the one "what this file is" fold — bytes + true size ⇒ `Text`/`Truncated`/`Binary` — shared by the live walk, the pinned `git show` (§5.1 #31) and the Work tab's patch (#32), so three seats never grow three vocabularies |
| `src/fs_watcher/mod.rs` | 140 | the watched root's watcher: the §7.1 allowlist subset exposed as a drainable stream of coalesced change notifications, pure Rust with no egui dependency |
| `src/fs_watcher/{roots,fold,hub}.rs` | 130+150+155 | per-root-kind allowlists (§7.1); the raw-event drain, coalesce and desync lead (§7.2); the process's one backend instance and its per-root fan-out (§7.1, bl-908c) |
| `src/git_env.rs` (+ `git_env/tests.rs`) | 60+70 | the ambient-git-env scrub at the spawn boundary; the crate's ONE `Command` constructor (bl-916a; `rules/no-bare-command.yml`) |
| `src/git_tree/cmd.rs` | 293 | the git CLI wrapper and log/diff parsing — no libgit2, and every invocation built by `git_env::command` |
| `src/git_tree/descent.rs` | 259 | hyphenated-descent ordering for the agent view (§7.1): hierarchy lives in the name, and lernie's narrow grammar is the authority |
| `src/git_tree/detect.rs` | 64 | the user-message preview, read off `steps/<agent-id>/001/request.json` — step records are never in a git tree |
| `src/git_tree/enumerate.rs` | 237 | commit-node and agent construction: raw git output bridged into the view-model's shapes, the trunk the config lineage |
| `src/git_tree/lock_probe.rs` | 209 | the executor-lock probe behind the §3.5 `live` classification — the inbox-directory `flock`'s holder is *the* driver |
| `src/git_tree/probe_cache.rs` | 201 | the 2 s TTL cache over any liveness probe (§10), wrapping the macOS `lsof` backend before the classifier ever observes through it |
| `src/git_tree/probe_stack.rs` | 88 | the platform probe stack held across ticks (§10, §15 Y11), so each tick re-derives through one stack instead of rebuilding probes and discarding the cache |
| `src/git_tree/state.rs` | 128 | the agent-state classifier (§3.5, §7.1): the four live-view states derived from the executor lock and the latest step's `response.json`, nothing stored |
| `src/git_tree/tools.rs` | 235 | the tool-call view-model over the on-disk `tools/<tool-id>/input.json` / `output.json` records, in-flight being an input with no output beside it |
| `src/git_tree/{fd_probe,lsof,terminal}.rs` | 135+180+140 | the `/proc/*/fd` writer scan; the pure `lsof -F` parser + the cfg(macos) spawn shim (§10); the §4.4 settled-tail classifier |
| `src/git_tree/marks.rs` | 170 | the `refs/lernie/*` namespaces — four read as oids (the §6 watermark evidence) and `held` read as a **value**, its blob parsed by `control::hold` (§8.6); the closed `AgentMark` set (§6) |
| `src/git_tree/{mod,model}.rs` | 123+218 | module wiring + the platform `cfg` probe stack; the inert view-model types it re-exports, incl. the §5.1 #28a call starts (`Agent::call_start_unix`, `ToolCall::start_unix`) and #28b's `Agent::last_delta` |
| `src/git_tree/probe.rs` | 110 | the tri-state probe traits (§10) |
| `src/git_tree/streaming.rs` | 169 | the live `response.json` fold — **one read, two facts** (§5.1 #10, #28b): the display text every tail seat reads, and the last content delta's kind that splits an open model call into waiting / thinking / inference |
| `src/inboxview/{mod,render,tests}.rs` | 180+75+200 | deposit parsing + the listing's per-file name/bytes (`InboxEntry`, §11 Raw) + the one `✉ from · at` header wording every §5.1 #11 seat shares (bl-929d); render, both modes; the tests cut out of `mod` when Raw landed (bl-1ff1) |
| `src/fleet/{mod,arming,facts}.rs` | 67+200+145 | the VISION §4.3 **armed loop**, off until armed (bl-66fb): the two-gesture family and the law it holds to (*it spawns and reaps; it never diagnoses*); the `cadence.yaml` `fleet:` block — the project, the cap, the optional lease, and the two fields yog refuses to guess; and the derivation the V4 board renders — cap, count, tick, lease, last act and the §3.5 ceiling asked over this workspace's bills, every one a query |
| `src/fleet/{row,pilot}.rs` | 148+296 | its acting half: the one ops-row shape a spawn and a reap each leave (§4.2) with a reap's reason stored as the *comparison* and no way to store a diagnosis, plus `last_act` — the board's "last tick", derived; and the level trigger and its thread (§7.2) — at most one move per tick, reaps before spawns, both fired through the boundary's own doors so the ceiling and the confinement gate hold by construction |
| `src/fork/{mod,composer,render}.rs` | 245+100+180 | the V2 **attempt** (bl-dc0c): `mod` what one fork *is* — the `lernie dispatch` argv, the skill pins, and the fire-time policy read off the workspace (§5.1 #34); `composer` the ×N and the readiness rule, where a cohort is a `Vec`'s length and nothing branches on it; `render` the seat at a pinned notch, every control a reading of the workspace rather than a yog list. Nothing here can touch a project worktree — the rung is read-only by construction (VISION §4.10, bl-2b8c) |
| `src/inspector/{mod,tests/mod,tests/tabs,tests/raw,tests/config,tests/pinned}.rs` | 240+150+100+70+75+125 | per-agent tab dispatch + governing-config view (§11), plus the pin banner it wraps every pinnable tab in — which since bl-1802 **is** the pin's release, the gutter that used to carry it having dissolved into the chat; the tests split at the cap on the seam between the shared fixture and what each half asserts |
| `src/jsonview/mod.rs` | 190 | the pure collapsible JSON row tree (§11) |
| `src/layout.rs` | 118 | §11 rule 5's arithmetic in one home (bl-9551): the share a container keeps for its own content, the ceiling that share leaves the **next** accessory in its stack, and the `None` that says an accessory the container cannot pay must not paint at all. Read by the shell's every docked accessory AND by `ui_state::Panel::max_size`, so a boundary the operator drags and a panel the pane docks divide one number |
| `src/login/{mod,auth}.rs` | 236+165 | §8.3 as amended (§15 M6 Z8): the streamed-piped `bz --login` flow, whose lines render verbatim and whose exit lands ONE outcome row, and the pure auth-shaped step-failure predicate that puts the affordance one click from the failure |
| `src/shell/pane.rs` | 140 | the conversation pane's own column (§11, bl-9551), split from `shell/mod.rs` at the cap on a real seam: the window divides itself between a top bar, a roster column, one world-level accessory and a remainder — this divides *that* remainder between the conversation's docked accessories (each drawing from the §11 rule 5 budget, painting nothing when it cannot be paid) and the bounded, clipped viewport that is the conversation itself |
| `src/keymap/{mod,tabs,center,spell,tests/mod,tests/planes,tests/center,tests/spell}.rs` | 272+81+117+97+275+94+92+87 | the pure §11 key → intent table and the `Held` plane rule over it — suppression, the modal plane (bl-d921) and the zoom combos included; the tests are cut at the module's own seam, the binding tables from the plane rule; `tabs` is the §11 inspector tab vocabulary itself — the enum, its digit map, its labels and `pinnable` (which tabs a rail pin reaches, §5.1 #31/#32) — split out at the cap, since only its digit map is a keymap fact, and `center` is its altitude-1 sibling: the §11 **center tabs** (bl-1ca2) — the four peers the center can be, their Ctrl+Shift+digit map, their strip labels and the words each says on hover — `hint` alone, and `focus_hover` pairing it with the combo that presses it (bl-478d rule 3), which the strip, the left-panel entries AND §9.4's remedy button all read, so one tab never grows three phrasings of one gesture (bl-91f1); both tab enums answer `digit()` off their own `all()` order, so the key a hover names and the key that works are one list; `spell` is the table's own **spelling** (bl-478d) — every press swept back out of `keymap` itself as an operator types it (`(f)`, `Ctrl+Shift+2`), plus the focus floor beneath it, which is the vocabulary the §11 hover scan holds every control against, compiled under `cfg(test)` because nothing the window paints reads it: a hover states its key in the seat's own sentence |
| `src/lib.rs` | 160 | module decls, Args, test_support (`src/test_support{,/world,/workspace}.rs` — the fake effects, the hermetic fixture world and its wall, and the real on-disk lernie workspace, each its own file at §12's cap) |
| `src/main.rs` (excl.) | 215 | entry, multi-call/namespace dispatch, the two `Engine::boot` faces (window and `yog headless`) and what a window adds beside the engine |
| `src/model_pick/{mod,header,grammar/mod,grammar/fields,grammar/models,grammar/roles,grammar/tools,query,remedy}.rs` | 210+161+192+176+255+54+29+208+73 | the §9.4 picker: pure plan + the provider-row gate and default (bl-bd89), the **two** scope sentences the one pane is handed (conversation vs. birth, bl-824e) and the **two** config lines — the conversation's freeze + derived drift clause with the two exits it earns (bl-9786, bl-2d19) and the birth block's pair-and-branch-head line, the anchored block grammars (no YAML dep) — `fields` the generic locate/read/replace every rewrite and the §9.5 pane share — the ONE row judgement (`is_unknown_row`) every §9 gate calls, and `remedy` the way out of a credential-shaped roster failure (bl-91f1, §9.4): §8.3's own `looks_auth` as the gate, `ProviderRow::login_blocked` as the words, a §11 tab as the destination — no wording and no classifier of its own, and no `Unrouted` state, because the picker named the row in the query it just fired |
| `src/monitor/{mod,arming,verdict,row}.rs` | 78+200+132+213 | the VISION §4.9 alignment monitor's data half: the anti-reinvention law stated where it must hold; the `cadence.yaml` `monitor:` block (arming, the model pin, the policy file it names) and the seed that file starts from; the three-valued verdict and the one reading of a model's reply; and the ops row that is audit trail, level-trigger memory and tuning dataset at once — with `latest`/`worst`, the queries that make a standing verdict a derivation rather than a field |
| `src/monitor/{window,check,sentry}.rs` | 128+178+191 | its acting half: the evidence one check reads (`goal.md` verbatim + the transcript delta derived from the last-checked sha by `git diff`, tail-clipped); the one bounded tool-less call through the embedded brazen adapter (§16.7 W10) behind a `Caller` seam, and the NDJSON read that takes the verdict and the provider's own counters; and the level trigger and its thread (§7.2) — one check per tick, only when a tip moved, retry by re-firing |
| `src/multiplex{.rs,/bl.rs,/lernie.rs,/help.rs,/landing.rs}` | 269+118+164+142+130 | the §16.7 namespace arms: each embedded crate's verb surface, dispatched from `main.rs` — plus the router's namespace table; `help` (bl-52ed): the argv seat's whole command table, the top-level roster rendered from it, every per-command page, and the discovery probe the `bl`/`bz` arms answer world-free (§8.5's every-command-answers-help rule at the argv surface); and `landing` (bl-7e54): the §16.3 repair the `bl` arm converges on the way in, re-deriving a pre-nesting landing's plugin schedule from balls' own seed |
| `src/names/mod.rs` | 101 | the §3.1 workspace-name validation, and only that. The §3.3 conversation mint left with bl-cd38 (bl-aca4's ruling, consumed at lernie 0.0.8): the wordlist, the injected-`Rng` seam and the bounded wraparound scan are `lernie::mint`'s, and `words.txt` is deleted |
| `src/nav/{mod,tabs,convs,convs/row,convs/expand,convs/doing,convs/flight,convs/group,menu}.rs` | 38+171+254+243+191+117+212+133+235 | the §11 altitude-0 view-models: tab bar + `Kind` marks, conversation list + the §3.3 display ladder + the header's derived when-seat (bl-16da, assembled through the shared `ui_state::format_iso8601` — bl-61db) + the §3.5 ball overlay, the §5.1 #28b per-agent `Doing` and the §11 live mark's seat roster (`convs/doing`, bl-b768 — the finest live fact, which `convs/flight` then *folds* into the #28 class rather than re-reading the snapshot), the #28 live-activity class, its priority **and the bottom strip's characteristics** — including the per-class elapsed each derives from a §5.1 #28a structural start or honestly omits (bl-9dfb), in `convs/row`'s own `age_label` (its own file because all three §11 seats read it, not just the row — bl-905f), the grouped-by-ball partition, the context-menu seat roster; and `ConvRow::verdict`, the VISION §4.9 standing verdict derived per build from the published ops tail (bl-8da1). **`convs/expand` is the unfold** (bl-fa82): the visible-row flatten over `git_tree::descent_order` given the shell's expanded set — the jsonview `flatten`'s shape, one altitude out — plus the two pure walks the §11 keyboard rides (`step` over the visible rows, `parent_of` read off their depths) and the ancestor chain a jump reveals. `convs/row`'s builder generalized with it: it projects **any** member's subtree slice, so the root-only build is one call per depth-0 subtree rather than the only shape it knew |
| `src/opslog/{mod,line,rows,rows/tests,exit,origin,live,detached,operator}.rs` | 272+168+146+166+142+106+166+114+131 | `ops.jsonl` append/tail + the sentinels (§4.2), the ≤4096 capper, OpRow's shape + its human-timestamp leading column (bl-61db) and collapsed summary (bl-0bf9 for the cap, bl-3aa1 for where it cuts — `elide::middle`, so the workspace leaf and agent id that tell two rows apart survive the cut; its tables split to `rows/tests` at the cap on this directory's own seam), `exit` the one reading of the `exit` field — `ExitKind` and the `failed`/`drift`/`exit_label`/`detached` half of OpRow that asks it (bl-afa9, bl-8433), `origin` the §7.3 attribution — which surface an op came from, and the one thing a banner filters on so a failure renders once and on its own seat (bl-48f8) — the §6 retirement projection + activity summary + the `Detached` outcome (bl-8433), the stderr-sink fold (§8.1), `operator` the two lines the operator writes: the ack watermark every alarm derivation reads past and the clear that ends a trail by logging itself as the next one's first row (bl-c417) |
| `src/paint_probe.rs` | 293 | the ONE headless egui paint harness (`collect` + `screen`/`input`) every render test drives, incl. the settled small-screen frame the §11 tail idiom is read on — as text (*what* is on screen), as positioned galleys (*where* it sits, bl-8c13), or as rect fills (*what hue* a glyphless mark painted — the §11 role stripes, bl-3acb). The text it reports is the galley's **glyphs**, row by row, not `Galley::text()` (bl-bc06): the text that went in survives truncation, so a dump read that way answers `contains("Login")` with yes about a button rendering a bare `…` — the one defect the paint layer is the only witness for. `Seen` also carries the run's **ink** (bl-7654) — its layout section's colour, or the shape's `fallback_color` where that section defers to it, which is what `Ui::set_opacity` dims — so "is this seat faded?" is a question of the frame rather than a guess about which widget drew it, and the §11 solidity is readable at the paint layer the way a role stripe's hue already was |
| `src/projects/balls.rs` | 185 | `Entry`→`Ball` projection + the closed-listing parse, status ladder, the §3.5 join table |
| `src/projects/join.rs` | 232 | the §3.5 join-state table (§3.2, §5.1 #7): the ball × workspace product enumerated once, bound iff the ball's claimant equals the workspace name — no operator identity, no stored fact |
| `src/projects/runner.rs` | 132 | the `bl` effect behind `projects::balls` (§5.1 #2/#4, §16.7 W8): the three ball reads of one project, typed, in-process since W8 and faked in tests |
| `src/projects/mod.rs` | 160 | clone enumeration, nested-delivery detection, roster labels (§11 rule 1) |
| `src/workdiff/{mod,plan,render,wire}.rs` + `tests/{mod,plan,read,paint}.rs` | 200+105+165+100 + 115+135+265+170 | the §5.1 #32 **project work-diff** (VISION §4.10, bl-3746), cut on four seams: `mod` the read itself — resolve each attempt's two ends, count the churn, read one file's patch; `plan` the pure half — which attempts a workspace holds and balls' own delivery-target rule re-derived over the snapshot's balls, plus the numstat parse; `render` the §11 Work tab; `wire` its §8.5 JSON shape, written beside the type whose vocabulary it spells (the reply roster still names the encoder). The tests are the S11 rung: the pure derivation, the read against a real project repo, and the paint |
| `src/git_tree/project.rs` | 60 | the **project** repo's four reads (§5.1 #32): name the integration branch, resolve a ref, `--numstat` a range, patch one file. It sits inside `git_tree` because `cmd` is the crate's one `git` doorway — the site that scrubs the inherited `GIT_DIR`/`GIT_INDEX_FILE` (§16) — and a second fork site would be a second place to forget it |
| `src/rail/{mod,cards,cohort,pin,place,tree}.rs` | 200+140+80+95+95+85 | the §11 step spine (VISION V1, bl-98da; re-seated into the chat by bl-1802), cut on five seams, all derivation and no paint — the seat is `src/transcript/spine.rs`: `mod` the spine's *shape* — the notch spine over the Steps view's `meta.commit` (§5.1 #29) and the row→notch lookup the chat renders through; `place` where each notch sits in the chat and how far its pin cuts, pairing one sealed model-output entry to each completed step (the ordinal alignment bl-929d and bl-98da both got wrong); `cards` a child's *placement* on it — the shared-commit-prefix fork point, the two edges, the fork label and the streaming tail, all pure over facts the snapshot already carries; `pin` the fold that threads one notch through the inspector (the transcript prefix, the budget as-of); `tree` the only new disk read — the Files tab out of `ls-tree`/`git show` at the pinned commit; `cohort` V2's fan, which is nothing but those cards grouped by the notch they were born at (§5.1 #33) |
| `src/shell/mod.rs` (excl.) | ≤250 | the *window assembly* — which panel sits where, in what order (bl-e160): §11's three altitudes wired into egui panels, every surface below painted inside one |
| `src/shell/activity.rs` (excl.) | ≤250 | §4.2 |
| `src/shell/alerts.rs` (excl.) | ≤250 | the §6 desktop escalation's whole shell face (bl-e160) — one call into the pure `crate::alert` decision, plus the two things only a window can supply: the OS's answer to *do I have focus*, and a thread to spawn the notifier on so the frame never waits for a desktop |
| `src/shell/banner.rs` (excl.) | ≤250 | the §7.3 last-failure widget every failing surface paints and the one ack that quiets them all, split out of `mod` at the cap on a real seam (bl-e160: `mod` is the *window assembly* — which panel sits where, in what order — while a banner is a widget painted inside one) |
| `src/shell/birth.rs` (excl.) | ≤250 | the §11 birth-config block the settings seat holds when nothing is selected: the work-directory box (bl-7927) + the same §9.4 model row, bl-824e |
| `src/shell/board.rs` (excl.) | ≤250 | the §11 balls section as the V4 board — the four column folds with their derived counts, and each row's gate, drone rows and figures over the covered `AppModel::board` (bl-9dd4) |
| `src/shell/bootstrap.rs` (excl.) | ≤250 | the one-time frame setup |
| `src/shell/center.rs` (excl.) | ≤250 | the §11 center-tab strip and the one dispatch behind it (the four `CenterTab` peers, the Config focus gesture that carries §9's re-read, and the Search tab offered exactly while there is an answer) |
| `src/shell/clock.rs` (excl.) | ≤250 | the three process-boundary mintings (the ops timestamp, the seconds every dated seat reads, the mint's entropy seed) — one home each, since two spellings of "now" would be two facts |
| `src/shell/config_edit/mod.rs` (excl.) | ≤250 | §9 |
| `src/shell/config_edit/status.rs` (excl.) | ≤250 | the config editors' status sentences (§9) — each Apply / Reload outcome as the one line the pane paints |
| `src/shell/config_edit/branch_pane.rs` (excl.) | ≤250 | §9.3 |
| `src/shell/config_edit/brazen_pane.rs` (excl.) | ≤250 | §9.1 — brazen's `config.toml`, the one Config surface that is a *workspace's* rather than the world's, so its whole pane (draft, status, effective dump, provider rows) is one field of `WallRam` (bl-5894) |
| `src/shell/config_edit/form_ui.rs` (excl.) | ≤250 | the §9.5 control-per-row renderer plus the one raw-fallback editor all three surfaces share (bl-2622) |
| `src/shell/config_edit/lernie_pane.rs` (excl.) | ≤250 | §9.2 |
| `src/shell/config_edit/yog_pane.rs` (excl.) | ≤250 | the §7.2 clock surface (bl-3381) |
| `src/shell/config_marks.rs` (excl.) | ≤250 | the §16.3 tracking-branch pane, split from `config_edit` at the cap: `world::marks` carries every decision, this only wires controls to it |
| `src/shell/conv_ball.rs` (excl.) | ≤250 | the ball-badge painters the conversation surfaces share (§3.5, §11), split from `navigator` at the cap: the ids, states and badges come from the tested `nav::convs`/`AppModel` derivations, so this only chooses the hue and lays the widgets |
| `src/shell/conv_list.rs` (excl.) | ≤250 | §11 altitude 0 — the list frame (the new-conversation affordance, the organizing toggle, the scroll and the visible-row iteration both organizing views share) with `conv_row` split off at the budget when the unfold landed (bl-fa82) |
| `src/shell/conv_row.rs` (excl.) | ≤250 | §11 altitude 0 — one row's whole paint: the prefix group, the depth indent and its `↳` elbow, the trailing metadata including the ▶/▼ subagent field whose click toggles the id in the shell's expanded set, the row menu, and the §11 tone scope a §7.2 pending row is painted inside (bl-915e) |
| `src/shell/conv_row/cells.rs` (excl.) | ≤250 | split off `conv_row` in turn at the budget on that file's own seam: `conv_row` assembles a row and wires its gestures, `cells` paints the three seats that carry a §11 shape rule of their own (the depth elbow, the state badge that is the whole prefix, the subagent field) |
| `src/shell/delete.rs` (excl.) | ≤250 | §3.6 |
| `src/shell/delete_agent.rs` (excl.) | ≤250 | the §3.6 one-conversation dialog + its inspector danger row (bl-f17a) |
| `src/shell/dispatch.rs` (excl.) | ≤250 | the intent→body table |
| `src/shell/fire.rs` (excl.) | ≤250 | firing the composer's start (§3.4, §8.1) — the bare / path rung resolved into `StartInputs`, then `start::prepare`, then the detached prompt; split from `input_bar` at the budget |
| `src/shell/flight_strip.rs` (excl.) | ≤250 | the §11 bottom in-flight strip (bl-905f) — the innermost bottom panel, hard against the chat tail above the goal box, created only while the open conversation has something in flight (its §11 rule 5 share is checked by `pane`, which owns the stack's whole arithmetic since bl-58e4) |
| `src/shell/focus.rs` (excl.) | ≤250 | the §11 composer-focus discipline (the one request bit and the one-frame repeat that is its delivery, bl-58e4 — not `app/focus.rs`, which owns the *selection* — plus the center seat that bit needs, since only the Conversation tab paints a composer, bl-1ca2) |
| `src/shell/inbox_queue.rs` (excl.) | ≤250 | the §11 inbox-composer's queue region (bl-929d) — pending rows + the draft under the derived fold line, painting `crate::composer`'s decisions and nothing else, and the one seam where the recall's ↑/↓ are taken off the frame's input before the box is added and the caret's painted row is read back off the galley (bl-f908) |
| `src/shell/{input_bar,verb_row,ball_bar}.rs` (excl.) | ≤250 each | §8.2 — the composer assembly, then one body per verb, shared by buttons, keys and menus (the composer carries the message and nothing else since bl-7927; `verb_row` split out at the cap when the queue landed, bl-929d — and carrying the §8.6 Approve/Decline pair, which appears only while the selected conversation is actually parked and vanishes with the mark, bl-765d) |
| `src/shell/inspector/mod.rs` (excl.) | ≤250 | §11 altitudes 1–2 — the per-agent inspector's own assembly, the tabs `vms` builds view-models for and `controls` steers |
| `src/shell/inspector/controls.rs` (excl.) | ≤250 | the §11 altitude-2 knobs the operator steers |
| `src/shell/inspector/fork.rs` (excl.) | ≤250 | the V2 composer's glue — the choices memo, the seat that lives and dies with the pin, and the one boundary gesture per candidate |
| `src/shell/inspector/rail.rs` (excl.) | ≤250 | the rail's own glue — the children gathered off the snapshot, each one's `steps/<id>` spend folded, and the pin applied to the transcript / files / preview / governing builds |
| `src/shell/inspector/vms.rs` (excl.) | ≤250 | the per-tab view-model assembly split off at the budget when the pin landed (bl-98da) |
| `src/shell/inspector/work.rs` (excl.) | ≤250 | the Work tab's two per-snapshot memos (§5.1 #32), which is where that tab's every `git` fork against a project repo is held to once per published snapshot |
| `src/shell/keys.rs` (excl.) | ≤250 | the §11 binding lift |
| `src/shell/login_pane.rs` (excl.) | ≤250 | §8.3 — the Login tab's content and the auth-failed banner's, one machinery in two seats since it stopped being a fold in the roster column (bl-1ca2) |
| `src/shell/menus.rs` (excl.) | ≤250 | the §11 context-menu attach/dispatch |
| `src/shell/modal.rs` (excl.) | ≤250 | the §11 modal-owns-the-frame invariant (the backdrop layer + the one dismissal the ✕ and Escape both spend, bl-d921) |
| `src/shell/navigator.rs` (excl.) | ≤250 | §11 altitude 0 — reduced to the roster, the balls section and the two entries that FOCUS the center's Config and Login tabs, painting no surface of its own (bl-1ca2) |
| `src/shell/new_ws.rs` (excl.) | ≤250 | §3.1 |
| `src/shell/ram.rs` (excl.) | ≤250 | the shell's own cross-frame RAM, split by **lifetime**: what belongs to the window sits on `ShellState`, what belongs to a workspace sits in `ram/wall`'s `WallRam` and a focus change *swaps that bundle whole* |
| `src/shell/ram/inspector.rs` (excl.) | ≤250 | the altitude-2 viewport ephemera and its per-snapshot memos |
| `src/shell/ram/login.rs` (excl.) | ≤250 | the §8.3 holder's own file — the wall-bound `bz` runner, credentials dir and sign-in spawn layer, §16.2 |
| `src/shell/ram/wall.rs` (excl.) | ≤250 | the workspace-lifetime bundle (brazen's config pane, the §8.3 Login holder, the §9.4 picker) that a focus change swaps whole, parking the outgoing wall's under the workspace it was typed in — so a draft survives A → B → A and no stream, roster or open picker crosses a wall (bl-5894, §16.2 as amended) |
| `src/shell/row.rs` (excl.) | ≤250 | §11 rule 1b — `control_last`, the one lawful spelling of a row that pairs greedy text with a trailing control, so the verb is allocated before the words that would otherwise eat it (bl-bc06) |
| `src/shell/search_pane.rs` (excl.) | ≤250 | the §8.5 results — a view of the published answer with no RAM of its own, whose rows are addresses to open, painted as the §11 Search tab focus and handed the answer the strip already read (bl-1ca2) |
| `src/shell/settings.rs` (excl.) | ≤250 | the conversation's config-shaped rows at the **foot** of the pane's bottom stack, below the goal box since the band-order ruling (bl-58e4) — the §3.5 spend figures + the §5.1 #35 context-fullness line under them + the §9.4 model row's two dropdowns and the picker extras `m` expands, capped at half the pane and scrolling past it |
| `src/shell/slash.rs` (excl.) | ≤250 | the §8.5 line seat (the `/`-draft's Run button and Enter, the start family through the frame's typed doors, and the answer note) |
| `src/shell/{start_pane,start_rows}.rs` (excl.) | ≤250 each | §3.4/§8.1 (the per-row affordances) |
| `src/shell/top_bar.rs` (excl.) | ≤250 | the §11 chrome strip |
| `src/shell/workspace.rs` (excl.) | ≤250 | §11 altitudes 1–2 — reduced to the identity header and its banners by bl-2e18, and to nothing else since bl-8905 deleted `members`, whose descent-tree rows the unfolded list had come to repeat |
| `src/shell/acceptance/{mod,smoke,screen,fixture,naming,overlap}.rs` (excl.) | ≤250 each | the excluded full-window smoke test, whose harness (`mod`: the window geometry, the three-frame settle) and whose claims (`smoke`: what that window must show) split at the cap on `screen`'s own seam — bl-bc06's `elision` and bl-9551's `overlap` each fit alone and together crossed it, so the seam is the parent's, not either ball's |
| `src/shell/acceptance/alerts.rs` (excl.) | ≤250 | the bl-e160 drive (rendering a frame folds the §6 queue into the window's own record of what it has announced, so a focused window absorbs its asks instead of saving them up — and, because egui's `RawInput::focused` defaults to `true`, no test can ever reach the send path and pop a real notification) |
| `src/shell/acceptance/bands.rs` (excl.) | ≤250 | the bl-58e4 stack-order drive (the conversation pane's bottom bands read off the composed frame at all four sizes, in one top-to-bottom list — the settings rows below the goal box and the in-flight strip still above it — with the census of which bands each size can seat pinned beside it, and the reader shown a frame laid out the retired way so it cannot pass vacuously) |
| `src/shell/acceptance/birth.rs` (excl.) | ≤250 | the birth-block reachability drive |
| `src/shell/acceptance/drafts.rs` (excl.) | ≤250 | the bl-a69a drive that a draft belongs to its target |
| `src/shell/acceptance/drift.rs` (excl.) | ≤250 | the §9.4 drift drive (bl-2d19): a conversation whose config lineage advanced past it states its freeze and offers **both** exits — the one that keeps it (`retarget`) and the one that starts over — on that sentence's own row inside the settings seat, read off painted glyphs rather than the strings handed to the widgets; and the other direction, an undrifted conversation offered neither, which is what makes the first beat evidence |
| `src/shell/acceptance/echo.rs` (excl.) | ≤250 | the bl-915e drive (a start and a follow-up each read on the frame *immediately* after Enter, with the substrate pinned to prove no derivation ran, then landed and re-read to prove the echo gave its seat up rather than doubling it) |
| `src/shell/acceptance/elision.rs` (excl.) | ≤250 | the §11 rule 1b regression on the two witness rows (the Login verb behind the longest provider name, `assign → <ws>` behind an arbitrary ball title), each asserted in both directions and against the panel's own edge, on painted glyphs rather than galley text — and beside them L4's other question, *where* a row cuts (bl-3aa1): two activity ops sharing the audit's invariant path prefix are laid in the real trail, and the glyphs show each row ending in the leaf and agent id that tell it from the other, where the head-keeping cut painted both rows as one identical line |
| `src/shell/acceptance/floor.rs` (excl.) | ≤250 | the §11 **focus floor** (bl-478d: Tab steps the frame's focus control by control, and Space presses what it reached — driven onto the balls fold, whose press is a durable §4.1 fact) |
| `src/shell/acceptance/{focus,walk}.rs` (excl.) | ≤250 each | the keyboard driver and the §11 focus-discipline drive it steers — `focus` its pointer/launch half, `walk` its keyboard half (bl-c21f: a roster step lands the composer in both directions, Ctrl+↓ continues the walk from inside the box, and Escape still releases to a live bare plane), each asserting the model's selection beside egui's `wants_keyboard_input` so a walk that never walked cannot pass |
| `src/shell/acceptance/geometry.rs` (excl.) | ≤250 | the §11 panel-geometry regression |
| `src/shell/acceptance/hover/mod.rs` (excl.) | ≤250 | the §11 discoverability invariant — the source scan that no control escapes, and its paint-layer proof under `everything_is_visible` |
| `src/shell/acceptance/hover/lex.rs` (excl.) | ≤250 | the reduction split off at the cap — source to skeleton, comments dropped and each literal kept as its own words, its parens and semicolons escaped so a sentence cannot fake structure |
| `src/shell/acceptance/hover/scan.rs` (excl.) | ≤250 | the walk over `lex`'s skeleton |
| `src/shell/acceptance/hover/spelling.rs` (excl.) | ≤250 | the rule-3 half (bl-478d: a control's hover names its key, its §8.5 line, or the focus floor, against a vocabulary read from `keymap::spell` and `boundary::help` rather than restated) |
| `src/shell/acceptance/inbox_composer.rs` (excl.) | ≤250 | the bl-929d drive (pending rows above the draft with fold arrows, the content-derived fold line with its floor and cap, the structurally-triggered snap-down, the strip's preserved seat) |
| `src/shell/acceptance/masthead.rs` (excl.) | ≤250 | the empty world's masthead (§3.4) — the first surface an operator ever meets, and until bl-37bf one no fixture could reach |
| `src/shell/acceptance/legible/mod.rs` (excl.) | ≤250 | rule 1c's G1 sweep over the whole window at all four sizes — no run's shown width narrower than the width it was laid at, against a `KNOWN` list that bl-5410 emptied |
| `src/shell/acceptance/legible/floor.rs` (excl.) | ≤250 | the rule-1d half that sweep cannot see (no run is a bare `…`, and the composer's verbs are whole at every size), split from it at the cap along the two failure modes themselves |
| `src/shell/acceptance/mint_seed.rs` (excl.) | ≤250 | the bl-28ba drive that a landed fire retires the seed it spent and a failed one does not |
| `src/shell/acceptance/modal.rs` (excl.) | ≤250 | the modal-ownership drive (Escape dismissal, the swallowed click and the Return submit, each asserted in two directions) |
| `src/shell/acceptance/name_column.rs` (excl.) | ≤250 | the same painted-geometry discipline one altitude in — on the row rather than the panel (bl-b9e3: the title's left edge is the row's fixed prefix and nothing else, so two rows differing only in attention align — measured on the painted galleys, since a string assertion would pass on a tree that deleted the flag outright) |
| `src/shell/acceptance/picker.rs` (excl.) | ≤250 | the bl-a842 drive that the §9.4 pane's **contents** reach the paint layer — the role strip the seeded `providers.yaml` declares, the blast-radius sentence a pick claims, and the §9.2 fault an undeclared model earns — all of it behind the "cannot read `roles:`" early return until the fixture carried a config lineage, and none of it asserted by the two picker beats that measure the seat's *height* |
| `src/shell/acceptance/raise.rs` (excl.) | ≤250 | the bl-9acf raise drive (one goal box, and a blank one fires nothing) |
| `src/shell/acceptance/recall.rs` (excl.) | ≤250 | the bl-f908 drive (↑ pages back through the operator's own turns — pending ahead of delivered — ↓ hands the half-typed draft back verbatim, and a caret below the top row keeps its arrow) |
| `src/shell/acceptance/settings.rs` (excl.) | ≤250 | the bl-2e18 seat drive (the spend figures and the model row paint inside the settings panel and nowhere above it, the empty selection answers the same question in the same seat, and an expanded picker cannot grow it past half the pane) + the bl-a48b context drive (the fullness percent and its evidence paint in that seat *beneath* the spend line they are not, and an empty selection — nothing measured — paints no such row at all) |
| `src/shell/acceptance/slash.rs` (excl.) | ≤250 | the bl-ec8f drive that a `/`-draft is a command (answered under the box, refused with its draft kept, and a bare `/` listing the roster) |
| `src/shell/acceptance/start_draft.rs` (excl.) | ≤250 | the bl-6ad8 drive that a pending start draft takes the composer's seat |
| `src/shell/acceptance/started.rs` (excl.) | ≤250 | the S0.3 drive that a fired start renders its own transcript (§3.4) |
| `src/shell/acceptance/tabs.rs` (excl.) | ≤250 | the bl-1ca2 no-full-cover-overlays drive: a combo focuses Config from inside the box and Escape comes home with the draft intact, every peer stays reachable from every tab while the composer is absent rather than buried, and the Login rows land right of the roster column (asserted by geometry, since the auth banner paints the same text) |
| `src/shell/acceptance/search_tab.rs` (excl.) | ≤250 | what is true of the **Search** tab alone, split off `tabs.rs` at the budget on that seam: it is offered while a search has been asked and retired when a `/search` with no text clears it — and a needle that matched nothing keeps its tab, keeps the operator on it, and paints `no matches for <needle>` with the paved path beneath (bl-648a) |
| `src/shell/acceptance/unfold/mod.rs` (excl.) | ≤250 | the unfold's own beats, split by hand — this file the bl-d5b9 paint half (the field's two numbers, the arrow revealing exactly the direct children indented under it, recursion two deep — each needle read off the derivation, since the §3.3 floor of a chained id is its terminal *generation* and a hand-spelled hash asserts the absence of a string nothing ever paints); every unfold beat reads the **conversation column's** galleys and not the window's (`screen::column`) — written when the altitude-1 descent tree repainted every member name in the centre, and kept after bl-8905 retired it because the centre's header still paints the open conversation's own title, which is the root row's needle |
| `src/shell/acceptance/unfold/drive.rs` (excl.) | ≤250 | the bl-89de drive, pointer half (the field's click unfolds and folds without selecting the row, and its hover states both numbers at the paint layer, which is where the hover scan's descent-less fixture cannot reach it) |
| `src/shell/acceptance/unfold/keys.rs` (excl.) | ≤250 | the bl-89de drive, keyboard half (→ unfolds the selected row and again one generation down without walking, ↓ steps over a folded subtree and opens nothing, ← pages up to the parent before a second ← shuts it) |
| `src/shell/acceptance/unfold/column.rs` (excl.) | ≤250 | the name column at depth 1 — `name_column`'s rule within a depth, its two complementary predicates being that the edges of one depth agree and that each title shares no pixel with the metadata pinned right of it |
| `src/shell/acceptance/walls.rs` (excl.) | ≤250 | the bl-5894 drive over two spheres (an unsaved brazen draft survives a workspace round trip, a live `bz --login` stream paints only under the wall that started it and is parked rather than killed by leaving, and a picker opened in one wall is closed in the next) |
| `src/shell/acceptance/wound.rs` (excl.) | ≤250 | the bl-55d8 drive (a conversation whose latest step has an empty `response.json` beside a non-empty `stderr.log`, rendered on the whole window: the adapter's own sentence is in the paint output, the retired *"activity trail below"* pointer is not, and the bl-90bf grace gate still withholds it on the frame before the window elapses — driven on a `FakeClock` swapped into `ShellState::wound_grace`, since the window is wall-clock time and a frame test must not sleep through it) |
| `src/search/{mod,corpus,excerpt,worker}.rs` | 175+140+50+90 | the §8.5 global search: the `Address`/`Field`/`Hit` vocabulary, the answer that carries its own needle (`Found::asked`, the strip's offer predicate, beside `Found::is_empty`, the pane's — bl-648a) and the empty answer's wording, the deterministic rank and bound, and `run` — the one engine all three seats end in; the corpus (the snapshot half free, the conversation half re-read from disk and the half cancellation is checked between); the matched-line window at char boundaries; and the window's searcher thread |
| `src/shell/model_pick/{mod,seat,lines,marks,ram,select,write}.rs` (excl.) | 200+217+112+58+134+292+96 | the §9.4 picker's widgets, worn by **two seats** with one implementation (bl-824e): `seat` paints the row both surfaces carry — the two dropdowns, the drift clause and its **two** exits (bl-9786's new conversation and bl-2d19's `retarget`, laid as a peer strip so neither is ever dropped), the write receipt, and the pane's extras while it is open, over one `Subject` value naming what the seat is about (its scope claim, and the conversation it belongs to when there is one) — `lines` derives and memoizes what that row says (the conversation's, keyed on agent tip + config tip + role; the birth block's, on the head alone), while `mod` holds the pane the row cannot hold (the role strip that re-scopes it, the dead-assignment fault), the two scope sentences it is handed, and the routes out, which are **one value**: the seat returns the §11 tab it was asked for, named by the `add a provider…` entry and by the credential fault's remedy alike (bl-91f1); roster fire on the model list's own open (bl-cd2a) + the three-layer failure paint (bz's line verbatim, the remedy between, the run-by-hand command beneath) + the commit-on-select wiring (no buttons, bl-fb6b); the dead-assignment marks; the surface's RAM, which holds brazen's rows **whole** so the fault's `auth` column costs no second read; the row's two brazen-sourced dropdowns and the pane's role strip; the two-file write plan |
| `src/spend/{mod,prices,ceiling,render}.rs` | 215+120+150+177 | the §3.5 join, pure over the worker's pre-walked bills (bl-9dd4) — selection, attribution, the honest-granularity label, and the unpriced remainder, with `of_workspace` the one deliberate fresh walk because a gate compares against now; the price table's parse and its micro-USD arithmetic; the §3.5 spend ceiling's policy half — the operator's number and the at-or-over comparison against the workspace figure (bl-56d5); the one figure widget every spend seat paints — the board's ball rows and the conversation's settings rows (bl-2e18) — whose attribution clause is independent of the price table, so the honest-granularity label survives deleting the cost column (bl-1765) |
| `src/start/{mod,goal,identity,exec,ensure,prompt,run}.rs` | 264+181+84+240+88+128+158 | the start flow (§3.4/§8.1): pure plan, goal compose, the §3.3 stamp and its inverses, the `bl`-facing gated executors, `ensure` the workspace's existence and its policy — `lernie new` plus the §8.6 control authoring that runs outside the create skip; the §9.2 birth-template gate that once sat here is retired (bl-00ee) — the detached fire |
| `src/state.rs` | 270 | the crate's lock chokepoint: the dirty hand-off, the snapshot cell, the §8.5 search cell and the §7.2 live-tail cell — the whole inter-thread interface (§7.2, §8.5, AGENTS rule 7). The tail cell is **appended whole below every line that was there before**, and takes the snapshot cell's *alias + free functions* spelling rather than a struct with an `impl` — including leaving the module doc's stale "three residents" line untouched. That is the hazard `rules/locks-outside-state.yml` records as the reason for both its carve-outs: llvm-cov mis-attributes phantom uncovered regions onto this file's `impl` headers when anything above them moves, and an added `impl` block draws one onto itself besides. This is genuine cross-thread hand-off state — what the chokepoint exists to inventory — so it belongs here and the spelling gives way instead of the rule. The watch hub's two singletons are its second declared carve-out (§7.1, `rules/locks-outside-state.yml`) |
| `src/steps_view/{mod,detail,columns,render,drill,wound}.rs` | 160+170+110+120+200+160 | the step inspector, incl. the §7.3 no-response wound (§11 Steps). Both tiers are cut twice, read from write: `mod`+`detail` are the list/drill-in **reads**, `render`+`drill` their **paints**, and `columns` is the §11 column table — header, hover explanation and cell in one home, so no field paints without its name (bl-3ffc) |
| `src/tail.rs` | 75 | the §11 tail idiom in one home (bl-8c13): the `stick_to_bottom` anchor and the top pad that seats an underfull body on the bottom edge, taken together by every tail surface and restated by none; since bl-929d it also hands back the body height it already measures, so a content-sized region (the inbox-composer) derives its extent from the one measurement |
| `src/test_support{.rs,/workspace.rs,/world.rs}` | 287+51+68 | the test-only scaffolding every test module in this binary shares: the spawn/env serialization locks (AGENTS.md rule 7's sanctioned carve-out), a real lernie workspace on disk for §8.6 control authoring, and the §16.2 fixture world every test that touches a §9 destination or a §16.3 space reads and writes through |
| `src/theme/icon.rs` | 242 | the mark's constants, its compass geometry and the `deep` hue derivation; assembles the layer order (§11), tinted or at rest |
| `src/theme/icon/shape.rs` | 93 | what the mark is made of — the three flat primitives, the `Shape::ribs` doorway the edge-walking emissions use, and the `Tints` one hue per circle (§11) |
| `src/theme/icon/arc.rs` | 89 | the compass: an arc from two endpoints and a sagitta, and the stroke and lune built on it |
| `src/theme/icon/{raster,vector,paint}.rs` | 184+66+60 | the three emissions of that `Shape` list — window-icon rgba (ribs resolved once, before the pixel loop), the checked-in SVG, and the live egui painter the §11 mark seats draw through; the PNGs are the raster encoded at build time |
| `src/theme/icon/tests/{mod,geometry,artifacts}.rs` | 134+165+51 | what the pixels say, what the compass guarantees, and that every checked-in artifact still decodes to the mark; `icon/paint/tests.rs` reads the painted layer back the same way |
| `src/theme/{mod,badges,mark,role}.rs` + `theme/tests/{mod,tone}.rs` | 250+296+149+103 + 283+30 | the single colour/visuals/font authority + the §11 badge mappings (glyph + hue + words in one home, `doing_badge` the mark's hue-and-words pair, `verdict_badge` the VISION §4.9 standing verdict's) + the two mark seats: the resting wordmark and the live mark's tint assembly and worded roster (bl-b768) + `role` the §11 role stripe's one home (bl-3acb): the byte-derived `Role` vocabulary, its hue-and-words pair, and the stripe painter both message seats draw through |
| `src/transcript/{mod,parse,rows,rows/project,rows/project/build,rows/turns,render,spine}.rs` | 263+159+172+228+95+292+250+157 | the **committed** transcript's enumeration (the live tail is the caller's fold, §7.2 bl-54f7), forgiving parsers, the §11 row vocabulary (classes, tones, roles — bl-3acb — and the auto-state rule incl. the in-flight input) and — cut at that seam when the §3.3 sender label landed (bl-2335) — the entry→rows projection with the labels and roles, over `project/build`'s row constructor and preview/body split (bl-54f7's own cut: *what an entry becomes* vs *what a row is made of*), then the §11 turn rollup over it (bl-1f21: the turn boundary, the aggregate line, what a shut turn omits), the folding render with the role stripe at every row's edge and the bl-7654 payload rule stated at the row rather than inherited (the expanded body and the Raw view wrap; the chrome and an abridged preview truncate; only an abridged preview fades, at `theme::tone_solidity`), and `spine` — the step spine drawn *through* the chat (bl-1802): the clickable operable-commit rule that pins, and the cards and cohorts born at it, which is the whole of what the retired `history-rail` side panel was |
| `src/ui_state/{mod,json,knobs,fields}.rs` | 238+45+100+115 | the `ui.json` schema, write-through save, echo-hash, seen API, the knobs + `zoom` + the §6 escalation's `notify_unfocused` (§4.1); the per-field accessors derived from it; the crate's one ISO 8601 formatter (`format_iso8601`/`iso8601_extended`, ported calendar math, no `chrono`/`time` dep — bl-61db) |
| `src/ui_state/panels.rs` | 125 | the §4.1 `panels` object: the `Panel` enum (key, default, floor, ceiling + the one clamp — one home per boundary) and its forgiving read / snapped write |
| `src/ui_state/{prices,ceiling}.rs` | 75+55 | the §4.1 `prices` object: the §3.5 price table's one read, forgiving and setter-free; the §4.1 `ceiling` number beside it, read the same way — absent is no gate (bl-56d5) |
| `src/app/panels.rs` | 60 | the model's panel-size fold (§11 rule 5's clamp on read and on settle) and the settle rule (one drag, one write) |
| `src/ui_state/prune.rs` | 130 | the §3.6 prunes: a deleted workspace's keys; a deleted conversation subtree's `seen` watermarks (bl-f17a) |
| `src/watch/mod.rs` | 250 | WatchSet reconcile, the ingest bridge thread, repaint hook (§7) — including the boxed hook that lets one `Engine::boot` serve both faces (bl-f6fe) |
| `src/world/wall.rs` (+ tests) | 110+95 | the per-workspace wall (§16.2, §3.1): the `YOG_WALL` layer, its layout and its read lens |
| `src/world/{mod,seed,marks,hatch,tools,seat,tests}.rs` | 188+95+259+192+199+80+200 | the composed world (§16.2): env + overrides, `lernie prime` seeding, the §16.3 **agent balls space** (the `YOG_MARKS` fold, balls' two home directories per space, and the one `tasks_branch` write/read — bl-e47b), the §8.4 hatches, the §16.4 shim roster (the §8.6 control shim and, since bl-3ff4, `yog` itself among them), and `seat` — which seat may open a window, the guard that keeps that `yog` shim from becoming an agent's way to paint on the operator's desktop. `template.rs` — the §9.2 gate over the workspace-birth template — is deleted with the gate (bl-00ee): yog reads that file nowhere now |
| `src/xdg/mod.rs` | 295 | env folds: balls layout (delegated to `balls::layout::Xdg`, over the §16.3 space's two home directories), lernie roots, yog roots, the wall (§16.2), percent-decode. brazen's ambient per-OS fold was deleted with the sharing it served |
| `tests/design_citations.rs` | 160 | the citation guard: every cited `§N`/`§N.M` resolves to a DESIGN heading (the header's retirement doctrine, machine-checked) |
| `tests/design_module_map.rs` | 168 | the module-map guard (bl-9f72): every production file under `src/` has a row in this table, and every `src/` path a row spells exists — brace lists expanded, test corpora excluded per the rule above |
| `tests/integration/glyph_coverage{.rs,/scan.rs}` | 125+180 | the tofu guard (§11): every non-ASCII `src` literal has a glyph in both font families — the assertion, and the tree scan + font probe behind it |
| `tests/integration/support/{mod,recorder,world,payload,clock}.rs` | 190+209+229+105+70 | the story harness (STORIES "Test harness"): the fake-`bl` runner and the one-agent workspace, the argv/env recorder script and its read-back parser, the **multi-agent** workspace builder (goal stamps, `refs/lernie/*` marks incl. the hold blob, dated commits, settled step framing), and the on-disk payload writers (`messages/`, `steps/`, `inbox/`, the balls clone dir) it composes — plus `clock`, the harness's own hand-driven `Clock` (bl-9006): `AppModel::boot` takes an `Arc<dyn Clock>` exactly so a test can supply one, and until INV-1 reddened under a nine-tarpaulin gate every beat in this crate had booted on the system clock and measured the machine instead of yog. Split at those three seams when the Z10–Z14 fake halves landed (bl-3b24) |

Testing per the house pattern: real-git tempdir fixtures (extended with a
balls-clone-layout and yog-ball-root fixture builder), argv-recorder scripts
under the binary-wide SPAWN_LOCK, fake `/proc` and fake `lsof` output
injection, injected clocks for every debounce/sweep branch, headless
shape-walk for every render fn, forgiving-read cases for every file parser,
`--test-threads=1`, tarpaulin 0.35.2 pinned, 100%.

### 12.1 Code style is governed by Rust Bootstrap v3 (see AGENTS.md)

DESIGN is the *architecture* authority; **code style is governed by the "Rust
Bootstrap v3" standard, whose yog-adapted, flat-numbered rules live in
`AGENTS.md`** at the repo root, machine-enforced by `rules/*.yml` (pinned
ast-grep 0.44.1), the clippy manifest (`Cargo.toml [lints]`, pedantic=deny), and
`cargo-deny` (`deny.toml`). `make check` runs the whole gate; the pre-commit
hook and CI mirror it. Read AGENTS.md before writing code — this note only
records that the standard exists. The deliberate skips (no workspace split,
no musl target, `unsafe` confined not forbidden, no `anyhow`, async rules
vacuous, no pre-commit framework) are recorded **in AGENTS.md's numbered
rules as yog ADAPTATION notes** — one home; do not restate them here.

### 12.2 The drive harness (`scripts/drive/`)

The other half of the source tree the 300-line cap governs: the real-substrate
harness, whose *doctrine* is STORIES.md's ("Real-substrate drive") and whose
*shape* is here. Bash, no repo deps, five tiers — a front door, a seat, a
fixture, a verdict, and the story beats — cut so that no tier knows the tier
above it.

| File | Est. lines | Responsibility |
|---|---|---|
| `scripts/drive/drive.sh` | 158 | the front door: the live-world refusal (a two-directional path-prefix test against `$XDG_DATA_HOME` — a run wipes its world before it starts), the `target/release` PATH prefix that makes the drive prove the build in hand, one scratch world per run verb under a stamped evidence root **outside the checkout** (a world nests `git init` fixtures and a path-mirroring delivery territory; inside the repo those come within a fixture's reach), and the log skeleton at the tail. The Makefile's `drive` family is a one-line wrapper over it |
| `scripts/drive/preflight.sh` | 157 | the host contract, named in full and at once: the seven binaries the scripts actually call (`Xvfb`, `xdotool`, `ffmpeg` for capture, `ffprobe` for `locate.sh`'s read of a shot's size, `python3`, `git`, the `yog` under drive), the two world-seed files every run verb copies, and — since §16.2 moved brazen inside the wall — the **wire** tier (bl-49c6, demoted to advisory by bl-00ee): per provider row the seeded birth template names, whether a NEWBORN wall's table ships it (asked of the binary under drive through an empty `YOG_WALL`, never a copy of brazen's defaults kept here to drift) and whether the host credential `seed_wall` copies into the wall exists. Advisory both, because birth no longer gates on either — only a beat that SPENDS does |
| `scripts/drive/yogdrive.sh` | 114 | the seat primitive: an isolated per-run Xvfb display and the window verbs on it (`launch`/`shot`/`type`/`key`/`bare`/`click`/`stop`), never the operator's seat (bl-4132). `launch` hands over the scratch data root and nothing brazen-shaped — the ambient-credentials symlink it used to lay fed a path no driven process has read since §16.2's ruling (bl-49c6) |
| `scripts/drive/locate.sh` | 252 | the pointer's answer to its own doctrine (bl-b9f2, extended to the whole harness by bl-5cce): where a beat must still click, the point is read off the frame it is about to drive rather than measured against a screenshot taken on an earlier day. Three balls (bl-2622 → bl-f8dc → bl-b9f2) re-measured the same §9.1 pixels before this existed, the last after bl-5410 gave every provider row a second wrapped line and moved the fold 119 px. None of these controls has a §8.5 name or a §11 key, but each has a **position relative to a structure**, and that is what is written down. **Four surfaces, three anchor families, one rule-finder.** ffmpeg flattens the shot the run already takes to one grey plane and a `python3` heredoc finds every horizontal rule in it — long, flat, brighter than a background it *reads* rather than assumes — split into two families by one test: a section separator starts at its own column's left edge, a window-wide panel's rule at x=0. So `brazen` folds the §9.1 fold, editor and Apply/Reload off the second rule of the §12 Config column; `inspector` folds the Raw toggle, the step selector and the record picker off the rule the centre paints between a conversation's header and its altitude-2 strip; `tabbar` takes the ⋯ overflow and the ★ in its menu off the top bar's own rule and the window's right edge; `activity` takes the trail's newest ops row off the window's bottom edge, since §11's tail idiom seats it there, and reads the panel rule only as the guard the beat otherwise lacks — a still-collapsed accessory is refused rather than clicked shut. No new host tool beyond `ffprobe`, and nothing assumes the 1150x760 window. The §9.1 Apply/Reload are the two the §11 focus floor cannot reach (bl-b9f2 re-drove the floor to check: `.code_editor()` is `lock_focus(true)`, so Tab and Shift+Tab are both indentation and the walk cannot leave the box) |
| `scripts/drive/harness.sh` | 266 | the tier every run shares, sourced by `stories.sh`: the ops/tree assertion helpers — including the ones that name a conversation by **identity** rather than by a count or a rank (`stopped`, `other_root`, `seen_kind`), since yog's list has ranked nothing since bl-cad5 — the two waiting primitives (`await`, `until_landed` — nothing waits on a clock, and `until_landed` takes a **no-op-on-miss gesture and a MONOTONE predicate**: it re-fires, so an equality on a quantity the gesture adds to is destroyed by its own retry loop, bl-0e44), `one_name_one_definition` (the guard over the one flat sourced namespace every `beats_*.sh` lands in — a duplicate top-level beat name silently deletes the earlier stage and leaves no verdict row to say so, bl-0e44), the per-run seat pair, the verdict in both halves — the printf PASS/FAIL line and the `verdicts.jsonl` row beside it, which is also where the binary under drive is resolved and recorded (bl-d1af) — and the file predicates two runners share, `file_has` and `md5of` (which answers `absent:<path>` rather than the empty string that made two absences compare EQUAL, bl-f16e) |
| `scripts/drive/wall.sh` | 67 | the §16.2 WALL FIXTURE, sourced by `harness.sh` and spent by `stories.sh`'s `seed` and `beats_s5.sh`'s `wall_config`: `BOOTSTRAP_WS`, `wall_dir`, `seed_wall`. The one tier that LAYS state instead of reading it, which is why it is not in `harness.sh` — that file's own header is assertions, waiting, seat and verdict, and a fixture that copies a host credential into a scratch world is none of them (split at the cap, bl-f16e). A wall is keyed by a NAME, and for every world this harness drives that name is §3.1's bootstrap constant `home`, so the wall goes down with the world seed *before* the launch rather than chasing a mint the first model call has already outrun (bl-49c6, bl-1851) |
| `scripts/drive/stories.sh` | 299 | the S0/S1 beats and the STEERING doctrine that aims every beat in the family (§11 keys where the subject is the window, the §8.5 line where it is not, a coordinate only for a view, and never a pinned one — `locate.sh` above); the world seed — lernie's roster and birth template **and the bootstrap sphere's wall**, both halves of one fact in one place (bl-1851) — the `gesture` transport, and the verb dispatch |
| `scripts/drive/beats_{s3s4s6,s5,s7,s8,s3res}.sh` | 274+299+244+156+178 | one file per world, sourced by `stories.sh` — same helpers, same seat. Split for the cap, and the split is also the fixture boundary: each world's beats need a different seeded state. **A fixture acts on a git repo only where that repo is the directory's own toplevel** — the delivery territory mirrors the project path, so a name match can land on an interior mirror component and `git -C` walks up from there into whatever real repo encloses it |
| `scripts/drive/beats_s6.sh` | 214 | the **S6 stage tier**, cutting the other way: not a world but a story. A run verb owns its world and its fixtures and reaches here for the S6 beats that world can support — `run_s7` for `s6_attention` (world C's S6-T1 budget/conflicted/mail stage), the forensic predicates, the pins and the ack convergence; `run_s3s4s6` for `s6_stop_ack`, the stop and its acknowledgement. **Two stages, two names** — they shared one until bl-0e44, and the later definition silently ate the earlier. So an S6 stage never sits inside a run's body, which is the seam `run_s3s4s6` was split along when carrying it put that file over the 300-line cap (bl-2d45) |
| `scripts/drive/beats_headless.sh` | 234 | the **SEATLESS run verb**, `run_headless` — the only one that claims no X display, opens no window and spends nothing on the wire (bl-bb20). `yog headless` is the same engine with no face (§8.4), so the whole run is `yog gesture` lines against a real world, and it is the one verb drivable on a box with no X server. It carries the graduated rungs whose claims the REAL substrate can falsify and a fake cannot: S13's board columns over real balls state and real blocker resolution, S11-T4's work-diff as real git over the worktree `bl claim` cut, S2-T1's prepared cwd, S14-T8 (its own premise) and S14-T5. The file's head records the whole scope decision, rung by rung, including every rung ruled OUT and why |
| `scripts/drive/cleanroom.sh` | 88 | the §16.7 W14 standing done-bar: a room whose only substrate on `PATH` is one `yog`, asserted rather than assumed — and, since §16.2 moved brazen inside the wall, asserted in the same two directions for brazen state: the room lays no ambient `$XDG_CONFIG_HOME/brazen` and refuses to run if one is there (bl-49c6) — then handed to `stories.sh` unchanged |
| `scripts/drive/logskel.sh` | 173 | the report generator: sha, host tuple, load, the driven binary and the beat table emitted from `verdicts.jsonl` — nothing re-asked of the generator's own PATH, which resolves the *installed* yog and not the build under drive (bl-d1af) — with every judgement left as an explicit hand-finish marker. A drive log starts generated and is finished by hand; the house style (evidence quoted, not summarized) is the operator's half |

**The verdict has two halves and one emission.** `pass`/`fail` print the human
line *and* write one JSONL row — `{run, beat, label, verdict, detail, evidence,
bin, at}` — to `<out>/verdicts.jsonl`, beside the `gestures.jsonl` the §8.5
transport writes. Neither half is a summary of the other, because both come from
the same call. The **label is the single source** of a beat's name; `beat` is that label
slugged, so no second name exists to drift, and a consumer needing exactness
reads `label`. `evidence` names the newest shot in the run directory at the
instant the verdict was taken — a measurement of what the beat had driven to,
which is why the field names a file rather than claiming it proves anything.
`bin` is the binary the run drove, resolved **inside the run** from its own
PATH — the only place the answer exists, since `yog` is launched unqualified
and `drive.sh` prefixes the checkout's `target/release` onto PATH. A reader
that re-asks `command -v yog` afterwards resolves the operator's *installed*
yog, a different sha, into the one field a drive log exists to pin (bl-d1af);
so the row carries it, and an unrecorded or divided answer is loud in the
report rather than plausibly wrong.

---

## 13. Deliberate interpretations (user-vetoable)

These are deliberate readings of the requirement, flagged for veto rather than
silently assumed.

### 13.0 The rule is no *state* in RAM, not nothing in RAM

The user clarified the durability requirement: **"no STATE in RAM" — not
"nothing in RAM"; views are fine in RAM.** This is the primary rule the rest
of §13 applies. A *view* is which data you look at and how the window is
arranged: focus, selection, scroll position, which nodes or panel sections you
have folded, live window geometry. *State* (durable data) is a user assertion
with no other authoritative home: the seen watermarks, pins, the last-used
identity. State must replicate so two instances agree (§4.1); a view may live
in RAM and be lost on crash. §13.1's focus/scroll ruling was the first
statement of this rule and is now just its leading instance. The persisted-view
keys (`collapsed` — §4.1) are the allowed converse: a view is
*permitted* to be kept durable for convenience, but is never *required* to
replicate the way state is.

### 13.1 Live focus/selection and scroll are per-instance viewport ephemera

The requirement: "effectively nothing is allowed to exist in only-RAM… two
yog instances side-by-side faithfully replicate the same data, excepting
unsubmitted user inputs." **This design interprets the requirement's motive as
data durability and replication: no *data* is RAM-only, and both instances
replicate the same *data*. Live focus/selection and scroll are *which data
you look at*, not data:** they lose nothing on crash (focus re-derives
deterministically at startup — next-attention, else first; scroll re-anchors),
and mirroring them makes side-by-side instances actively hostile — every
click in one yanks the other's view, which two of three judges found defeats
the point of running two instances. `ui.json` therefore keeps the
genuinely-converging *state* — the four seen watermarks, pins,
`identity_last_used` — plus the persisted *views* kept for convenience
(`collapsed`; §13.0). **Which rows are unfolded is this rule, not that one**
(bl-fa82): the §11 conversation list's expanded set, the jsonview collapse set
and the transcript's fold overrides are all "which data you look at", keyed by
content rather than by a named section, and all three live in RAM (§5.3). The
`collapsed` array stays the deliberate counter-example it always was.
Everything the operator would call data — including acknowledgements, which a
zero-durable-state stance structurally cannot represent — is durable and
converges. **Veto path:** if the strict-literal reading is wanted (mirrored
focus/scroll), add `focus`/`scroll` keys back into `ui.json` with content-
anchor scroll representation; the write/adopt machinery already supports it.

### 13.3 Detached prompt defers error immediacy to disk

`lernie prompt` is spawned detached (§8.1), so a prompt-time failure surfaces
from disk rather than from a pipe yog holds. This is the
robustness-over-immediacy trade: yog's death must never kill a running loop,
and the short piped steps (`bl claim`, `lernie new`) still surface their errors
directly in `ops.jsonl`.

**Deferred to disk, not discarded (bl-a649).** stdin and stdout stay null;
**stderr goes to a per-spawn sink file** (§4.2 / §5.2:
`$XDG_STATE_HOME/yog/detached/<ts>-<workspace>.err`) — without it, a driver
that launched and then died was byte-identical to a clean launch. The sink is
a *file*, not a pipe: yog holds no fd, so the §8.1 lifetime guarantee is
untouched — the child outlives yog and keeps writing to an inode nobody must
be alive to drain. The ops row's `stderr` is folded in from that file **at
read time**, on the ops sweep (§7.2), and a non-empty capture makes the row a
rendered failure — the existing §7.3 machinery. Only the *immediacy* is still
deferred: the death is visible on the next sweep, not at fire.

**A failure to *launch* is not deferred at all (bl-afa9).** The one detached
failure yog observes synchronously is the fork itself, and it is logged as what
it is — the §4.2 `-3` synthetic-failure line, with `CliError`'s wording (a bad
work directory blames the directory, bl-6191) in `stderr` — never as a `-2`
handoff row. The rule the two halves share: `-2` is written when, and only when,
a child was actually handed off.

**The conversation surface owes its own half (bl-7f2e).** The sink answers
*why* on the ops surface; it cannot answer *where*, because a driver that dies
**mid-life** has already written a step, and that step — `response.json` at zero
bytes, no `meta.json` — read as a quiet one (§4.4 framing has no vocabulary for
"produced nothing", so it says `Killed`, the same ash badge a mid-stream kill
gets). The **no-response wound** (§7.3, §11) is that vocabulary: derived at read
time from the two files plus the agent's §3.5 liveness, stored nowhere, rendered
in ichor beside the step and bannered at Altitude 1.

**The division was drawn one file too far left (bl-55d8).** It read: the
conversation surface says *this died*, the ops surface says *what it said on the
way out*, and the wound's own text points across it. That pointer is empty for
the class the operator actually hit. Only a driver **yog** fired has a §8.1
sink; a turn continued by `lernie message` is driven by a child lernie launched,
so no `-2` row exists to fold anything into. Meanwhile the adapter's own words
were already sitting inside the step yog was reading — `stderr.log`, the third
file of the record (lernie ARCH §2.3). So the conversation surface owes *both*
halves for this class, and the wound states the reason instead of pointing
across at a trail that may hold nothing. The ops sink is unchanged and still
answers for the spawns yog itself fires.

---

## 14. Rejections

Recorded so they are not relitigated:

- **`toml_edit` or any TOML/YAML dependency** — the brazen structured view is
  `bz --dump-config`; balls data is `bl list --json`; a yog-side parse is a
  second authority that drifts. (Unchanged by §16.7's embedding: yog links
  brazen and calls *its* parser, which is the rejection honored, not evaded —
  a direct `toml` dependency in yog's own manifest would still be a second
  authority.)
- **flock-acquire liveness probing** — perturbs the substrate: a transient
  yog hold makes a `lernie message` writer skip launching a driver and strand
  the deposit (I8).
- **Direct `tasks/*.md` parsing** — re-implements bl's frontmatter parser and
  status/admits ladder; `bl list --json` is the sanctioned bedrock contract.
- **Any yog-side ball↔workspace registry file** — drifts and needs its own
  merge discipline. The claimant field is not a registry: it is balls' own
  first-class metadata under balls' own merge discipline, which is why binding
  lives there (§3.2).
- **Path-convention binding**
  (`yog/balls/<mirrored-project-path>/<ball-id>/`, the original §3.1, with its
  verbatim-vs-percent-encoding debate) — *superseded by the claimant join
  (§3.2)*: a location is congenital and immutable where the requirement is
  late-mutable, and it hard-coded 1:1 where agents author and pick up several
  balls per workspace.
- **Zero-durable-state stance** — structurally cannot represent notification
  acknowledgement (no lernie ack verb exists; marks are level-triggered) and
  violates the governing requirement.
- **Ops-log-free design** — gate/close/scan output would be RAM-only and
  non-convergent; the current stderr-and-drop hole would persist.
- **The word "session" in code or UI** — lernie bans it for cause; the
  concept dissolves (start = claim+new+prompt, the unit is the workspace, the
  list is workspace enumeration).
- **Daemon/socket/IPC between instances** — disk is the bus (every
  substrate's religion). *Scope narrowed by the client/server split (bl-b9a2,
  `docs/REMOTE.md`): a seat may reach the §8.5 boundary over one mTLS channel —
  that is a client transport to one engine, not instance coordination, which
  stays disk-only.*
- **Linking lernie/brazen crates *as a blanket rule*** — *superseded by
  §16.5.* The original stance (CLI + disk is the whole contract; brazen's
  library API is explicitly unstable) is now the *phase-1* posture only; the
  phase-2 end state embeds `balls`/`brazen`/`lernie` as **exact-pinned** crates.
  Instability is answered by exact-pinning (brazen's own README posture), and
  process semantics stay non-negotiable regardless of linking — drivers are
  processes holding flocks; plugin dispatch stays subprocess (§16.5).
- **A reproduction hatch at the wound** (a "re-run this prompt with stderr
  attached" button beside a §7.3 no-response step, running `yog exec lernie
  prompt …`) — *rejected on the evidence, not the ergonomics*. `yog exec` is how
  the bl-8e07 skew was diagnosed, but that was **before** the detached child's
  stderr sink (§8.1/§13.3): the driver's own words are now captured and rendered
  on the ops surface, so the button would ask the operator to re-create a fact
  yog already holds — and would fire a *second* driver at a conversation that
  already has one, with a goal yog would have to re-compose. The wound instead
  names where the cause lives, and the two surfaces stay disjoint the way §6
  keeps them: the conversation says *this died*, the ops trail says *why*.
  `yog exec` stays exactly what §8.4 makes it — a hatch for a human at a shell,
  not a verb yog fires at itself.
- **Async runtime** — the eframe loop + notify threads + repaint scheduling
  is the entire concurrency story.
- **yog aping lernie's seeding** — the nested `LERNIE_HOME` is seeded by
  lernie's own bootstrap verb (**`lernie prime`**, landed upstream as
  bl-6d83: `LERNIE_HOME=<dir> lernie prime`, seed-if-absent, idempotent,
  silent on success; `models.yaml` at the home root is the seeded marker),
  never by yog reproducing lernie's seed logic; a second seeder drifts from
  the first (§16.2).
- **Overriding `XDG_DATA_HOME` in the world env** — `$XDG_DATA_HOME` is the
  world's *anchor* (the world lives under `$XDG_DATA_HOME/yog`; overriding it
  recurses). The original rejection also kept `XDG_CACHE_HOME` ambient to
  share brazen's credentials and cache with the ambient world; that half is
  superseded by the blast-radius ruling — brazen state resolves
  per workspace (§16.2) and only the anchor stays ambient. The anchor half
  stands and is load-bearing twice over: it is also why the wall cannot be an
  `XDG_DATA_HOME` override and why yog supplies brazen's credential/cache
  seams instead (§16.2's wall paragraphs).
- **yog shipping or installing tool binaries as the end state** — a yog-owned
  pinned bin dir with `cli_outbound` preferring it is phase-1 scaffolding,
  retired wholesale by phase 2 (§16.4). In the end state the version mechanism
  is the exact-pinned crate, and the only binaries are the user's ambient CLIs
  (orthogonal to yog) and the embedded-crate agent-tool shims (§16.4).

---

## 15. Implementation epic (retired)

The v1 implementation epic — milestones M1–M6, tasks Y1–Y24 and Z1–Z9 — is
complete and retired to git history (bl-43cd). `git log --grep '[bl-'` and
the closed balls listing reach every landing; STORIES.md remains the
acceptance ladder for the conversation-first surface. The heading stays so
citations (`§15 M6`, `§15 Z9`, …) keep resolving.

---

## 16. The yog world

yog owns a **world** — a nested substrate environment it composes under its own
data root and hands to every child it spawns. This section is the world's
normative home; §0, §1, §2 (I1/I2/I7), §3, §5.1, §8, §12, and §14 are amended
to point here.

### 16.1 Application, not a layer

The naïve framing has yog as a thin viewer over the user's ambient
`bl`/`lernie`/`bz` state. It inverts: yog composes its own nested world
(§16.2), and the substrate state yog drives is *yog's*, under yog's data root.
Playing on top of the user's *direct* tool usage stays possible — an agent's
task branch can be pointed at the project's shared store branch at launch
(§16.3's launch clause) — so **compatibility with an ambient workflow is the user's decision,
not a structural given.** The world encapsulates lernie, balls, and brazen
state so completely that yog and the human's own shell never collide unless the
user chooses overlap.

**Rejected:** yog as a pure ambient overlay (no nested state) — the coordination
point would be the user's live working tree and clones, so every yog action
would perturb the user's own `bl`/`lernie` work; encapsulation is what makes yog
safe to run beside a working human.

### 16.2 The composed world environment

yog reads the ambient environment once (`xdg::Env::from_env`), computes its
data-root anchor `$XDG_DATA_HOME/yog`, and composes **one** world `Env` — the
ambient snapshot plus a fixed override set — used both to derive every substrate
path yog reads *and* to spawn every child. Overridden, to nest:

| Var | World value | Nests |
|---|---|---|
| `LERNIE_HOME` | `<yog-data-root>/world/lernie` | lernie config **and** data (the `Env::lernie_home` collapse) |
| `XDG_STATE_HOME` | `<yog-data-root>/world/state` | balls clones/worktrees/op-logs **and** yog's own `ui.json`/`ops.jsonl` |
| `PATH` | `<yog-data-root>/world/tools:$PATH` | the tool an agent's bash *finds* — yog's own `bl` shim ahead of any host binary (§16.4, §16.7 W9) |

The first two nest **state**; the third nests the **toolchain**. It is the same
encapsulation argument one layer up: env inheritance already makes an ambient
`bl` read the *right paths*, but it is still not yog's balls implementation
(§16.4's (b)), and under W14's clean room there is no host `bl` at all. It is a
**prepend, not a replacement** — everything else on the operator's `PATH` still
resolves — and the fold is **idempotent**: a `PATH` already led by the tools dir
is returned unchanged, so re-deriving the override set from the world `Env`
(which `world::marks` and the brazen config pane do) never stacks a second
entry.

Left ambient, deliberately:

| Var | Consequence |
|---|---|
| `XDG_DATA_HOME` | the **anchor** — the world lives under `$XDG_DATA_HOME/yog`; overriding it would recurse |
| `XDG_CONFIG_HOME` | ambient **as a var**, so an agent's `git`/editor/shell config still resolves — but balls' own two config-home folds do **not** ride it: yog supplies balls' homes at its `bl` seam instead (§16.3's space). Left whole, balls read the operator's `~/.config/balls` from inside the "nested" world, and a stale seed template there pruned the plugin schedule out of every landing yog founded (bl-e47b) |

**Brazen is not ambient (the blast-radius ruling — reversing the ruling settled
at W10).** A workspace is an app-wide blast radius (§3.1) — different sets of
conversations, settings and providers, all of it. Provider rows are
credential-adjacent workspace settings, so brazen's config — and with it the
credentials it points at and the model cache beside them — resolves **inside
the focused workspace's wall**, never against the machine's own
`$XDG_CONFIG_HOME`/`$XDG_DATA_HOME`/`$XDG_CACHE_HOME` brazen state. The W10
reasoning ("one shared host `bz` reads the file, so nesting it would only
orphan the credentials it points at") died with the host `bz` itself: the
linked brazen is yog's own code (§16.7 W10), so there is no schema skew to
protect against and no shared binary to orphan — and what remains is the
sphere wall, where a corporate workspace's providers and sign-ins are exactly
what must never shine into a personal one. Everything version-fragile
(lernie's home, balls' store layout) stays nested as before.

**The wall (bl-c0e2).** One layer inside the world, keyed by the workspace's
§3.1 name — which is what that name already *is*, "the dir leaf and the ball
claimant, a boundary's label":

```
<yog-data-root>/world/walls/<name>/brazen/config.toml
<yog-data-root>/world/walls/<name>/brazen/credentials/<provider>.json
<yog-data-root>/world/walls/<name>/brazen/models/<provider>.json
```

The layout is **yog's own**, so it is those three leaves on every §10 target
and the per-OS branch that used to reproduce brazen's ambient dirs is gone
with the sharing it served. Leaves are unique across the three enumeration
roots by construction (§3.1 refuses a name equal to an existing leaf under any
of them), so a foreign or replay workspace gets its own wall by the same fold.
§3.6's deletion removes the wall with the workspace: an orphaned wall would
hand a dead sphere's credentials to the next workspace to take its name.

**A wall is born empty, and its rows arrive after birth (bl-00ee).** *yog* seeds
no wall, ever. A newborn workspace answers brazen's shipped rows and nothing
else, and every row beyond them — a custom provider entry through the §9.1
editor, a sign-in through §8.3's Login roster — is a per-workspace act the
operator performs afterwards. That is the sphere wall working as designed, not a
gap: a corporate sign-in must not shine into a personal workspace, so there is
deliberately no inheritance to make birth richer. Any yog surface that judges a
provider row must therefore judge it against a wall that exists — §9.2 records
the one gate that did not and was retired for it.

**"Nothing can seed a wall before the birth" was this section's stated reason,
and it holds only of a MINTED leaf (bl-1851).** A wall is keyed by a *name*, so a
hand that knows the name can lay one whenever it likes — and the empty-world
start's name is not minted at all: §3.1 fixes it at the constant `home` ("a
constant, not a config … and not a mint"). Only the deliberate §11 `w` sphere
carries a leaf nothing can precede, and even there the leaf is the operator's own
typing. Correcting the reason changes no behaviour and un-retires no gate — what
§9.2's birth gate lacked was never a knowable name but anything of yog's own to
put in the wall — but it is the whole of `scripts/drive/`'s wire premise. The
harness lays `<world>/walls/home/brazen/` **with the world seed, before the
launch**, because yog's bare start is one gesture (the mint and the detached
first `lernie prompt` fire together, §8.1), so a fixture laid "after the mint" is
always later than the first model call: every scratch-world drive run's first
turn died `unknown provider` for two days while the beat asserting its reply read
as a wire outage.

**One var carries it: `YOG_WALL`**, layered onto the world's override set for
a workspace-bound read or spawn, never part of the world set itself (the world
is workspace-free — it is a pure function of the anchor). Everything else is
derived from it, so there is one fact and no second var to drift. Setting it
once at the edge that knows the workspace is enough for the whole descendant
tree: the fired `lernie` loop inherits it, lernie hands its own environment to
every tool subprocess (lernie ARCH §3.3), and a bare `bz` in an agent's bash
is the world's shim re-entering yog (§16.7 W9/W12), which folds the wall back
out of its own process env.

**Why yog's own var rather than brazen's knobs**, and why yog supplies
brazen's credential/cache seams: `BRAZEN_CONFIG` selects only the config file,
while credentials and the model cache fold off `$XDG_DATA_HOME` /
`$XDG_CACHE_HOME` — and `$XDG_DATA_HOME` is the anchor, which must not be
overridden (§14). brazen's shipped shim reads those from the **process**
environment, so no injected `Env` could move them for an in-process call
either. brazen's own exposure note names the exit — *"a host wanting isolation
can supply its own seam impls instead"* — so yog roots the credential store
and model cache at a path (`bz_host::store`), which makes the in-process seat
and the spawned `yog bz` seat resolve one wall through one fold.

**No wall, no `bz`.** A seat inside no workspace has no providers, no sign-ins
and no model cache, so the embedded `bz` refuses (exit 64, brazen's usage
class) rather than reading the machine's own state. Every yog surface guards
before it calls — the §9.1 pane, the §8.3 login roster and the §9.4 picker all
render the guard — and a `bz` under an agent inherits its wall, so the refusal
is only ever reached by a `bz` invoked outside any workspace. That includes a
bare `yog bz …` at a shell a bare hatch (§8.4) dropped into: a hatch with no
`--ws` hands out the **world**, which names no sphere. `yog env --ws
WORKSPACE` / `yog exec --ws WORKSPACE …` is how a headless seat names one
(bl-b589) — the supported sign-in spelling, and the only way a `bz` outside the
window reaches a wall at all.
Because `$XDG_DATA_HOME` is *not* overridden, re-deriving the anchor through
the world `Env` yields the same path — the composition is self-consistent, one
lens, no bootstrap special case.

**The wall owns the RAM over those files, not just the files (bl-5894).**
Moving the config, the credentials and the model cache inside a workspace is
only half the ruling: the GUI state *about* them — brazen's unapplied draft,
the Login pane's live `bz --login` stream, the §9.4 picker's open flag, role,
half-made pick and fetched roster — is equally a workspace's. Held as one box
per window and re-lensed when focus moved, it failed in both directions at
once: re-loading the box threw away a draft the operator had typed (§5.3 says
unsent input lives until it is sent or dismissed, and a focus change is
neither), while everything the re-lens did not touch stayed on screen and
clickable under the *next* sphere. So the wall holds the box: `WallRam`
(§12, `shell/ram/wall`) is folded once per workspace and swapped whole when
focus moves, with the outgoing wall's parked under the workspace it belongs to.
Preservation and isolation stop being two rules — A's surfaces survive because
they never left A, and B cannot see them for the same reason. Two consequences
worth naming: a running sign-in is **parked, not dropped** (dropping the
`Stream` SIGTERMs a sign-in mid-flight, and it is writing *A's* credential
either way), and the world's own editors — lernie's global config, the yog
clock's `cadence.yaml` — deliberately stay out of that bundle, since one file
with one draft must not become one draft per sphere.

**I1/I2 hold against the nested disk:** every §5.1 derivation resolves through
the world `Env`, so "disk is the app" means yog's world; two instances compose
the identical world from the identical ambient env and converge exactly as
before. **Severability widens (§3.1):** one `rm -rf $XDG_DATA_HOME/yog` erases
the entire world — nested lernie home, nested balls state, and yog's own
artifacts — and leaves the *ambient* substrates untouched.

The `LERNIE_HOME` seed is written by lernie's own bootstrap verb
(`lernie prime`, upstream bl-6d83), never by yog — yog composes the env and
calls the verb; it never apes lernie's seeding (§14).

### 16.3 The balls task branch: per agent by default, settable at launch

**Each agent tracks on its own balls branch (the per-agent ruling).** By
default each agent gets its own balls branch for tracking; an agent's branch
can be set at launch; subagents are passed their parent's space by default; and
an agent can amend its own branch to change its config, which is what the
launched-then-told-to-work-on-a-project case needs. Four clauses, and they are
the design:

- **The default is per-agent.** A launched agent's task tracking lands on a
  branch of its own, so two agents' task churn never collides and one agent's
  working set is legible on its own ref. The shared `balls/tasks` store branch
  remains the *project's* stable contract — but it is a destination an agent
  is pointed at deliberately, not the water every agent swims in.
- **The branch is set at launch.** Launch is where the operator (or the
  dispatching parent) names the branch when the default is wrong — an agent
  raised to work an existing project is pointed at that project's branch from
  birth.
- **Subagents inherit their parent's space by default.** A descent works one
  set of tasks, so a child lands on its parent's branch unless dispatched onto
  its own.
- **An agent can amend its own branch.** The launched-then-pointed-at-a-project
  case: the agent retargets its own tracking mid-life, through the same write
  path as everyone else.

**The framing that stood here is superseded.** The old section — *"shared by
default, with a no-marks knob"* — cast a private branch as **stealth**,
invisibility traded against coordination, where this ruling makes the private
branch the ordinary case and the shared branch the deliberate join. A branch is
a tracking space, not a hiding place, so the three modes (`shared` / `stealth` /
`branch <name>`) collapse to the one value they were always about: a branch
name, with `balls/tasks` saying "the project's board" outright. The knob no
longer writes `task-remote` at all — a store remote is the *project's* fact, and
asserting it as the literal word `origin` was writing a remote name into a
clone's binding as if it were a URL, shadowing the real one (bl-e47b).

**How the per-agent key meets bl's per-checkout config: it doesn't — it meets
bl's *space* (bl-e47b).** This was the open question and it has one forced
answer. `bl conf set task-branch` is scope-keyed to the **landing**, and a
landing belongs to a *clone*, which balls keys on `(state home, invocation
path)`. So that write binds a project, never an agent: two agents working one
project share one landing, and one agent working three directories has three.
Worse, a clone holds **one** store worktree, so two branches in one clone
thrash it — and yog's board reads that very checkout. The unit that can be
per-agent is therefore not the branch inside a clone but the **space** the
clone lives in:

> An agent's **space** is balls' state home and balls' config home, together.
> One var names it — **`YOG_MARKS`** — layered onto a spawn exactly as
> `YOG_WALL` is, and inherited by the whole descendant tree for free.

- **Absent = the world's space**: state stays `<world>/state` (every clone yog's
  board already reads) and config becomes `<world>/config`. yog's own `bl`
  verbs, the §5.1 #2 reads, and every agent *pointed at a project* run here, so
  the project's board is one store, instantly consistent, with no sync hop.
- **Present = the agent's own space**, `<wall>/marks`: its own clone bundle and
  its own balls config home, keyed by the §3.1 name that is already the ball
  claimant (§3.2) — one name, one wall, one claimant, one branch, so the
  claimant and the space it claims into can never disagree.

The branch **inside** a space is `tasks_branch` in balls' own §4 layer-2 config
(`<space>/balls/config.toml`) — the one layer that covers every clone in a
space, which is exactly what "the agent's branch" means, and a layer balls
itself ranks above the landing and names by layer on every read. yog writes that
one key and nothing of its own shape; `bl conf` stays the authority on what a
checkout resolves. Severability is unchanged and stronger: delete the space and
the policy is gone, with no yog config file to leave behind.

**The four clauses, and where each lands:**

| Clause | Mechanism |
|---|---|
| the default is per-agent | a workspace's own space is `<wall>/marks`, a private clone bundle; nothing written reads as balls' default branch |
| set at launch | §3.4's payload ladder already decides it — the **ball** rung was offered on a project's board and its `bl claim` landed there, so it carries no `YOG_MARKS` and its `bl` *is* the board's; every other rung is launched onto its own space. No new axis, no new flag |
| subagents inherit | `YOG_MARKS` rides the spawn, lernie hands its environment to every tool subprocess (lernie ARCH §3.3), and a bare `bl` is the world's shim re-entering yog. Zero mechanism of its own |
| an agent amends its own | `Action::SetMarks { workspace, branch }` / `Query::Marks { workspace }`, `/marks [<branch>]` (§8.5) — the same verb the launch spends, differing only in when it fires, which is why "at launch" and "amend" are one gesture and not two |

**The seam balls supplies, and why the config home had to nest anyway
(bl-e47b).** balls' library does no env reads — the host builds an `Edge` with
balls' two home directories — so yog folds the space in at `multiplex::bl`, the
one place it already folds `YOG_NAME` into balls' default actor. Nesting the
config home was overdue on its own: §16.2 nested balls' *state* and left
`XDG_CONFIG_HOME` ambient, so balls inside yog's world read the operator's
`~/.config/balls` for both `config.toml` (the layer that decides
`tasks_branch`) **and** `default-config/` (the template `bl prime` founds a
landing from). On this box that template was a stale copy naming the retired
`tracker` plugin, so every landing yog founded pruned its whole schedule: no
`bl-tracker` at any phase and no `show` hook — yog's stores never fetched,
never pushed, and `bl show` never printed a worktree. Proven by control: the
same binary against an empty config home wires `bl-tracker` at eight phases and
pushes `balls/tasks` to the project's origin.

**Nesting the config home repairs new clones only, so the `bl` arm converges the
old ones (bl-7e54).** A landing's `config/plugins.toml` is seeded ONCE — `prime`
seeds only when founding, and `rebind` never rewrites a committed schedule — and
balls' rename convergence cannot reach this damage either, because it rewrites a
*retired* name to its current spelling and here there is no old name left, just
an absent one. So every landing founded before the fix stays silently local
forever. The repair is **yog's**, and the reframe is what makes it so: balls
draws its convergence boundary at the landing because *"an old name in the XDG
layer is the user's file"* — but inside yog's world there is no user file, since
yog supplies balls' state home, its config home and its `exe_dir`. A landing
under the world is therefore yog's own generated artifact, exactly like the
`world/tools/` shims (§5.2), and it converges the same way: **on the way into
every `bl` verb**, at the same seam that already converges the shims, skipped for
a discovery probe (§8.5, bl-52ed). The gate is one `rev-parse` and one parse of
the schedule — does it name every balls plugin the world provides — so a
converged landing costs nothing and no verb waits on a repair. **yog never
restates balls' schedule**: the repair re-runs `balls::seed::seed_landing`, so
the phase names, the plugin names and their order stay balls' single source of
truth (the rejected alternative was yog re-implementing that default through `bl
conf`). Only `plugins.toml` is re-derived — `balls.toml` is carried across the
re-seed, so an operator's `bl conf` scalars survive a repair. The damage on this
box was wider than a pruned plugin: the stale template carried an OLDER balls'
phase vocabulary (`drop.post`, `claim.pre`, `unclaim.pre`) and no `show` hook at
all, which is why the whole schedule is re-derived rather than patched.
**Unaffected residual, and still the better home: an upstream balls ask** — `bl
prime` converging a schedule that is missing first-party entries the current seed
wires would repair every box, not just yog's.

### 16.4 Binaries are agent tools, not yog's payload

**yog ships and installs no tool binaries in the end state.** Binaries matter
only as the plane on which lernie's worker agents drive tools by bash — an
essential surface — and there they split cleanly:

- **Ambient-world work** uses the user's own installed CLIs. It is orthogonal
  to yog — not yog's concern, not yog's to pin.
- **Embedded-world work** — an agent operating on yog's nested state, mostly
  balls (e.g. closing the ball yog claimed in the nested state root) — needs a
  tool that (a) resolves the **nested** clone/worktree paths and (b) shares
  yog's **exact** balls implementation and state view. An ambient `bl` fails
  (a): it reads a *different* `$XDG_STATE_HOME` and computes the wrong
  clone/worktree paths. No host binary guarantees (b): none is built from yog's
  pinned crate.

The correctness argument, not mere convenience, drives the mechanism, and
**both (a) and (b) are met structurally** in the landed end state (§16.7 W8–W13):
yog seeds `<yog-data-root>/world/tools/{bl,lernie,bz}` — re-exec shims of the yog
binary (the `--editor-apply` multi-call pattern) speaking each tool's argv
contract and dispatching to the **embedded crate against the nested roots** — and
puts that directory at the head of the world's `PATH` (§16.2), so the `bl` an
agent types *is* yog. Beside them sit `bl-delivery`/`bl-tracker` shims
(bl-2930): not agent tools but balls' sibling plugin binaries, bound by the
embedded `prime` so the plugin chain re-enters yog too. No version and no path
is left to drift.

**And `yog` itself is on the roster (bl-3ff4).** The world seeded every
substrate tool an agent might type *except the one that drives yog*, so §8.5's
promise — every operator gesture drivable headlessly — was unreachable from
inside the world, and VISION §4.9's floor grant for a monitor responder
(`flag`, i.e. a boundary action) had no binary to be granted. In a clean room
there is no host `yog` to fall through to; where one exists the fallthrough is
*worse* than the absence, silently resolving the operator's **installed** yog
rather than the build under drive — bl-d1af's defect class exactly, one layer
out. Its shim is the odd one twice over, and both oddities are load-bearing:
it carries **no verb word** (`exec <yog> "$@"`, never `exec <yog> yog "$@"` —
yog is the argv surface, not a namespace of it), which is why the resolution
switch is an enum over `Namespace(word)` / `SelfBare` rather than a second
boolean beside the first; and yog does not *spawn* it, so it is on the roster
only because the roster is what the world seeds.

**Ruling — a window is the operator's own act, so an agent seat is refused
one.** That shim passes argv through verbatim like every other, and a bare
`yog` with no gesture and no subcommand is the GUI: unguarded, the shim that
lets an agent *ask* yog something also lets it paint on the operator's desktop.
The guard spends an **existing explicit signal** rather than a new flag —
`YOG_NAME`, already stamped on every workspace-scoped spawn and already riding
the whole chain (driver → tool subprocess → the agent's bash → the shim, §3.3),
whose presence *is* "this seat is inside an agent". Such a seat asking for a
window gets a refusal naming what it can drive instead (the gestures,
`headless`, the three namespaces) and exit `64`, the usage class the wall-less
`bz` already refuses with. It is keyed on the **environment, not the shim**, so
it holds however yog was reached — including an agent that finds the operator's
installed yog on the ambient `PATH`, which is the very drift the roster entry
exists to end. The operator is never caught by it: `yog env` prints
`LERNIE_HOME`, `XDG_STATE_HOME` and `PATH` and nothing else (§8.4), so a human
who ran `eval "$(yog env)"` carries no `YOG_NAME`; and every yog-spawned child
names a namespace verb, so none reaches the window seat at all.

**Version coherence is structural, not gated.** The phase-1 capability gate
(per-verb `--help` probes, a toolchain pane, dispatch-layer refusals) could
only *detect* skew, and only some of it — a `lernie` linking a different
brazen than the `bz` beside it was capable on every probe and fatal on every
dispatch. The end state **dissolves** the class instead: one `Cargo.lock`
names one balls, one brazen and one lernie, and every process image in every
chain is the yog binary those pins built — a skewed pair is not guarded
against, it is unrepresentable. The lockfile is the version (§16.5); a gate
every tool passes by construction is dead code, so the gate is gone (§16.6).
After-the-fact rendering stays for a different reason — the §7.3 no-response
wound and the §8.1 driver-stderr sink surface *any* dead driver, embedded or
ambient, and neither was ever gate machinery.

**The discovery mechanism is bash, not lernie's tool slot (W9).** lernie
discovers `lernie-tool-<name>` under its own data root with a JSON-on-stdin
contract, a schema file, a skill, and a `providers.yaml` grant — a surface
yog could reach only by aping lernie's config authoring (the §14 rejection),
and one that speaks nothing like bl's argv. Agents drive tools by bash, so
the shim is named plainly (`bl`), placed on the agent's `PATH`, and invoked
exactly as a host `bl` would be — which is what lets the goal preamble carry
no tool instruction at all (§3.3).

**Identity rides the env, not the argv.** "Stamping `--as $YOG_NAME`" is
implemented where balls reads the default — the multiplex arm's
`Edge::default_actor` (`$YOG_NAME` → `$USER` → `"unknown"`) — so it is one rule
over every verb, an explicit `--as` still wins, and verbs with no `--as` need no
exception. The shim itself parses nothing; it forwards argv verbatim.

**The full verb surface runs (bl-2930).** balls binds its sibling plugin
binaries (`bl-delivery`/`bl-tracker`) from `Edge::exe_dir`, and upstream
promoted both boundaries as lib entrypoints (`delivery_bin::run`/
`tracker::run`, U-balls-3) — so the multiplex carries `bl-delivery`/
`bl-tracker` arms, the world tools roster carries their shims beside the
three agent tools, and the `bl` arm hands balls `world/tools/bl` as its
executable: `Edge::exe_dir` is the tools dir, the seed's sibling rule finds
the plugin shims there, and a `prime` binds a plugin chain that IS yog.
**Nothing is refused**: no verb of the embedded `bl` requires a binary yog
cannot be (the clean room has no host `bl` to fall back on, so a refusal
would strand the ball rung entirely).

The human counterpart to these agent tools is `yog env` / `yog exec` (§8.4).

### 16.5 Crate adoption, phased by upstream readiness

All three substrates are embedded as **exact-pinned** crates — the pin *is*
the version mechanism (brazen's own pin-exact README posture, generalized).
**This end state is mandated, not aspirational** — installing yog ships the
batteries; no runtime dependence on any system-installed `lernie`/`bl`/`bz`
survives, and the machine's local checkouts/installed binaries may differ
freely from yog's linked versions. (The forcing incident: a machine-wide
brazen upgrade against lernie's exact pin killed every prompt silently — the
skew class the pin-in-lockfile abolishes.) The upstream seams each embedding
stands on: balls' promoted typed read surface (`reads::Catalog`,
`reads::Entry`; upstream bl-9901) and the plugin-binary entrypoints
(`delivery_bin::run`/`tracker::run`; U-balls-3) — mutations stay behind the
verb surface, the worktree-open → seal(ff-only CAS) → unseal protocol and the
plugin chain never bypassed by a raw file write; brazen's feature-gated
`native-host` exposure of `brazen::native` (the feature is the purity
boundary its upstream test pins); lernie's `lernie::cmd` API (CLI-parity
CI-enforced) with the injected `Fx::driver_target`/`Fx::adapter_target`
re-entry seams — yog re-execs **itself** as driver and adapter.

**Process semantics are non-negotiable regardless of linking:** drivers are
processes holding flocks, and plugin dispatch stays subprocess. Linking changes
what code yog *calls*, never the concurrency model — a linked lernie still runs
drivers as flock-holding processes, and yog re-execing itself as the driver is
exactly that process, not an in-process task.

### 16.6 Phase 1 — binaries (retired)

Phase 1 shelled to host binaries and is complete; its task record (W1–W7) is
retired to git history (bl-43cd). What it built and kept is stated above: the
world env (§16.2), seeding via `lernie prime` (§16.2, §14), the store-branch
knob (§16.3), the escape hatches (§8.4). Its capability gate and toolchain
pane (W5) were deleted outright once the embedded crates made every verdict
but `Ok` unreachable — a gate every tool passes by construction is dead code
(§16.4); the skew incidents that shaped W5 are the argument for §16.4's
structural answer, and the Login pane is the surviving half of its pane
(§8.3, §11).

### 16.7 Phase 2 — batteries included (landed)

**The one mechanism: self-multiplex.** Every spawn that today targets a host
binary instead targets **yog's own executable** with a verb-namespace argv —
`yog lernie <argv…>`, `yog bl <argv…>`, `yog bz <argv…>` — dispatched by
`main.rs` to the embedded crate exactly as each upstream's own thin bin does
(the `--editor-apply` / `yog env` / `yog exec` multi-call pattern, §8.4,
generalized). The principle from §16.5 holds unchanged: **linking changes what
code yog calls, never the concurrency model.** Every seam that is a process
for a *reason* stays a process — lernie's detached drivers and execve
lease-baton hops, balls' per-op change-worktree + ff-only-seal CAS, the
GUI-vs-stdout boundary on mutating bl verbs — only the binary on the other
side becomes yog itself. Seams that are *data-shaped* go in-process: balls
store reads (typed `Catalog` load replaces `bl list/show --json`
spawn-and-parse), brazen config projection. Skew death is structural: cargo
resolves ONE brazen in yog's graph (yog's pin and lernie's pin must agree or
the build fails — the lockfile is the parity check), the multiplexed verb
implementations ARE their versions, and lernie's runtime `bz --version` guard
never fires because the adapter target is the same process image that linked
the types. Three distinct skew incidents bit in the 24h before this ruling
(binary-vs-pin, config-schema-vs-binary, local-build-vs-registry-build);
all three are unrepresentable in the end state.

**Dependency shape (bl-89a4):** exact-pinned crates.io releases (`=x.y.z`)
for all three substrates — **the pin authority is `Cargo.toml`/`Cargo.lock`,
never this doc.** yog's graph has **zero git dependencies**; `deny.toml`
carries no `allow-git` list and `make publish` works. A pin is
lockfile-fixed and independent of local checkouts (a `path` dependency is
never lawful: it couples yog's build to the machine's checkout state, the
exact disease). An exact `git`+`rev` interim while an upstream publish is in
flight is lawful but re-incurs two recorded costs — crates.io refuses a
package with a git dependency, and a rev is not a stable name (bl-4537) —
AGENTS.md rule 6 carries that rule and its named exit.
**Residual host deps, documented and deliberate:** `git` (both yog and lernie
shell to it; a battery not worth including) and the platform probes
(`lsof`/`/proc`). `$EDITOR` never escapes: yog already re-enters itself
(§9.3). The `Binary` env overrides (`LERNIE_BINARY`/`BL_BINARY`/`BZ_BINARY`)
survive as test seams and escape hatches; default resolution becomes
`current_exe()` + namespace, not PATH.

**Landed shape (the W8–W14 waves, retired to git history at bl-43cd).** All
three substrates are embedded, every wave has landed, and the clean-room
proof stands: `scripts/drive/cleanroom.sh` — only yog and `git` on `PATH` —
drives S0/S1 green to a live wire with every substrate process image a
`yog <ns> …` namespace of yog itself (recorded in STORIES as the standing
real-substrate done-bar). The rulings the waves produced live where they
bind: foundedness-not-exit-code and the in-process catalog read at §5.1 #2,
the shim roster and the `Edge::exe_dir` plugin seam at §16.4, the `--as`
default at §3.3, the logical-vs-physical ops argv at §8.2. The wave labels
(W1–W14) survive in citations as historical labels git history resolves.
Two facts live only here:

- **lernie's `Fx` re-entry targets are spelled as the world's shims**
  (`world/tools/{lernie,bz}`), never the bare yog executable: both targets
  are single paths lernie spawns verbatim, so the bare exe would drop the
  namespace word and re-enter the GUI. The arms converge the shim roster on
  the way into every verb — one read, no write in the steady state — so the
  first invocation has valid re-entry targets with no ordering dependence on
  a start.
- **One read is still a subprocess: the closed listing** (`yog bl list -s
  closed --json`, §5.1 #4) — balls' dead-set history walk is not on the
  promoted read surface. **Residual upstream ask, U-balls-2: promote the
  dead-set walk** (`reads::history`); landing it deletes yog's last spawned
  read and its last JSON parse together.
