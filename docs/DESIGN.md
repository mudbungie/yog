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

yog is the **server** of the four-component split (REMOTE §12): the standalone
holder of a world of litany loops organized around **named workspaces**, with
balls as the work items workspaces pick up. It knows every project on the
machine, every ball in each project, every litany workspace, every agent and
subagent in each workspace, and every byte of diagnostic data litany produces;
it can start work from nothing, a path, or a ball (§3.4), drive agents, and
edit brazen and litany configuration, all through the substrates' own write
paths (`bl`, `litany`, `bz`-validated file writes).

**It has no face and it executes nothing** (bl-7942, bl-37fd). Every read and
every act crosses one surface — the §8.5 control boundary, over REMOTE §9.5's
mTLS wire or a deposit in the world's gestures inbox — and a **seat** on the
far side of it decides what a human sees. The seat is its own crate and its own
repository (`lernie`, REMOTE §12); an android client is another; `yog gesture`
from an agent's own bash is a third. Execution is a **thrall**'s, enrolled by an
explicit operator act, and a yog with none is valid and is the default: a tool
call with no thrall to route to refuses in band, which is the posture working.

**yog is an application that owns a nested world, not a layer draped over the
user's.** The naïve reading — yog as a thin viewer over the ambient
`bl`/`litany`/`bz` state a human already runs — inverts: yog composes its own
nested environment (§16.2) for itself and every child it spawns, so the
substrate state yog drives is *yog's*, under yog's data root. Playing on top of
the user's *direct* tool usage stays possible — an agent's task branch can be
pointed at the project's shared store branch at launch (§16.3) — so
**compatibility with an ambient workflow is the user's decision, not a
structural given.** The world is the subject of §16.

**Governing invariant (I0): two yog instances running side-by-side faithfully
replicate the same data, with nothing RAM-only except unsubmitted input text.**
yog is a pure derivation of disk plus a small set of gesture dispatchers. The
only durable state yog itself owns is one UI-state document (`ui.json`), one
action-outcome log (`ops.jsonl`) and one clock-settings document
(`cadence.yaml`, §7.2, bl-3381), plus a per-client pane document beside them
(REMOTE §7). Everything else already has an authoritative home in balls,
litany, brazen, or git, and yog derives it.

Crate `yog`, bin `yog`, repo `github.com/mudbungie/yog`, published on
crates.io. **Bare `yog` boots the engine and parks**; there is no verb to
select it, because there is one face (REMOTE §8). Runtime dependencies: clap,
libc, notify, rustls, serde_json, thiserror — plus the three embedded
substrates `balls`, `brazen`, `litany` as **exact-pinned** crates.io crates,
the pin being the version mechanism (§16.5; the pin authority is `Cargo.toml`).

**Reading this document after the severance.** §11 is retired and carries the
reading rule for every sentence elsewhere that still describes a window: yog
states a fact, a seat shows it. Where prose here says the frame paints
something, the question to ask is which §8.5 query or reply carries it — and if
none does, that is a defect to file rather than a face to put back.

**The ruling's own home is REMOTE §12** — the four components, the front-door
and ship-inert invariants, and the migration order — not here.

---

## 1. Taxonomy

litany bans the term "session" (TAXONOMY §3: "underdefined, per-framework
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
| **workspace** | A litany workspace: a directory containing `repo.git`; yog-started workspaces live at `$XDG_DATA_HOME/yog/workspaces/<name>/` (§3.1) | litany (contents), yog (location) |
| **name** | Two names at two altitudes: a **workspace name** is operator-chosen at creation (validated shape, §3.1) — the sphere wall's label, the dir leaf, **and** the claimant stamped on every ball claim the workspace makes (§3.1, §3.2); a **conversation name** is minted (two words in PascalCase from an embedded wordlist — litany bl-79a2, consumed by bl-0219) — yog draws it at preview and at fire and passes it via `--name` (§3.3, bl-08f2), **from `litany::mint`'s embedded list** since bl-cd38 consumed the bl-aca4 ruling (§3.3's "state of the move") — the context window's own identity, durably litany's `name` blob beside `goal.md` | litany (the stored fact, and the mint); yog (the seed, the fire-time draw + preview) |
| **binding** | The derived association between a ball and a workspace: ball claimant = workspace name (§3.2). Balls-owned metadata, explicitly late-mutable via `bl claim`/`bl unclaim` — never a yog-stored fact | balls (claimant field); yog joins |
| **agent** | `agents/<id>` branch; the id is a chain of `<ts>-<short>` descent segments (ARCH §2.3 — two hyphen-free tokens each), which is where the hierarchy lives | litany |
| **conversation** | A root agent plus its descent subtree — the §11 organizing unit. Its **identifier** is the root agent id; its **name** is a minted wordlist label litany stores on the root's branch (§3.3, reversing bl-68d9's no-name rule) — minted for agent self-identity, rendered as the row title | litany (the agents, the stored name, the mint); yog (the seed and the derived view) |
| **exchange** | litany's presentational span (ARCH §2.4: root agent's history between a user message and the terminal response) | litany |
| **attention** | A derived per-agent predicate (§6): unacked notify / stop / budget / conflict, or pending mail with no driver | yog (pure function) |
| **seen / pin / collapse** | The operator's durable, converging UI facts (§4.1) | yog (`ui.json`) |
| **draft** | Text typed but not sent, **for one target** — a new conversation in a workspace, or a message to an agent (§11) | RAM (the requirement's carve-out) |
| **world** | The nested substrate environment yog composes under its data root — the `LITANY_HOME` / `XDG_STATE_HOME` / `PATH` override fold that redirects `litany` and `bl` state into yog-owned roots and fronts the search path with yog's own `bl` shim (§16.2, §16.7 W9; brazen state resolves per workspace since the blast-radius ruling, §16.2) | yog (composed) |

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
  state via `bl` verbs; workspace state via `litany` verbs only (never a direct
  write inside a workspace — ARCH §3.5; removing a *whole* workspace dir is a
  write to yog's own names root, not inside one — the §3.6 delete verb); brazen
  config as a
  `bz --dump-config`-validated atomic file replace; litany global config as an
  atomic file replace (litany declares these hand-edited).
- **I3 — All yog file writes are temp-in-destination-directory + `rename`.**
  Never in-place truncation, never a temp on another filesystem (EXDEV). Temp
  names are dotfiles (`.<name>.yog-tmp-<pid>`) so no substrate reads them;
  leftovers older than 24 h are swept at startup. The name and the sweep are
  **one home** — `src/scratch.rs` — because a sweep that spelled the temp
  differently from the writer would delete nothing, or delete something else
  (bl-e47c: the write half had three sites and the sweep half was never
  written). `ops.jsonl` is the one
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
  recovery rule, arch §13). litany repo state: litany's writer/driver
  discipline (ARCH §2.11); yog is a pure reader.
- **I6 — The RAM whitelist is closed.** Only the items in §5.3 may exist
  without a disk home. Every addition requires amending this document.
- **I7 — yog never mutates any substrate except on an explicit user action.**
  No auto-prime, no auto-scan, no auto-push, no background repair. Two
  instances can never race a spontaneous mutation because neither has any.
  *Composing* the world env (§16.2) is pure — no mutation, always safe;
  *materializing* the world (creating the subtree, writing the `world/tools/bl`
  agent-tool shim, seeding `LITANY_HOME` via
  litany's own bootstrap verb, priming the nested balls clone, binding an
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
  projects sort by path, workspaces by path (the tab bar's named strip
  by name), agents by descent order. Two
  instances render identical order without sharing any ordering state.

---

## 3. The organizing unit: the named workspace

Both roots below — `yog_data_root`, `litany_data_root` — resolve through the
composed world env (§16.2), so the paths name locations *inside yog's nested
world*. The balls state root enters only through `bl` verbs (claims and
listings), never through path arithmetic: **the workspace tree encodes no
project paths and no ball ids** (§3.2 supersedes the original path-convention
binding).

### 3.1 Names: an operator-chosen sphere label, a flat root

**A workspace is long-lived and low-volume** — a sphere of work (personal,
corporate, a client) whose wall is litany's isolation boundary; conversations
are root agents *inside* it (litany §7.3), and balls flow through it (§3.2).
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
  `<litany-data-root>/workspaces/` are **foreign** (litany's auto-id
  territory — rendered, unnamed, never created by yog);
  `<litany-data-root>/replays/*` render as read-only replay workspaces. Three
  roots, one shape, classification by path alone.
- **Severability:** the root is yog's own territory, *not* litany's
  machine-populated `workspaces/` tree. Deleting `$XDG_DATA_HOME/yog` erases
  yog's entire workspace footprint and leaves litany and balls untouched —
  the same choice balls made in placing delivery worktrees under its own
  plugin territory (arch §1). **With the nested world (§16.2) this widens:** the
  nested `LITANY_HOME`, the nested balls state root, and yog's own artifacts
  all live under `$XDG_DATA_HOME/yog`, so one `rm` erases the whole world and
  leaves the *ambient* litany/balls/brazen untouched.
  Rejected: `<litany-data-root>/workspaces/yog/…`
  — squats in litany's retention-governed, auto-id-populated territory.
- litany accepts any path for `litany new [path]` (CLI §1); yog `mkdir -p`s
  the root (outside any workspace — the ban is on writing *inside* one) and
  passes `<root>/<name>`.

**Renaming, and the pre-reversal leaves (migration).** There is no rename
verb: a sphere with the wrong name is **replaced, not renamed** — raise the
chosen-name workspace (New workspace), re-home its bound balls with a
per-ball unclaim/claim through the hatches (§8.4, the operation spelled by
hand and lawful while nothing runs there, optionally with a hand `mv` of the
dir), and let the old workspace's conversations age out under litany's
30-day retention. A ball whose claimant still names the old leaf renders
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
  <name>`; release = `bl unclaim <id>`. Both are first-class UI verbs (§8.2),
  legal whenever balls allows them, at any point in a workspace's life; a
  re-home is the pair, spelled through the §8.4 hatches (bl-6c28 retired the
  one-gesture Move).
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
delivery worktree (`<worktree>/.litany/`) — captured by close's squash into
the project repo, disqualifying.

Parallel attempts, reprompts, and follow-ups remain *multiple root agents in
the one workspace* — exactly litany §7.3's concurrent-exchange model ("new
question → `litany prompt` forks new root"); the workspace stays litany's
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

Litany has no target-repo concept, and its tools do **not** inherit the
driver's cwd: the executor runs every tool subprocess in the agent's working
directory — the agent worktree by default, movable only by the agent's own `cd`
built-in, which writes the id-scoped mark (`refs/litany/cwd/<agent-id>`) the
executor reads back at every spawn. **The work target is a typed parameter
(bl-6654, landed on lernie `=0.0.8`):** the fire passes the rung's binding as
`litany prompt --cwd <path>` (upstream bl-d0b4), which seeds that mark at
creation — so *every* tool step of every later turn runs at the target, not
just the initial process. One channel, per rung: **ball** binds the
claim-derived `work/<id>` worktree, **path** binds the directory box's value,
**bare** binds nothing and lets litany's own default (the agent worktree)
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
- **The per-target `current_dir` is gone.** Yog set the initial `litany prompt`
  process's directory per rung. It reached that one process and no tool step,
  which made it look like a binding while binding nothing; DESIGN recorded it
  as misleading redundancy. `Prepared` carries no cwd field at all now — the
  detached driver simply stands in the workspace it drives.

**The binding is also what the project's own standards are discovered from**
(§3.7, bl-aa8b): the fire walks from the binding's authority root down to the
binding, and freezes each instruction file it finds as a `--pin` on the same
argv. One typed target, two consequences — where the work happens and which
rules govern it — and neither is prose.

Edits at the target still land in balls' external project worktree, outside
litany's agent branch, commit-per-side-effect history, child inheritance,
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
`--name` — litany commits the name beside `goal.md` and **states the stored
fact in its assembled context** (litany bl-d55f, released 0.0.4:
`compose_system` derives `Your name is <name>.` into the system slot from the
`name` blob, never a second copy). The retired `You are <name>.` stamp — one
line, no instruction, the bl-df65 interim channel — survives only as a
**legacy parse** (below); no live path composes it.

**The workspace never enters the prompt (bl-df65).** The workspace name is
*operative* in exactly one channel, the harness's: `YOG_NAME` rides every
workspace-scoped spawn and every tool subprocess inherits it (§8), the shim's
`--as` default stamps it on every claim (§3.2). The spawn's `current_dir`
selects the initial prompt process's directory (§3.4), but pinned litany
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

The mint — **litany's `litany::mint`** since bl-aca4, consumed by bl-cd38 (see
"state of the move"): **two words in PascalCase** (`PeachHollow`) drawn from a
wordlist embedded in that crate, a pure function over an injected RNG and an
occupied set. On collision with the occupied set the candidate is discarded and
the next drawn — the retry is bounded by the index space itself (one wraparound
scan), erroring loudly on exhaustion rather than looping. Yog calls that one
function at preview and at fire, and holds no wordlist of its own.

The shape has moved twice and yog inherited both with no code change, because
`litany::mint::mint` is the only draw it makes: bl-d12f retired an early
two-word compound for a single lowercase word, and litany's bl-79a2 — consumed
here by bl-0219 with the 0.0.11 pin — went to the ordered PascalCase pair,
because one lowercase word read as a word and not as a name. One RNG draw
apiece, the same pure scan, the same exhaustion bound; the space widened from
541 words to 541 × 540 = 292,140 ordered distinct pairs. It was never sized
against a birthday bound in either shape — the occupied set it actually races
is one workspace's living agents, tens of them, recycled by retention. **The
one thing that changed for yog is width on the glass**: a §11 row lays the
title and the weak subtitle on one truncating line, so a two-word title spends
more of that line (`shell/acceptance/echo.rs`).

- **The occupied set is per-workspace, and already derived (bl-08f2):** the
  names the target workspace's living agents wear, read off the same name
  fact the display ladder reads — the litany-stored `name` blob, with the
  legacy goal-stamp parse as fallback while pre-0.0.4 roots live. Children
  count too, and must: litany refuses a name any living agent already wears,
  so a mint blind to a named child would fail at fire. No cross-workspace
  enumeration: workspaces are isolation walls — two agents in different
  spheres never meet, so global uniqueness would buy nothing.
- **The pool does not burn:** occupancy is one workspace's live roots
  (dozens at most), litany's 30-day retention recycles it, and a recycled
  name is lawful — the name discriminates the living; the root id stays the
  identifier.
- **Nothing is stored that can be computed — and yog stores nothing:** the
  name's one durable home is litany's `name` blob on the agent's own branch
  (below; formerly the goal's first line, the bl-df65 interim). No yog
  registry, no `ui.json` field, no second home — reading a name is
  `git show agents/<id>:name`, a query.

**Ruled (bl-50f3): the name's durable home moves to litany.** Prose-only
naming failed live: agents message peers through litany's `message` tool,
which resolves only `agents/*` ref ids, while the operator and every UI
surface speak names — an agent told to message `shudder-storeroom` could not
resolve it, twice. The name is agent identity, agents live in litany, so the
fact becomes litany's: `litany prompt`/`dispatch` grow an optional `--name`
(the name committed under the agent beside `goal.md`, so `agents/*` refs stay
the only registry and retention recycles names with zero cleanup; uniqueness
among the living is refused at creation), and `message` resolves exact id
first, else unique living name (litany bl-c8ed; released as 0.0.4, litany
bl-4c15). yog stays the minter — the mint above is unchanged, still previewed
before spawn (I7; the name exists before the agent id does, which is also why
deriving the name *from* the id was rejected as impossible — and name-as-id
was rejected because ids must never collide while names lawfully recycle) —
and becomes a pass-through and reader — **wired by bl-08f2**: the fire
spawns `litany prompt --name <minted> <workspace> <goal>` (the goal stays
last in the argv, which is also what keeps the ops-log clip and the
detached-sink join key positional-from-the-tail), on a lost-race re-mint the
re-derived name is what passes, and the ladder's rung one reads the litany
fact back (`Agent::name` at enumerate time, `git show agents/<id>:name`).
The `You are <name>.` stamp is no longer a fact home — and by the later
bl-6920 ruling (**landed**) it is retired as a channel too: the first user message reaches the model unmutated; self-identity
belongs in the harness-assembled context, which is litany's job (litany
bl-d55f, verified in the pinned 0.0.4: `compose_system` states the stored
name fact in the system slot). The stamp's compose is deleted. Its parse
survives as the one legacy rung (`Agent::name_fact`'s fallback), kept
**solely** because pre-0.0.4 roots carry no `name` blob — the stamp's first
line is the only record of their name — until 30-day retention ages them
out; then the rung is deleted. New roots never match it: nothing composes
the shape anymore. A workspace registry file yog writes and litany reads
was rejected outright: two representations of the living-agent fact that
must drift, in a format neither crate owns.

**Ruled (bl-aca4): name is an exposed dispatch parameter; omission
auto-mints — and the mint's one home moves to litany.** A name belongs on the
dispatch command itself: it already tells the depth, it keeps subagents'
identities and tasks clear, and it simplifies the whole naming question —
an omitted name is generated dynamically, a supplied one is honoured, and
either way the parameter is exposed. This **amends bl-50f3's "yog stays
the minter"** — the storage half of that ruling (the name's one durable home
is litany's `name` blob) stands untouched; what moves is the mechanism's
address. bl-50f3 could leave the mint in yog because only yog's fire minted;
the moment *every* creation path must mint on omission — the `dispatch`
tool, `litany dispatch`, `litany prompt`, none of which pass through yog — a
yog-resident mint would leave litany either calling up the stack (inverting
the dependency) or growing a second list. So the wordlist + draw + bounded
retry move **into litany beside the uniqueness check they race**
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
  validation) and calls litany's through the crate it already links (the
  §16.7 multiplex proves the linkage). **Landed at bl-cd38** — see
  "state of the move" below. Everything operator-facing is
  unchanged: the composer still previews the predicted name before anything
  spawns (I7), the seed lives exactly as long as the prediction it backs
  (bl-28ba — preview timing and seed lifetime are yog *policy* over
  litany's *mechanism*), and the fire still mints and passes `--name`,
  because the fire's return — the minted name — is the §3.4 focus-claim
  handle that must exist before the agent id does. Yog never omits the
  parameter; omission-minting is for the callers that have no preview to
  keep.
- **Preview parity:** preview and spawn draw from the same function. The
  occupied sets differ only by yog's legacy goal-stamp rung, which makes
  yog's a *superset* of litany's living-names scan — the safe direction
  (yog may avoid a word litany would allow, never predict one litany would
  refuse) — and the discrepancy dies with the legacy rung. The lost-race
  story is bl-50f3's, unchanged: the preview is a prediction, the fire's
  fresh mint is the truth, and `require_available` at creation stays the
  actual uniqueness gate — the residual race (two mints landing the same
  word between scan and commit) is refused there, loudly, exactly as a
  hand-typed collision is.
- **Rejected — litany fallback-mints with its own list, yog keeps its
  own:** two lists and two draws are two representations of one behavior
  and must drift; yog's preview could predict names litany's fallback would
  never produce, and every wordlist curation lands twice or diverges. The
  mechanism is one function or it is two facts.
- **Rejected — `name` required, every dispatcher always supplies:** directly
  against the ruling ("if a dispatch command omits it, we can provide it
  dynamically"); it multiplies minters — every model, script and human
  becomes one — and turns a forgotten name into a refused dispatch, a
  failure apiece where one default dissolves the class.
- **Rejected — litany mints always, yog stops passing `--name`:** the
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
call `litany::mint::mint(rng, occupied)` over the crate's `Rng` trait and its
`SplitMix64`; the wordlist stays behind the function, unexported, per litany
ARCHITECTURE §3.4. Yog's own seat in the seam is the **seed**: `shell::clock`'s
`entropy_seed` is still the one home for "now" and feeds
`SplitMix64::from_seed`, which is what makes the §3.3 preview and its fire
agree and what bl-28ba's per-prediction seed lifetime is expressed in — read
**once per session**, every later seed being one stream step off the one a fire
spent (bl-dd3d). The
corpus that lands is litany's clean-room 541-word list (litany bl-b59c), which
replaced an EFF-derived CC BY 4.0 list — the licence and the hostile-word
problem left the tree with yog's copy of it, rather than moving into litany.
The acceptance fixture pins `MINT_SEED` and the whole word sequence it mints
(`MINTED`, of which `MINTED_FIRST` is the opening preview), so a corpus change
in the crate fails loudly at `shell::acceptance::mint_seed` and names its
cause.

**Scope: every creation names; the ladder names whatever litany names.**
Since bl-aca4 no creation path yields a nameless agent — a dispatch child, a
hand-run `litany prompt`, and yog's fire all end named, supplied or minted.
The name fact rides any `agents/*` ref and the enumerate read
(`Agent::name`) makes no root/child distinction, so the ladder shows every
new agent by rung one with no yog special case (bl-08f2's read, unchanged —
which had already lifted the bl-df65 honest-scope limit: back then no fact
carried a child's name, so none was invented); the lower rungs keep naming
the pre-aca4 stock until retention ages it out.

**Display: the name is the title; the first payload line is the preview.**
One function derives what a conversation is called, as a ladder (bl-08f2):
the litany-stored name fact → the legacy goal-stamp parse → the first
payload line (the goal with the stamp stripped) → the root agent id. The two
name rungs fold in one place (`Agent::name_fact`), so retiring the legacy
rung is one deletion. The §11 row title, the center header, the descent-tree
member row (bl-df72 — that seat painted the raw id, the operator's
"incoherent timestamp"), the in-flight strip, and the composer's
`message <x>` target label (bl-2f30) all read the fold and fall through;
foreign, hand-typed, and post-bl-6920 unnamed roots land on the payload
line or the id. **No seat formats an agent id as a display name** — the id
is a fact whose display seats are the ladder's own floor and the hover (the
member row and the center header both keep it there); an acceptance scan
holds the rule **on values** (bl-45c7) — it reads the painted window and
asks of every token whether it is stamp-shaped, so a seat leaks under any
field name or none. **The floor
spells the terminal generation only** (bl-63a1): a litany child id embeds
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
`sender`. That is bl-63a1's own lesson repeating verbatim, so the fix was
both halves: `inboxview::header_line` stopped spelling the chain, and **the
scan stopped enumerating names at all**
(bl-45c7). A vocabulary is the wrong kind of strength — it decays on every
rename and nothing fails when it does, so the scan went on passing on the
subset it happened to know. It now asserts the rule over the *painted
window*: every id-shaped run in it must be the ladder's own floor spelling,
where id-shaped is `nav::convs::is_stamp`, the one grammar the floor itself
reads. A carrier nobody has named yet is caught by the same sentence,
because the sentence names no carrier. What that trades away is the seats a
fixture cannot reach; §11's fixture reaches the window, and a guard that
knows three of a fact's names is worth less than one that knows the fact.
The strip is
the retired compose's other inverse — parse and strip live in one module
(`start::identity`), where the retired shape has its one written record.
The legacy rung has **one** shape to recognize (bl-2706): `You are <x>.` A
legacy root whose `<x>` was a workspace name parses by the same rule — an
accepted, bounded, display-only misread until retention ages the goal out;
for every goal without the stamp — which is every new one — the strip is
the identity function, the general path. **A legacy-rung title says it is
display-only** (bl-8068): litany resolves a message target by exact id, else
unique *stored* name — never the goal-stamp prose — so a title with no `name`
blob behind it (`Agent::name_display_only`) is unaddressable. Every seat that
shows it says so: the §11 row title and the centre header hover
`theme::NAME_DISPLAY_ONLY`, which names the agent id as the address that
works, and **the boundary withholds the `name` key entirely** for such a row
(`display` still carries the ladder's answer) — the machine-facing surface
never hands a peer a name litany will refuse. Diagnosed from the field: an
operator read "marbling-lake" off a pre-fact row, told a peer to message it,
and litany refused with `no agent "marbling-lake" in this workspace`. The
headline invariant is what
makes rung two worth showing: **every prefill yog composes
leads with its headline** — the ball rung's first line is
`Ball <id>: <title>`, the path rung's is `Working directory: <dir>`
(reworded from a sentence that buried the path on line two), and the bare
rung is the operator's own ask, which leads with intent by nature. Two
conversations with the same preview stay distinguishable by name, state,
age, and the root id (the true identifier): a preview is a subtitle, not a
key — the same distinction balls draws between a task's title and its id.

**The payload's home is `goal.md`, and rung two reads it there** (bl-368d):
`agents/<id>/goal.md` is the file the dispatch writes and the same read that
yields the goal's two stamps, so one fact has one home. What litany *sent* the
model is a different document and never this one — an assembled context leads
with the §3.7 pinned-instruction frame (litany ARCH §2.5) and carries a deposit
inside its `---` envelope, so `steps/<id>/001/request.json` heads with
`<file path="instructions/…">` or with `---` and with nothing the operator
wrote. Rung two was derived from that record until the freeze made the
divergence total, and the operator read the frame's opening tag as the name of
every fresh conversation.

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
`litany prompt` fires; **and a seat with no composer says the same thing by
saying nothing** (bl-06a1). `/prompt`'s goal IS that whole payload at every
seat — yog prepends nothing to it, ever — so the prefill reaches the model only
because some seat sent it. A composer seat pre-loads its box and sends the
edited whole; a terminal, where each `yog gesture` is its own process, held the
prefill in the `/prepare` reply and had no way to fire it, so every
terminal-fired conversation went out with the operator's typed sentence alone:
a path rung with no `Working directory:` headline, and a **ball rung with no
ball** — no body, and not even the `Ball <id>:` header §3.2 calls the
conversation→ball join. So `/prompt`'s goal is **optional**, and an unsaid one
falls to `prepared.goal` whole (`boundary::line::args::goal_or_prefill`); a
bare start prepares no prefill, and there the unchanged *"the goal is
required"* refusal stands, which is this rule with empty inputs rather than an
arm of its own.

**It is a default and not a concatenation, and that was the ruling** (bl-06a1).
`/prompt` composing `prepared.goal` + a tail would read the help page's older
wording — *"this goal as its whole tail"* — but it cannot be done in yog alone:
a composer seat sends the edited whole, so composition would fire every
composer-typed prefill **twice**, and no test on the wire can tell one seat's
send from the other's without a new flag on `Action::Prompt`. A prefix test
against `prepared.goal` is worse than a flag, because a composer invites the
operator to edit the prefill and an edited one fails the test silently. The
composed shape therefore stays available, exactly where it was already
available: the operator joins the two texts and sends them as one goal, which
is what editing the composer's box already is. The help page states that in
place of the retired *tail* wording. **identity is the harness's fact, passed as `--name`
at fire time** (the same ownership line as `YOG_NAME`/W9: the harness carries
identity, the model reads it from litany's assembled context, the operator
never types it). The composer *previews* the predicted name greyed above the
box (`will be named <name>` — worded as the prediction it is, never as a goal
line) — the mint is a pure read (the target workspace's derived name facts +
RNG, drawn through `litany::mint` since bl-cd38 landed the bl-aca4 move), so the predicted
name renders before submit with
nothing spawned (I7 intact) — and on the rare lost race (another instance
took the name between preview and Enter) the mint re-derives and passes
the fresh name; the preview is a prediction, the fired mint is the truth.
**A seed
lives exactly as long as the prediction it backs** (bl-28ba): the RNG seed is
held across frames so preview and fire agree, and retired the instant a fire
lands, because that prediction has been spent. **Its successor comes off the
spent seed's own stream, not a second entropy read** (bl-dd3d): one
`SplitMix64` step, so entropy enters a session exactly once — where the first
seed is minted — and a *known* opening seed determines the whole run of names,
not merely its first. That is what makes the acceptance drive assertable: with
two clock reads, "the third name differs from the first" was a coin flip over
litany's 541-word pool (it flaked twice in one day, once failing an unrelated
close gate); with one, the sequence is a fact of the pinned seed and is written
down as words. Nothing operator-facing changes — a successor nobody can predict
without the seed is as unpredictable as a successor nobody can predict without
the clock. Held longer — one seed per
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

**And the boundary reads that mark back** (bl-1015). One channel for the
binding meant no channel at all for *reading* it: `/files` walks the agent
worktree, a bound conversation writes nothing there, and the reply was an
ordinary listing of goal and soul with nothing said. So a conversation that
built its whole deliverable in the bound directory and one that built nothing
answered alike, and the operator's only route to the work was to already know
the host path and look outside yog. `Query::Files`' answer now carries
**`working_dir`** — the mark's own value, present exactly when it is not the
worktree the listing walked, so its absence is the case where the listing IS
the working directory and a reader never compares the two. It is a
**statement, not a second listing**: yog does not walk the target, and this is
QUALITY H2 (*"a fact yog cannot derive is answered as absent and never as a
zero"*) applied to a listing rather than to a field. The mark rather than the
trail's `--cwd` row, because §3.3 above already ruled which of the two is the
channel — the trail's row is the §3.8 cohort's join and answers a different
question (*which attempt was this fire bound to*), and reading it here would
be the second spelling this section deleted. `/work-diff` is unchanged and
still honest by its own page: it is per-ball over the project repository's
work branches, so a path-rung conversation has no attempt for it to be about
(VISION §4.10 item 8).

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
| **bare** | — | (none) | (none — the empty composer) | (none — litany's default, the agent worktree) |
| **path** | a directory | (none) | target preamble, path verbatim | the directory |
| **ball** | a ball, picked or freshly created | `bl claim <id> --as <name>` | `Ball <id>: <title>` + body | the work worktree |

- **Creating a workspace is the rare, deliberate verb** (New workspace,
  §11): raising a sphere wall — a client, corporate vs. personal — not a
  per-conversation act, which is why it is also where the operator types the
  name (§3.1). The everyday gesture is the composer: a new prompt is
  a new root in the focused workspace (litany §7.3), and a ball pickup claims
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
  derivation reads back off every root (litany-stored, legacy goal stamp as
  fallback), unique per workspace by the
  mint's own occupied set — and **that name is a selection at once**
  (bl-2e8f): the §7.2 echo below mints a row keyed by it, the seat folds that
  row into the answered forest ahead of every reader, and ↓ could already land
  on it by hand — so the claim is spent through the ordinary `focus_agent` path
  (the one ↓ takes, acknowledging §6 identically) on the frame of the fire, and
  again on the first roster that carries the root, when the conversation swaps
  the name it was born under for the id it acquired. Until bl-2e8f only the
  second of those happened, which left the operator's own new conversation the
  one row in the §11 list nothing highlighted, behind the birth placeholder,
  for as long as the driver took to write a branch — *"you start a new chat,
  start typing, and the new chat isn't immediately selected."* The **timing was
  the defect and not merely the wait**: with the selection arriving whenever the
  driver got round to it, whether the operator's *next* Enter started a second
  conversation or messaged the first was a race. It is now always the second,
  and `new conversation` (§11's button, `n`) is how they say otherwise.
  - **And the second is HELD, because it has nowhere to go yet** (bl-56c6).
    The ruling above was made true of the *selection* and left false of the
    *address*: until the driver writes its branch the minted name resolves
    nowhere, so the second Enter posted `Message{agent: <minted name>}` and was
    refused *"unknown conversation"* for the whole window. It was the second,
    and the second bounced. So a send aimed at **this window's own unresolved
    mint** is taken by yog rather than posted — one more undelivered deposit on
    the §7.2 echo that already stands for the first one, painted in the same
    faded §11 queue — and posted, in the order it was said and addressed by the
    id the branch brought, at the instant the claim resolves. Nothing addresses
    a name that resolves nowhere, so the refusal has no site left to happen at.
    It is not a queue beside the echo and not a start-window mode: the hold is a
    field on the one pending value, the predicate is that value's own, and every
    other target is posted exactly as before.
  - **A start in flight refuses a second start** (bl-56c6). The §8.1 fire is two
    posted acts, and both facts a start spends — the §3.3 mint seed and the §3.4
    claim — are spent on the *second* one's receipt. Replacing the outstanding
    hold left the first `Prompt`'s aftermath unrun while its detached driver
    launched anyway, and chained a second `Prompt` **with the same unspent seed
    against the same occupied set**: one minted name, two roots, and every later
    address to it "ambiguous conversation" forever. The one-act-at-a-time rule
    that governs every other gesture (§9.8: *newest wins*) is therefore not the
    rule here, and the gate is the existing signal — an act outstanding — rather
    than a flag. Nothing is lost by refusing: the draft is untouched, and the
    ruling above is what the very next frame does with it.
  A claim whose root never appears is
  inert — the conversation it selected stays selected and stays faded, which is
  what a start whose driver died honestly is — and no claim survives being
  spent, so the operator's own later selection stands. **A claim is spent where
  it was made** (bl-56c6): it moves the selection only while the selection is
  still the name the claim put there, because a start can take a minute to write
  its branch and an operator who read something else in that minute must not be
  yanked back by their own conversation arriving. The rule is *a start focuses
  what it started*, not *a start outranks whatever you did next* — the same
  reason a follow-up's echo claims no focus at all. The claim is per-instance
  RAM like the focus it becomes (§13.1); nothing about it is written down, the
  §6 acknowledgement included: that records the evidence an agent *has*, and a
  conversation with no branch has none.
  **A pending conversation's state is what nothing observed** (bl-56c6): the
  synthetic row a start mints carries no lock and no completed step, flagged
  uncertain, which is exactly what `git_tree::state::classify` answers for an
  agent it cannot probe — there is no inbox directory to hold a lock and no step
  to frame. It read `live` until that ball, claiming a driver yog had never
  looked at and offering §8.2's `Stop` on a conversation no signal could reach.
  For the same reason **nothing is asked about it**: every §11 inspector
  question refuses at the address, and painting those refusals told the operator
  their own new conversation was unknown for the whole of a healthy window. The
  empty view is what the world honestly holds; their text is in the queue.
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
- Re-opening is the same path as opening: an existing dir skips `litany new`,
  dissolving any "resume" special case. During work, everything is litany's
  normal surface; yog renders and dispatches verbs (§8.2), including
  late assignment of further balls (§3.2).
- **`bl close`:** the ball file is deleted; the closed listing's claimant
  still names the workspace, so *that query is the "delivered" status* —
  delivered balls group under their claimant workspace on demand
  (`bl list -s closed --json`; obituary via `bl show <id> --json`;
  delivered-in commit via `git log --grep "[<id>]"`). yog deletes nothing *at
  close*: workspace retention is litany's (§9.2, 30-day default); `litany
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
learns a price (its stated boundary); litany commits that count into the step
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
- **A rate table describes disjoint slices, so the prompt is partitioned before
  it is priced** (bl-6621). The four brazen counters are not four disjoint
  slices — on an OpenAI- or Google-shaped row the cached tokens sit *inside* the
  prompt counter — so charging each counter at its own rate billed the cached
  slice at the input rate **and** the cache-read rate, and the §5.1 #16 token
  figure summed it twice beside that. Both fold once now: the prompt is
  `max(input, cache_read + cache_write)`, and the money is that prompt cut into
  three non-overlapping parts — the cached read, the cache write, and the
  uncached remainder `max(input - (cache_read + cache_write), 0)` — each at its
  own rate, plus output. **The tokens priced are exactly the tokens counted**,
  so the dollar figure, the token figure and the §3.5 ceiling that gates on the
  dollar figure cannot drift apart; and both are a floor where a provider slices
  disjointly, never an over-statement, because a spend figure that over-reads
  refuses a birth the operator's number would have allowed.
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
  function — the §8.5 dispatch match's `Prompt` arm delegates to it, and since
  bl-1747 the frame's own fire *is* that arm, posted over the wire — so one
  gate covers the click, the slash line, the deposit and `yog gesture` at once. **There is no second
  gate anywhere**, which is the whole point of seating it at a chokepoint.
- **A birth is the only thing gateable, and that is a ruling, not an
  omission.** `Message` is *not* gated: refusing to answer a drone that is
  mid-ball strands exactly the uncommitted work the ceiling exists to protect,
  which is the same expensive failure as killing it. `Prepare` is not gated
  either — a claim spawns nothing and is releasable, so the refusal belongs at
  the one irreversible step rather than one step early. The bound on a drone
  that is *already alive* is litany's own `max_total_tokens`, one layer down,
  where the loop that spends it runs; yog's ceiling bounds the fleet's
  births and says so.
- **The value is one `ui.json` number** (§4.1 `ceiling`), USD, beside the price
  table it is denominated in — **severable in both directions**: delete
  `ceiling` and the gate is gone; delete `prices` and it is gone too, because a
  ceiling is a dollar figure and yog refuses to bound dollars it cannot compute
  rather than inventing a token proxy for them. No new config artifact, no
  setter, no verb, no flag.
- **The figure it compares is the whole world's** (bl-a80a) — every workspace
  the §3.1 roster names, folded into one number. At-or-over refuses. The
  comparison is against the figure's **floor** — unpriced tokens are reported
  (above) and never guessed at — so the gate refuses only on spend it can
  actually name, and a literal `0` is honored as written: the deliberate hard
  stop.

  **It was the target workspace's, and that was a defect, not a granularity
  choice.** The argument for the narrow scope was attribution's — a workspace is
  the sphere a drone lives its ball in, the one scope a spawn names outright —
  but *what a figure is honest about* and *what an operator's allowance covers*
  are two questions, and the number answers the second. `ceiling` is **world
  config**: one `ui.json`, one key, beside the one `prices` table it is
  denominated in (§4.1). A world-level number read as a per-sphere bound gives
  one operator figure as many meanings as there are workspaces, so arming a
  second project silently doubled the allowance the operator wrote — and §4.3
  arming is one entry per workspace, so a whole-day operator's portfolio
  multiplied it by however many projects were live.

  **One key, one comparison, one gate, one home.** The multiplication is gone by
  construction rather than policed by a second rule: no world ceiling *beside* a
  workspace ceiling (two ceilings over one concern is exactly the shape bl-56af
  deleted), no precedence table, no stored counter, no new durable fact. The
  fold is a query over bills yog already publishes — `spend::of_world` at the
  gate, walking the roster at the instant of the refusal, and
  `spend::priced` over `Snapshot::bills` for the V4 board's copy of the same
  verdict. Severability is untouched: delete `ceiling` or delete `prices` and
  the gate is gone, exactly as before.

  **The cost is the ruling's, and it is stated rather than mitigated.** An idle
  workspace is refused because a busy one spent, and an operator who wrote `25`
  meaning *per sphere* now has a world allowance of `25`. That is the safe
  direction: a ceiling that binds sooner refuses a **birth**, which is the one
  thing it may refuse, and never touches a live drone. Because the figure is
  lifetime-cumulative (nothing windows it and nothing resets it — the ceiling
  has always meant *"this has spent enough, ever"*), a long-lived world
  eventually latches the gate; the remedy is raising the number, which is the
  remedy it had before, arriving sooner.

  **Deliberately not built** (each would be a new fact needing its own ruling,
  not a derivation off this one): a *day window*, which needs a clock origin, a
  reset and a stored or derived boundary — the first thing here that would not
  be severable by deleting a key; and a *world concurrent-drone cap*, because
  sum-of-caps is already arithmetic over entries the operator wrote themselves,
  the birth rate is already bounded at one per full sweep (§4.3 — one pilot
  thread, one move per tick, world-wide), and a drone count is a poor proxy for
  the thing being protected when yog can price the money directly.
- **A refusal renders where refusals already render.** It writes the §4.2
  `["yog-step","ceiling"]` failure line before it rides back, carrying the
  start's own `Origin`, so it banners at the rung that fired it (§7.3) and
  counts toward §6 attention like any other failed action; the composer keeps
  its goal and its unspent mint seed, exactly as any other failed fire does.
  The text names both figures and the key to edit. Nothing is refused
  silently, and nothing running is touched.

### 3.6 Deletion: unmaking a workspace (bl-ef89)

**Deletion is the raise's inverse, at the raise's altitude.** Raising a
workspace is `mkdir -p` the names root + `litany new <root>/<name>` under the
operator's chosen name (§3.1, §3.4); deletion is the
release of the sphere's live claims followed by removal of the workspace
directory. litany has no workspace-delete verb, and that is not a gap to work
around: `litany new` accepts any path — the caller owns placement, so the
caller owns disposal. (An earlier draft grounded this in "litany's retention
is branch-level GC inside a living workspace" — **false against the shipped
crate**: lernie through 0.0.4 ships no retention or GC of any kind, and since
0.0.4 it ships an *agent*-scoped `litany delete` — the verb the
one-conversation delete below spawns. The conclusion stands on the placement
argument alone, corrected bl-f17a.) Removing the whole dir is a write to
yog's **own names root** (§3.1: "the root is yog's own territory"), never a
write *inside* a workspace, so I2's litany-verbs-only rule stands untouched.
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
the wall's contents go with it. Until `litany bundle` is surfaced (§8.3,
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
   and `collapsed` override — yog's own file, one ordinary write-through
   save (§4.1: every mutation lands before the call returns; the debounce
   this line used to name was retired by bl-b54e). Not mere hygiene: the name-reuse case below must not inherit a
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
semantics (deposits, epitaphs — litany ARCH §2.9), and the refusal names the
live conversations so the operator stops them first. Verbs stay orthogonal.

**Confirmation doctrine — destructive vs recoverable (normative for every
future verb).** A verb is **destructive** iff it destroys facts no derivation
can recompute. Everything else is **recoverable** through the ordinary
primitives — Stop resumes by message, Release re-claims, Close is a
*delivery* gated by the repo's own hook — and fires
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

**Scope: named workspaces only.** Foreign workspaces are litany's auto-id,
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
(d) *an upstream `litany delete` verb* — **scoped to workspace disposal**
(amended bl-f17a): disposal belongs to whoever owns placement, yog places
workspaces, and that is the whole of the rejection. litany places *agents*,
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

**The removal is litany's verb, spawned** — never a yog write inside the
workspace (I2): `litany delete <ws> <agent> [--children]` (0.0.4), short,
piped and logged like every §8.2 litany verb, cwd the workspace,
`Origin::Conversation`. What it removes is litany's own subtree cut — the
agent's branch and worktree, its `steps/` and `inbox/` slices, its
`refs/litany/*` marks, and (under `--children`) the `<id>-*`
hyphen-descendants — and an absent agent is a quiet success (delete's
postcondition already holds), so the verb is convergent on re-run.

**The gate is the workspace verb's, member-scoped:** refuse while the root or
any member of the conversation probes Live/InFlight, the §10 "?" counting as
live — fail closed, naming them, so the operator Stops first (rejected (c)
holds here too: no kill is folded in). litany's own `Driven` decline is the
substrate's independent fail-closed under the race; yog gates first, at fire
time, off the published snapshot.

**The census is the substrate's, not a yog re-derivation.** The dialog
enumerates what dies from `litany delete --children --dry-run` — the
descendants by name and the pending-deposit count, straight off litany's
`DeleteReport` (its `--dry-run` is documented as "the census a caller's
confirmation enumerates"). One source of truth: the process that performs the
act computes what the act takes. The dry run mutates nothing and is fetched
once, at dialog open (an explicit gesture, I7) — unlogged, the `bl conf` read
seam's idiom, so opening a dialog does not append trail rows.

**Arming is the amended doctrine above, and it is also the argv:** the typed
conversation name is the *only* thing that fires `--children`; an unarmed
fire is the **bare** verb, which litany declines for an agent with
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
clears rather than naming a gone branch. Until `litany bundle` is surfaced
(§8.3) there is no archive step to offer; the dialog states
irrecoverability, and bundle-then-delete composes in front later without
changing the verb.

Implementation: `delete::agent` (gate, arming, census parse, the two spawns),
the `DeleteAgent` boundary action (§8.5 — gated in dispatch exactly as the
dialog gates, whichever frontend fires), the §11 seats in `nav::menu` /
`shell::delete_agent`.

### 3.7 Project instructions freeze at the binding (bl-aa8b)

**The verified gap (bl-e249's Claude Code comparison).** Claude Code discovers a
project's instruction files from the hierarchy it is working in *before* the
first inference. Yog did not, and neither does pinned litany: litany's shipped
worker soul is harness mechanics only, and §3.3's goal is the operator's payload
— so a drone born in this very repository met its `AGENTS.md` only if it thought
to go looking. Repository standards were a thing the model might find, not a
thing the harness handed it.

The comparison is evidence for *a* project-context mechanism, **not** a licence
to copy Claude's: no `CLAUDE.md` filename is hardcoded anywhere here, no memory
system is imitated, and yog stores no copy of anything.

**The whole feature is one sentence: the fire pins the instruction files the
binding's project declares, and yog's `config/default` says the worker composes
them.** Everything below is that sentence's five mechanical halves.

#### 1. Discovery is a walk from the authority root to the binding

The input is the §3.3 typed work target — [`Prepared::binding`], the one
channel, already the ball rung's claim-derived `work/<id>` worktree and the path
rung's directory. **No binding, no instructions**: the bare rung and a
not-yet-created ball bind nothing, so they discover nothing — the general path
with empty inputs, not a rung table (§3.3's own argument, one level up).

- **The authority root is the nearest ancestor of the binding, inclusive, that
  holds a `.git` entry** — a directory *or* a file, since a `work/<id>` worktree's
  `.git` is a gitdir pointer file. No `.git` anywhere above it: the binding is
  its own authority root.
- **The walk never ascends above that root.** That single rule is the whole
  answer to "untrusted parent instructions": a `$HOME/AGENTS.md`, a
  `/srv/AGENTS.md`, anything outside the project is not *skipped by a check* —
  it is unreachable by construction. Ambient material never enters a yog world
  (§16); instruction bytes are not the exception.
- **The chain is outermost → innermost**, root first, binding last, each
  configured filename in the policy's declared order at each level. That total
  order **is** the precedence: the most specific instructions arrive last.

Each candidate is admitted only if `symlink_metadata` says **regular file** — a
symlink is skipped, because the freeze is byte-exact and a link can point out of
the root — and its length is within [`MAX_BYTES`]. An over-size file is
**skipped whole, never truncated**: half a rule reads exactly like a whole rule,
so a truncated instruction is worse than a missing one. At most [`MAX_DOCS`]
documents ride. A path yog cannot spell — non-UTF-8, or carrying `=`, which is
`--pin`'s own separator — is skipped rather than mangled.

**No include resolution, ever.** A pinned document's `@other/file` line is text.
Following it would make the frozen bytes a function of bytes nobody froze, and
would re-open the escape the authority root just closed.

#### 2. The freeze is litany's, and yog reads no bytes

The fire appends one `--pin <dest>=<src>` per discovered file to the *same*
argv §3.3 already builds (litany ARCH §2.5, released 0.0.4 as bl-fb5c, and in
the pin since). litany loads and validates every pin **in the CLI layer, before
any branch, ref or inference exists**, writes the bytes into the fork's worktree
and commits them on the dispatch commit beside `goal.md` and `soul.md`.

So yog opens no instruction file. It stats candidates and names paths; the read,
the validation, the snapshot and the commit are all one upstream mechanism with
no filename policy of its own. A source that stats clean and then cannot be read
fails the fire in litany's own voice, pre-fork — loud, and nothing exists to
clean up. That is the explicit policy for unreadable files: **skipped when the
stat says so, refused loudly when the read says so, never silently partial.**

#### 3. The destination *is* the provenance

Each document freezes at `instructions/<NN>/<rel>` — `<NN>` the zero-padded
discovery rank, `<rel>` the file's path relative to the authority root.

- litany frames every composed block with its worktree-relative path (ARCH §5.3
  "file path as hint"), so the model reads *where* a rule came from with the rule.
- litany's assembler sorts **lexically within a category**, which plain relative
  paths do not survive (`AAA/AGENTS.md` sorts before `AGENTS.md`). The rank
  prefix is what makes precedence survive that sort — the same device litany
  uses on itself, where `summary/NNN.md`'s zero-padding is what makes lexical
  order age order.
- `instructions` is a legal pin destination: it is none of litany's reserved
  first segments (`goal.md`, `soul.md`, `name`, the control files,
  `descriptions/`, `messages/`, `summary/`).

#### 4. Freezing is not composing — so yog authors the manifest glob

**The premise that would have shipped this broken:** that pinning puts a
document in context. It does not. Whether a pinned blob composes into assembled
context is the question of the `manifest.yaml` the conversation **resolves**
(litany ARCH §5.2; §5.1 #17 and §9.4 for which commit that is), and the
shipped worker role pins `goal.md`, `soul.md`, `descriptions/**` and orders
`summary/**`, `skills/**` — nothing else. A pin at `instructions/…` under the
stock manifest is a committed file no model ever sees.

So `roles.worker.pinned` gains one glob, `instructions/**`, authored onto
`config/default` **at every start**, by the same fixed-point convergence §8.6
already runs for `tool_control:` — read the committed file, compute the wanted
file, stage only if they differ, and let the §9.3 scripted-`$EDITOR` drive of
`litany config` (the only lawful writer of `config/*`) commit it. Authoring an
authored manifest reproduces it byte for byte, so the steady state reads one
file out of git and spawns nothing. **Since bl-e654 that authoring reaches the
running conversations too**, for §8.6's reason exactly: the manifest is read
from the commit a conversation follows, re-resolved at every step boundary
(§9.4), so a glob authored onto `config/default` takes effect at the next step
of every conversation on that lineage rather than only at the next fork. **The
documents it composes are still the fork's**, and that half is untouched (point
2 above): the glob says *include `instructions/**`*, and what answers is
whatever the agent's own dispatch commit froze there — nothing for a
conversation forked before the freeze existed. The composition rule follows the
tip; the bytes it composes do not.

**One drive, not two.** §8.6's control block and this glob are two files of one
policy, so `start::ensure` collects both drifts and converges them in a single
`litany config` pass — one checkout, one commit, one ops row. Each author owns
its own file's fixed point and knows nothing of the other.

`pinned:` rather than `order:` deliberately: pinned is included regardless of
budget, so instructions cannot be silently shed. They still *count* toward the
budget, which is what [`MAX_BYTES`] and [`MAX_DOCS`] bound. A manifest with no
`roles.worker.pinned` anchor is an operator's own manifest, and yog leaves it
alone rather than fighting it.

#### 5. Policy is severable; provenance is derived

**Filename policy.** The default is `AGENTS.md` and lives in code. The override
is `instructions.yaml` in the workspace's config commit, beside
`capability.yaml` and read the same way — at `config/default`'s **live tip**,
never a governing commit, because it is the operator's policy and not the
agent's structure (§8.6's own argument). The grammar is one line shape:

```yaml
- AGENTS.md
- CONTRIBUTING.md
```

Reading is total, like §8.6's: a line that is not `- <bare filename>` is not a
name. **The file, when it exists, is authoritative including when it names
nothing** — that is the explicit opt-out. Deleting it restores the default, so
severability is exact: removing policy deletes a file, it does not edit code.

**Provenance stores nothing.** Three surfaces already carry all of it:

| what | where | how |
|---|---|---|
| which files, from which absolute source | `ops.jsonl` row of the fire | the argv is logged whole; `clip_goal` trims only the final element (§4.2), so every `--pin dest=src` survives — the §11 activity trail opens each row to its full argv |
| the exact frozen bytes | the dispatch commit | ordinary blobs: the §11 Files tab lists `instructions/**` in the agent worktree and previews them, and the pinned rail reads that same tab out of git |
| what the model was told | assembled context | each block framed with its `instructions/<NN>/<rel>` destination (ARCH §5.3) |

No yog registry, no manifest file, no second copy — the pins are the record, and
the record is git's. **The operator's goal stays their payload** (bl-6920): the
freeze is a spawn *parameter*, never a concatenation into the goal text.

#### Children inherit the fork, which is the freeze

VISION §4.10 forbids a child inheriting stale instruction bytes. Nothing does:

- A child's dispatch commit forks its parent's tree, so it carries the parent's
  frozen `instructions/**` verbatim — the same bytes, of the same lineage, for
  the same target, because **a child born today binds no directory of its own**
  (§4.10 items 2–3: "nothing is inherited … a child of a `--cwd` agent is back
  in its own worktree"). There is no second target for the bytes to be stale
  against; the divergence the rule guards has no way to exist yet.
- When a child *does* gain a binding — bl-8746's attempt fan over balls
  bl-4eac — it is created with one, so it freezes against it at creation. That
  is this same section one level down, not a new rule (bl-a1a4's fractal law).
- A child that walks out of its root with `cd` is §4.11's open-world
  adjudication, an existing gate, not a hole here.

**Implementation:** `start::instructions` (the walk, the rank, the pin specs),
`start::instructions::names` (default + the `instructions.yaml` override),
`start::instructions::manifest` (the glob's fixed point), the `--pin` arms in
`start::prompt`'s one argv, and the single convergence in `start::ensure`.
### 3.8 The mutating fan: N isolated candidates over one obligation (bl-8746)

**Governed by VISION §4.10 (bl-2b8c); this section is only where yog's half of
it lands.** A start binds one working directory (§3.3, bl-6654's typed `--cwd`).
A **fan** is that same start spent N times, each spend bound to a different
directory — and the directories are balls', not yog's.

**The mechanism is entirely upstream** (balls 0.5.10: bl-a1a4's source-owned
delivery law, bl-4eac's attempt capability). `crate::fan` is a thin, in-process
consumer of it, because the capability has no `bl` verb and upstream rules that
it must not have one — *"a verb would be a second entry point to a capability
whose whole point is that the N = 1 ball path and the N > 1 alternative paths
are ONE mechanism"*. So:

- **The target is asked for, never spelled.** `Project::target(Some(<ball>))`
  answers the ref `bl close` already derives — `work/<id>` — so accepting a
  candidate advances the ball's own branch and the ball's later close is that
  same delivery one level up. `None` is the bare project-repo obligation: the
  integration branch the project itself names. `fan::Obligation` carries the
  pair, because a project without its ball is half a target.
- **The handle and the worktree are balls'.** yog constructs neither; it reads
  balls' `attempt_path` back to *verify* a binding rather than to build one.
- **N = 1 is not a case.** `fan::spread` with `n <= 1` materializes nothing and
  hands back the ordinary claim binding, so the single start and the fan are one
  code path walked once and walked N times.
- **One fan, one base.** `Attempt::open` takes a *ref*, resolved per call, so
  the shared fork point is not structural upstream. `fan::open` proves it and
  refuses a fan whose members report different bases — members that do not share
  `(target, base)` are not siblings and yog will not present them as one.

**The gesture is `Fan(fan::Verb::Spread)`, and it fires nothing** (§8.5): it
answers with the
prepared start once per candidate, and each is spent through `Prompt`'s own
door — so the §3.5 spend ceiling gates every birth exactly as it gates a single
start, per-variant overrides are the caller's, and the trail carries N ordinary
fire rows. It is nonetheless one gesture rather than N, because N attempts off
one pinned tip cannot be N gestures without losing the shared base.

**The cohort is derived, and from yog's own trail** (`fan::cohort`). Membership
is the fire rows' `--cwd` bindings that balls' attempt formula reproduces,
joined by the workspace they fired in and the claim that names the project.
Deliberately **not** litany's `refs/litany/cwd/<agent-id>` mark, for the reason
§8.6's writable root gives: the mark is rewritten by the agent's own `cd`, so a
cohort read from it would be a set the candidates could edit themselves. There
is no fan registry, no membership field, and no winner — acceptance is the
delivery the target's history records, and rejection is the absence of one.

**The writable root grows with it** (§8.6, VISION §4.11 item 3). With N > 1 no
work happens in `work/<id>`; each candidate writes in its own attempt worktree,
so `control::root::candidate_worktrees` puts exactly those directories in the
root beside the claim's. Same two yog-owned facts, same refusal to read the
agent's own mark.

**Retirement is two calls and one severable policy** (`fan::retention`). A
retirement always releases the worktree and keeps the source ref, so a rejected
candidate stays fully addressable; the ref goes only when `cadence.yaml`'s
`retention:` block declares a `keep_min` for that project and the candidate has
outlived it. Absence is *never discard*, for the reason the armed loop's
`lease_min` is absent by default (§4.3): deleting a ref destroys the only
record of a rejected candidate, and yog does not do that on an opinion.
Severability is deleting the entry.

**Rework needs no mechanism at all** (VISION §4.10 item 5). A stale candidate is
reworked by `Message`-ing the agent bound to it to incorporate the current
target in its own worktree and redeliver; balls refuses a stale source before it
merges, gates or moves anything. yog never reconciles, and the absence of a
reconcile path in `crate::fan` is that rule's implementation.

**The fan resolves — V3's half of this surface (bl-c2bd).** Four rulings, each
spending what this section already built rather than adding a mechanism:

- **Acceptance is `Fan(fan::Verb::Deliver)` — "Deliver candidate", never
  Adopt** (VISION V3.2, §4.10 items 5–6). It is balls' one delivery law spent
  by handle (`fan::delivery`): the target must already be incorporated — a
  stale source refuses before anything merges, gates or moves, and yog never
  reconciles — the repo's own hook gates the exact source tree, and the squash
  lands tagged `[<handle>]`. Delivery advances the obligation's own target
  (`work/<id>` for a ball), so it neither closes that ball nor changes what the
  ball's later close delivers; after one candidate lands every sibling is stale
  by construction and reworks by the standing rule above — sequential synthesis
  out of the law, no primitive anywhere. The reply is the four identities the
  delivery computed (`fan::Delivery`), a receipt and never a stored winner. All
  three serializations carry it: the RAM variant, `{"op":"deliver"}`, and
  `/deliver <handle> <summary…>` — the summary is the verbatim tail, because a
  delivery subject is the operator's statement of what landed.
- **The acceptance mark is derived, and there is no other kind**
  (`fan::delivered_commit`): the `[<handle>]` tag-scan over the target ref's
  own history — the same tag balls' retry-standing greps for — answering the
  delivery commit or the one absence that covers pending and rejected alike,
  because rejection *is* the absence of a delivery. No reject verb, no outcome
  field, nothing stored.
- **The candidates read on the work-diff surface** (`workdiff::candidates`,
  VISION V3.3): one row per cohort member at the ruled
  `work/<id>..attempt/<handle>` range, wearing the derived mark, on both
  frontends through `Query::WorkDiff`; the patch drill-in is addressed by ball
  **and** handle, because a fan's candidates all wear the obligation's ball.
  The obligation is read from the same last-claim rule the §8.6 writable root
  spends (`control::root::claimed`) — one rule, one home, consumed twice.
- **Judge and Synthesize needed no mechanism** (VISION V3.1): both are V2's
  fire path — an ordinary dispatch whose *goal* carries the candidates' exact
  terminal refs, which the work-diff rows already state per candidate
  (`source_oid`, the attempt branch, the base, the target). The verdict is the
  judge's ordinary committed message; a read-only synthesis child holds its
  merged result as its response, and a synthesizer that would write project
  bytes is itself an ordinary attempt on the same target (§4.10 item 1). No
  fan-in primitive exists, deliberately.

**The §11 seat landed (bl-77bc), and it spends what this section built and
nothing new.** The fan group card renders on the Work tab over the §3.9
science rows (whose `diff` *is* this section's candidate row), the N picker is
a control on the pending start's own pane — Send with N > 1 posts one
`Fan(Spread)` and its receipt walks the rebound starts back through `Prompt`'s
ordinary door, so the ceiling gates every birth exactly as it gates one — and
all four affordances **compose dispatches instead of adding doors**
(`science::compose`): Judge and Synthesize seed the ordinary new-conversation
composer with a goal carrying each candidate's exact refs, Deliver seeds
`/deliver <handle> ` awaiting the operator's summary, Retire seeds
`/retire <handle>`, and in every case the operator's Enter is the fire. The
Adjudicator story graduated with it (STORIES S19). The per-attempt science
projection landed as §3.9 (bl-40ab), and it left one correction here: **the
claim attempt wears the derived acceptance mark too.** `workdiff::resolve` filled `delivered`
for candidates only, because V3's surface was about candidates — so the ordinary
single start was the one attempt whose acceptance could not be read, which makes
N = 1 a case exactly where §4.10 item 8 says it is not. The scan is unchanged
(`fan::delivered_commit`, whose own doc says the tag it reads is *"an attempt
handle or a ball id"*); for a claim the id is the ball, so the mark answers that
ball's own delivery onto the branch its close delivers into.

### 3.9 The science projection: one attempt, joined at read time (bl-40ab)

**Governed by VISION §4.10 item 7** — *"The science projection is a query:
frozen inputs (goal, pins, governing config commit — model and skills ride it),
base/source/target/delivered OIDs, terminal response, usage, wall time, project
diff, verdicts (messages), and the accepted/rejected/reworked outcome — all
derived at read time from litany step records, balls delivery identities, and
git ancestry. Nothing stored."* `crate::science` is that sentence and nothing
else: it owns **no fact**, and its whole content is the join. **One term of that
sentence has since been amended** (VISION §4.10 item 7 as amended, yog bl-e654):
the config commit is no longer a *frozen* input — a conversation follows its
config lineage's tip at every step boundary (§9.4) — so the column below names
the policy an attempt resolves rather than one its fork froze. `goal` and `pins`
are unchanged; those really are the fork's.

**One typed query through the §8.5 boundary, addressed at a workspace.**
`Query::Science { workspace }` → `Reply::Science(Vec<science::Attempt>)`, spelled
`/science` and `{"op":"science","workspace":…}`, answered by the one
`boundary::answer`. It is a workspace read for the reason `Query::WorkDiff` is:
the attempt set of a world is the attempts its workspaces hold, and the seat
looking at one is asking about that one.

**The row set is the work diff's row set, and the diff column *is* a work-diff
row.** `science::project` calls `workdiff::read` — the ordinary claim attempt
plus each §3.8 fan candidate — rather than enumerating attempts a second time,
and each science row *carries* that `workdiff::Attempt`. Two representations of
one fact drift (VISION §4.5), so the identity, the two refs, both OIDs, the
churn and the acceptance mark have one home, one wire spelling (`workdiff::wire`, both
directions) and one set of `Change` arms. What the projection adds is the
**agent side**, and every column of it names an authority that already had it:

| column | authority |
|---|---|
| the bound conversation, and the `--pin` documents frozen at its fire | the §4.2 trail's own fire row (`fan::fires`) |
| the goal | the agent worktree's `goal.md` — the dispatch commit's own copy |
| the config commit the attempt **follows** (model and skills ride it, so neither is a column) | §5.1 #17 whole — the `governing_config` ancestry walk *and* the `follow` derivation over it. Since bl-e654 this is not a frozen input: fork settles the lineage, control resolves its tip at every step boundary, so the column names the policy the attempt resolves **now**. What governed a given step is a fact about when the step ran and is filed upstream (litany bl-e4a0); what the projection compares candidates on is unaffected, because each step's `request.json` already carries the policy's effects (§5.1 #13) |
| usage, wall seconds, step count | litany's step records, off the walk already published as `Snapshot::bills` |
| the terminal response, and the verdicts | the committed `messages/` tree (`crate::transcript`) |
| the base both ends departed from | git: `merge-base target source`, which is balls' own base formula |
| accepted / rejected / reworked | git: the target's history, and ancestry |

**A compacted conversation's row says so** (bl-fde5). The verdicts and the
terminal response are read off the committed `messages/` tree, and litany's
compactor deletes from that tree (§5.1 #12): verdicts delivered in a squashed
span are deleted files — unrecoverable, and not guessed at. The row therefore
carries `compacted`, how many entries the counter proves deleted — derived from
the same spliced markers §5.1 #12 already seats, stored nowhere, zero on an
intact record and absent from the wire then — so a short verdict list over a
compacted arm reads as *the surviving record's verdicts*, never as the arm's
whole history. The §11 fan card states it beside the figures it bounds.

**The base OID rides the science row, not the diff row.** Item 7 names three
OIDs beside the delivered one; the diff row already carries source and target,
and the third — the commit the two ends departed from — is a *cohort* fact
rather than a churn fact, so `Query::WorkDiff` would pay a git read it never
renders. Item 6 makes balls the authority for it and balls' authority is a
**formula**: *"the exact commit this attempt started from:
`merge-base(target, source)`, derived, never stored"*. yog spells the formula
because the only way to ask balls is to resume the attempt, and resuming
re-materializes a worktree — a read must not write. Absent, honestly, when
there is no resolved pair, when the project cannot be located, and when the two
ends share no ancestor at all.

**Neither ancestry read is new git** (a correction inside this ball). `merge-base`
and `merge-base --is-ancestor` were already spelled once, in `git_tree::cmd`, for
the §9.3 config fold, and this projection spends those two rather than adding a
project-repo copy of each — a second spelling of one git command is exactly the
drift `git_tree::project` exists to prevent. What is local to `science` is the
degradation: an unanswerable ancestry probe is `false` (no claim that a rework
happened) and an unanswerable `merge-base` is absent (no guess at a base).

**The binding is one rule at every N.** The conversation bound to an attempt is
the **last** fire whose `--cwd` names that attempt's worktree, and both worktree
formulas are balls' own — `attempt_path` for a candidate, `work_worktree_path`
for a claim (both leaf spellings, `<id>` and `<id>-<claimant>`, for
`control::root`'s reason: which exists is a disk question the join need not
ask). So N = 1 is not a case here either. The fire row is parsed **once**
(`fan::cohort::fires`, extracted for this): the cohort asks whether the bound
directory is an *attempt*, the projection asks which attempt of any kind it is.
That is where the module split falls too — `bound` answers *which* conversation,
`observed` answers *what about it*, and the two share nothing but the agent id.
Never litany's `refs/litany/cwd/<agent-id>` mark, for §3.8's reason — the agent
rewrites it with its own `cd`.

**It reads nothing twice.** The step-record columns are an in-memory filter of
`Snapshot::bills` by the attempt's own conversation tree (bl-9dd4's ruling
applied: the tree is walked once on the worker, and every later figure is a
predicate over the result), so a workspace of ten attempts costs no extra pass
over `steps/` at all. **Wall time rides that same walk** — `StepBill.wall_secs`
is the step's `meta.json` `started_at`→`ended_at` span, summed per step exactly
as litany's own `budget::derive::wall_seconds` sums it, because *wall is wall*:
the span covers a step's backoff sleeps, and a first-to-last reading would bill
an idle hour between two calls. The one calendar routine yog owns gained its
inverse for it (`ui_state::epoch_from_iso8601` beside `iso8601_extended`) —
still no `chrono`, and deliberately **not** an RFC 3339 parser: it accepts
exactly the shape litany's clock writes, because that clock is the only writer
this crate reads and a tolerant parser would invent an answer for bytes no
litany produced.

**The terminal response is committed-only, and that is a ruling.** §8.5 folds the
in-flight tail onto `Query::Transcript` so two seats never describe one moment
differently — but *terminal* is a settled fact, and a tail is not one: folding
it would make this column say something a re-read a second later contradicts. A
live tail has a query, and this is not it.

**A verdict is a message, and yog classifies no prose.** Every message delivered
into the attempt's conversation rides, with its sender, in order. VISION V3.1
rules that a judge's verdict is an ordinary committed message; which messages an
operator counts as verdicts is the reader's question, and a filter on wording
would be yog deriving meaning from prose — the thing §4.10 item 2 forbids
everywhere else.

**The outcome is four arms over three git facts, and no stored anything.**

- **Accepted** — the target's history records *this* attempt's delivery
  (`fan::delivered_commit`, the mark the row already wears). The only acceptance
  there is.
- **Rejected** — this attempt's delivery never happened and something else's
  did: `by` names the sibling whose delivery the target records, or is absent
  when the attempt was discarded and its source ref is gone (which the diff row
  already states, by naming the source among the refs that did not resolve).
  Siblings share the **target**, not merely the ball — a fan's candidates target
  `work/<id>` while the claim targets the branch that ball closes into, so a
  ball-only test would read the claim's own close as a sibling's win.
- **Reworked** — rejected above, and then reworked: the source has incorporated
  the target, so balls' delivery would no longer refuse it as stale. **This is a
  reframe of the ball's own wording** (*"the source advanced after a refusal or
  verdict"*), and the reframe is the design: a clock-based reading needs three
  clocks yog does not share — the trail's unix seconds, a `messages/` filename
  counter, and a commit date in the project repo — while what that reading is
  *for* is whether a superseded attempt can deliver again, which §4.10 item 5
  already defines as having *"incorporate[d] the new target in its own
  worktree"*. So the test is `git merge-base --is-ancestor`
  (`git_tree::is_ancestor`), the delivery law's own precondition read from outside:
  one exact git question, no clock, and true of a rework done by hand. A refusal
  is the *occasion* for a rework, never its evidence.
- **Pending** — none of the three. It is their absence rather than a fourth
  policy, and naming it is what keeps them statements: an attempt yog can say
  nothing about must not read as rejected.

**The seat (bl-77bc).** The §11 fan-group card renders this projection on the
Work tab — `science::render` the card, `science::respdiff` V3.3's response
comparison (a pure line LCS over the two `response` columns, capped and honest
about the cap — its own forty lines rather than a dependency, because the two
responses live in two different conversation repos and there is no one tree to
ask git about), `science::compose` the affordances as composed dispatches —
and the Work tab's listing *is* this answer since the same ball: the attempt
rows drill into each row's own `diff`, so the card and the listing cannot
disagree about which attempts exist, and `Query::WorkDiff` is asked only for a
picked file's patch. Neither module owns the other's columns.

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
  kind, keyed to ref oids (notify/budget/conflicted = the `refs/litany/*` ref
  target; stopped = the branch tip oid at acknowledgement, stamped for any
  ***at-rest*** agent, not only a stopped one — §6 rule 2 as amended bl-2194.
  The key keeps its historical name because the watermark's identity is the tip
  oid, unchanged: that is what makes the widening migration-free — every
  `ui.json` already on disk stays exactly as valid). litany's marks are
  level-triggered and yog may not delete refs ("the UI is a pure reader"; no
  ack verb exists) — so "the user has seen this" is a yog fact: **the mark is
  litany's, the acknowledgement is yog's.** A moved ref re-notifies.
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
- **`ceiling`** — the §3.5 spend ceiling: one number, **USD**, the bound *this
  whole world's* spend must stay under for yog to start a *new* conversation
  anywhere in it. World-scoped since bl-a80a: it is a world-level key in a
  world-level file, so it names a world-level allowance, and arming a second
  project does not multiply it. Read exactly like `prices` and for the same reasons — read-only, no
  setter, no editor, no verb, live within a tick through the whole-file
  `adopt`. Absent, non-numeric or negative all read as **no ceiling**: deleting
  the key deletes the gate, not a code path. An empty `prices` deletes it too
  (§3.5: a ceiling in dollars needs the table that makes dollars). A literal
  `0` is honored — the hard stop that starts nothing new. It bounds *births*
  only; nothing already running is ever stopped by it. **It is also the only
  ceiling a yog-dispatched conversation has** (bl-56af): §8.6's workflow fixed
  point strips litany's `budgets:` block from every workspace's `config/default`,
  so there is no second, token-denominated bound that could drift from this one.

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
(`litany prompt`; its own `stderr` is always empty, and whatever the driver
says afterwards folds in from the sink — a death, or an operator notice,
bl-1296), `-3` a synthetic failure line — a spawn that never
launched, piped or detached, or a non-spawn `["yog-step",…]` step.** An error
class with no ops row is an error the UI cannot render — the §7.3 failed-action
row depends on this:

```json
{"ts":"2026-07-17T12:00:00Z","argv":["bl","close","bl-4db6"],"cwd":"/home/u/dev/brazen","exit":0,"origin":"balls","stdout":"…","stderr":"…"}
```

- **`origin` is the §7.3 attribution** — `balls` / `conversation` / `world`,
  the [`opslog::Origin`] tokens (bl-48f8). A *fixed* field like `ts`/`cwd`/
  `exit`: never truncated, and the one thing a banner surface filters on. It is
  **stored, not derived, because it cannot be derived**: `bl close` and `litany
  message` are told apart by their `argv`, but a ball-rung start and the
  composer's Enter write byte-identical `litany prime` / `litany new` /
  `["yog-step","mkdir"]` / `litany prompt` lines, so any read-time
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
  `litany prompt`'s composed goal — to a bounded head with an explicit
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
  gate/close output, `litany scan` summaries, and error text are *not* on disk
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
  to join them. A row whose folded `stderr` holds anything the notice classifier
  does not recognize is a rendered failure by the `-2` rule above — that is how
  a driver that died *after* launching stops being invisible. **Non-empty was
  the whole test until bl-1296 and it was too wide** (the notice class below):
  litany's driver stderr is an operator-notice channel as much as a dying one,
  and this sink is append-only for the driver's whole life while the fold
  re-reads its tail every sweep, so one benign line held the newest row of its
  origin in ichor until it was acked.
- **One sentinel, one fact — and the field is never rendered raw (bl-afa9).**
  `-2` used to be written for both a detached `litany prompt` that handed off
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
- **The `-2` sentinel's failure is a STATE, not what its sink said (bl-b95e).**
  This is §13.3's `driver.log` ruling — *"its **content** is never the trigger
  — a stale line from a healed crash must not alarm — only the diagnosis"* —
  finally applied to the one capture file yog still read the other way round.
  The pinned litany ARCH binds a detached driver's stderr to
  `steps/<agent-id>/driver.log` and states what that file is for: *"it is where
  every driver states what it **declined** — a compaction landing declined or
  superseded (§2.6), a launch that failed into the accepted crash class, a §6
  budget stop — and those lines are addressed to an operator"* — every one of
  them printed on a path that returns `Ok(())`. yog's `-2` row has no observed
  exit to read, so `OpRow::failed` substituted *"the driver said anything at
  all"* for one, and the ops sink is append-only for the driver's whole life
  while the §7.2 fold re-read its tail every sweep: one line therefore kept the
  newest row of its origin ichor-red — §7.3 banner with argv and stderr tail,
  ⚠ on the §11 chip — until the operator acked it, however many turns the
  driver went on to run. **bl-1296's answer was a marker table** over the
  sentences litany prints, a fifth outcome (`OpOutcome::Notice`) for the lines
  it recognized, and its own doc calling the table knowingly fragile and meant
  to be temporary. It was: a phrase list keyed on prose somebody else owns is
  not a classifier, and no widening of it could have reached the defect above,
  which is about *time*, not words.
  **The rule now**: `opslog::launch::stillborn` asks what the launch produced,
  and the §7.2 ops refresh folds the sink **only** when the answer is nothing —
  so a folded tail is the derivation's verdict and the bytes in it are
  diagnosis. The state is the orphaned-mail pair (§13.3) asked of the launch's
  own target: the row names it (`prompt --name <conversation>`, `advance <ws>
  <agent>` — the tokens live in `opslog::launch` so the spawn and the reading
  cannot drift), and the launch is **stillborn** when no matching agent is
  being driven (§3.5) and none has acted since the row's stamp. Both hold
  vacuously when the target is not on disk at all, which is the class the sink
  was added for: a start whose driver died before writing a branch. Ahead of it
  sit the §7.3 grace window (a launch younger than it has not had time to
  produce anything, and bl-18e8's rising edge is indistinguishable from a death
  until yog has looked again) and §10's rule against a false definite (a
  workspace that derived no tree is **no verdict**, never an accusation).
  **What this retires**: the phrase table, the fifth outcome and its badge. A
  driver that filed a notice and carried on is a handoff like any other — its
  sink is never read, so it needs no bucket of its own to be spared the alarm,
  and the `-2` sentinel is back to **two** readings that partition it,
  `OpRow::detached` (a live handoff) and `OpRow::detached_died` (`failed`).
  **What it costs**: a healthy launch's ops row carries no stderr, so the
  operator cannot expand it to read litany's notice lines. That is the trade
  the rule states — a capture log is diagnosis, read where something is wrong —
  and the lines themselves keep their durable home in `driver.log`, which
  §13.3 gives a seat of its own in the step drill-in.

---

## 5. The complete state inventory (normative)

Every piece of state in the application, classified. **This table is
normative: code review rejects any state not placeable in it.**

### 5.1 Derived-from-disk (fact → home → derivation; never stored by yog)

Every path fold below resolves through the composed world env (§16.2), and
every **balls** fold through the §16.3 space standing in it — the world's when
no `YOG_MARKS` is layered on, which is every read yog itself makes:
`LITANY_HOME` and `$XDG_STATE_HOME` name nested locations, and
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
| 5 | Work-worktree path | formula + `bl claim` | bl-delivery formula recompute (`binding::work_worktree_path`), cross-checked against `bl claim`'s own stdout (`start::exec::cross_check_claim`; a mismatch is a `Drift` refusal). **Two readings, not three:** `git worktree list` is git's registry of the same fact and yog does not read it — a third reading could only disagree with the two that already agree, and the claim's stdout is the authority balls itself prints |
| 6 | Workspace list + names + foreign + replays | `$XDG_DATA_HOME/yog/workspaces/*`, `<litany-data>/workspaces/*`, `replays/*` | readdir for `repo.git`; leaf = name per §3.1 |
| 7 | Join state per (ball, workspace) | #2–#6 | the §3.5 claimant join, a pure function |
| 8 | Agent set, descent, tips | `repo.git` refs | `git for-each-ref agents/*`, then the §2.3 grammar over the ids (existing `git_tree`): an id's parent is that id less its last **descent segment** — `<ts>-<short>`, exactly two hyphen-free tokens — and it is a child only if that derived parent is **present in the ref set**. Absent (an id outside the grammar, or a deleted intermediate) ⇒ a root row, never a re-attach to some shorter prefix; the registry intersection is litany's own ruling (ARCH §8) and the two must not fork. **The id-derived tree is the *provenance* fact** — who dispatched whom; since litany bl-a693 a `--from` child's *context* (git ancestry) can diverge from it, and the two are distinct edges, never conflated (VISION V1.3's two-edge taxonomy): §11 membership stays this descent-id tree, while V1's spine states the distinction in the child card's fork-label wording (`from here` / `from <Name>@<oid>` vs `from config/<name>`) — the solid/dashed strokes went with the gutter (bl-1802), and a drawn descent graph was **declined** (bl-5cf8): the words are the rendering — the list's indentation already draws provenance, the fork label already words context, and a stroke between rows is a fact the text-reading acceptance harness (bl-bc06) could never hold to account. **Since bl-fa82 the §11 conversation list is a rendering of this tree, and since bl-8905 the only one**: every visible row is the subtree rooted at its agent, `children_of` is the row's `direct` count and the subtree size less one is its `total`, and the all-collapsed case is the root-only list this seat had before. The strict rule is what the list obeys — the Stop menu's looser `+children` prefix test is a different question and stays where it is |
| 9 | Agent state {Live, InFlight, Quiescent, Stopped} | inbox flock + response.json framing | LockProbe + WriterProbe (tri-state, §10) + the §4.4 settled reading (ARCH §3.5/§4.4). **The settled tail yields two facts, not one** (bl-fb87): the transport `Framing`, and beside it the *semantic* `Ending` its canonical `finish.reason` names. Quiescent is `Settled::whole` — complete **and** ended on the model's own terms; a tail that framed cleanly around a turn the request's `max_tokens` cut off (`Ending::OutputLimit`) is **Stopped**, because transport completion is not task completion. Still four states — bl-d816's badge ruling stands, the badge answering "needs me?" and the workspace pane answering "why" — and the why rides beside the state as the truncation reading (#13's wound at Altitude 1/2, and §8.2's Nudge gate) |
| 10 | Streaming text, tool calls | `steps/<id>/NNN/` | existing `streaming.rs` / `tools.rs`. The response fold is **one read yielding one value** (bl-b768, bl-54f7): a `Stream` carrying the answer text, the reasoning text and the *kind of the last content delta* (#28b), and `Agent::stream` holds that value whole rather than splaying it into fields that could be filled from three reads. Two reads would cost a second syscall per agent per tick and could catch two different mid-write states of one file, so the fold returns all of it or it is not one file's answer. **On the focused conversation the same fold runs again, off-derivation, at the rate the model writes** — §7.2's live tail. It was a fold onto the rendered `Agent` until bl-73e7; it is an *answer* now (`Query::Follow`, held on its own lane), spliced onto the committed transcript at `Transcript::with_live`, so nothing on a rendered snapshot is put there by anything but the derivation |
| 11 | Pending messages, inbox contents | `inbox/<id>/*.md` | count + parse `---from/deposited_at/epitaph---` frontmatter. **The painted `✉ from · at` header's sender rides the §3.3 display ladder** (bl-b6d0): `from:` is an agent id, so the header states what #8's roster calls that agent — the name fact, else its payload line, else the ladder's floor (the id's terminal generation, bl-63a1), which is where a sender no living agent carries lands (`user`, a foreign id, a reaped subagent whose mail outlived it — one call, no branch). The file's own `from:` is untouched and stays the fact: it is what the §11 Raw toggle shows and what the §8.5 `inbox` reply carries, because those two are *about the file*. Derived where it is painted, never stored on the deposit, so a row and the roster of the same frame cannot disagree |
| 12 | Transcript, and **how many messages have landed** | `agents/<id>/messages/NNN-<origin>.*`, plus `agents/<id>/summary/NNN.md` | readdir + sort; origin from filename; "tool in progress" = tool_use with no tool_result. The **count** (`Agent::messages`) is the `NNN` counter's **high-water mark** — how many messages have *ever* landed — never a count of files present (bl-fde5), and rides on the same readdir the §11 recency fact (`Agent::last_action_unix`, bl-cad5) already performs — one directory walk yielding both, the #10 discipline — because it is the one honest observation of "the message I just sent has landed" that the §7.2 pending echo reconciles against: a landed message advances the counter and a compaction never lowers it (the compactor deletes only *below* the surviving counter), where a file count goes DOWN when a compaction lands mid-flight — a movement the echo's passed-the-baseline predicate has no reading for, which would strand the echo behind a baseline no landing could pass. **The directory is NOT append-only** (bl-7bd2): litany's compactor `git rm`s message files and squashes the span they lived in, so a readdir alone renders a *rewritten* record as the whole record — a reply on screen whose question is gone, which is what an operator sighted. The removal is **derived, never stored**: the `NNN` counter is monotonic from `001`, so a first entry above it or a mid-sequence discontinuity IS the deletion, and `transcript::compaction` seats a virtual `EntryKind::Compacted` marker in each hole — the same move the §7.2 live tail makes for the entry no file backs. Two limits of that query are the honest floor rather than cases it handles: entries deleted off the **end** leave no hole to bound, and a conversation compacted **whole** leaves no counter to read. The text that *replaced* the span is `summary/NNN.md` at the agent worktree root — a **sibling** of `messages/`, numbered branch-globally across passes — and **nothing on disk links a summary to the span it replaced**: one pass may delete disjoint runs (one summary, many gaps) and two passes' deletions may abut into one hole (one gap, many summaries — the ordinary shape, since the shipped template retains no tail). So no positional pairing is made; the summaries are the conversation's **compaction record**, seated whole on the earliest gap, which asserts nothing about which replaced which. Each marker asserts only what the counter proves |
| 13 | Step diagnostics | `steps/<id>/NNN/{meta,request,response,staging}.json`, `stderr.log`, `tools/`, `steps/<id>/driver.log` | full-file reads, jsonview rendering — every byte inspectable. The §7.3 **step wound** derives from the same bytes, in two disjoint classes. *No response*: an empty-or-absent `response.json` with no `meta.json`, on an agent nobody is driving (#9) — and since bl-55d8 its **reason** is a third read of that same directory, `stderr.log`, gated on the predicate so a healthy step pays no syscall for it. *Output limit* (bl-fb87): a settled tail whose `Ending` is `OutputLimit` — the class whose framing is `complete`, which is why it needed a carrier at all and why its badge cannot be recovered from the framing. Four readings, one derived value (`Wound::{None, Mute, Spoke, OutputLimit}`), nothing stored. Since bl-83d6 both capture logs are **records in their own right** — a picker seat each, offered when the file has bytes, read on demand beside the drill-in and never in the standing list |
| 14 | Marks ×4 | `refs/litany/{conflicted,budget-exhausted,abandoned,notify}/*` | `for-each-ref` (marks.rs extended from 2 to 4 namespaces) |
| 15 | Attention (per agent / rollups / totals) | #9, #11, #14 + `ui.json.seen` | §6 predicate — pure |
| 16 | Budget *spent* | Usage events across `steps/<root>*/` | fold; limits displayed only as raw `workflow.yaml` text (no YAML dep). **The four counters are not four disjoint slices, so the fold is not their sum** — it is `max(input, cache_read + cache_write) + output` per usage line, the same reading of a prompt #35 takes and at the same one home (`BudgetSpend::prompt_tokens`), because a provider that reports the cached slice *inside* its prompt counter would otherwise be billed for it twice (bl-6621; measured at +25% on an OpenAI-shaped tree, and rising with the cache-hit rate). **Lockstep with litany**, whose `budget::spend` folds the identical shape since its bl-68f5: this figure is a preview of the one that exhausts `max_total_tokens` one layer down, so the two are one arithmetic changed only together. Exact under containment, a floor under disjointness, plain `input + output` where no cache counter is reported — and it collapses back to a plain sum the day brazen normalizes the overlap in its decoders (brazen bl-d192), the named exit |
| 16a | Spend **attributed and priced** (per conversation, per ball) | #16's fold + each step's `request.json` model + `ui.json.prices` (§4.1) | the §3.5 join — Σ(usage over the agents tied to the ball) × the table, priced per step by that step's own model. Attribution is #8's goal stamps when any name the ball, else the workspace (the §3.5 ruling, labelled as such). Derived per ask, never stored; an empty table yields no cost at all. **The rates are per-slice, so the prompt is partitioned before it is priced** (bl-6621): a table with distinct `input` and `cache_read` rates describes disjoint slices by construction, so the cached read, the cache write and the uncached remainder — `max(input - (cache_read + cache_write), 0)` — are each billed at their own rate and the cached slice is never charged at both. The tokens priced sum to #16's fold exactly, which is what stops the dollar figure and the token figure telling different stories about one usage line |
| 17 | The config commit an agent **resolves** | git ancestry + the `config/*` heads | two chained pure git derivations, and the answer is the second one's (§9.4, bl-e654). The **governing** commit is unchanged — nearest ancestor of the agent tip reachable from any `config/*` ref (merge-base over config refs, litany ARCH §2.2), still unmoving, still `config_edit::branch::governing_config` — but it is now the *input*: `config_edit::branch::follow` takes the `config/*` heads whose history contains it and reduces them to their **distinct tips**. One distinct tip is **followed**, and control resolves from that tip at every step boundary (upstream bl-403b/bl-e580: fork settles the lineage, not the commit); two or more is divergence the derivation refuses to guess between, so the fork commit resolves **held**. `GoverningConfig` carries the resolved oid and a `Governance::{Follows(name), Held{diverged_lineages}}` saying which arm produced it, and `GoverningConfig::label()` is the one wording every seat prints. Held-ness is derived here, from git, and never read out of litany's stderr notice — see §9.4 |
| 18 | Config branches + contents | `repo.git` `config/*` refs | `for-each-ref` + `git show <ref>:<path>` |
| 19 | brazen file config | `<wall>/brazen/config.toml`, the focused workspace's own (§16.2's wall layout) | raw text. No workspace in focus is **no path at all** (`BrazenPaths::of` answers `None`) — the surface renders its guard rather than falling back to the machine's `$BRAZEN_CONFIG`/`$XDG_CONFIG_HOME`, which is the whole of the blast-radius ruling in one row |
| 20 | brazen effective config | `bz` itself | `bz --dump-config` stdout verbatim (bz is the authority on the value fold; yog never re-implements TOML semantics). *Since §16.7 W10 this is a **function call**, not a spawn — the linked `brazen` at yog's exact pin, driven through `src/bz_host.rs`; same bytes, same authority, no foreign binary* |
| 21 | brazen built-in rows | compiled into bz | static read-only hint beside the dump (which drops the defaults operand and so can never show them). *Since W10 the rows are additionally **listed**: `bz --list-providers` keeps that operand, so the login surface offers built-ins and file rows alike — one in-process read (§5.1 #20's sibling), and the source of the credential-presence rows too*. **When** it is read: at the login surface's RAM construction (bl-e290), so the pane opens with its roster already in it; `↻ providers + credentials` is the re-ask for the two things a start-up read can go stale against — a config edit made since, and a credential written since (bl-402f: it re-reads #22 in the same gesture). Never on first click — a surface that shows nothing until you refresh it reads as a surface with nothing in it |
| 22 | Credential presence | `<wall>/brazen/credentials/<provider>.json` existence — the focused workspace's own | existence only; contents never read, never written. Paired with #20's `auth` column it is the whole of what a surface may say about a provider, so the *words* are derived once (`brazen::row_views`). **One painted seat: the §8.3 Login rows** — bl-402f gave the §9.5 config pane a second copy of the same sentences and bl-20cb took it back, because the roster belongs to the surface that can act on a row and a verb-less duplicate of it is QUALITY H1's violation exactly. Read at the same gesture that asks #21, never per frame — and the §9.1 pane, painting no credential column, no longer reads it at all |
| 23 | Model cache | `<wall>/brazen/models/*.json` — the focused workspace's own | read-only display; refresh = `bz --list-models`, whose write lands in the same wall because the caller carries it — the picker's spawn in its env, the in-process runner in its `Env` (bl-dff8) |
| 24 | litany global config | `<config-root>/models.yaml`, `workflows/*.yaml` | raw text |
| 25 | Action history | `ops.jsonl` | tail + parse (§4.2); ambient error prominence is the §6 retirement projection over that tail — never a stored flag |
| 26 | A provider's offerable model roster | the provider, through `bz` | `bz --list-models --provider <row> --json`, fired **on every picker open** and never stored (§9.4) — and, since bl-dff8, on every `Query::Models` (§8.5), which is the same read in-process through the linked brazen. Distinct from #23: that row is brazen's on-disk cache rendered read-only; this one is the live answer the picker offers from |
| 27 | Role → (provider, model) per workspace | `providers.yaml` in a config commit | `git show <commit>:providers.yaml` (#18) + the §9.4 anchored-block read. Read at the **config-branch tip** for what the picker is about to change, and at the commit the agent **resolves** (#17) for what the open conversation actually runs under — which under follow-the-tip is the *same* commit in the ordinary case, and a different one only where the conversation is held on a divergence or follows another lineage. The model line still states both, because that difference is exactly the case worth painting (§11's bottom settings rows, §9.4's apart clause) |
| 28 | Conversation live-activity class {inference, tools, subagents} | #9 + #10, over the §2.3 subtree | `nav::convs::flight` — **inference** = any member `InFlight` (#9: the open `response.json` fd); **tools** = a *running* member holding a tool call whose `output.json` has not landed (#10); **subagents** = any non-root member holding a driver. The three overlap by construction and the operator's priority resolves them to ONE: **inference > tools > subagents** (§11). The classes are a query per tick and could not be a stored flag — yog never observes a start or an end, only disk at this tick. **Three seats read this one derivation** (the list row's pulsing name, the altitude-1 chip, the bottom in-flight strip, bl-905f); the **altitude-1 chip is the one that states the class in words** (bl-3f70 — the strip printed the identical sentence until that ruling, and the row always hovered it), and only the strip adds characteristics, and every one of them is a field of the same snapshot (`Agent::stream.text`'s length, `ToolCall::name`, a running-child count, and the two **structural starts** below) — never a second derivation. **Since bl-b768 the class is a *fold over #28b*, not a second reading of #9/#10:** `inference` = any member whose `Doing` is a model call, `tools` = any member whose `Doing` is `Tools` (the live-driver guard moved there with it). The priority is unchanged and is applied to those answers |
| 28b | What **one agent** is doing {waiting, thinking, inference, tools, idle} | #9 + #10 + the last content delta of the live `response.json` | `nav::convs::doing` — the finest live-activity fact, and the whole vocabulary of the §11 live mark (one circle per agent). Under an open response fd (#9) the **last content delta** splits the call three ways: none yet = *waiting on the API*, a `thinking_delta` = *thinking*, a `text_delta` = *inference*. Else a running member holding an unfinished tool call (#10) = *tools*; else *idle*. **Idle is not "stopped"** — a quiescent agent, a killed one and a circle with no agent in it all read the same, because whether a branch ended well is a different question with its own carriers (the §3.5 badge, the §6 marks); putting it on the same circle would be two facts on one carrier. The delta kind rides the snapshot as `Agent::last_delta`, off #10's one fold, and is consulted **only** while the agent is `InFlight` — so a settled step's trailing delta is never read and no expiry rule is needed. #28 folds over this; nothing folds the other way |
| 28a | When the call in flight **began** — the strip's elapsed (bl-9dfb, amending bl-905f) | the world record that opens the call: `steps/<id>/<NNN>/request.json` for a model call, `…/tools/<tool-id>/input.json` for a tool call | `Agent::call_start_unix` / `ToolCall::start_unix`, both stamped at enumerate time beside the presence checks they ride with (bl-cad5's rule). Each file is written **once, immediately before the call it opens, and never rewritten**, so its mtime is the call's *start*, not its last sign of life — verified against the pinned litany: both drivers (`run_exchange`'s loop and the `litany advance` hop) land `request.json`, then take the very timestamp they will later write as `meta.json`'s `started_at`, then invoke the adapter; the tool executor writes `input.json` atomically, then takes `output.json`'s `started_at`, then spawns. `meta.json`/`output.json` cannot serve: each lands only *after* its call returns, so it is absent for exactly the call being timed. Nothing under `steps/` is git-tracked (§2.3), so neither file has a commit timestamp to prefer. **Elapsed = now − start, per frame, nothing stored** — the wall clock is minted at the shell boundary (`shell::now_unix`, the same one the §11 list's ages use) and the label is `age_label`'s, so the two seats cannot drift |
| 29 | Operable-commit rules — the persistent faint lines through the chat, one per step, each the gesture that pins it (bl-929d, re-seated bl-1802) | #12's transcript entries + `steps/<id>/NNN/meta.json` `commit` and `response.json` framing (#13) | `rail::place`, folded onto #30's notches. **Position pairs a model-output entry to a completed step, one for one** — a call that reaches `Finish` seals `messages/NNN-<model-id>.json` and one that does not seals nothing (litany ARCH §2.3), so the transcript's `Model` entries are the `Framing::Complete` steps in step order. Each step's rule sits at the first entry it read that its predecessor had not (its predecessor's tool results, then the boundary drain's deliveries — ARCH §2.11 orders the drain after the tool entries), and the index of its own output is #31's cut. **This replaces the ordinal alignment bl-929d shipped**, which paired the i-th delivered run with the i-th step: a litany step is one model call and a tool loop is many steps behind one drain, so every rule after the first tool-using turn carried the wrong commit. Absence of a commit = no line; a call that sealed nothing takes a seat only as the last step (its read state is the tail of the chat). Derived per snapshot, never stored |
| 30 | The **step spine** — one notch per step, its two edge kinds, its seat in the chat, and each dispatched child's card (VISION V1, bl-98da; bl-1802) | #13's `meta.json` `commit` (the notch), #8's `Agent::steps` (both edges), #8/#9/#10 (the card's identity, state and streaming tail), #16 folded per agent (its spend) | `rail::build`, a **pure** fold with no git call of its own, over #13, #8 and #12's entries. Each notch carries its chat seat (#29's `rail::place`) — the row its rule paints above and the cut its pin reads to, so the spine and the transcript's rules are one derivation rather than two. The notch spine is #29's read — the Steps view's `meta.commit` in step order, reused not re-derived, so the spine and the transcript's boundary rules can never disagree about what a step read. The **two edges** (VISION V1.3) come off one fact, and since bl-1802 the context edge is spent on the fork label's wording rather than kept as an index — a chat rule has no gutter to stroke, and the label states in words what the strokes drew (`from here` / `from <Name>@<oid>` vs `from config/<name>`); a drawn descent graph was declined (bl-5cf8 — the words are the rendering; see #8): `Agent::steps` is `git log --first-parent <branch> --not --branches=config/*`, so a *fork* child's list opens with its parent's commits up to the fork point and a *clean* child's shares nothing with it — the longest common prefix **is** the fork point (the *context* edge, git ancestry) and its emptiness **is** cleanness. The *provenance* edge is located for both kinds alike: the last notch whose commit is no later than the child's own first commit, which is the rule the card hangs under. §11's descent tree stays the descent-**id** tree (#8); this is a label on a card, never a second membership tree. Derived per snapshot, stored nowhere. The old `Rail::navigable` gate is gone with the gutter it kept from claiming width: the chat's rules are the ones bl-929d already drew, so an operator who never clicks one sees today's transcript exactly — the burden check, with nothing to gate |
| 31 | The inspector **as of** a notch — transcript, agent-context files, the config, budget (VISION V1.2) | #30's pinned commit + #12 / `ls-tree` at that commit / #17 / #16 | one commit, four reads, no new mechanism per tab (the shape STORIES §S7 point 3 named when it declined a per-tab checkbox). **Transcript** is a *prefix* of #12, cut at the notch's own `Place::cut` (#29) — everything ahead of that call's own model output. **What is exact and what is not** (amended bl-7bd2 — the original claim here was "`messages/` entries are append-only under a monotonic counter", and compaction falsifies it: litany's compactor DELETES from that directory, #12). What survives is exact: a `messages/NNN-*` file is never rewritten, so every entry the prefix shows **is** the pinned tree's bytes, and the Raw toggle keeps showing verbatim bytes with no `git show` per message. What is not exact is the prefix's **extent** — entries the pinned tree held may since have been squashed away, so the cut lands short of where that call really read to. This is not coded around: the compaction markers #12 splices into the listing make the difference **visible** in the pinned view instead of silent, so a prefix that crosses one is a record rewritten after the pin was minted. A cut is never wrong about an entry, only about how many there were. **Files** is the one new disk read: `git ls-tree -r -l <commit>` for the listing (blobs with their sizes as of then — a `--name-only` listing could not state a size, and a zero would be a lie) and `git show <commit>:<path>` for one file. **Config** is #17 asked with the pinned commit as the agent tip instead of the live one. **This tab is the one place the follow-the-tip ruling cost something, and it says so rather than implying otherwise** (§9.4, bl-e654): #17 now resolves a *followed* commit, and following is a fact about **now**, so the tab answers *what governs this conversation today* and cannot answer *what governed step N*. Under the freeze those were one question; they are two now, and yog derives only the one it can. What is exact at the notch is the policy's **effects** — the step's own `request.json` (#13) holds the model id, soul, tools and retry it actually ran under — and the missing document is filed upstream rather than worked around (litany bl-e4a0: the resolved config commit recorded in each step's `meta.json`), at which point this tab gains a second, honest reading with no new mechanism. Both are answered at the boundary now (`Query::Files`'s `at`, bl-44e9; `Query::Governing`'s, bl-13f9) rather than derived by the frame and memoized: the commit rides the query as a **selection** — which tree, which commit — so the fold itself never crosses and §8.5's *views gain no boundary representation* holds of both. Of the four, these are the two whose subject really is a different tree; the transcript and the budget are folds over answers the seat already holds. **Budget as-of** is the notch spine's own per-step tokens summed through the pin. Nothing here is stored; the selection itself is §5.3 viewport ephemera. **The release** is the pin banner, which paints above every pinnable tab: the gesture that raises the pin is a rule in the chat, so the way back has to be reachable from the tabs that have no chat (§11) |
| 32 | The **project work-diff** — what a workspace's delivery attempts have actually changed (VISION §4.10, bl-3746) | #2's live balls (the §3.2 claimant binding, the ball graph) + the project repo's own refs | a pure git read of the project repo, `target..source`, spelled exactly as the §4.10 ruling spells it. The **source** is the claim's branch, `work/<id>` — balls' own `delivery_path::work_branch`, never a literal here. The **target** is balls' own `target::derive` re-run over facts the snapshot already carries: the parent ball's work branch when this ball close-gates a *live* parent (`parent = X` AND `X` carries `{this, on: close}`), else the project's integration branch — `git symbolic-ref --short HEAD`, again balls' own spelling, so yog and `bl close` can never name two targets. The listing is `git diff --numstat` (counts, not bytes) bounded at the Files tab's own cap; one file's patch is `git diff <range> -- <path>` read only when picked, classified by `files_view::classify` — the one "what this file is" vocabulary. **Three declines, never one silent empty listing:** a project repo that names no branch is *unreadable*, a ref that does not resolve is *absent* naming which end, and a resolved pair with nothing between them is an empty diff. A workspace holding two balls has two attempts and both are shown — there is no rule that picks one. Nothing stored, no index, no verb spent; memoized per snapshot by the seat that asks (§7.2), never per frame |

| 33 | A **cohort** — the candidates dispatched from one notch, and the ancestry they share (VISION V2.3, bl-dc0c) | #30's cards, grouped | `rail::cohort::cohorts`, a **pure** grouping of #30 by `provenance_notch` — the birth notch V1.7 reserved as the fan anchor. **Membership is not a fact anybody records.** Firing the fork twice from one mark *is* firing a cohort and firing it once *is* a cohort of one, so there is no fan registry, no fan verb and no winner field to keep: the group is a `group_by` over cards yog already derived. The **common ancestry** is the fork label when every member wears the same one and nothing when they differ — the same fact said once at whichever level owns it, and absence is a value (the columns then say their own). The four side-by-side facts a candidate is judged by need no new derivation either: state is #8's, usage is #16 folded per agent, and the **terminal response** is #10 — `Agent::stream.text` is the latest step's accumulated text re-read every tick, so while a candidate runs it is the live tail and once it settles the same bytes are the last thing it said. A second "terminal response" reader would be two readers of one open file, disagreeing at every moment it matters |
| 34 | The **fire-time policy** a pinned notch may fork into — the fork points, and the model each role names at each of them (VISION V2.2, bl-dc0c) | #18's config branches + the pinned commit; #17 asked at each; `providers.yaml` from that commit's tree | `fork::choices`. The points are **here** (the pinned commit — a fork carrying the conversation's own history) then every `config/<name>` head (a clean start): one control with two kinds of value, which is VISION V1.3's *"one spawn gesture with one parameter — the fork point"*. Each point's roles are read from the `providers.yaml` of the config commit the fork will **resolve** — #17's followed answer, the very file litany resolves the run against — through §9.4's own `grammar::roles`, so the picker and the fork composer can never disagree about what a config says. Under follow-the-tip (§9.4) that reading is *more* durable than it was, not less: a clean start off `config/<name>` follows that lineage, and a fork from the pinned commit follows the lineage the pinned conversation already resolves, so the roles the composer offers are the roles that will still be in force at the candidate's tenth step and not only at its first. **That is what makes the model visible without yog owning a model list**: a role *is* a model binding, and giving an attempt a model no config declares is a config write (§9.4's `PickModel`), not a dispatch flag. A ref whose config yog cannot reach declares no roles and the point paints as offering nothing — a fact about the workspace, never a silence. Memoized per snapshot with the pinned commit in the key (it is `for-each-ref` + a `merge-base` walk + a `git show` per point), never per frame |
| 35 | **How full a conversation's context is** — the percentage §11's settings rows state per chat (bl-a48b: the context-window percentage is shown per chat) | the **last `Usage` line** of the root agent's **latest** `steps/<root>/<NNN>/response.json`, the model that step's `request.json` names (#16's walk), and the `context_window` §9.2's global `models.yaml` declares for it (#24) | `context::of_conversation`, a pure filter over `Snapshot::bills` — no second disk pass, no stored counter, nothing cached. **Fullness is not spend, and the difference is load-bearing:** #16 folds every attempt of every step of a whole descent (what exhausts `max_total_tokens`, and what keeps growing after a compaction empties the context), while this is one number off ONE step — so the walk carries two extra columns, the step's own `seq` (making "which is the latest" an in-memory question, exactly as bl-9dd4 made scope one) and its **last** attempt segment's counters (a step retried three times must not read as a context three times its size). **The root's own latest step, never the descent's** — a dispatched child runs its own context in its own tree. **The prompt is `max(input, cache_read + cache_write)`, and since bl-6621 it is one formula with one home** (`BudgetSpend::prompt_tokens`) that #16's spend fold reads as well — this row stated the rule while the spend fold summed all four counters beside it, and that divergence was the double-count. brazen's canonical `Usage` is deliberately unnormalized about overlap because its providers disagree — Anthropic reports the three as **disjoint** slices of one prompt while OpenAI's `prompt_tokens` and Google's `promptTokenCount` already **contain** the cached slice beside them — so summing over-states one shape and taking `input_tokens` alone under-states the other by nearly everything (brazen marks Anthropic prompts for caching unconditionally). The maximum is exact where the slice is contained, degrades to plain `input_tokens` where no cache counters are reported at all, and is a **floor** where they are disjoint. It never over-states; normalizing that overlap is brazen's job, not yog's to guess at. **The denominator is the declaration, and since bl-d9cb yog is the only program that reads it:** lernie 0.0.10 retired the `models:` table (litany's bl-35e2), so the block is yog's own hand-configuration — authored by §9.2's Declare control, edited by the §9.5 form, read here and nowhere else. One home, one number, operator-correctable — and since bl-3ffa the entry is that number and nothing else, the block's `provider`/`capabilities` columns having had no reader anywhere (§9.2). brazen serves `Model.context_window` only for the providers that publish one (Google), which is its own empty-set rule — *"a harness hand-configures only what no provider serves"* — and reading brazen's cache **as well** would be two representations of one fact. bl-848f had answered that by moving the discovery to the WRITE path, seeding the declaration from the roster a pick was made off; bl-d9cb deleted the pick's write entirely (it landed in a table nothing loads), and the seed went with it rather than being relocated. The number is the provider's fact and it moves without yog's involvement, so a field seeded at pick time is a stale snapshot; if this figure should ever prefer a served window, the shape is a read-time query over `model_cache_at`, not a write. Today: whatever the operator declared, or no figure. **No window, no figure** (no step, no model on the step, an undeclared or zero window): the row is absent, never a percentage of a default — and since bl-d9cb an undeclared window is the ordinary state of a fresh world rather than a corner, because no gesture seeds one — the same no-capability-theater rule §3.5's unpriced remainder keeps. The windows ride the snapshot (`Snapshot::windows`), read at boot and on the 15 s full sweep like the ball fetch: one hand-edited world-global file, so a fifth watch root would buy latency nobody can perceive |
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
are swept: I3 temps (`.<name>.yog-tmp-<pid>` in the destination dir — `ui.json`'s
directory, the §9.2 config root and its `workflows/`, and each wall's three
brazen destinations) and the scripted-editor staging directory
`$XDG_STATE_HOME/yog/stage/<nonce>/` (§9.3). Neither is an authority; leftovers
>24 h old are swept at startup — both halves off one clock read in
`Engine::boot`, the temps by `scratch::sweep` over `scratch::dirs` and the
staging dirs by `config_edit::branch::edit::sweep_staging`. The sweep takes only
a regular file whose name yog itself would have written, directly in one of
those directories: never a directory, never a symlink, never a walk.

One more *class* of yog-owned file exists and is deliberately **not state**: the
world's tool shims, `<yog-data-root>/world/tools/<name>`, one per
`world::tools::ROSTER` entry (§16.7 W9's `bl`, extended to all three agent tools
at W11, to balls' two sibling plugin binaries at bl-2930, to the §8.6 capability
control at bl-fec8, and to `yog` itself at bl-3ff4). **The roster is the
authority and the set is not restated here** — this line said "five" and named
five while the tree seeded seven, which is what a second copy of a list buys.
Each is a *generated artifact* — a pure function
of the `Cli` yog resolves that tool through — re-derived and rewritten on any
drift at every start and every hatch (bl-44a5), holding no fact yog cannot
recompute.
Deleting it loses nothing; that is the test this row asserts, and why it is
listed here rather than beside `ui.json`.

### 5.3 Legitimately RAM (the closed whitelist, I6)

| Item | Why RAM is legitimate |
|---|---|
| Unsubmitted input text (prompt/message bars, claim-dialog fields, the goal composer, config editor buffers before Apply, the two §3.6 delete confirmations' open target + typed name — the agent dialog also holds the dry-run census it fetched at open (bl-f17a), a derivation cache the fire never trusts: the argv is re-armed from the typed name and the subtree re-cut by the verb itself) | the requirement's explicit carve-out: "text typed in a box can live in RAM until sent". RAM, but **per target**, not per box (§11, bl-a69a): the docked composer keys its drafts by what it is pointed at, so switching the selection switches drafts rather than re-addressing one. **The target of a config editor buffer is the workspace wall that holds the file** (§16.2 as amended, bl-5894): brazen's `config.toml` is a workspace's, so its draft is keyed by wall and survives A → B → A, while the litany-global and cadence files are the world's and keep exactly one draft each. Re-loading one box on focus change is not per-target keying — it is the box relabelled, and it discarded the draft |
| **Live focus/selection and scroll position** — including **which tab each of the two strips shows**: the altitude-2 inspector tab, and the §11 center tab (`keymap::CenterTab`, bl-1ca2) | per-instance viewport ephemera — *which data you look at*, not data; loses nothing on crash; re-derives at startup (§4.1). Deliberate interpretation, §13.1. Scroll is *represented* as content anchors (topmost visible message ordinal / step number), never pixels, so the viewport stays stable across live re-derivation of the tree beneath it |
| Subprocess handles, drain threads, `Stream`s, the detached reaper threads | a process is not data; the fact each represents lives on disk (driver running = flock held; op outcome = `ops.jsonl` line + substrate state). Long-lived drivers are spawned fully detached (§8.1) so yog's death cannot kill or starve them — the handle a detached spawn keeps lives in its reaper thread, never in app state, and carries no fact (its whole job is to take the status and drop it) |
| Watcher registry, notify channels, dirty flags | reconstructible plumbing; the sweep (I4) makes their loss harmless |
| Memoized derived snapshots (`HashMap<PathBuf, GitTree>`, ball lists, parsed transcripts) | caches of §5.1 facts; discarded and rebuilt at will |
| Live window geometry, egui layout/font caches, GPU state | instance-physical, not data (§13.0: window arrangement is a view, not data). **The panel proportions inside that window are not this row** — where the operator dragged a boundary is an assertion with no other home, kept durable in `ui.json.panels` (§4.1) on the same terms as `zoom`; the window's own size and position stay the desktop's business, and yog neither reads nor writes them |
| Probe result TTL cache on macOS (§10) | a cache of an observation with a 2 s bound |
| The model picker's open/closed flag, its selected role, the half-made pick beneath it, and the in-flight `bz --list-models` run with the roster it produced (§9.4) — **all of it per workspace wall** (bl-5894) | the run is a live subprocess (the row above); the roster is #26 — *a query's answer, held only as long as the surface that asked*. Storing it would make yog a second authority on a set the provider owns, and the picker re-asks on every open by design. The wall scoping is the blast-radius ruling read onto RAM: a roster listed against workspace A's providers, and a role/provider/model chosen from it, may not paint or be clicked under B — a pick there would write B's config lineage from A's candidate set |
| Live streamed-verb output (§8's streamed-piped class: the `bz --login` sign-in lines, off stderr — §8.3 as amended), **held with the wall it was fired in** (bl-5894) | instance-local by nature (a device code is for the human at *this* keyboard); it converges to its `ops.jsonl` outcome line at exit, so the other instance renders the durable fact from the pane, never diverges. **The last failure is *not* here** — it was a RAM item until bl-4895 proved a cached copy cannot be right: the banner is derived from the ops tail every frame (§7.3), so it has no RAM home to lose. **Its attribution is on the row too, never in RAM** (bl-48f8): which surface a failure belongs to is a fact of the durable line's `origin` (§4.2), so it survives a restart, reads the same in both instances, and cannot be lost with the frame that dispatched it. **Parked, never dropped, when focus leaves its workspace** (bl-5894): the run is writing *that* sphere's credential, so it may not paint under another — and dropping it would SIGTERM a sign-in the operator is halfway through, which is why the wall owns the holder rather than a focus change clearing it |
| What this window has already told the desktop (§6 as amended, bl-e160): the last observed alert set | a desktop belongs to a **window**, so two instances each announce their own and neither should converge — the §13.1 viewport-ephemera argument carried one step out of the frame. Losing it loses nothing: a restart is a new window, and a new window announces nothing it merely inherited (its first fold is its baseline), which is the same rule that keeps a fresh launch from flooding. Nothing here is a fact — the facts are the `refs/litany/*` marks and the `ui.json` watermarks that decide the queue it is a difference against |
| The §3.4 start claim and the **pending echo** it carries (§7.2, bl-915e): the workspace, the target (a minted §3.3 name or an agent id), the operator's text, and the landed-message baseline it reconciles against | the unsent-input row above, one instant later: a message yog has *sent* and the driver has not yet flushed is still only this instance's word for it, and the moment disk says so the derivation says it instead. Losing it loses nothing — a restart re-derives from disk, which is the same convergence a landed message already has. It is deliberately not durable and deliberately not in `Snapshot`: writing it down would make yog a second authority on a fact litany owns, and two authorities on one message is exactly the drift §7.2 measures |
| The §3.4 **raise claim** (§7.2, REMOTE §9.7, bl-7407): the wall a landed start founded, until the derivation enumerates it | the row above it one noun up, and held for its reason exactly — the focus names a workspace by its §3.1 name, and yog has made a wall the worker has not read. It is the *same* argument as the pending echo's: yog's own word for a thing it just caused, retired by the derivation showing it, never written down because the enumeration is disk's to answer. Losing it costs one derivation's worth of a focus that resolves to nothing, which is what a restart already is |
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

1. `refs/litany/notify/<id>` exists and its oid ≠ `seen[ws][agent].notify`.
2. State **at rest** (Quiescent or Stopped), no `refs/litany/abandoned/<id>`,
   and tip oid ≠ `seen[ws][agent].stopped` — a conversation waiting at a tip you
   have not seen. Rest is the general condition; a stop is only the wounded way
   of coming to rest. The clean end and the failed end differ in the state
   badge, never in whether your turn has come *(widened bl-2194)*. Which
   way it came to rest is the *word* the signal wears, not a second signal:
   a rest whose latest response was refused at the provider rung says `refused`
   rather than `stopped` *(bl-b43b, below; the refusal is read off the failure
   sentence rather than a flag since bl-9b88)*.
3. `refs/litany/budget-exhausted/<branch>` oid ≠ `seen[ws][agent].budget`.
4. `refs/litany/conflicted/<id>` oid ≠ `seen[ws][agent].conflicted`.
5. Pending inbox > 0 **and** lock Free — mail nobody is driving (the
   writer/driver stall case). **Not seen-gated**: it is actionable (flush via
   `litany scan`), self-clears when a driver picks it up, and hiding it would
   hide a stall.
6. `refs/litany/held/<id>` exists — the capability control parked a tool
   invocation before it executed (§8.6, bl-765d). **Not seen-gated**, for rule
   5's reason carried one step further: a park costs the drone no process and
   no tokens and *nothing but an answer releases it*, so a watermark could only
   ever hide a conversation that cannot move. It self-clears when litany lifts
   the mark, which happens exactly when the answer's re-adjudication runs.
7. A **flag** was raised on the conversation and the raising row's `ts` ≠
   `seen[ws][agent].flag` *(VISION §4.9, bl-6f2f)*. `/flag <why…>` is the
   signal-out verb and the alignment monitor's **floor grant** — a responder
   granted only `flag` is a pure judge — and it wrote its one exit-0 ops row
   into a trail this predicate did not read. So the verb whose entire purpose
   is letting a machine raise its hand raised it where nobody looks: VISION
   §4.9's ladder table, `Reply::Flagged`'s doc and the gesture's own help all
   promised *"attention item + ops row"*, and only the row existed.

   Seen-gated like 1–4, with the **row's timestamp standing where an oid
   stands**: nothing about a watermark needs a ref, only a value that changes
   when the fact does, and a later flag is a later stamp. Nothing is stored —
   the row is the home and the signal is a query over it
   (`monitor::flag::latest`).

   **The fact rides the agent, not a seventh parameter.** `attention` and
   `evidence` both take `&Agent` and nothing else, and that is the shape of a
   signal rather than an accident of one: a rule reached by an extra argument
   would have to be threaded through the rank sort, both rollups, the roster
   walk and every caller of each, and the acknowledgement path could not follow
   it at all. So `Agent::flagged` is stamped by `monitor::flag::fold` at the one
   place the ops trail and the derived trees are both final — the snapshot's
   publish — which makes it the single field on that type the workspace does
   not answer, said out loud where it is declared.

   The **reason** rides the §6 queue row beside the word, on `held`'s
   precedent: a queue row exists to be answerable without a second read, and a
   signal that says *look at this* and cannot say why sends the operator
   hunting through `/ops` — which is where the flag already was.

Signals 1–4 and 7 are seen-gated on the `ui.json` watermarks (§4.1); focusing an
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
badge; rule 5's is the `✉n` accessory and the Inbox tab; rule 7's is the flag's
own `ops.jsonl` row, which `/ops` renders whether or not it was acknowledged;
rules 1, 3, 4 and 6 are the agent's `refs/litany/*` **marks**. The marks are one closed set
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
badge, the `✉n` accessory, the `refs/litany/*` marks — and a row that says the
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

*The baseline is per window, and it advances on every frame that was told
something.* What has been announced is §5.3 RAM (a desktop belongs to a window;
two instances each own theirs and neither converges), and the fold runs
regardless of focus and regardless of the knob — with only the announcing gated.
That is what stops a burst of stale news the moment a window loses focus or the
knob is switched on. It also dissolves the first-boot flood one level up: a
window that has just opened has witnessed no *arrival*, so its first fold is the
baseline and says nothing — the general path with no prior observation, not a
first-run branch.

**Since bl-f297 the queue crosses the wire** (REMOTE §9.7): the window asks
`Query::Attention` on the asker's standing set like every other migrated read,
so the count the strip paints, the ask the desktop names and the list a headless
`/attention` hands back are one answer rather than one derivation run twice.
What that changed is the word *unconditionally*: **an unanswered frame is not a
reading of the queue**, and neither is a refusal. A frame the wire has not
answered holds no queue at all, so folding one would read as everything having
departed and then, on the next answer, as everything arriving at once — the
first-boot flood, re-armed twice a second. The baseline therefore moves only on
a frame that was told something, which is the *same* rule that makes a freshly
opened window silent: no observation, no arrival. A refusal is the engine
declining to say, and this seat has no surface to paint that on (a notification
is output, not a pane), so the window stays quiet rather than announcing on a
guess. The cost, stated: the fold runs at the asker's cadence rather than the
frame's, and the focus gate is read on the frame the answer lands — a window
buried and re-focused inside one ask period folds once instead of thirty times,
which is a difference no difference detector can feel.

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
tokens — binary plus subcommand (`bl close`, `litany prime`, `yog-step mint`) —
because the argv tail carries per-run operands (a ball id, a composed goal) that
never repeat, so keying on the whole argv would retire nothing; `cwd` scopes it,
so a clean `bl close` in one project leaves a failed one in another alone.
Success is the pane's own failure classifier negated (`OpRow::failed`), so no
second definition of success can drift from the one it paints. A retired failure
keeps its row and its ⚠ in the expanded accessory — it loses only ichor and the
chip's count. **Absence of a live failure is the record; the log is the
history.** The wound this closes: a three-day-old `litany prime` failure, since
fixed and re-run green, read as THE error when an unrelated action failed,
sending diagnosis down a false trail.

A conversation whose latest step **failed** stirs the strip through rule 2: a
failed or killed latest `response.json` classifies the agent Stopped
(§4.4/§3.5) — an auth-failed step included — so an unseen dead conversation is
never "nothing stirs". Acknowledging it clears the *signal*, not the fact: the
conversation list's state badge and the §11 Login affordance keep rendering
the settled failure (the badge is state, not attention).

**And since bl-b43b it stirs in its own word.** The clause above was true and
insufficient: an auth-failed step classifies Stopped, and `stopped` is the word
an operator's own `/stop` earns — so a conversation that was refused at its
first model call told the operator they had done a thing they had not done, on
a plain row, with an empty transcript, while the remedy (sign a provider in)
was named on no surface they were looking at. The fact was computed, stored and
answered correctly, and reached exactly one surface: the per-step `auth_failed`
/ `auth_row` pair, behind a second query nobody opens on a conversation that
looks stopped. That is §11's own reading rule failing on itself — *"anything
this document says yog paints is a fact it answers"* — with this section as the
document.

The remedy is a **word, not a badge and not a rule**. Three things stay exactly
where they are: the badge set is frozen at four (§5.1 #9, bl-d816), the rule set
does not grow, and the count does not move — a refused conversation fires rule 2
once, as any rest does. What changes is *which way* it is said to be waiting,
which §6 has always made the row's job:

- **`AttentionKind::Refused` stands where `Stopped` would**, never beside it.
  One firing, said in the word that is true of it, with its sentence in the same
  one home every other rule's lives in (`AttentionKind::says`) — so the bl-e160
  desktop escalation names the act rather than reporting a stop nobody made.
- **The fact rides beside the state on the agent**, exactly as bl-fb87's
  truncation reading does and off the same read: the §3.5 classifier already
  reads the latest `response.json` once, and the refusal is one more reading of
  those bytes rather than a second syscall (§5.1 #10). Every derivation pure
  over the view-model gets it for nothing — the queue's `signals`, the roster
  row, the `agent` answer.
- **The roster row wears it as a hue** (`Tone::Bad`), which is the operator's
  one *passive* sighting. The hue is never the explanation (§11's glyph
  doctrine): the word is the signal above, and the provider **row** that refused
  — the credential to sign in — stays the steps surface's `auth_row`, one fact
  in one home, one query deeper on the conversation the operator opens.

What is deliberately NOT done: the conversation's transcript still paints
nothing for a refused conversation. Seating a virtual notice entry there is the
`Compacted`/`Streaming` move a third time and is worth doing, but it is a new
`EntryKind` and its own ball rather than a rider on this one.

**The half that never reaches the contract, and the words nobody carried
(bl-9b88).** bl-b43b read a refusal out of the settled tail's `error` event —
the in-band half, which is what brazen writes when it reached the provider and
the provider said no. A **credential-less provider row does not get that far**:
the adapter dies at startup, litany lands an *empty* `response.json` beside its
own `stderr.log` (litany ARCH §2.3: *"Empty on an ordinary run … bytes here mean
the adapter failed outside that contract"*), and the settled reading is `Killed`
— indistinguishable, to bl-b43b's predicate, from a driver someone killed. That
is the branch a live deployment actually hit: a workspace whose role pointed at
a credential-less row launched a driver per conversation, every model call
refused, every seat painted a list of conversations that simply never answer,
and the exact failure sentence sat in files no roster reads.

Three corrections, and each dissolves a case rather than adding one:

- **The fact is the sentence, not the flag.** `Agent::failure` is *why the
  latest model call failed*, verbatim, and `Agent::refused()` is the auth-shaped
  **reading** of it — a query, never a second stored field, so the fact and the
  reading of it cannot disagree. §6's `refused` signal and its remedy are
  unchanged; they now fire for a startup failure that says `credential` as
  readily as for a 401, because both are one sentence asked one question.
- **The framing decides which file is read**, so nothing is a special case and
  a healthy conversation pays nothing: `Failed` spends bytes already in hand,
  `Killed` opens the step's `stderr.log`, `Complete` opens nothing. The §7.3
  wound's `meta.json` observation is not repeated here — the wound must tell a
  settled step from an unsettled one for a *per-step* badge, and the framing has
  already answered that for the agent's latest call.
- **`Tone::Bad` widens from the refusal to the failure.** bl-b43b painted the
  auth-shaped subset; a transport reset, a malformed config and a dead adapter
  are the same sighting to a scanning operator — *this one did not run* — and a
  roster that reddens for one of them and not the others is a roster that
  teaches the wrong rule. The badge set is still frozen at four and the signal
  set at six.

And the words now ride the rows that carry the hue: the §11 conversation row and
the §6 queue row each carry the failure's **first clause** (`git_tree::clause` —
the provider's `message` when the evidence is an event, else the evidence, first
line, capped), so a roster of red rows says what is wrong with them instead of
asking the operator to open each one. The whole of it — the adapter's captured
stderr, the provider row to sign in to — stays the steps surface's, one query
deeper. Gaining that field on three shapes is a wire change, so it bumps
`PROTOCOL` to 3 (REMOTE §9.9's rule, mechanised by the corpus ledger).

**The prompt that never became a conversation (bl-a649).** Rules 1–5 are all
per-*agent*, so they can only stir once a conversation root exists. A detached
`litany prompt` that dies before writing one — a tool version-skew refusal at
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
different drivers and only the step's copy exists for a `litany message` turn.

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
| WorkspacesRoot | `<litany-data>/workspaces/`, `replays/` (top) | dir create/remove |
| BallsClones | `$XDG_STATE_HOME/balls/clones/` | clone dir create/remove; per-clone `tasks/tasks/*.md` and `config/config/**`; the per-clone `log` (multi-MB, no rotation) is **filtered out** to avoid event storms |
| YogState | `$XDG_STATE_HOME/yog/` | `ui.json`, `ops.jsonl`, `cadence.yaml` (the `detached/` sinks are **not** watched: a chattering driver would storm the watch, and the 15 s sweep's re-read of the ops tail folds them in anyway, §8.1) |

`packed-refs` is not decoration. yog reads refs through `git for-each-ref`,
which reads the loose tree **and** the packed file. `git gc` runs `git
pack-refs`, which empties `repo.git/refs/` into `repo.git/packed-refs`; deleting
a ref that is only packed then rewrites `packed-refs` **alone**, touching nothing
under `repo.git/refs/`. Without the entry that deletion is invisible to the
watcher and reaches yog only via the 15 s sweep — a reproducible dropped event
(bl-49f4; proven in `src/fs_watcher/tests/drift/`).

Rejected: one recursive watcher over the whole litany data root — an agent
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
3. **`LitanyConfig` cannot be armed as specified without becoming the watcher
   this section already rejects.** §16.2 collapses `LITANY_HOME` onto
   `world/litany`, and `litany_config_root()` and `litany_data_root()` both
   return it — so the "config root" *is* the litany data root, `workspaces/` and
   `replays/` and every agent worktree included. `Watcher` arms
   `RecursiveMode::Recursive`, so that root is exactly the "one recursive
   watcher over the whole litany data root" rejected two paragraphs up.

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
| **follow lane** (`wire::lane::Lane`, §7.2 live tail; bl-73e7) | its own seat on the wire and its end of the frame's hand-off | holds one connection open on the **focused** conversation's live tail and lands each frame the engine writes, waking the face — its ONLY thread, since the fold itself is the engine's (`boundary::follow`, one open `response.json`, a byte offset and the fold so far, on the connection thread that answers the held read). It was a follower thread in the window until bl-73e7, publishing into a `TailCell` the frame painted from; the remote split left that fold with no reader, so the mechanism moved to the engine and the carve-out retired with it |

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
- **The frame renders the derivation plus exactly three non-derived facts: the
  operator's own last send, the focused conversation's live tail, and the wall a
  start just raised.** One function folds all three onto the snapshot a frame
  paints (`echo::compose`), so "what does a frame see that disk does not say?"
  has one answer to read and a fourth such fact would be a fourth argument there
  rather than a fourth mechanism. The first is below; the second is **The live
  tail** further down; the third is the §3.4 **raise claim** (`src/app/raise.rs`,
  REMOTE §9.7 class 2, bl-7407) — added as exactly that third argument, which is
  the sentence above being taken at its word rather than amended.

  The raise claim exists because focus is a §3.1 **name** now (the wire
  spelling), and a name resolves against the enumerated workspaces: `litany new`
  returns before the derivation has read the wall, so for one pass the focus
  names a workspace no set carries. The claim holds it — same holder as the
  start claim below it, same retirement predicate (*the derivation shows it*),
  one noun up — and because it is folded rather than resolved per reader, every
  door reads one enumerated set and none of them knows a claim exists. It is
  retired **before** the fold runs, so the painted set can never carry one
  workspace twice.

  **The engine does not hold this claim, and must not** (bl-6c9e). The same
  ordering ran backwards at the *boundary* — where it refused gestures rather
  than mis-aiming a focus — and the answer there is the opposite one: the
  intake **asks disk** for the §3.1 enumeration per gesture (§8.5,
  `app::addressable`), because it is off-frame and the authority is three
  readdirs. So the two are not two mechanisms for one fact. A frame does no IO,
  which is exactly why it needs an optimistic claim; an intake does, which is
  why it needs none, and why a gesture is still decided by *the derivation,
  never the fold*.

  **A migrated surface reads rows, not a snapshot, so the echo has a second
  projection** (REMOTE §9.7, bl-44e9): `echo::rows::with_echo` folds the same
  value onto an answered §11 conversation list. Two projections of one fact is
  not two sources — both live in `src/app/echo*`, so the sentence above still has
  one place to read — and the ruling behind it is that **a seat's optimism reaches
  whatever that seat actually reads**. Without it, migrating the §11 list would
  have deleted §3.4 from the one surface it exists for. As more surfaces migrate
  the snapshot projection shrinks toward nothing; neither is the other's
  fallback.
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
    - **The queue seat asks about a name exactly as it asks about an id**
      (bl-56c6). The composer aims at whatever the §11 selection is, and since
      bl-2e8f a fired start makes its minted name that selection — so the seat
      asked `Query::Inbox` about the name, was refused, and painted an **empty
      queue for the whole start window**: the operator's first message with no
      representation at the one seat this bullet names as the seat they meant.
      The echo declined the fold there on the premise that *"a start has no id
      yet, so no seat is asking"*, which bl-2e8f had already retired. A name and
      an id are two spellings of one target; the seat folds on whichever it
      holds, and what it folds is **every** send the echo stands for — the one
      that made it and each held follow-up, oldest first.
  - **The reconciliation key is the landed message count, never the text.**
    Every echo records its target's `NNN` counter high-water when it was made
    (§5.1 #12 — messages ever landed, never files present, so a compaction
    landing mid-flight cannot move the figure down and wedge the echo,
    bl-fde5) — zero for a start, whose root does not exist yet — and
    is superseded on the first derivation whose count exceeds that baseline: the
    file the echo stood in for has landed. Matching on message text is fragile,
    and every cheaper proxy is a conflated predicate of the kind §7.2's own
    drift instrumentation exists to avoid: `last_action_unix` moves on a single
    streaming token and `tip_oid` on any step commit, so either would retire the
    echo while the operator's message was still missing — the defect, restored.
  - **The QUEUE seat has a second key, narrower and faster, and needs one**
    (bl-78d8). The count above answers *"has the derivation shown the message?"*,
    which is the right question for the §3.4 claim and the wrong one for the §11
    inbox-composer queue: the echo's seat there is the **inbox listing**, and a
    follow-up's deposit reaches that listing an entire step boundary before
    `messages/` moves. Between those two moments the queue held **two rows for
    one message** — the solid deposit and the faded echo, the same words, side by
    side — which is the promise below broken at the one seat it was written for.
    So an echo records a second baseline: **how many deposits its seat could show
    when the act was QUEUED**, and it yields that seat the moment the listing
    exceeds it. Two facts, two predicates, and neither stands in for the other.
    - **Queue time, not receipt time.** The §8.2 verb is piped and run to
      completion, so by the receipt that mints the echo the deposit is already on
      disk; a count taken then would include the very file the echo stands for
      and retire it at birth. The baseline is therefore the *seat's* to supply —
      read off the standing `Query::Inbox` the region above the box already
      paints from — and it is carried on the held act, which is the last moment
      it is knowable.
    - **Still a count, never the text.** The §11 rule holds unchanged: the echo
      and the deposit say the same words by construction, so words cannot tell
      them apart. A listing longer than the one the act was queued against is the
      deposit, and nothing was read to say so.
    - **Zero is the general path, not a case.** A start is queued against no
      inbox at all, and a seat whose standing ask has not answered showed
      nothing; both baseline at zero and the rule reads correctly for each.
    - **What it does not do.** It retires a *seat*, not the echo — §3.4's claim
      still spends on `messages/`, per the bullet below. A drain that did not also
      flush would shrink the listing back under the baseline and seat the echo
      again, which no substrate does: the delivery commit that empties the inbox
      is the same commit that writes `NNN-user.md`, and that write retires the
      echo outright.
  - **Landing and the §3.4 claim are one event, not two.** One predicate retires
    the echo and spends the focus claim, in `adopt_started` — so the pending
    value has one lifetime and there is no state in which yog holds half of it.
    Only a *conversation* target moves focus (§3.4: a start focuses what it
    started); an *agent* target does not, because the operator was already
    there, and their own message landing must not yank them back from wherever
    they have since navigated.
    - **Resolving is that same event, and everything the window held rides it**
      (bl-56c6). The claim spending, the target taking its id and the §3.4 held
      sends going out are three things that happen because *one* thing became
      true: the conversation has an address. So they are one place, and the
      third needs no receipt — the draft those sends came out of was emptied
      when they were held, and each is already in the §11 queue as the
      undelivered deposit it is; a refusal earns the durable `ops.jsonl` line
      every act leaves (INV-2).
    - **The half-held state this rules out was reachable** (bl-56c6). In the
      disk-fallback race where a second send *did* land mid-window, its receipt
      raised a follow-up echo on the minted NAME, overwriting the start claim:
      focus could never migrate to the real id, the synthetic row vanished, and
      the new echo's own predicate was false forever because nothing on the
      roster ever wore that name as an id. Holding the send closes it at the
      source rather than by a rule about which echo wins.
    - **What the seats keyed by the old spelling do** (bl-56c6). Two of them
      hold state keyed by the identity a conversation *had*: the composer's
      draft buffer (§11, one box over many targets) and the §11 row list, whose
      answer lands an ask period behind the derivation that resolved the claim.
      Both read the swap off the echo rather than being told about it — it is
      true for the echo's whole remaining life, so each is idempotent and no
      event has to be caught. The draft is **carried**, never destroyed; the row
      keeps leading the list under the name it was born with until the answer
      carries the real one, because the derivation is what said that root
      exists.
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
    - **"The next send replaces it" was written for two follow-ups** (bl-56c6),
      and it is wrong for a follow-up arriving on an **unresolved start**. A
      follow-up replacing a follow-up is honest — the older send is not unsent,
      and the newer one is what the seat is now waiting on. A follow-up
      replacing a start CLAIM destroys the claim before it is spent: the focus
      can never migrate to the id, the synthetic row disappears, and the
      replacement echo waits on a name no roster will ever wear. So the rule is
      narrowed to what it was about: **one send at a time to a conversation the
      world carries**. A send to a conversation it does not carry yet replaces
      nothing — it is HELD beside the start's own (§3.4), and both go out when
      the address exists.
    - **A draft is never destroyed by a receipt, either** (bl-56c6). A send is
      answered frames later and the box is not disabled across the gap, so what
      the operator types there is a draft like any other: the receipt removes
      exactly the words that were deposited, and if the buffer was edited under
      the send it is left alone rather than half-cut. The same rule one noun up
      is why the buffer is carried across the name→id swap instead of being
      stranded.
  - **The second and later messages ride the same mechanism, and must.** The
    §8.2 `message` verb is piped and its deposit becomes `NNN-user.md` only on
    the driver's next step boundary, so the identical hole was open there. One
    `Echo` with two target arms covers both; there is no start-only path, and no
    second thing to build when the same complaint is made about a reply.
    - **What that sentence left out, and what it cost** (bl-78d8). A follow-up's
      deposit is two writes at two rates, not one: the piped verb writes
      `inbox/<agent>/` **synchronously**, and only the flush into `messages/`
      waits for the step boundary. Reading the two as one hole made the
      `messages/` count the only reconciliation an echo had — so from the first
      fresh inbox answer until a whole-workspace derivation caught up, the queue
      appended the echo beside the deposit it stood for and the operator saw
      their own message twice. The gap the echo actually fills at that seat is
      an **ask period**, not a step boundary: what the queue paints between
      Enter and the next `Query::Inbox` answer. The queue-seat baseline above is
      that hole's own key, and the two arms still share one `Echo` — the second
      key is a field on it, not a second mechanism beside it.
- **The live tail: the focused conversation's open `response.json`, followed at
  the rate the model writes it** (`src/boundary/follow.rs`, `src/wire/lane.rs`;
  bl-54f7, rebuilt bl-73e7). Text did not stream back in while the model was
  thinking or writing: yog reads directly from the stream-in file in the litany
  workspace and shows every character as it lands. The mechanism was already
  there — the fold (§5.1 #10), the virtual transcript entry, the `Doing` split
  (#28b). Its **cadence** was the defect: the fold ran only as a byproduct of a
  whole-workspace derivation, so it inherited the watcher poll, the `DirtySet`
  announcement, the 100 ms debounce and the re-derive. Characters arrived in
  clumps at the watcher's rhythm.
  - **It has been built twice, and the second build is the one that stands.**
    bl-54f7 put a **follower thread in the window**: a reader of the open file
    publishing into a `TailCell` that `echo::compose` folded onto the snapshot
    the frame painted, under an explicit in-memory carve-out (purely display, a
    dead end, not re-derivable). That was right while the window derived its own
    content. The remote split (REMOTE §9.7) then moved every §11 read to the
    wire — the transcript with it — and the follower kept running with **no
    reader left**: it folded onto `AppModel::snap`, and every seat that paints
    the tail now reads a `Reply`. Its only remaining effect was up to sixty
    repaint requests a second for content nothing painted, and its own
    regression test asserted against the unpainted value. bl-73e7 deleted the
    thread, the cell and the overlay, and moved the **mechanism** to the engine.
  - **The tail is an answer now, so the carve-out is retired rather than
    weakened.** There is no RAM the window holds and cannot re-derive: the tail
    is `Query::Follow`, answered by the engine from the same fold the pull read
    uses, and the frame paints a decoded reply exactly as it does for every other
    §11 surface. The rule the carve-out needed — *no accessor from the model to
    the tail* — is retired with the thing it guarded, and I1 is untouched for the
    reason it always was: the frame does no IO.
  - **What made it fast is what still makes it fast: following, not
    re-reading.** The read holds the response file's path, a byte offset and the
    trailing partial line (`boundary::follow::open`); each look reads only what
    was appended and folds the complete lines through the one shared parser. The
    engine holds the connection and writes a frame per growth, so the bytes
    cross at the rate they are written and the cost is the new bytes rather than
    the conversation.
  - **The pull path is the fallback and is never deleted.** `Query::Transcript`
    still folds the tail at `ASK_PERIOD`, so a window with no lane — one that
    never came up, one whose stream just ended, one with no wire at all — paints
    the chat exactly as the migration left it: the tail at ask cadence. That is
    the whole reason a lane is worth having; it can fail without the chat
    failing.
  - **Superseded, never merged.** When the step commits, `NNN-<model>.json`
    lands and the derivation carries it; the tail's stream **ends** at that
    boundary — a response file belongs to one step, so a new step is a new
    stream rather than an accumulator to reset — and the seat swaps to the
    committed entry. `Transcript::with_live` *replaces* a live entry rather than
    appending beside one, which is what makes the swap a swap: the two texts are
    never reconciled character-by-character, the committed entry is the truth and
    the tail was a preview of it.
  - **Scope is the focus, and it is the seat's own declaration.** One
    conversation is open; that is the one that streams. Tailing every agent in
    every workspace at that rate is the version of this that burns the machine.
    The seat declares its subject the way it declares any standing question, so
    focus moving hangs the lane up and asks for the new one — and the
    conversation just left reverts to the derivation's own fold of the same
    bytes on the pull path's cadence, which loses nothing, because nobody is
    reading a conversation they navigated away from at character rate. Nothing
    here expires on a clock; the engine's own bound is a *quiet* hold, which a
    frame written resets, because writing a frame is what discovers a peer that
    went away.
  - **Following, not re-reading.** The read holds the response file's path, a
    byte offset and the trailing partial line; each look reads only what was
    appended, folds the complete lines through the **one shared parser**
    (`git_tree::fold_stream`) and absorbs the result (`Stream::absorb`, whose
    contract is `fold(a).absorb(fold(b)) == fold(a ++ b)` on any line boundary).
    Re-reading and re-folding the whole file on every look is the naive shape and
    it degrades as the answer grows. That contract is also **why the lane is not
    a second reading of the tail** (bl-6233): the derivation folds the whole file
    on the worker's schedule and the lane folds the suffix on the writer's, and
    `absorb` is what makes those one description rather than two that happen to
    agree. Partial-write tolerance is structural twice: the bytes after the last
    newline are held back, and the parser skips a line it cannot read.
  - **A frame carries the whole fold, never a delta.** A seat replaces what it
    holds; there is nothing to reassemble, a frame the lane misses costs
    nothing, and a re-ask after any interruption asks for the tail rather than
    for a suffix nobody can address. It is also what lets the *value* be the
    thing compared: bytes moving is not the same as the tail moving, so an event
    the operator cannot see — a `message_start`, a tool-argument delta — advances
    the offset and earns no frame.
  - **What the tail says is what `Agent::stream` says.** No seat learns a new
    vocabulary: the §11 live mark, the flight strip's `N chars streamed`, the
    roster preview and the transcript's live rows all read the one fold. The
    engine's gate on whether there *is* a tail is `inspector::live_tail`,
    unmoved — a settled step's trailing text is already committed, so a response
    file with nothing in flight is not followed at all.
  - **The two halves keep different clocks, and that is the whole split.**
    `transcript::build` reads the committed `messages/` and moves when the step
    commits; `Transcript::with_live` folds the tail on and moves as the model
    writes. Merging them into one build is what made the tail as slow as the
    derivation. Since bl-73e7 the tail reaches the seat by **two** routes at two
    cadences — the pull answer's fold and the lane's newer one — and `with_live`
    replacing rather than appending is the only reconciliation either needs:
    newest wins, and the answer can never be painted twice.

    **What newest-wins is over changed with bl-3655, and the reconciliation did
    not.** A follow frame carries what landed since that read's previous frame,
    not the whole answer, so what a seat replaces its live entry with is its own
    accumulation — every frame of the read absorbed in order onto an empty fold
    — rather than the frame it just received. `Stream::absorb` is that
    accumulation and it is the same operation the engine gathers the frames
    with, so the two agree by contract (`fold(a).absorb(fold(b)) ==
    fold(a ++ b)`) and not by coincidence. The pull answer is unchanged and
    needs no case: it is a stream of one frame absorbed onto nothing, which is
    the accumulated value. What the whole-text frame bought was idempotence per
    frame; what it cost was **quadratic** wire bytes in the answer's length —
    20x amplification measured on a two-sentence reply, and that is the floor.
    The property is kept where it was load-bearing (a seat that lost its
    connection re-asks and its new read opens at byte zero) and dropped where it
    was only convenient. REMOTE §5's follow-lane ruling is the authority, and a
    seat building against PROTOCOL 2 must consume it: the frame's field
    *signature* did not move, so the corpus ledger cannot see this.
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
desktop's, accusing yog of a storm litany was creating. Per-frame work caps were
rejected as the fix: a roots-per-tick cap cannot bound the cost of re-deriving
**one** root, which is exactly what happened. The standing rule is now that UI
and backend operations are totally isolated: the UI never freezes, which means
it does as little as possible.

**What that costs, and how it is paid honestly.** The frame no longer renders
this instant's disk; it renders the last *completed* derivation. That is a real
loss of a real guarantee, so it is measured rather than assumed:

- Every `Snapshot` carries `derived_at_unix` — when the pass **completed**, on
  the wall clock. The §11 ops accessory renders `derivation N s behind` once the
  age exceeds two full-sweep periods — silent below that, because a full sweep
  re-stamps the snapshot every 15 s even when nothing changed, so exceeding two
  of them means *passes are not completing*, not that the world is quiet.

  **The stamp is wall-clock, and it had to become one** (REMOTE §9.7, bl-b4b5).
  It was an `Instant`, which is a process-local monotonic reading with no
  spelling at all — so the one number the accessory paints could be derived by
  the window and by nothing else, and the §8.5 chokepoint, whose `now_unix` is
  minted at the process boundary precisely so every derivation is deterministic
  under test, had nothing to date it against. One stamp, in the unit both ends
  of the wire speak: the age is a subtraction the boundary makes, the line is
  the engine's wording, and it rides the `Query::Workspaces` answer beside the
  §7.2 growth note — the read every window already makes every frame.
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
snapshot that found it, exactly like a `Drift`. One glance now names litany
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
`src/fs_watcher/tests/drift/` and `src/app/tests/drift.rs`): the backend's own
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
| Crashed/killed driver | agent classifies Stopped from framing; attention rule 2 fires; `litany scan` deposits died epitaphs and flushes inboxes |
| yog crash mid-write | rename atomicity: whole-old or whole-new; dotfile temp debris swept at startup |
| yog signalled (SIGTERM from `pkill`, SIGKILL) or crashed between a §4.1 gesture and disk | cannot arise: `ui.json` is **write-through** (§4.1) — the gesture is on disk before the call returns, so there is no in-flight window and no shutdown hook to miss (bl-b54e) |
| yog SIGTERM'd with **work** in flight (a `systemctl restart`, a `pkill`) | bl-b54e's ruling covers the engine's own state and never covered the work. Since bl-269a the engine catches `SIGTERM` (§8.5, `src/engine/stop.rs`): the flag ends the face's loop, the `Engine` drops, each thread finishes its pass and joins, and every piped child goes through `Stream`'s polite SIGTERM-then-SIGKILL rather than being cut down with the process. Bounded by the unit's own `TimeoutStopSec`, never by a second deadline in yog. A SIGKILL still reaches nothing, which is what a SIGKILL is |
| yog crash with drivers running | drivers are detached into their own process group, holding no yog-owned pipe (stdin/stdout null, stderr on a file) — unaffected; next launch re-derives their state from locks/refs |
| Detached driver dies right after launch (tool version skew, missing model config) | past the §7.3 grace window the launch's **target is missing from the derived tree** — no conversation of the minted name, or an agent that has not acted since the row's stamp with nobody driving it — so `opslog::launch::stillborn` holds and the ops sweep folds that launch's §8.1 stderr sink into its `-2` row, which becomes a rendered failure — banner + ⚠ chip, with the driver's words as the diagnosis. Without the sink this was invisible: exit `-2`, empty stderr, a prompt that "does nothing" (bl-a649) |
| Detached driver **declines something and carries on** (a compaction landing superseded, a launch in the accepted crash class, a §6 budget stop) | **nothing at all**, because the sink is never read: the conversation is on disk and its driver is at work, so the launch is not stillborn and the fold does not run (bl-b95e). No banner, nothing on the ⚠ count, an ordinary `OpOutcome::Detached` row. The rule this replaced equated *any* stderr on a `-2` row with death, and because the sink is append-only and re-read every sweep, one benign line banners forever otherwise; bl-1296's phrase table over litany's own sentences narrowed that without reaching it. Its cost is stated where it is paid (§4.2): a healthy launch's row carries no stderr to expand, and the driver's lines keep their durable home in `driver.log` |
| Probe backend unavailable (lsof missing) | tri-state `Unknown` → uncertainty badge, never a false definite state (§10) |
| Driver dies leaving an empty step (version skew, OOM, kill before the first event) | the step is a **no-response wound**, not a quiet one: an empty-or-absent `response.json` **and** no `meta.json` **and** no driver on the agent (§3.5) renders "driver produced no response" in ichor beside the step and banners it at Altitude 1 (§11). Framing alone reads this `Killed` — the ash "stopped" badge over a `0 attempts · 0 tok` row, which is how it read as a quiet step (bl-7f2e). **The banner states the *reason*, in the adapter's own words** (bl-55d8): the tail of that step's `stderr.log`, which litany ARCH §2.3 defines as *"the adapter subprocess's stderr, appended once per attempt across the model call. **Empty on an ordinary run**: brazen speaks every failure in-band on stdout, so bytes here mean the adapter failed outside that contract — a startup failure (a malformed brazen config, an unreadable credstore) that produced no events at all"* — which is this row's class exactly, so the file is not a hint about the cause but the cause itself. The predicate is unchanged and the reason is not a second fact: one derived value carries both (`Wound::{None, Mute, Spoke}`), and the `stderr.log` read is gated on the wound so a healthy step never pays for it. A wound whose `stderr.log` is empty too (a SIGKILL mid-call) is `Mute` and **says so** — "nothing on disk says why" — never a bare glyph and never a pointer at somewhere that has nothing (§11 glyph doctrine). **What this retired:** the banner used to say *"the driver's own stderr is in the activity trail below"*, which was wrong for the class the operator actually hit. A turn continued by `litany message` is driven by a child **litany** launched, not by a yog detached spawn, so no §8.1 per-spawn sink exists for the ops sweep to fold into a `-2` row at all — the falsifying run's two sinks belong to its two `litany prompt` starts and the one matching the wounded turn is zero bytes. The step's own `stderr.log` was the only copy of the answer, and yog was reading past it. The operator's whole signal was *"it looks like the second message in a conversation always fails"* — the absence of a reply. **The residual this left is closed** (bl-83d6): the banner still quotes a *tail* on the two bounds the crate already had (`opslog::detached::captured`'s 4 KiB of file, then `opslog::rows::stderr_tail`'s last three lines — the same tail every other §7.3 surface shows) and still names the file, but the file itself is now a **seat in the drill-in's record picker** (§11 Altitude 2), offered whenever it has bytes and shown to the bounded-file cap. Two bounds, each answering its own question: how much a one-line sentence quotes, and how much a reading surface shows |
| A **healthy send** classified as that wound for a moment (bl-90bf) | the wound's two halves do not share a clock: the disk half is read through the §7.2 per-snapshot memo (once per published snapshot since bl-e90a; per frame before that), the liveness half rides the probe cache inside that same snapshot, and a driver *taking* its flock emits no fs event — so between the send and the §7.2 poll that finds the lock, a genuinely-in-flight empty step reads as a wound. The predicate is right on the inputs it is given; the cache is what is behind. The banner therefore holds a **grace window** before it paints (`src/app/grace.rs`, `WoundGrace`): a wound that clears inside the window never reaches the screen, one that outlives it banners and stays. The window is `Cadence::wound_grace` — **the rising edge's own latency**, spelled as a sum over the **live** cadence off the rendered snapshot (bl-3381) so a re-tuned period carries the grace with it, never a magic number. Four legs, because that is the whole distance between disk changing and a frame holding the snapshot that says so (bl-18e8): one **cheap sweep** (the coarsest signal that can mark the root at all), one **debounce** (the coalescing window the mark then waits), one **pass** at its widest bound (`late_pass` of a full sweep — a derivation publishes once at its end and may be queued behind a full sweep of every workspace, which is the period that sweep is budgeted, bl-4b28), and one **`ASK_PERIOD`** (the boundary's own poll between the worker publishing and the frame holding the answer, REMOTE §9.7). It was cheap sweep + debounce alone until bl-18e8, stated here as *the catch-up bound itself* — but that is the bound on *marking* the root, not on the fact reaching the frame, and the two legs it omitted are the larger two. The rising edge is not covered by the cheap sweep at all: the targeted re-probe looks only at agents already Live/InFlight (`derive::liveness::needs_liveness_reprobe`), so a resting agent coming alive is purely fs-event driven and the sweep tick that leg stood for never happens. The excess was visible — roughly a second of ichor red on a healthy send, which is the alarm the window exists to prevent, arriving on schedule. **And the send now schedules its own catch-up** (bl-18e8's other half, which shortens the true window rather than papering it): an act's receipt marks the *workspace* it named dirty beside its substrate root (`src/app/acts.rs`), so the deposit that creates the mail-on-tail state is the same act that requests the re-derivation clearing it. The gate is **render-layer RAM on the injected clock**, mirroring `Schedule`'s debounce; the predicate stays pure and Clock-free (§5.1 #13). A genuinely dead driver is therefore banner-ed *late*, never not at all — by the window plus the frame cadence's own ≤2 s poll floor (I4), which applies to it exactly as to every other rendered fact. Only the **banner** is graced: the §11 Altitude-2 Steps row paints the same flag ungated, because a cell in a table you opened is as fresh as the rest of that table, while a banner is an unrequested alarm and an alarm that retracts itself teaches the operator to distrust it |
| A model call that **framed cleanly around a turn it did not finish** (bl-fb87) | Transport completion is not task completion. A request sets `max_tokens = N`; the model emits thinking and no text and no `tool_use`; usage reaches N; the stream ends `finish{reason: length}` then `end`, with no `error`. Every §4.4 transport promise is kept, so the settled tail framed `Complete` and §3.5 read **Quiescent** — a clean rest with no answer in it — and §8.2 then offered **Nudge**, which cannot advance it: litany's `advance` derives `Warrant::NothingDue` from an assistant-side tail with no `tool_use` and exits without creating a step. The correction is a second fact off the same walk — never a second walk, and never an inference from token counts (`usage == max_tokens` would duplicate what the reason already states): the canonical `finish.reason` is the authority, and `length` is the one word whose consequence differs. It yields `Ending::OutputLimit`, which makes the state **Stopped** (§5.1 #9 — no new badge; bl-d816's ruling stands), suppresses Nudge (§8.2), and raises the §7.3 wound's second class — `✖ output limit ended the turn` in the Steps row's badge seat, and at Altitude 1 the banner that says the reply stops where the output budget ran out, that Nudge cannot resume it, and that a **message** carries it on. Partial text is untouched: the §5.1 #10 fold still hands the fragment to the glass, and the framing stays `Complete`, so `rail::place` still pairs the step with the entry litany sealed off it. A turn that really did end with a tool call to make finishes `tool_use`, not `length` — the reason is one value and the provider names it — so the continuation case needs no content sniffing to stay Quiescent. **What this is not**: raising the 4,096 default, which only moves the failure; and painting an explanation while leaving the agent Quiescent and Nudge enabled, which leaves the mechanism false |
| `bz --list-models` fails, or returns an empty roster (§9.4) | the picker banners in ichor with the captured stderr and the exact command to run by hand (the §8.3 fallback grammar); an empty roster is named as itself ("the provider offered no models"), never rendered as a picker with nothing in it. The current assignment stays on screen throughout, so a failed query never looks like a lost model. *An **auth-shaped** failure additionally names what this row needs (`login_blocked`'s own sentence) and carries the control that goes there — Login for a row that signs in, the Config tab otherwise (bl-91f1). The banner and the fallback command are unchanged beneath it* |
| A config file the §9.4 anchored-block grammar does not recognize | the surface declines **loudly** (ichor, naming the file and the shape it expected) and points at the §9.2 / §9.3 raw editors. yog never guesses at YAML it cannot recognize, and cannot half-write: a pick is one file since bl-d9cb, and the text is composed — every gate passed, the grammar satisfied — before anything is staged |
| A spawn whose requested cwd is not an existing directory | **the directory is named, never the program** (bl-6191): `std::process` fails such a child *between fork and exec*, and reports the resulting ENOENT against the **program path** — so a start into a typed-wrong work directory read `failed to spawn <yog binary>: No such file or directory`, telling an operator their binary was missing. Every spawn shape routes its failure through one constructor (`CliError::spawn`), which asks the cwd's own question first (`work_dir_fault`) and answers `work directory does not exist: <path>` — or `is not a directory` for a path that is plainly there, since a second lie fixes nothing. Not gated on the OS error kind: a cwd that is not a directory could not have forked for any other reason. The same question is what the §11 field pre-flights, so this error is the *unreachable-by-the-form* residual — a directory deleted between the flag and the Enter, or a non-form caller |
| Failed action (short verb or start step) | **a rendered fact, never stderr-only, and rendered exactly once**: the full `ops.jsonl` entry (argv, cwd, exit, origin, stderr) is expandable at the ops pane, *and* the **originating surface** — and only it — renders the failure in ichor red with argv + stderr tail. *A **failure** is `OpRow::failed`, which for a detached `-2` row is stderr the notice classifier does not recognize (§4.2, bl-1296) — a driver notice is not a failure and reaches no banner at all.* The banner is **derived every frame** from the refreshed ops tail (`AppModel::last_failure(origin)`), never cached at dispatch: the dispatch handler runs microseconds after a detached spawn, before the child can die, so a snapshot taken there is `None` forever and the sink row above surfaces to nobody (bl-4895 — three live prompts, three populated sinks, zero banners). No `eprintln!`-only error path may exist in `src/shell/` (STORIES INV-2). **The originating surface is the op's `origin` field (§4.2), stamped at dispatch — see the row below for the rule it obeys** |
| **How a banner ends** (bl-c417) | Two ways, and until bl-c417 only one existed. (1) **Retirement**: a newer op of the same `origin` that did not fail — the §6 rule, per-surface (the row below). (2) **The operator's ack**: `AppModel::last_failure` queries `opslog::since_ack`'s rows, so the §4.2 ack line quiets every surface's banner at once, whether or not anything was retried. That second exit is the whole of the complaint — *"I need a way to make the failed notification go away"* — because an operator who reads an error and decides **not** to retry could not otherwise put it down: the only exit was a *successful re-run of the same verb*, so a failure the operator has understood and chosen to leave alone banners forever. The ack is not a widget flag: it is a durable line, so the dismissal converges to the other instance and survives a restart exactly as the failure did. It is also not amnesia — a **new** failure of that origin lands after the watermark and banners again. The dismiss control sits on the banner itself and in the §11 ops pane, both spelled from one home (`opslog::operator`), both explaining themselves on hover (bl-68ac) |
| Which surface is "the originating surface" (bl-48f8) | **The op's *subject*, recorded at dispatch, one banner surface per subject.** Three subjects exist and each has exactly one seat: a **ball** op (every `bl` verb, and every step of a ball-rung start — `bl create`/`bl claim`, but equally its `litany prime`/`litany new`/`["yog-step","mkdir"]` substrate steps and its detached `litany prompt`) banners in **the roster's balls section**, where the ▶ Start / ▶ Continue / Create-&-Start row that offered it is (§11, bl-6ad8); a **conversation** op (`litany message`/`stop`/`scan`, and every step of a bare- or path-rung start) banners at **the composer** — the empty world's bootstrap box being that same box before a workspace exists (§3.4), never a second seat; a **world** op (the §9 config writes, the §16.3 space knob, the §8.3 login flow, the §3.6 unmaking, and yog's own §7.2 drift lines) banners on **no §7.3 surface at all**, because each of those surfaces states its own outcome in place and a config-write failure is not news the composer has any business breaking. §6's retirement is per-surface with it: a surface's banner clears when *that surface's* next action runs clean, and never when someone else's does. **The subject is not the pointer's position:** one gesture has one body however many hands reach it (`ball_bar::close_ball` is the composer's Close button, the §11 `c` key and the row menu), so a ball verb is about a ball wherever it was clicked — forking that body per hand to record a pixel would record a distinction no operator makes. What this closes: `last_failure()` was one global query over the ops tail and three surfaces rendered it unconditionally, so a single failed start painted the balls fold, the composer *and* the bootstrap box at once, a config-editor failure accused the composer, and any surface's clean run wiped every other surface's live banner |

---

## 8. The action surface (v1) and exact argv

All spawns go through `cli_outbound` (generalized: binary resolution
parametric over env var — `LITANY_BINARY`, `BL_BINARY`, `BZ_BINARY`, default
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
   `litany new <root>/home`** — the bootstrap is this empty case, not a
   separate flow.
   The explicit **New workspace** verb (§11) runs the same `litany new` with
   the operator's typed, §3.1-validated name, deliberately. **The resolved
   workspace then becomes the focused one**
   (§3.4) — every rung, before step 2 opens its composer, so the composer and
   every surface beside it name one workspace.
   **Birth judges only what exists at birth (bl-c3a9, retired by bl-00ee).**
   `litany prime` lays down `<world>/litany/template/providers.yaml` and every
   `litany new` commits it verbatim as the workspace's first `config/default`,
   so that one file decides which provider row every conversation in the new
   workspace dispatches through. bl-c3a9 therefore ran §9.2's provider gate over
   it *before* `litany new`, and while brazen's table was the machine's that was
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
   nothing on top (bl-7fc8).** The pinned litany template authors
   `worker.tools: [apply_patch, bash, cd, dispatch, load_skill, message,
   multi_tool, read_file]` — the entire shipped pool, `message` and `dispatch`
   included — so a root agent in a workspace yog just created can already
   message a sibling and dispatch a subagent with no second write. An earlier
   creation-time rewrite (`grant_worker_tools`) once re-asserted those two
   names against a stale template that lacked them; against every template
   since, it read the role, found both already present, and authored no
   commit — a no-op path kept alive only by its own stale comments. bl-7fc8
   deleted it, its staging/editor call and the tests that existed only for it.
   The first `config/default` is litany's/operator's one home from the start,
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
   `litany prompt --name <minted> [--config <lineage>] <root>/<name> <goal>`
   — the lineage §8.7's birth policy selected off the ball's tags, absent for
   every start that selected none — with the goal **verbatim**
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
substrate step (the seed, `litany new`) precedes every `bl` mutation, so a
failed or missing substrate aborts before anything half-commits — *the start
flow* can never mint an orphaned claim. (A claimed ball whose workspace was
later deleted remains a legal state; §3.5 renders it claimed-elsewhere.)

The planner (`start::plan`) is a pure function returning the command sequence;
the executor runs it step-by-step with per-step outcomes in `ops.jsonl`.
Steps are individually idempotent-or-convergent: re-running after a crash at
any step converges (double-claim refuses benignly; `litany new` skipped when
the dir exists; prompt just adds a root; **a ball already claimed by a local
workspace name re-plans as a prompt into that workspace — resume, not a
second mint**). A **new** ball defers the id: the plan is a single
`bl create`, and the freshly-minted (ready, unclaimed) ball is re-planned as
an existing one — the new→existing transition *is* the convergence, not a
special case. The claim's stdout worktree path is cross-checked against the
bl-delivery formula (both the `<id>` and `<id>-<claimant>` variants match;
anything else is a convention drift surfaced loudly, never silently
accepted). The short steps (`litany prime`, `bl create`, `bl claim`,
`litany new`) log their piped outcome; the detached `litany prompt` logs only its **spawn** — argv,
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

**The start pane's FIRST rung is provider sign-in when the wall has none
(bl-1fd0, operator ruling).** Verbatim: *"On a wall holding no usable provider
credential, typing a goal and hitting Enter will work zero percent of the time —
the conversation is born, immediately dies on no-models, and the operator learns
it from a dead row (or from nothing at all). The pane is inviting the one act
that cannot succeed while hiding the one act that must come first."* It was hit
live twice in one evening and both first goals were wasted.

- **The predicate is brazen's own `credential` column**, projected beside the
  three §5.1 #20/#21 columns yog already reads and folded by the same §8.3 `ask`
  that populates the Login roster (`crate::start::WallCredit`). No network, no
  spawn, no per-frame cost — the ruling's "must not add a round trip to the
  pane" kept by reading a table the frame already holds.
- **A keyless row is not a credential.** The ruling's own wording is *"any row
  `stored` or `not required`"*; taken literally it is vacuous, because brazen
  merges its built-in table under **every** config, so `ollama` and
  `claude-code` read `not required` on every wall there can be. Nor are they
  what a doomed start was routed to: both claim no model prefixes and are
  reached only by an explicit `--provider`. So `not required` does not ready a
  wall; it only adds a clause to what the rung says. Every other spelling does
  ready it — `stored`, `ambient`, `inline`, and any this build cannot read,
  because refusing a wall whose rows carry a credential a run would spend blocks
  a working setup, and no surface refuses on an unanswered question.
- **Three states, total** (`crate::start::StartGate`): Ready paints nothing at
  all and today's flow is untouched byte for byte; SignIn paints the sentence
  and the §8.3 roster beneath it, and Send says the sentence instead of firing —
  through the same read the §11 Enter binding makes, so pointer and keypress
  cannot disagree; Unknown is a workspace a §8.2 entry hosts, where this box
  reads its OWN wall's brazen and says so rather than answering with the wrong
  table. **Unknown is bl-61bf's seam** and never a refusal: an unread wall is
  not a wall known to be empty.
- **The goal box stays draftable and the draft is never spent.** A refused Send
  is a no-op — the pending start, its goal and its §3.3 seed all stand — which
  is the whole point: the ruling is about a typed goal that was lost.
- **It is a band of the pane, not content inside the goal box's panel** (§11
  rule 5). Written inside the composer first, it does not fit there by
  construction: the start box is 240 points and the rung is a sentence, ten
  provider rows and a live command stream, so the roster took the box's room and
  the pane clipped Send off the bottom. It docks directly above the box and asks
  for a share of its own, exactly as the settings band does. It is not a fifth
  settings band and bl-2e18's ordering ruling is untouched — it is not a
  setting, it is the reason the box below it cannot fire, and like the in-flight
  strip it is conditional as a whole.
- **It dissolves on the sign-in's own outcome.** The §8.3 holder folds a clean
  streamed exit back into its rows the one frame the run settles, so the wall's
  credit flips and the next frame is a signed wall's — with §11's one focus door
  handing the keyboard to the goal box, whose draft is untouched. The start goal
  box takes that hand-off because it is a composer like the other two (§11: one
  box, one Enter) and the message composer is not painted while it is up.

**What the gate cannot see, recorded rather than hidden:** *which* row a start
routes to. That is `roles.<r>.provider` on the config branch — several git reads,
and §9.4's subject rather than the pane's — so the gate judges the **wall**, not
the route. It is exactly right when nothing at all is signed in, and
conservative for an operator whose roles all name a keyless row: they are told to
sign in when their setup needs no sign-in, and the remedy is one sentence they
can read past. The same defect one seat over — the docked message composer and
the empty-world bootstrap box, whose Enter is §3.4's bare rung — is not covered
here.

### 8.2 Per-workspace / per-agent verbs

| UI action | argv (cwd) | Spawn mode |
|---|---|---|
| New prompt (new root) | `litany prompt <ws> <text>` | detached |
| Message agent (also the resume gesture — no resume verb exists, ARCH §2.9) | `litany message <ws> <agent> <text>` | short, piped (it self-detaches its driver) |
| Stop | `litany stop <ws> <agent>` (+ `--stop-children` toggle) | short, piped |
| **Send-and-interrupt** — cut the conversation off mid-work and give it this text instead (bl-a33d) | `litany stop <ws> <agent>`, then `litany message <ws> <agent> <text>` | short, piped **×2, two `ops.jsonl` rows** — the interrupt and the deposit are independently observable mutations (§4.2) and a composite row would hide that a stop fired. **No new verb, and no trigger of yog's own**: the deposit's own driver-start is the trigger (litany ARCH §2.9 — there is no resume verb, a deposit into a quiescent branch starts a driver), which is precisely the state the stop just put the branch in. **No `--stop-children`**: the gesture's subject is the conversation being talked to, and a subtree is `/stop children`. Fail-closed on the first half only when it could not *spawn* — a stop that ran and was declined (nothing in flight) carries on to the deposit, which is the same gesture at zero work rather than a case of its own, so no seat has to know whether a driver is up. It rests on litany bl-b98d: a stop landing in a **tool window** settles that window before depositing (one in-band `is_error` `tool_result` per unanswered `tool_use`), so the tail a deposit revives warrants an ordinary model call instead of litany's `UnpairedToolUse` decline. Before that landed this gesture would have bricked any conversation it caught mid-tool-call, and yog may carry no guess at litany's step state — that would be a race, not a mechanism |
| **Nudge** — fire inference on a conversation from the state it is already in (bl-9bef): no new message, no goal retyped | `litany advance <ws> <agent>` (ws) | **detached**, logged like the §8.1 fire. litany derives what is due from the transcript tail (ARCH §6 *warrant*): a tail ending user-side means a model call is due, so a first turn whose call died — the credential was missing, the row was wrong — re-dispatches **in place** on the same branch, the same goal, the same conversation. Not `message` with an empty body: a deposit would put a second user turn on the wire saying what the first already said. Detached and never piped, because an advance runs the conversation until it goes quiet. **Offered exactly where Stop is not** (Quiescent/Stopped): a Live or InFlight branch already has a driver holding the lease, and litany's hop would take its clean no-op branch — a control that fires and does nothing is QUALITY H4's theater. **One resting shape is exempt for that same reason** (bl-fb87): a conversation whose latest turn the output limit cut off (§5.1 #9's truncation reading) leaves an assistant-side transcript tail with no `tool_use`, which litany's `advance` derives as `Warrant::NothingDue` — it releases the lease and exits without creating a step. So the partition gives way rather than offering theater, and the recovery is **Message**, which needs no gate because a deposit lands user-side and warrants a call. The §7.3 wound says exactly that, in words, at the same moment the control disappears — a control that vanishes silently reads as a bug. It is the §8.6 hold release's own launch, shared as one body |
| Fork an attempt from a pinned notch (VISION V2, bl-dc0c) | `litany dispatch <role> <ws> <parent> --goal <text> --from <ref> [--pin skills/<s>/SKILL.md=<pool>/<s>/SKILL.md]…` (ws) | short, piped (it detach-launches the child's own driver). **Piped, not detached, on purpose:** a refusal — an undeclared role, a ref the workspace has not got — must come back in litany's own words as a rendered failure, not as a click that did nothing. A cohort is this row fired N times, one ops line each |
| Scan / flush | `litany scan <ws>` | short, piped; summary line surfaced |
| Move a live conversation onto **another config lineage**, and settle one that is held on a divergence (§9.4, bl-2d19 as re-scoped by bl-e654) | `litany retarget <ws> <agent>` (ws) | short, piped. It writes `refs/litany/retarget/<agent>` and returns; the conversation's **own** executor lands the re-fork at its next step boundary (litany ARCH §2.2), so nothing yog spawns advances the branch. It is **not** how a config edit reaches a running conversation — under follow-the-tip an edit reaches it with no verb at all. No `--config`: litany's default lineage is the one §9.3 writes, §9.3 writes exactly one, and that one is the lineage a held or foreign-lineage conversation is being settled onto — a knob with one lawful value, and the value is right |
| Close ball | `bl close <id>` (project) | short, piped; capture/fold/gate/squash output in `ops.jsonl`, gate failures verbatim (claim+worktree stay up, bl's own semantics) |
| Assign ball → workspace (§3.2) | `bl claim <id> --as <name>` (project) | short, piped |
| Release ball | `bl unclaim <id>` (project) | short, piped |
| New ball | `bl create "<title>" [--body B] [-p N] [-t TAG]… [--parent ID] [--needs ID[:OP]]` (project) | short, piped; new id captured on stdout |
| Update ball | `bl update <id> [--title T] [--body B] [-m NOTE] [-p N\|--no-priority] [-t TAG\|--no-tag TAG]… [--parent ID\|--no-parent] [--needs ID[:OP]\|--no-needs ID]` (project) | short, piped |
| Delete workspace (§3.6 — typed-name confirm; refused while any agent is Live/InFlight) | `bl unclaim <id> --as <name>` per live bound ball (project), then the `ui.json` prune, then the dir removal logged as `["yog-step","delete-workspace"]` | short, piped ×N + the §4.2 non-spawn step line; the §8.1 planner idiom, order load-bearing (§3.6) |
| Delete conversation (§3.6 one deep, bl-f17a — confirm scaled to blast radius; refused while any member is Live/InFlight) | `litany delete <ws> <agent>` (+ `--children` iff the typed name armed it); the dialog's census is `litany delete <ws> <agent> --children --dry-run` beforehand (unlogged — a read, the `bl conf` seam's idiom) | short, piped; a clean removal then prunes the subtree's `seen` keys (`ui.json`, §4.1) |
| Refresh models | `bz --list-models --provider <row> --json` | streamed-piped where a frame paints it (§9.4's picker, physically `yog bz …` since §16.7 W10 — the logical argv logged is unchanged); **in-process** through the linked brazen where nothing paints it, which is `RealBzRunner` and therefore every off-frame caller (§8.5's `Models`, bl-dff8) |
| Login provider | `bz --login --provider <row> --browser` (Login pane; also offered beside an auth-failed step) | streamed-piped (§8.3); same physical retarget. The row is offered only when brazen's table says `auth = "oauth2"` (§8.3 as amended, bl-b4e5); `--browser` is unconditional at the current pin and becomes the browser-only arm of §8.3 rule 1 as amended (bl-61bf) once a row declares a device endpoint. The spawn's locus moves to the engine as a boundary act (REMOTE §8.3, bl-c285) |

A short verb is **piped inside the gesture** — `actions::verbs::run_logged`
runs it to completion and appends its outcome — so there is no window for a
busy state and none is kept: the feedback is the durable `ops.jsonl` line
(§4.2) that the activity chip and the §7.3 banner read back per frame, the
same fact both instances see. Only the long verbs detach (§8.1).

**Claimant rider (Z4; "identity rider" pre-bl-68d9).** Every `bl` claim/close/unclaim yog issues is stamped
`--as <workspace name>`, **not** the operator's `$USER` — the claimant delivers
its own ball (§3.2's ownership line, the same fact as the start flow's `bl claim
--as <name>` and W9's `YOG_NAME`). Concretely: **close** and **release** stamp
the ball's *bound* workspace name (its claimant); **assign** stamps the
*target* workspace name. The operator identity survives only as the
*author* of a standalone `bl create`/`bl update` (§8.2 New ball / Update ball),
where a workspace is not the reporter.

**The workspace-bound rider (bl-bf79).** Every `litany` row in the table above
is *about one workspace*, and such a spawn carries two env facts: the
workspace's **wall** (`YOG_WALL`, §16.2) and its **name** (`YOG_NAME`, §3.3).
Both are laid **at one seam** — the bound `litany` a workspace verb takes
instead of a bare command handle — and never per verb. The rule is normative
and total: *a §8.2 `litany` verb takes a workspace-bound spawn, so there is no
per-verb judgement about which facts it owes.* `stop` launches nothing and so
gains nothing from the wall; it is bound anyway, because an exemption is a
decision, and it was three such decisions that shipped the bug.

That bug is what this rider records. `message`, `fork` and `scan` each laid
`YOG_NAME` alone (or nothing at all). `litany message` deposits and then
**detach-launches a driver** when the branch is quiescent (litany ARCH §2.9 —
there is no resume verb; the deposit restarts a driver), and that driver
inherited yog's fold *without the wall*: its first `bz` died with `no workspace
in this environment — providers, sign-ins and the model cache belong to a
workspace`, and the turn produced a zero-byte `response.json`. The first turn of
a conversation always worked (`litany prompt` is fired from §8.5's chokepoint,
which did lay the wall), so the operator saw "the *second* message always
fails" — really "any message that has to revive a quiescent driver fails". This
is §16.2's own rule applied where it had not been: *set once, at the edge that
knows the workspace, and no downstream seat has to be told.*

### 8.3 Deliberately not in v1 (with reasons)

- **Manual `litany dispatch`** — workflow-driven dispatch is the designed
  path; a manual role-dispatch button invites mis-goaled children. If ever
  surfaced: `litany dispatch <role> <ws> <branch> --goal <text>`, role list
  derived from the roles that carry souls in the `providers.yaml` the run would
  resolve (§5.1 #17).
- **Fork-from-history** — no litany CLI verb exists; yog may not write refs
  (ARCH §3.5). Upstream gap, tracked as a litany ball, not worked around.
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

  **Once a workspace can be held from another box, the spawn's locus is the
  ENGINE (bl-61bf; REMOTE §8.3).** The sign-in is a boundary act executed
  inside the NAMED workspace's wall and streamed to the invoking seat, so the
  credential lands in the wall the workspace's agents read (§16.2) whichever
  box the browser is on — a sign-in fired at the seat wrote the seat's wall
  and left the host's empty, which is the live defect the ruling closes. The
  pane's in-process spawn is the local case of the same act and retires with
  the pane's migration (bl-c285 the act and lane, bl-1ddb the pane). Nothing
  credential-shaped crosses the boundary in either direction: yog still
  renders the flow, bz still owns the credential (§5.1 #22), and the streamed
  lines are the whole of what moves.

  **The flow-selection rule and the capability source (bl-b4e5), three
  rules:**

  1. **The flow is the row's own capability (amended bl-61bf): the device
     flow where the row declares a device endpoint, the browser flow
     everywhere else.** The rule was "always `--browser`", argued from the
     desktop — a GUI with a browser at hand — and the argument died when the
     workspace stopped being the box's own (REMOTE §8.2): the loopback
     AuthCode flow (RFC 8252) completes only where the browser can reach bz's
     loopback, which for an engine-side sign-in (REMOTE §8.3) is the engine's
     box and not the seat's. The device flow (RFC 8628) is the
     seat-independent one — bz needs no browser and binds nothing; the URL and
     user code stream to whichever seat asked; the human completes in any
     browser anywhere — so where a row can serve it, it is the flow that
     completes from every seat, the engine's own box included (the window
     paints the verification URL as an opening link, so the co-located case
     keeps its one-gesture feel). `authorize_url` and `token_url` are
     *required* fields of every `oauth` block while `device_url` is `Option`,
     so `--browser` remains the floor every oauth row can serve; a
     browser-only row signed in from a remote seat gets the stated loopback
     remedy (REMOTE §8.3), never a silent hang. Still no flow selector on any
     surface and nothing yog guesses at: the branch is one row-declared fact,
     read off brazen's own table (rule 2). At the current pin no builtin row
     declares a device endpoint and the projection does not carry the fact, so
     the spawn is `--browser` unconditionally — byte for byte the old rule —
     until the upstream ask lands (bl-c5fe, consumed by bl-7c9f).
  2. **Loginability is brazen's `auth` column, never a yog-side
     reclassification.** `bz --list-providers --json` projects
     `{name, protocol, auth, credential}` per row; brazen's own resolve
     invariant is that `Provider::oauth` is "present exactly when
     `auth = "oauth2"`". So `auth == "oauth2"` *is* "this row has an `oauth`
     block", answered by the crate that owns the invariant. `bz --login` signs
     in oauth rows only, so every other spelling (`none`, `api_key`, `bearer`)
     is a row it can only refuse: those rows get **no Login button at all** —
     see rule 4. The projection carries no device-endpoint fact at the
     pinned brazen — and rule 1 as amended (bl-61bf) makes that fact the flow
     key, so gaining the column is half of the bl-c5fe upstream ask; until it
     lands the absence reads as browser-only and the spawn matches the old
     constant.
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
     projection it reads (`src/config_edit/brazen/providers.rs`) — and this pane
     is its **one painted seat** (bl-20cb): the §9.1 config pane rendered the
     identical struct verb-less until that ruling moved the roster to the
     surface whose verb it is about, so there is no second surface that could
     state a different thing about one provider. The Login pane's `↻` re-asks brazen
     *and* re-reads presence in one gesture (§7.2: never per frame), which is
     also how a just-completed sign-in becomes `signed in`.

     **A default install has a browser sign-in, and yog spends no code on it**
     (ruled at bl-8c2d, consumed at bl-0219). Every row brazen used to ship was
     keyed or keyless, so the rule above — correctly — put the *reason* where the
     button goes on all of them, and a stranger's only path was authoring a key
     in the §9.1 editor. That was §8.3 working as written and it was not the
     intent, so the fix was brazen's: its default table now carries an oauth2
     row, and `row_views` renders the verb for it the moment the crate is
     pinned. Nothing here reclassifies a row (the keyed rows still name the
     editor); the whole consume is a beat pinning that the empty world's roster
     paints a pressable Login **beside that row's name** rather than a sentence
     — `shell/acceptance/first_run.rs`, which is where an upstream that renames
     or drops the row will fail.
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
     failing step's `request.json` names the model it was dispatched with (litany
     ARCH §4.2 — the id rides the canonical request verbatim); the config commit
     the agent **resolves** (§5.1 #17, the followed one) binds each role to a
     `(provider row, model id)`
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

  6. **The config-kind fault gets the same shape** (bl-dd7f, ruled at bl-9b52).
     Rule 5 is about credentials; the other class an operator meets on their
     first dispatch is a **provider row that does not resolve** — a name in the
     workspace's `providers.yaml` that brazen's table has not got, which dies as
     `litany prompt: provider error (Config): unknown provider \`openai-chatgpt\``.
     That rendered on the §7.3 banner with a **Dismiss and nothing else**, and
     Dismiss puts the sentence down without touching the file. So the banner now
     pairs it with one sentence and one control, additive above the Dismiss and
     below brazen's verbatim words: the remedy names the row and routes to the
     §9.1 raw-TOML editor, which is the one place a provider row is authored
     (`config_edit::fault`, the §11 `Config` tab).

     Three things this deliberately is **not**. It is not a birth gate: §9.2's
     was retired (bl-00ee) for judging a workspace's providers against a wall
     that did not exist yet, and this reads a failure that already happened —
     the row's existence is brazen's fact, resolved at call time (litany ARCH
     §4.1), and this is that answer arriving. It is not a re-wording: the
     classifier adds a sentence and changes none. And it is not a join: brazen
     quotes the name it could not resolve, so the row is already in the words
     being classified, and a second derivation could only disagree with them.

     **The class is wider than litany's `Config` wrapper, and a marker table
     cannot find all of it** (bl-5252). The other failure whose only remedy is a
     config file is a row that resolves fine and whose **dialect** cannot carry a
     yog turn: `/model worker claude-code <id>` written before §9.4's capability
     gate existed, or written by hand through the §9.1 editor — which is the
     operator's own authority and gates nothing — dies at brazen's *encoder*.
     That whole family (no tools, no `tool_choice`, no multi-turn transcript, no
     non-text block) leaves one `reject` helper stamped `ErrorKind::ParseInput`,
     so litany wraps it `provider error (ParseInput) …`: the two markers match
     nothing in it, and the banner offered Dismiss for a failure a file fixes.
     The error KIND cannot be the key either — it is the same `ParseInput` a
     malformed image block gets, and keying on it would hand a config remedy to
     every one of them.

     So the second way in is keyed on **the dialect the decline names**. brazen's
     habit is to lead with its own `ProtocolId` spelling (*"`claude_code`
     carries no tool declarations…"*), so `dialect_decline` scans the failure's
     own words for a whole `[a-z0-9_]` token and hands it to the **same match**
     `ProviderRow::tools_blocked` reads the `protocol` column into — the
     judgement bl-3d22 landed, not a second table, so the banner and §9.4's
     provider control cannot disagree about why the row is unusable. The match is
     exact and case-sensitive on brazen's spelling, which is the whole of its
     narrowness: the row NAME `claude-code` carries a hyphen and rides every
     other failure line through that row, and a tool-capable dialect that names
     itself (*"`anthropic_messages` requires max_tokens"*) is not refused. The
     added sentence is the next move — a tool-carrying row, chosen in §9.4's
     picker or authored in the §9.1 editor — and brazen's words stay verbatim
     above it. When brazen bl-5053 serves the capability column, this route
     follows `tools_blocked` to it; it holds no dialect fact of its own.
     `tests/brazen_claude_code_decline.rs` drives the **linked** brazen to earn
     the sentence it keys on, so a rewording upstream reddens rather than
     silently un-classifying the family.

     **The picker beside that step names the row that failed** (bl-dd7f's other
     half). §9.4's dropdown steers off a row brazen has dropped (bl-bd89) — the
     right call, since asking a dead row for its models is the dead end the
     picker exists to leave — but it steered *silently*, so a conversation that
     died on `openai-chatgpt` showed a picker reading `anthropic`, brazen's
     first row, and the operator read it as a report of what ran. The
     substitution is now a fact the caller is handed (`model_pick::Scoped`) and
     the seat says it: *this conversation was dispatched through
     `<row>`, which brazen does not have*. Steering is unchanged; steering
     silently is what ended. The note retires the moment the operator picks a
     row themselves — their selection is their own answer to it.

     **Both seats say it, and the pane still opens.** The wordings live on
     `AuthFailure` beside the classification that decides them (the
     `GoverningConfig::label` discipline, §9.3), so the §11 conversation
     banner and the Steps-tab mark cannot drift: `⚠ the last step failed on
     <row>'s credentials` and `⚠ auth: <row> — Login ↙`, degrading to today's
     unrouted wording where no row was derived. The Login pane still paints
     beneath the banner in both cases — a *wrong* derivation must never become
     the only way out.

### 8.4 World escape hatches (`yog env`, `yog exec`)

Two subcommands of the yog binary — the multi-call pattern beside
`--editor-apply` (§9.3) — expose the composed world to a human at a shell:

- `yog env [--ws WORKSPACE]` prints the world's `export` lines (`LITANY_HOME`,
  `XDG_STATE_HOME`, `PATH`); `eval "$(yog env)"` drops the current shell *into*
  the world, where a bare `bl`/`litany`/`bz` is the world's own shim — yog's
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
flow and the litany arm use). The tools dir is a generated artifact of *the
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

**A gesture addresses by NAME, never by path** (REMOTE §8, bl-f5f6). Every
`Action`/`Query` field that identifies a workspace or a project carries the
name: a workspace's is its §3.1 directory leaf — which §3.2 already makes its
`--as` identity, so nothing new is invented and a foreign workspace is addressed
exactly as a named one is — and a project's is derived, since a balls invocation
path has no name (the shortest trailing run of components no other enumerated
project shares, `src/naming`). The engine resolves a name to a path **once, at
the chokepoint, ahead of the match** (`Action::project` in `src/boundary/address.rs`;
`Action::workspace`/`Query::workspace` in `src/boundary/address/workspace.rs`), so no arm re-derives an
address and an unresolvable name refuses naming the token before anything runs.
The frame reads the same mapping backwards where a seat's *selection* becomes a
gesture (`Snapshot::ws_name`/`project_name`). Two directions, one mapping,
nothing stored. A path still crosses where the path **is** the fact rather than
an identity — a written file's location, a worktree the operator opens, and the
`--cwd` binding a `Prepared` carries back to the engine that minted it.

**A conversation is addressed by an agent id or the unique stored name a living
agent wears** (bl-49bc, REMOTE §8). That is the third noun, and its rule differs
because the noun does: a workspace and a project resolve over an *enumerated set
of paths*, while a conversation's identity is a **pair** — litany's id (its
branch name, and the only thing that addresses a path) beside the §3.3 name it
wears. The two are disjoint spaces by construction, since every id opens with the
compact `YYYYMMDDTHHMMSSZ` stamp and litany refuses a name that reads like one,
so the resolution never guesses: the needle is one or the other.

**Why it had to become a contract.** A successful `/prompt` answers
`{"kind":"started","conversation":"<minted-name>"}` — the minted name is all a
fire knows, the root having no id until its detached driver writes `agents/<id>`
— and the terminal's own usage said `--agent ID`. So the receipt's handle
composed with `message` alone, that being the one litany verb which resolves a
display name: `/agent` read `present:false`, `/steps` and `/transcript` answered
empty rows, `/stop` and `/retarget` refused. The dangerous half was the verbs
that **succeeded**: `Floor`, `Monitor(Flag)` and `MarkSeen` write yog's *own*
rows keyed by agent id, so a row landed under a display name read as policy,
logged as policy and governed nothing. Two deliberate local contracts became
non-composable the moment both were exposed as one headless API, and the fix is
one vocabulary rather than a translation table: **the receipt keeps publishing
the name, and every agent-addressed `Action` and `Query` accepts it.**

The tables are `Action::agent` / `Query::agent` (`src/boundary/address/agent.rs`,
answering *through* `monitor::Verb::agent` exactly as the workspace table does),
and the ladder beside them is three rungs, each answering what the one above
cannot. An **id-shaped** needle is an id — returned untouched, with no
enumeration and no existence claim, so every seat that already spells one pays
nothing and `litany delete`'s admission of an id no ref answers to (its §9.2
debris cleanup) survives. Otherwise the **published derivation** is asked,
because it is the very set every boundary *read* answers from — addressing and
answering must not be able to disagree about which conversations there are — and
it holds the foreign and hand-made ids litany's stamp grammar does not recognize
beside every stored name. Otherwise **disk**
(`git_tree::living_agents`), which is bl-6c9e's barrier one noun down: a fire
returns the instant its detached driver is launched, so the name its receipt
hands back is addressable before the next §7.2 pass has read the branch. Unknown
refuses and ambiguous refuses, in the resolver's own words. **One window stays,
and it is litany's**: between the fire answering and its detached driver
committing `agents/<id>` there is no branch anywhere for any rung to find, so the
handle refuses — *naming the token*, which is the honest answer and the one thing
the old contract could not give (it read empty rows, or wrote a policy row that
governed nothing). Closing that window would mean waiting on the driver inside
the fire, which §8.1 refuses on its own terms. **The §3.3 ladder's
legacy display-only rung is not an address** (bl-8068): a `You are <x>.`
goal-stamp parse has no stored `name` blob and no ref answers to it, so it
refuses exactly as an unknown name does — it renders as a title, which is all it
ever was, and the seats that paint one already hover that fact.

**The set that resolution reads is the live enumeration, and that is what makes
a birth a barrier** (bl-6c9e). The engine's intake builds each gesture's
environment with the §3.1 enumeration as disk holds it — three readdirs, folded
in at `ConsumerCtx::deps` through `app::addressable` — in place of the workspace
set the last derivation cached. Without it, an act that **founds** a workspace
answered before the worker had read it, so the very next boundary call refused
the name that reply had just made addressable: the documented `/prepare` →
`/prompt` flow could not compose two processes deep (`{"error":"unknown
workspace \"home\""}` immediately after a prepared reply for `home`), and the
window's own posted receipt earned the same refusal for the wall its previous act
had founded, so a first turn reached `litany new` and `litany config` and never
`litany prompt`. **When an action returns a newly addressable resource, its
success reply is a barrier for every boundary call after it** — and the way that
is kept is a *query*, not a claim: nothing is stored, nothing is republished, and
no sleep is spent waiting for a derivation. Three properties come with the shape
and each is load-bearing. It costs nothing in the steady state (the two sets
agree, so the published `Arc` is handed straight back rather than cloned). It
runs backwards for free (a workspace the §3.6 unmaking deleted stops resolving
at once instead of at the next sweep). And it does not touch §7.2's partition:
the enumeration is disk answering, never the frame's optimistic fold, so
`boundary_deps`' rule — *the derivation, never the §7.2 fold* — is intact, and
the frame keeps both its cached copy and its §3.4 raise claim because a frame
does no IO. The *derived* per-workspace facts (trees, bills, the §3.5 join) stay
exactly as published: those are the walks that are not cheap, every read of them
is aimed by the path this resolution produced, and a newborn wall answers with
the zeros it honestly has.

**And it is both nouns, since bl-3377.** The ruling above is stated as a rule
about *existence* — *"birth is a barrier because existence is a query"* — and
bl-6c9e folded one set, leaving the **project** noun on the derivation's cached
copy. So a project primed into the world was refused by every ball gesture for a
whole sweep, in a sentence indistinguishable from a typo: `yog bl prime` in a
repo inside the world, then `/create` on it, earned `unknown project "proj"`,
and one full sweep later the byte-identical line succeeded. A project is
enumerated exactly the way a workspace is (§5.1 #1 — one readdir of the balls
clones dir, `projects::enumerate`), it was simply not included, and it inherits
every property above: it costs nothing in the steady state, it runs backwards
for free, and the derived per-project facts (the ball lists, the §3.5 join) stay
as published. One readdir joins the three.

**The refusal now names the set, and that is why no listing verb was added**
(bl-3377). The other half of the defect was discoverability: `--project` takes
the §5.1 #1 *derived* name — the shortest unique trailing run of path components
— and no gesture answered that set. `/balls` and `/board` carry a `project`
field only for balls that already exist, so on a world with a primed project and
no balls, which is exactly the state after `bl prime`, the operator had no way
to learn the word the flag wanted; the refusal named the token they typed and
offered nothing. A `/projects` read beside `/workspaces` would answer it, and is
one more verb, one more reply shape and one more thing to keep in step. The
refusal is the place the question is *already* being asked, and `naming` is
already the one home of both names — so `by_leaf` and `resolve` append what
could have been typed (`known_of`, bounded at a dozen names and then a count,
because a refusal is a sentence and not a listing). An empty set says so
outright, since *"nothing here answers to a project name"* is a different world
from *"you typed the wrong one"*.

It discloses nothing new. REMOTE §4's narrowing runs before this — a scoped
client's refusal lists that client's own registered workspaces and no others,
which is what makes naming them safe — and project names already cross to any
connected seat on every `/board` and `/balls` row.

**The taxonomy is the existing invariants, not a new concept.**

- **Actions** are the ops trail's rows (§4.2): everything that mutates a
  substrate. Their carrier is `boundary::Action` — one enum variant per
  gesture, parameters and all — and one chokepoint, `boundary::dispatch`,
  routes every variant to its §8 executor. Today's roster: the §8.2 short
  verbs (message, stop, **interrupt** — bl-a33d's send-and-interrupt, the one
  arm that composes two acts and so leaves two trail rows for one gesture —
  scan, close, assign, release, create, update — the last two carrying
  their whole payload as `verbs::edit::Create`/`Update` rather than as loose
  variant fields, bl-dbde, so the roster, the codec, the line and the executor
  read **one** vocabulary of what a ball is made of),
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
  Beside them, the §3.8 **mutating fan's** family (VISION §4.10, folded in
  bl-a33d) — one `Fan` variant over `fan::Verb`: `Spread` (N candidates off one
  pinned target), `Retire` (release one, discarding its ref only when
  retention says so), and `Deliver` (**Deliver candidate**, VISION V3.2 —
  accept one by the ordinary source-to-target delivery, bl-c2bd). It folds for
  the monitor's reasons exactly, and on a seam every layer beneath the roster
  already drew — `boundary::fan` holds the executors, `codec::fan` the
  spellings, `line::fan` the readers. The fold is what made room for
  `interrupt` in `action.rs`, which was **at** §12's cap: the split a full file
  demands is a real seam, and the seam was already there in three places.
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
  `Search`, `WorkDiff`, — bl-40ab — `Science`, the §3.9 attempt projection,
  which *composes* `WorkDiff`'s rows rather than restating them, `Help`, — bl-0164 — `ReadConfig`, `Marks`,
  `Providers`, — bl-dff8 — `Lineages` and `Models`, and — bl-6233, REMOTE §9
  step 1 — the §11 **inspector family**: `Transcript`, `Steps`, `Step`,
  `Files`, `Rail` and `Inbox`, the six addressed at a *conversation* rather
  than a workspace, without which no seat but the window could read a chat)
  and the chokepoint is
  `boundary::answer` — functions over the
  published snapshot (§7.2) plus the durable `ui.json`, *the snapshot
  derivation run without a frame*. Several read the world's **bytes**
  rather than this snapshot's derivations — `Search` (§8.5 below),
  `WorkDiff` (§5.1 #32, which asks the snapshot which balls a workspace
  claims and the project repos for the rest), `Lineages` (§9.3's browse,
  which is the workspace's own git) and the whole §11 inspector family
  (bl-6233, which reads the conversation's own `messages/`, `steps/`,
  worktree and inbox) — and all are answered straight
  through, because every seat that reaches the chokepoint is already
  off-frame. **The inspector family also reads the snapshot**, and that is a
  ruling rather than an accident: an in-flight model call's live tail is the
  snapshot's own `Stream`, so `Transcript` folds it on exactly as the window
  does. Answering the committed half alone would have been cheaper and wrong
  — two seats describing one moment differently is the divergence this
  chokepoint exists to prevent. The *pin* is not answered at all, and
  neither is any other fold: a fold is a view (below), so what changed with
  bl-44e9 is the **altitude of the answer**, never the rule. `Conversations`
  answers the whole descent forest with its per-row rollups and each seat selects
  the rows its own expanded set makes visible (`nav::convs::visible`); `Rail`
  answers notches carrying the budget **as of** each of them, so resolving a pin
  is reading one field off the notch the operator picked rather than folding a
  prefix. A commit, by contrast, *is* a selection where a query's subject is a
  tree — which is why `Query::Files` carries `at` and the pinned listing has a
  headless spelling, on the same footing as `Step { seq }`. The §9 config family's reads (bl-0164, extended by bl-dff8 to the
  lineage browse and the §9.4 roster) read the world through the
  same `Deps` `dispatch` takes instead, exactly as their writes already do
  (§8.5 below), which is why `answer` returns a `Result` and can refuse as
  `dispatch` can. The frame's view-models delegate to these same functions
  (`workspace_stats`, `conversation_names`, `delete_confirmation` are thin
  delegations), which is the parity mechanism: one implementation, two
  serializations. **`AppModel::conversations` is not among them any more**
  (bl-44e9): the §11 list is a `Reply::Conversations` the window reads over the
  wire like any other seat, so what the frame holds is a payload and a fold of
  it, not a delegation.
- **Views** never cross it: they are exactly §5.3's closed RAM whitelist (I6)
  — focus, scroll, tab selection, drafts — **and the §4.1 presentation
  durables beside them** (pins, collapse, zoom, panel sizes, seen
  watermarks). The ruling's own line decides the durable case: "switching tab
  doesn't" — durability does not promote presentation state into an
  operation. Views gain no boundary representation, by design.

**The paint side takes only what a `Reply` could carry** (REMOTE §9.4,
bl-1eb0). The read half of the sentence above needs a line of its own, because
`AppModel` sits on both sides of it: it may **orchestrate** — hold the focus
(§13.1), resolve it against the published snapshot, memoize a heavy build per
derivation (§7.2 `SnapMemo`), hand a search off and paint whatever landed — and
none of that crosses the boundary. What crosses into *paint* is a payload,
never the engine's derivation. Concretely: `src/shell/**` and the three
rendering modules beside it (`inspector`, `composer`, `inboxview`) may name
`AgentState`, `AgentMark`, `Flight`, `Tone` — the vocabulary a decoded reply
hands back — and may not name `GitTree`, `Agent` or `CommitNode`, which ride
nothing and never will (`rules/no-engine-tree-in-paint.yml`). The three had
paint-side answers minted for them: `Query::Agent` → `Reply::Agent(AgentView)`,
the §11 centre pane's whole read of its selection; `nav::convs::Titles`, the
§3.3 ladder's id→title input, which a seat builds either from the engine's
agent set or from a conversations reply's own rows; and `model_pick::ConfigTip`,
the config-lineage tip's two strings. The rule is not *which module a type lives
in* — it is **whether the wire can say it**, so a `pub` engine type is fine
wherever the codec spells it and banned where it does not.

**The GUI is one serialization of this surface, and it is a WIRE one.** The
shell's click-glue constructs variants and reaches the chokepoint — but since
bl-4841/bl-1747 it never reaches it in process: REMOTE §1.2 makes the window a
wire client of its own engine, so a click *posts* the codec envelope over
loopback and the receipt lands frames later (REMOTE §9.8), against the same
`boundary::dispatch` a phone seat reaches. `AppModel::dispatch` is **gone**: it
existed for the four gestures whose reply gated a frame-side fact, and those
facts hang off the receipt now (`shell::acting`). **The headless serialization is the `boundary::codec` JSON
envelope** — one flat object, `op` the discriminant — and both codec
directions plus the dispatch match are exhaustive over the enums, so **a new
gesture without a headless spelling fails to compile**, never review.

**The transport is deposit-based** (I4 — the litany inbox discipline applied
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
worker, polling the inbox (a latency knob; correctness is the deposit's
persistence, not the poll). It rides `Engine::boot`, which is the whole binary
since bl-7942 — a bare `yog` parks on it until a signal — so a deposit is
answered by the engine whether it was deposited by a seat, by an agent's own
bash or by an operator's terminal. `yog gesture '<json>'` is
deposit-and-wait sugar over exactly this path (validate against the codec,
deposit, poll, print the reply; exit 0/1 on the reply's verdict, 2 for an
envelope that never deposited, 124 when no consumer answered — the deposit
remains). The argv verb is a depositor, never a second dispatch
implementation (VISION §8).

**The wire is the second intake, and it opens onto this same room** (REMOTE
§9.5, bl-b6fa). `src/wire/` adds an mTLS listener to `Engine::boot` — beside
the consumer, and for the consumer's own reason: it rides the ENGINE, so a
*seat* reaches it exactly as a deposit does. A connection's request frame goes to `ConsumerCtx::answer` — the very
function the inbox poll calls — so the listener never sees an `Action` or a
`Query` and **there is no place a wire-only verb could be added** (REMOTE §3).
Two doors, one room: the inbox is the world's own residents' door (same
machine, same disk, disk is the bus) and the wire is the door for a caller
across a trust domain, whose whole authentication is the certificate it
presents. **Every seat is on the far side of it** (bl-7942): `yog seat` was a
terminal seat in this crate and is the seat crate's now, along with the window
it shared a transport with. **Absence WAS the off switch and is not since
bl-ae05** (REMOTE §8): a box with no material founds its own loopback trust root
at boot — one `openssl` recipe, the operator's own act performed on the
operator's own box — because every read a seat makes crosses this listener and
an engine with none is an engine nothing can look at. Material yog minted for
itself serves loopback only; an address naming anything else is written by an
operator, and that is the whole of the distinction.

**A stop is the engine's drop, and the whole mechanism is the catch**
(`src/engine/stop.rs`, bl-269a). The engine parked forever under the *default*
`SIGTERM` disposition, so a signal killed the process where it stood: no `Drop`
ran anywhere, the `Engine` was never dropped, its five threads were never
stopped or joined, and `Stream`'s drop — the one thing that SIGTERMs a piped
child politely and waits for it — never executed at all. Nothing had to be
*built* to drain, because dropping the engine already is the drain: every
thread it spawns owns the §7.2 shutdown shape (stop flag, park loop, a `Drop`
that unparks and joins), so an engine dropped at the end of a consumer pass is
one that finished the deposit it was answering and refused the next. So the
change is a disposition, a flag and a loop that ends — and
`Engine::park_until_stopped` takes the engine **by value**, which is what makes
parking-without-dropping unspellable rather than merely discouraged.

Three things it deliberately is **not**. It adds no verb, flag or config key:
the signal is the existing explicit signal. It adds no deadline of its own —
`yog.service` states one (`TimeoutStopSec=30s`, until now unused) and the
kernel's `SIGKILL` at the end of it is the only bound that cannot be
out-waited, so a second timeout in yog would be a worse copy. And it does not
touch a running turn: a turn is a *detached* `litany` driver in its own process
group (§8.1), which yog's exit has never killed and must not start killing —
under the unit it takes the cgroup's own `SIGTERM` beside yog's, and the pinned
litany catches that and deposits its branch's result with a `stopped` epitaph on
the way out (its ARCH §2.9). yog neither drains nor signals a turn; it stops
being in the way of one.

**One face, one mechanism** (VISION V5.4). There were two, and the window
consulted the same flag from eframe's loop — a drain on only one face would
have been exactly the second implementation V5.4 refuses. Since bl-7942 there
is nothing to hold apart: the flag ends `Engine::park_until_stopped`, which
takes the engine by value, so returning from the loop and dropping the engine
are one act.

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
- **The verb table is the single source.** `boundary::help::table()` — the three
  consts `ACTIONS`, `STANDING` and `QUERIES` composed into one roster of
  (verb, usage, summary, detail) — is what refusals print, what a seat completes or helps from,
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
deposit's reply file carries. The start family needed the frame's own typed
doors until bl-1747, for the aftermath rather than the act: the §3.4 workspace
adoption, the held start claim, the §3.3 mint seed a landed fire spends — which
a headless consumer must not do and a window must. It posts the ordinary
`Prepare`/`Prompt` gestures now and hangs all three off the **receipt**
(`shell::acting`), so the seat keeps its aftermath without keeping a door. And the terminal:
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
serve --help` booted the engine and parked, and `yog tool-control --help`
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
  does not make the world unsearchable, and it is never silent. A **compacted
  conversation** rides the same list (bl-fde5): its deleted span (§5.1 #12) is
  bytes that no longer exist to read, so the answer names the span — the
  surviving record and the compaction summary *are* searched, the summary's
  bytes riding the spliced marker — rather than posing as an answer over the
  whole conversation. No new field: a source the search could not read, named
  with why, is exactly what the channel already says.

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
  `config.toml` **in a named workspace's wall**, litany's `models.yaml`, one
  `workflows/<name>.yaml`, yog's `cadence.yaml`, or one file on a per-workspace
  lineage) and the destination
  **decides the pipeline**: `bz` validates a brazen draft (§9.1), brazen's
  provider table gates a litany-global one (§9.2), `litany config` commits a
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
one home, no drift, and every intake gates identically. The consumer's interim
(leave the table empty because brazen was unasked) is retired. bl-3f46 wrote
this for the §9.2 workspace-birth gate as well; that gate is gone (§9.2,
bl-00ee) and the seat-side plumbing that handed the GUI's cached rows into the
start flow went with it, so what asks today is the §9 config family, each
against the wall of the workspace whose file it judges.

**Two frame-only entries remain, and they are the §8.1 pattern, not exceptions.**
The three *file* editors' Apply buttons (§9.1/§9.2 and the `cadence.yaml` pane)
stay on their own `Editor`/`BrazenEditor`, because a pane holds a **long-lived
RAM draft** with a load-time snapshot and the §9 hash guard is over that draft:
it refuses when the file moved under the operator. A deposit has no such draft —
it states its whole text in one atomic instruction, so load and apply are
microseconds apart and the guard degenerates to the must-not-exist check a new
file wants. Both enter the same §9 pipeline, exactly as the §8.1 start
family's two typed doors are the arms the `Prepare`/`Prompt` gestures delegate
to: one implementation, whichever spelling asked for it, with the frame-only
state beside it rather than inside it (REMOTE §9.8: the window posts both
gestures like any other and holds their aftermath on the receipt). Every other
config seat — the lineage Send, the marks buttons, the picker's selection —
constructs a variant and posts it.

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
brazen and §9.2 litany-global editors' `Reload`, the hash-guard's own re-diff
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
words (*"no workspace in context"*) and an
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

**And the remedy is the seat's, not the sentence's (bl-e66f).** That refusal
read *"no workspace in context — focus one, or use the envelope"* until bl-e66f,
and it is shared: the line parser is one implementation, reached by a window's
composer and by `yog gesture` alike. At the terminal there is nothing to focus —
`boundary::sugar::argv`'s module doc opens by saying so, *"it holds no
selection … a line typed here states its targets outright"* — so the sentence
sent the operator to a control that seat does not have, while `--ws`, the flag
that exists for exactly this, was named nowhere in it. Worse, the flags were
named nowhere at all: `argv::usage` printed **only on a refusal**, and the
refusal for a *missing target* was the one refusal that did not print it, so the
way to learn how to aim a gesture was to type one wrong.

Two corrections, both keeping one implementation:

- **The shared sentence states the fact; the seat states the remedy.** What is
  missing is true at every seat; how to supply it is not. So `args::workspace`
  says only *"no workspace in context"*, and `argv::read_gesture` appends the
  seat's usage to **every** refusal it hands back — the line parser's included,
  where before it appended only to the two the flag reader raised itself. One
  rule, one place, and a refusal at this seat now always ends in the flags.
- **`--help` prints the seat's line around the shared answer.** The rewrite
  into `/help` exists so one answer serves both seats, which is precisely why
  the argv flags may not live inside that answer — they are one seat's fact. So
  `sugar::help_answer` is the seat's usage line, a blank line, then
  `help::render`, and `yog gesture --help` finally names the flags it aims with.

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
every seat's start shares the `Prepare`/`Prompt` bodies.

**Forwarding needs no verb.** "Read, answer, or forward" is three things to do
with a row and two gestures: forwarding an escalation is `Action::Message`
aimed at somebody else, carrying the row's own text. A `/forward` would be a
second spelling of a gesture that already exists (the §8.3 login precedent).

**What it deliberately does not cover.** The queue's rows are the §6 signals and
nothing else. Capability holds — a drone parked at a tool call awaiting a
verdict — are **bl-765d**'s `PendingHolds`, whose facts come from
`refs/litany/held/*` rather than from this predicate; when they land they extend
this query's signal set rather than opening a second queue beside it.

**One engine, and now one face (bl-f6fe, completed by bl-7942).** V5.1 was
verbatim *"headless mode is the same binary, minus the window"*, and V5.4
*"nothing here is a second implementation"*. `src/engine.rs` is where that
stopped being an intention: `Engine::boot` is the whole of a running yog — the
§5.2 startup sweep, the §7.1 roots, the model's first synchronous derivation,
and the derivation worker, watch bridge, gesture consumer and (since bl-b6fa)
the REMOTE §9.5 wire listener spawned beside it. It was two calls with two
repaint hooks while a window rode beside it; the window is the seat crate's,
so it is one call and there is no hook — nothing in this process has a frame to
wake, and a seat asks on its own cadence (`wire::ASK_PERIOD`).

**One engine per world is what that buys**, and it is why a seat never has to
arrange for a server: a second engine over one world — two pilots, two
sentries, two derivation workers — is the instance-coordination shape §14
rejects. That is also why a bare `yog` from inside an agent seat is refused
(`world::seat::boot_refusal`, §16.4): the shim passes argv through verbatim, so
the word that used to open a window on the operator's desktop now founds a rival
engine on the world the agent is already speaking to.

Before `engine.rs`, `main.rs` carried the assembly **twice**, in the one file
`tarpaulin.toml` excludes, so the copies were free to drift and no test could
notice; `src/engine/tests.rs` boots an engine into a hermetic world and reads a
deposit's answer back, which is the V5 claim end to end.

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
enforcement point is **litany's own tool-control seam** (litany ARCH §3.3
*Tool control*, bl-de6d — in the 0.0.6 pin): `workflow.yaml`'s
`tool_control:` names one executable consulted before every granted tool
invocation executes, answering `pass` / `refuse` / `hold` on stdout for the
`tool_use` + `role` + `agent_id` JSON on stdin, failing closed. yog's job is
to *be* that executable and to own every fact it reads.

- **The control is a world-tools shim** (§16.4's re-exec pattern, seated
  beside `bl`/`litany`/`bz`): the yog binary in a consult mode, addressed by
  **absolute path** in the authored block — never PATH-resolved, so no host
  binary can shadow it. It is **side-effect-free per consult** (the seam
  demands idempotence — release is re-adjudication): it writes nothing, ever.
  Two moves per consult: **classify** the invocation into VISION §4.11's
  effect vocabulary (intrinsic class map for built-ins; the workspace
  ruleset over `bash` commands, unmatched = open-world; `cd`/`apply_patch`
  judged against the writable root — the bound attempt worktree plus the
  agent worktree, the agent's cwd read from litany's
  `refs/litany/cwd/<agent-id>` mark, the ball worktree computed by the
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
- **Fact homes, one each.** The *request* is litany's hold mark
  (`refs/litany/held/<agent-id>`), which unlike the other four `refs/litany/*`
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
  lawful writer, the §9.3 `litany config` drive, reached by the existing
  `ApplyConfig` gesture on a `Branch` destination (`/config branch default
  capability.yaml …`). The capability boundary therefore adds a *reader* and
  no writer at all, which is what keeps its whole surface one action wide.
- **Authoring rides an existing write class — but not the one the ruling
  named (bl-fec8, verified against the pin).** The ruling authored the block
  into `LITANY_HOME/template/workflow.yaml`; three premises behind that turn
  out to be false. `litany prime` never seeds `template/` — it is an
  *override* root, absent by default ("policy lives in config, not code", at
  litany's own constant). The override is a whole-file `fs::copy`, not a
  merge, so a `workflow.yaml` carrying only `tool_control:` would delete
  `events:` — and with it every dispatch — from every workspace born after
  it. And authoring a *complete* override needs litany's embedded default,
  whose module is private: there is no lawful read of it. **So the control is
  authored per workspace, onto `config/default`, at every start**, through
  the one lawful writer of `config/*` — the §9.3 scripted-editor `litany
  config` drive — with the base taken from the workspace's own committed
  `workflow.yaml`, which is exactly what litany put there. yog still never
  writes inside a workspace. This is *stronger* than the template route
  rather than a retreat from it: the template would only have reached
  workspaces born after it, while this reaches every workspace on its next
  start, and every agent forked after that commit is controlled. **Since
  bl-e654 it also reaches the agents already running, and the sentence that
  stood here is reversed**: it read *"agents already running keep the policy
  they froze — that is litany's per-branch freeze, not a gap authoring could
  close"*, and follow-the-tip closes exactly that gap. `workflow.yaml` is read
  from the commit a conversation **follows** (§9.4), re-resolved at every step
  boundary, so a `tool_control:` shim authored onto `config/default` adjudicates
  every running conversation on that lineage from its next step — no retarget,
  no restart, nothing per agent. Convergence is by comparison, not
  memory: the authoring is a fixed point, so a tip that already names the
  shim reads one file out of git and spawns nothing. A drive that fails
  aborts the start (`["yog-step","control"]`, Z5) rather than handing back a
  workspace whose drones nothing adjudicates.

  **The drive is shared, the fixed points are not (§3.7).** This block and
  §3.7's `instructions/**` manifest glob are two control files of one yog
  policy, so `start::ensure` collects whichever drifted and converges them in a
  **single** `litany config` pass — one checkout, one commit, one ops row, and
  the same abort. Each author owns only its own file's fixed point and knows
  nothing of the other.

  **The workflow fixed point holds one other block, and holds it empty: a
  yog-dispatched conversation has no budget** (bl-56af). litany's
  `workflow.yaml` may carry `budgets:` — `max_total_tokens`,
  `max_wall_seconds`, `max_depth` (litany ARCH §6) — and every axis of it is a
  **whole-tree** consumable: one allowance a root and its entire descent spend
  together, checked at every model-call boundary, exhaustion writing
  `refs/litany/budget-exhausted/<branch>` and ceasing the loop. litany's
  pre-`0.0.11` template shipped that block *set* (`max_total_tokens: 2000000`,
  `max_wall_seconds: 3600`, `max_depth: 4`), so every workspace born before
  that release committed those numbers to its `config/default` and caps every
  agent that follows that lineage — an hour of accumulated tree-wall, and a
  dispatch tree four deep, both of which bind on ordinary work. litany has
  since retired the seed, but **a template only ever reaches workspaces born
  after it**, which is this section's own argument for authoring per workspace
  rather than into `template/`. So `authored` also **strips** any top-level
  `budgets:` block and leaves one comment line stating that it did — same file,
  same transform, same drive, no second drift entry, and a fixed point because
  a file with no such block strips to itself.

  **Unconditionally, not down to a smaller number.** A whole-tree ceiling ends
  a conversation that is still working, and early termination destroying
  uncommitted work is the expensive failure §3.5 already reasons about — the
  reason yog's own ceiling gates a *birth* and never a live drone. And yog
  already has that ceiling: the `ui.json` `ceiling` key (§4.1), denominated in
  dollars rather than tokens or tree-seconds, absent by default so deleting the
  key deletes the gate, and spoken on the V4 board with the gate's own words
  ahead of the spawn it will bind. Two ceilings over one concern is the second
  representation that drifts; the one yog authors is the one it can remove.
  **This dissolves the config-tab question rather than deferring it**: there is
  no per-conversation budget for the §9.5 pane to surface as an option, because
  a control over a value the next start deletes would be a knob that lies.
  `workflow.yaml` stays browsable as raw text like every other file in the
  commit a conversation follows, so nothing is hidden from an operator who
  reads it. **The strip converges the running conversations too, and that
  sentence is a reversal** (bl-e654): it read *"agents already running keep the
  ceiling they froze — litany's per-branch freeze again, not a gap this could
  close"*, and follow-the-tip closes it. This was checked at the pin rather than
  assumed — litany's step loop calls `resolve_worker` at the top of every
  iteration past the first and takes `budgets:` off *that* resolution, so the
  ceiling a running conversation is judged against is the followed commit's,
  re-read per step. The strip therefore converges every conversation on the
  lineage at its next step, not merely every one forked after it.

  **And a conversation the old ceiling already killed is not dead — the strip
  alone reaches it (bl-d710, amended bl-e654).** The strip converges
  *workspaces*, so the ball asked what becomes of the branches marked before it,
  and the answer is a gesture yog already paints rather than anything to build.
  **The mark is a record, not a gate**: `budget::check` derives the tree's spend
  live at every model-call boundary and compares it to the **resolved**
  `budgets:` (litany ARCH §6, "no stored counter"); it never reads
  `refs/litany/budget-exhausted/*`, and every axis is an `Option`, so a config
  with no such block bounds nothing. **So the raise lands with no verb at all**:
  the block the strip removed is gone from the very commit the killed branch
  resolves at its next boundary, and there is nothing left to compare against.
  Under the freeze this needed `retarget` to move the branch's fork point onto
  the stripped ref first; that step is deleted, and with it the last reason the
  §9.4 clause painted for these branches — they are not apart from the lineage,
  they follow it. **The park is only the missing launch**, and that half is
  unchanged: exhaustion is an ordinary terminal state, and litany's exit
  protocol declines to self-relaunch on that epitaph alone (an epitaph-spam
  cycle against a hard ceiling), while an ordinary deposit probes and launches
  with no epitaph gate at all — so the message *is* the next step. The
  operator's whole remedy is therefore **a message, and nothing else**. Two
  residuals keep `retarget` a real answer rather than a dead verb, and both are
  §9.4's: a conversation **held** on a diverged lineage resolves its fork commit
  and so still carries the old ceiling until the lineage is settled, and a
  conversation following some *other* lineage was never converged by a write to
  this one. Fork — the hatch the ball guessed at — is the *discarding* answer
  beside them and always worked; it is simply the worse one, since it leaves the
  history behind. What bl-d710 changed is that the §11 mark says so
  (`theme::mark_badge`), the one mark of the five that named its cause and no
  remedy. **Clearing the mark
  is still not yog's** (§5.1 #14): a revived branch keeps it, as the record of
  what happened, and the acknowledged §6 signal stays acknowledged.
- **A hold is a parked drone, not a deadlock; a deny is a decline, not a
  stop.** The parked branch is derived state (the mark plus the unpaired
  tail) surfaced as an attention item naming the tool, an input summary,
  the computed class, and the reason; the ball renders waiting at its gate.
  Answering is a boundary action (§8.5 variant, headless spelling by the
  codec) that writes its ops row and fires `litany advance` — the explicit
  user action continuing (I7); litany re-adjudicates and the branch moves.
  **No enforcement path calls stop**: stop mid-tool-window wedges the
  branch permanently (litany bl-b98d), so refusal is always in-band or a
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
  which is litany's mark. `pass` and `refuse` both **release** (one executes,
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
- **The confinement gate is a birth gate, at the two doors a drone is born
  through** (VISION §4.11 item 8): `dispatch::prompt` (every start, above the
  §3.5 spend ceiling — a birth that will not happen has no spend to judge) and
  the `Fork` arm (every attempt). Since bl-bca4 yog wires **one
  platform-explicit backend**: Linux's bubblewrap (`bwrap`), shelled as a
  subprocess exactly as §16.7 mints certificates by shelling to `openssl` —
  no crate, no `unsafe`, and its absence on a box *is* the derived
  unavailability. The gate's availability is **derived at each birth, never
  stored**: a probe runs the exact sandbox shape a wrap spends, and a
  `confinement: required` workspace fires only when it passes — on any other
  OS, or under a failing probe, the standing refusal names exactly why. Past
  the gate the fire is **wrapped**: the backend argv rides yog's own litany
  spawn seam (`Bound::at` and the prompt door — the same two folds the §16.2
  wall takes, so every workspace-bound litany spawn a policy confines is
  confined by construction, revived drivers included), and the wrap is
  *unconditional under the policy*, so a backend that vanishes between gate
  and spawn fails the exec loudly, naming `bwrap` — never a silent fallback.
  The **support boundary** is explicit: *filesystem writes* are clamped — the
  host tree re-bound read-only, writable exactly the derived set: the
  workspace, the composed world root, the host `/tmp`, and the **bound project
  repo** (the paragraph below). *Process access*, *environment* and *network*
  are **not** clamped, each for a named reason: a pid namespace dies with its
  init and litany's short verbs detach drivers that must outlive them (its
  ARCH §2.9; `litany stop` also signals by host pid); the env is already
  composed explicitly at yog's spawn boundary; and the drone's model calls
  are HTTPS from its own tree — unsharing the net severs the loop from its
  brain. An absent layer still gets no affordance anywhere — the only surface
  it earns is the refusal that names it. V4's armed loop (bl-66fb) reads this
  same gate.

  **The bound project repo is the set's fourth member, and it is the §3.2
  claimant join** (bl-34b1). bl-bca4 shipped the first three and stated the
  consequence as a v1 bound: a project repo lives outside the world, so a
  confined drone found it read-only and the ball rung's own `bl close` — which
  advances a ref *in that repo*, and whose `work/<id>` checkout keeps its gitdir
  inside that repo's `.git/worktrees/` — could not deliver. The bound is lifted
  and nothing is stored to lift it. A workspace encodes no project path (§3.5),
  and only the prompt door ever sees a payload — `litany message` and `litany
  advance` detach-launch a driver through the same `Bound::at` fold with the
  workspace and nothing else — so a project carried as a *birth parameter*
  would have confined every revival more tightly than the fire it resumes,
  which is the shape that produced the bound in the first place. Instead the
  set reads the derivation the §4.11 writable **root** already spends
  (`control::root::claimed`): the last `bl claim <id> --as <name>` row on yog's
  own ops trail, stamped with this workspace's leaf, whose `cwd` **is** the
  project the claim ran in. The trail is durable, so both doors derive the
  identical set — one rule, no payload, no field to go stale, and a third
  consumer of one claim rule rather than a second copy of it. Two bounds
  survive, both inherited from that same rule and stated once for both readers:
  a workspace that never claimed through yog has no project in its set, and a
  ball an agent claimed **for itself** mid-conversation leaves no yog-side row
  (§3.2's limit), so its project stays read-only. And one rule joins the set:
  **what is not there is not bound** — `bwrap` refuses a `--bind` whose source
  is absent, so §3.5's legitimate *orphaned-project* state (the clone gone,
  workspaces unaffected) narrows the set instead of failing every birth in the
  workspace that claimed it. It can only ever narrow.

  **And it is a fact yog owns, not one yog trusts** — a distinction worth
  spelling here, because `ops.jsonl` lives under the world root and the world
  root is *in* the writable set, so a drone that forged a `bl claim` row could
  name any directory as its next birth's project. That is not this member's
  hazard and not a regression: the §4.11 writable **root** has read the same
  rows out of the same writable place since bl-fec8, so a drone that can write
  the trail already owns its own adjudication — and the threat model above says
  exactly this, the OS layer bounding the write-*accident* class with
  adversarial evasion out of scope by construction. What does hold is the
  invariant the root states: the derivation reads no fact the agent is *meant*
  to control. No `cd` mark and no payload is in it.
- **What this is not.** Not total confinement: the host tree stays *readable*,
  the network stays open, and the deliberately-shared brazen credentials
  (§16.2) stay reachable by an *allowed* invocation — the OS layer bounds the
  write-accident class, rule classification bounds the rest of accident, not
  adversarial evasion. VISION §4.11 item 8 carries the honest threat model,
  the alignment monitor covers drift, and litany's reserved v1.1 sandbox seam
  (its ARCH §3.6 — a per-tool capability clamp, finer than this per-drone
  envelope) remains later and litany's own.

Shipped-state landed with bl-fec8 (shim, classifier, fold, authoring), bl-765d
(policy config, the hold-answer variant, the sixth attention signal, the
confinement refusal), bl-94b4 (the floor writer above — the monitor's revoke
rung over the same fold, whose *reader* `Answers::floored` came with the shim),
bl-bca4 (the Linux backend behind that refusal — probe, gate and wrap) and
bl-34b1 (the writable set's fourth member).

### 8.7 The birth policy (VISION §4.2 / §4.6, bl-380f)

**A ball's tags select the config lineage its drone is born on.** VISION §4.2
promises *"Skills are seeded at spawn, keyed on ball tags. A drone's context is
fresh exactly once; that is the one moment a standard cannot be crowded out.
yog selects skills and model from the ball's tags at fire"*, and §4.6 fixes the
shape: *"Policy table, yog config, severable. No crate below yog ever names a
model."* Until bl-380f neither existed — `BallSpec` carried id, title, body and
join, so a fleet birth read the whole ball off the snapshot and dropped its
tags at the payload.

**The lineage is the policy, so there is no table.** A litany config commit
already *is* the (model, skills) pair yog would otherwise have to name
separately — `providers.yaml` holds `roles.worker.{provider, model}`,
`descriptions/skills/**` is the skill set the tools composer offers, and
`manifest.yaml` says what composes — and `litany prompt --config <name>` forks
the new agent off its head (litany ARCH §2.3, in the pin since 0.0.6). So the
whole mechanism is one sentence: **a ball tagged `deep` is born on
`config/deep` where the workspace has one.** No new file, no new verb, no new
flag, and nothing below yog names a model.

The four questions a policy layer has to answer, answered without a surface:

- **Conflict — the ball's own tag order.** The first tag naming a lineage wins.
  Tags are an ordered, operator-authored list, so the precedence is already
  written where the tags are; a priority column or a longest-match rule would
  be a second home for one fact and the two would drift.
- **Default — no match, no flag.** An untagged ball, a ball whose tags name no
  lineage, and the bare/path rungs are one path with different inputs, not
  three cases: the selection answers `None` and the fire omits `--config`. The
  default is *litany's* `config/default`, never a word yog spells.
- **Severability — the ref is the config.** Installing a policy is
  `litany config <ws> <tag> --from default` (or §9.3's editor drive); removing
  one is `git branch -d config/<tag>`. Removing a default deletes config and
  edits no code, which is the test stated the right way round.
- **Authority — the ball, once.** Nothing is mirrored into yog state. The tags
  ride the §3.4 payload from the board row that carried them, are read by
  `start::lineage::select`, and are gone.

**Resolved once, consumed twice** (`src/start/lineage.rs`, called from
[`prepare`](#81-start-the-composite-verb--34s-axes-as-argv)). The answer
governs two acts, and they must not be able to disagree:

1. **§8.6's policy convergence.** `execute_ensure_workspace` authors the
   capability control and §3.7's instruction glob onto **the lineage this start
   will fork off**, not onto `config/default`. Converging `default` while
   birthing on `config/deep` would make a tagged birth the one birth nothing
   adjudicates — the §4.11 confinement silently absent on exactly the drones an
   operator gave a special policy to. §3.7's filename policy
   (`instructions.yaml`) follows for the same reason: the manifest that
   composes the frozen documents is authored on the fired lineage, so reading
   the filenames off another one would let one lineage's answer compose
   another's files.
2. **The fire's `--config`.** `prepared.lineage` rides the one argv beside
   `--cwd` and the `--pin` specs, spawned and logged from the same list (§3.3).

Deriving it separately in each would be two homes for one fact, and a lineage
created between the two reads would converge one branch and fork another. So
`Prepared` carries it — the same shape and the same reason as `binding`: a
value the engine minted, carried back verbatim by a seat that never reads it.

**What this does not close.** VISION's ledger row G6 asks for *model by
complexity heuristic, not tag map*, and §4.6 is explicit that tag→model *"is
the functional heuristic and stays available"* while the target is one rung up:
a complexity estimate written at decomposition time, tuned against the
estimate-outcome-spend triple. The policy **layer** that row calls missing now
exists and a complexity estimate is just another tag; what is still missing is
the **dataset**, which is G5's join (§4.5) and not this section's.

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

**§9.5 amendment.** The pane around this editor is now controls over facts, and
the raw TOML draft is folded behind them. The editor below is unchanged and
stays raw for the reason this section already gives, restated as §9.5's first
justified fallback.

**§9.1 amendment (bl-20cb): the pane references the roster, it does not paint
it.** The §9.5 amendment first spelled that as the effective provider table
itself — row name, `auth`, credential presence — rendered here as read-only
rows. That put the same ten `row_views` sentences on two surfaces, and this was
the copy that could not be acted on: *"signed in"* is an answer from the
credential store, and a blocked row's own sentence (*"api-key provider — set the
key in Config"*) was pointing at the very pane it was painted in. **The roster
has one seat, the §8.3 Login tab** — the surface that carries the sign-in verb
owns the rows the verb applies to. What stops this pane being *blind* is what
the file itself owns and Login does not state: how many rows it ends up routing
(counted from brazen's own answer, never pinned), and the standing hint that the
built-ins are in that count and not in this file. The rest is one control that
focuses the Login tab — the same "name the remedy, carry the thing that goes
there" shape §9.4's credential fault takes (bl-91f1), spending the one tab-focus
gesture and growing no second way to open Login.

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
  never land, so `bz` (and every litany loop calling it) never breaks.
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
  about to hand to every litany loop are literally the same code. yog still
  declares no TOML dependency of its own; the one `toml` in the graph is
  brazen's, reached only through brazen's API. What did die is the phase-1
  `name = "…"` line scan that stood in for parsing (it fed the login rows and
  the credential-presence rows): both now read brazen's own `--list-providers`
  projection.

### 9.2 litany global config (`models.yaml`, `workflows/*.yaml`)

One editor per file; Apply = hash-guard + temp-in-dir + rename (litany declares
these hand-edited; yog is the hand, minus torn writes). yog still adds no YAML
dep. New workflow = same path, new name; templates copyable. **Nothing here
judges the file's contents** — see the retired gate below.

**§9.5 amendment: `models.yaml` is edited as controls, not as text.** Each
entry's declared fields are a typed row — since bl-3ffa **two of them**:
`model_id` a scalar field and `context_window` a bounded number, read from and
written back into the very draft this Apply commits. `workflows/*.yaml` keeps the
raw editor (§9.5's second justified fallback: litany's workflow DSL is litany's,
and yog holds no grammar for it), which is the general path with an empty schema
rather than a branch on which file is open.

**The provider gate, and why it is gone (bl-53be, retired bl-3ffa).** The
original text said "no validator exists … the operator's risk is identical to
`vi`", and a shipped `models.yaml` promptly offered two Claude models on
`provider: anthropic` — a row that was uncredentialed, against a table nobody had
asked. `models.yaml`'s own header stated the contract it was breaking:
*"`provider:` on each model is a brazen provider-row NAME (§4.1) — endpoints,
auth, and wire dialects live in brazen's own config"*. That made one field
checkable without parsing YAML and without a second authority on anything —
brazen publishes the row set (`bz --list-providers`, the linked projection of
§16.7 W10) — so an entry naming a row brazen did not have was refused on Apply,
every offending entry named and nothing written.

**It was right while a role's model resolved THROUGH that declaration, and the
declaration stopped resolving anything.** litany retired the `models:` table
(its bl-35e2) and bl-d9cb re-pointed the picker at `providers.yaml`, whose
`roles.<r>.provider` is now the single home of the pointer. What was left was a
gate over `models.<id>.provider` whose **only remaining reader was the refusal
itself** — `declared` → `unknown_rows` → this Apply, and nothing else in either
program looked at the field. Two representations of zero facts, and worse than
inert: the gate could refuse an Apply whose whole purpose was correcting
`context_window`, the one line anything reads, on the strength of a dead one it
had no way to fix except by hand. bl-3ffa deleted the field from the write, the
`capabilities:` field beside it (no reader in either program, ever), their §9.5
controls, the chain, the gate, its `Rejected` arm and the Apply hover that
promised the refusal. The section's original posture is its posture again: the
operator's risk here is `vi`'s, and a future `litany config --check` still slots
into the same pipe.

**Nothing migrates, because reading was never the problem.** The grammar is
anchored line reads (§9.4), so an operator's existing four-field entry keeps
parsing exactly as before and `context_windows` still keys on `model_id` wherever
one is written. **yog writes less and reads the same** — that is the whole shape
of the change, and it is why a file hand-authored to the older block needs no
edit and gets no warning.

**Where the row judgement lives now.** The `is_unknown_row` question — does this
name a row brazen has? — survives at its three real sites, all of them over the
LIVE pointer: §9.4's pick gate, §9.4's role marks, and §9.5's `provider` control
over `roles.<r>.provider`. With it survives the principle this section stated
under the retired gate, which was never about the gate: **an empty provider table
judges nothing.** An empty answer from `--list-providers` means brazen could not
be asked, not that no rows exist, and no surface may refuse on the strength of a
question that went unanswered. The other non-special-case (*"the gate runs over
every §9.2 file, because a `workflows/*.yaml` declares no `models:` block"*) is
deleted rather than re-sited: with no gate there is no per-file difference left to
dissolve, and the three plain-file destinations run one unjudged pipeline.

**The gate's other retired site: the world's workspace-birth template (bl-c3a9,
retired bl-00ee).** The gate had a third site before it had none — a judgement
made *before*
`litany new`, over `<world>/litany/template/providers.yaml`, the file `litany
prime` authors and every `litany new` commits as the workspace's first
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
google, ollama, claude-code`). That is the very posture this section
forbids above — *"no surface may refuse on the strength of a question that went
unanswered"* — arrived at from the other direction: not an empty table, but a
table that is merely **early**. It is worth reading beside the gate's own
retirement, because the two failures rhyme: this one judged the right field at a
moment with no answer, that one judged a field with no consumer at all.

**So the judgement is not moved, it is already elsewhere.** yog has a designed
seat for "this conversation's provider is not usable", and both halves of it
read a wall that exists: the §9.5 pane faults `roles.<r>.provider` with the same
`is_unknown_row` the moment the workspace's own `providers.yaml` is rendered,
and a row still dead at the fire surfaces as the §8.3 auth-shaped step failure
with Login one click from the wound (§11 altitude 1). Birth now judges only what
birth can see. The template still needs no editor for the same reason as before —
its contents are litany's seed and litany's to fix — and yog reads it not at all.

The reader is the §9.4 anchored block grammar applied to the other file — the
same primitives `roles:` is read through, so `models.yaml` has exactly one
reader in yog.

**And since bl-d9cb the `models:` block is yog's own table, not litany's.**
lernie 0.0.10 retired it (litany's bl-35e2): the file's whole load shape there is
one optional `adapter:` field, and a leftover `models:` block *"is ignored on
parse"*. Two consequences, and bl-3ffa is the second one carried to its end. The
§9.4 picker no longer writes here at all — see §9.4 — so **this surface is the
only writer**, through its Declare control and the §9.5 typed rows. And the block
is now **one fact wide**: `context_window`, the §5.1 #35 fullness denominator, is
the only thing anything reads out of it, so it is the only thing the Declare
control writes and — beside `model_id`, which is how that number is keyed when an
entry is filed under an alias — the only thing §9.5 offers a control over. The
entry yog authors is `<model-id>: { context_window }` under a comment naming the
figure the number feeds.

**Which leaves an open question this section deliberately does not answer.** If
the block is one number that only yog reads, its home may not be a litany config
file at all (bl-d9cb raised it and kept the file). The argument for staying is
that the operator already hand-edits `models.yaml`, the read costs one watch root
nobody can perceive, and a new file would be a second place to look for one
number. The argument for moving is that yog's own facts live under yog's own
state root (§7.2's `cadence.yaml` is the precedent). Nothing is blocked either
way, and a move is a migration where this ball was not.

### 9.3 Per-workspace config branches (the scripted `$EDITOR`)

Browsing: `for-each-ref refs/heads/config/` + `git show <ref>:<path>`
(read-only, via the existing env-scrubbed `git_tree::cmd`), including the config
commit each agent's conversation actually resolves (§5.1 #17, `policy follows
config/<name>, now at <short-oid>` — or `policy held at <short-oid> — <n>
diverged config lineages` where the lineage has forked, §9.4).

**§9.5 amendment: the pane picks, it does not type.** The lineage is a dropdown
over the branches that exist (ending in *new lineage…*, the escape that reveals
a name field — §9.4's "each list ends in its own escape"), the file is a
dropdown over that commit's own `ls-tree`, and **Load** fills the body from
`git show`. A `providers.yaml` so loaded renders as typed role rows (§9.5); every
other path in a config commit — `souls/**`, `descriptions/**`, `workflow.yaml`,
`manifest.yaml`, `version` — is prose or litany's own schema and keeps the raw
body (§9.5's third justified fallback). Before this the pane authored a config
commit from a free-text branch name and a free-text path over an **empty**
buffer: a blind write against a file nobody had read.

**The reads are gestures, not frames** (§7.2, bl-ee0a). The listing used to
spawn `for-each-ref` inside `App::update`, once
per frame. It is now read when
the Config pane opens — the same read-on-demand gesture §9 already uses for the
file editors — when the lineage selection changes, and after a `litany config`
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

Editing — `litany config <ws> [name]` is the only lawful writer of `config/*`
and is $EDITOR-interactive, so yog drives it. Since bl-3f46 the drive is the
boundary's `ApplyConfig` on a `Branch` destination (§8.5), carrying the
workspace, the lineage, its origin, the checkout-relative path and the file's
whole text — so the pane's Send, `/config branch <lineage> <path> <text…>` and a
deposit are one gesture. The steps it runs are unchanged:

1. User edits the branch's files in RAM buffers; Apply writes the full
   drafted file set to `$XDG_STATE_HOME/yog/stage/<nonce>/`.
2. Spawn `litany config <ws> <name>` (plus `--from <src>` / `--orphan` when
   forking) with `EDITOR="<yog-binary> --editor-apply"` and
   **`YOG_EDIT_SRC=<staging-dir>` in the environment** — the staging dir
   rides in env, the checkout path arrives as the shim's argv, because litany
   composes the editor line through `sh -c` and its exact arg-passing shape
   is a flagged open question. The shim tolerates both `$EDITOR <dir>` and
   per-file invocation.
3. `yog --editor-apply` (a tiny non-GUI mode of the yog binary; its copy
   logic is a pure, fully-tested lib function) copies **only the drafted
   files** over the materialized checkout — **never a full-tree sync**:
   `litany config` has just refreshed `descriptions/**` from the data-root
   pools at commit time and the shim must not clobber that. Exits 0; litany
   commits and tears down. Empty diff is declined by litany; surfaced as "no
   change".
4. Staging dir deleted on completion; leftovers swept (§5.2).

**Diligence task 0 for this feature: source-read litany's exact `$EDITOR`
invocation shape** (how the checkout path is passed through `sh -c`) before
building the shim, and record the finding in the task.

This is the only path that advances a config branch, honoring "never write
inside a litany workspace except via the litany CLI" — yog writes only its own
staging dir; litany performs the commit.

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
at all**. Nothing was clickable, so nothing was written (no `litany config` op
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
model brazen does not list. That entry can assign an *unserved* model — the
operator's own call, on a row brazen has. It cannot assign an unroutable one, but
**not because the row beside it is brazen's** — that was the reasoning, and
bl-3d22 falsified it. The row's
existence is checked *and so is its protocol's capability*, below.

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
assign `g`, `gp`, `gpt`… every one of them a `litany config` commit on the
workspace branch. A half-typed id is not a choice.

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
  listed for a row brazen has, so the assignment a pick writes can never name a
  dead row, and the role mark it produces can never be faulted.

  **Row existence is not request-shape compatibility, and this claim used to
  assume it was** (bl-3d22). `bz --list-providers` ships `claude-code`, whose
  `claude_code` dialect is single-turn and **tool-less by contract**: its encoder
  rejects every request with nonempty `tools`. Every yog turn has nonempty tools
  — `tool_host::Injection::tools` declares the `clients` tool unconditionally and
  litany splices the host injection into every canonical request — so
  `/model worker claude-code <id>` advanced both config halves and then failed
  the next worker start before any network call, on
  *"claude_code carries no tool declarations; use the `anthropic` row for
  tools"*. A role's own `tools: []` buys no exemption, because the injection is
  not the role's election.

  The capability is therefore judged too, and **from the protocol, never from a
  row name**: the effective table's `protocol` column carries brazen's own
  `ProtocolId` spelling, `ProviderRow::tools_blocked` parses it back into that
  public closed enum through brazen's serde rename and answers over a **total
  match**, so a new upstream dialect fails to compile until its arm is added, and
  a spelling this build cannot name is *no answer* rather than a refusal. The
  picker's provider list shows such a row with its reason and does not offer it,
  which is how this invariant is kept true rather than weakened; the row is
  **not** removed from the catalog, because the VISION §4.9 alignment monitor's
  check is structurally tool-less and pins a model rather than a role.

  brazen projects no capability column of its own — `--list-providers` serves
  `name`/`protocol`/`auth`/`credential`, and the `Protocol` trait that owns the
  rejection is crate-private. **The upstream ask is already filed as brazen
  bl-5053** (per-row capability declines on the read surface, derived from the
  protocol's own reject arms and never a config field); until it lands, the total
  match over `ProtocolId` is the closest derivable thing, and it is keyed on
  brazen's enum rather than on a table of names yog invented. That ball was filed
  off this same defect seen once before and yog never gated on its own side,
  which is why it recurred — the local gate is not a stand-in for the column, it
  is the half yog owns.

  **The gate is not the whole remedy, because a config can predate it** (bl-5252).
  This bullet gates the picker; it reaches no assignment written before the
  gate existed, and none written by hand through the §9.3 editor, which is the
  operator's own authority. Such a step still dies at encode — and it did so with
  a §7.3 banner offering Dismiss and nothing else, because the classifier there
  keyed on litany's `Config` wrapper while brazen stamps every dialect decline
  `ErrorKind::ParseInput`. The same `tools_blocked` judgement now answers that
  failure too, reached from the dialect the decline names rather than from a
  column (§8.3 rule 6): one match, two ways in, so the banner's reason is the
  sentence this control paints.
- **`plan` refuses a row brazen lacks** (`PickError::UnknownProvider`), before
  the file is touched. It is the picker's own gate and always was the only one
  that could be: the §9.2 Apply gate judged a different file, since bl-d9cb a
  pick did not go near it, and since bl-3ffa it does not exist — this is where a
  written provider row is judged. Since bl-3d22 it takes the effective table **whole**
  rather than its name column, because the row gate asks two questions and only
  one of them is answerable from a name.
- **`plan` refuses a row whose protocol cannot carry a yog turn**
  (`PickError::Incapable`, bl-3d22), in the row's own words, at the same point
  and for the same reason: a config that would die at the next step's encode is
  refused instead of committed. A row the table does not carry is an unknown row,
  not an incapable one — there is no protocol to judge — so the gate needs no
  case of its own for it, and an unanswerable table gates nothing.
- **`plan` refuses an id the block grammar cannot hold**
  (`PickError::NotAnId` — blank, or carrying whitespace / `:` / `#`). Only the
  custom entry can produce one; a listed candidate is a string brazen itself
  printed.

**And one thing the same column says that is NOT a refusal** (bl-671d). A
dialect can be tool-capable and still leave a fact the turn depends on to the
server. `ollama_chat` is the case: brazen's encoder maps the output cap to
`options.num_predict` and emits **no `options.num_ctx` at all**, so the Ollama
server's own default context governs every yog turn — not the model's capacity,
and not any `context_window` `models.yaml` declares (that number is the §5.1 #35
denominator, reaches no request, and since bl-d9cb no pick writes one). A drive through the offered row
reached local inference and produced nothing usable: 4095 input tokens on the
platform payload, one generated token, `finish_reason: length`, against a model
whose own context was 262144.

This is **stated, not gated**, and the difference is the whole ruling. yog cannot
see what the server was started with, so a refusal would be a refusal on the
strength of a question that went unanswered — the same discipline
`is_unknown_row` applies to an unanswerable table — and it would refuse a
correctly-raised server. `ProviderRow::context_caveat` is therefore a caveat
beside a **selectable** row, with the remedy on its hover, and `plan` does not
consult it. That is not the dead-end warning this section retired, because it
carries the operator's next move: the row gets an explicit context in the
workspace's own brazen `config.toml` — `unsupported_body_keys = ["max_tokens"]`
plus `body_defaults = { options = { num_ctx = …, num_predict = … } }` — which is
config in the file that authors a row, and the operator's number rather than a
default yog guessed.

**Both halves of that recipe are load-bearing, and the reason is the second half
of the defect.** The encoder inserts its typed `options` first and folds config
passthrough with `or_insert`, so a `body_defaults` `options` beside a typed
`max_tokens` is dropped **whole and silently** — the obvious fix does nothing and
says nothing. Clearing the typed cap is what opens the valve, which is why the
cap is restated inside the object handed over. All three facts — the missing
`num_ctx`, the silent drop, the recipe landing both limits — are asserted against
the linked brazen in `tests/brazen_ollama_context.rs`, so the caveat is true
because a test says so and dies the day brazen changes. **The upstream ask is
brazen bl-f19d** (a first-class context declaration, or a passthrough that
composes with the typed `options`); when it lands, the caveat and the remedy are
deleted together.

**One gesture, ONE file (bl-d9cb).** A pick writes `providers.yaml` on
`config/<name>` through the §9.3 path (staged draft + `litany config`, the only
lawful writer) and nothing else. Since bl-3f46 the gesture is the boundary's own
`Action::PickModel { workspace, role, provider, model }` (§8.5): the pane
constructs the variant and calls the chokepoint, and `/model <role> <provider>
<model-id>` or a deposit reaches the same executor — one implementation,
whichever seat fires it.

**It was two writes in a normative order, and the premise for that died
upstream.** This section used to read: *"litany's cross-check
(`config::cross::check_roles_against_models`) refuses to load any config whose
`roles.<r>.model` is not declared in the global `models.yaml`, and refuses one
whose declared `provider` differs from that model's. A role assignment and a
model declaration are therefore two halves of one fact, and the picker writes
both"* — `models.yaml` first, *"normatively"*, because a role naming an
undeclared model **bricked every step in the workspace**. lernie 0.0.10 retired
that check and the table under it (litany's bl-35e2). Read at the pin,
`config/cross/mod.rs` says:

> There is no roles-against-models check any more (bl-35e2): the global
> `models.yaml` carries no `models:` table, a role's `providers.yaml` assignment
> is the single home of its (provider row, model id) pointer, and id validity is
> brazen's fact caught at the first live model call.

and `config/models.rs` deserializes the whole file to one optional `adapter:`
field, adding that *"A leftover `models:` block in an operator's file is ignored
on parse (serde's default for unknown keys), so existing installs load
unchanged."* So the first write reached nothing that reads it, and the ordering
rule it justified protected nothing. **Three things dissolved with it and none
was replaced:** the second write, the order between them, and the class of
half-written state an order exists to bound. `plan` returns one text.

**The pointer's single home is `providers.yaml`, and yog now says so in one
place.** A role's `provider` + `model` is the whole binding; nothing mediates it.
The §9.4 role marks therefore judge **that** pointer — `roles.<r>.provider`
against brazen's effective table, in `PickError::UnknownProvider`'s own words, so
the mark and the refusal one gesture later cannot phrase it differently (below).
The picker reads one file too: it had held the global `models.yaml` per open
beside brazen's rows, for a judgement about a table litany no longer loads.

**What became of `models.yaml`, and of the one fact still read out of it.** The
file is still litany's — it carries `adapter:` — and the `models:` block inside
it is now **yog's own table, read by exactly one consumer**: `context_window`,
the §5.1 #35 fullness denominator (`grammar::context_windows`). Nothing else in
either program reads a byte of that block. So:

- **The fact is not deleted, its redundant writer is.** The window keeps one
  authoritative home and one reader. What went away is the *seeding* of it by a
  gesture that had no business declaring it.
- **The seat that authors an entry is the §9.2 Declare control**, which was
  always there beside the §9.5 typed rows that edit one. Its generated entry
  takes `DEFAULT_CONTEXT_WINDOW` under a comment saying both generated lines are
  declared defaults and naming the figure the number feeds — the operator's
  reason to edit it, said where they are reading.
- **bl-848f's distinction dissolves rather than being preserved.** That ball
  found a fabricated 200 000 sitting beside a number brazen had served *in the
  same request*, indistinguishable from a considered one, and answered it by
  seeding the entry from the roster and writing a second comment variant that
  said which was which. With the picker's write gone there is no seat with a
  roster behind it: every generated number is a declared default, one note says
  so, and the two-kinds problem has no instance. The seed, the second note and
  `query::served_window` went with it.
- **brazen's served window is a query, not a field.** It is the provider's fact
  and it moves without yog's involvement, so copying it into a file at pick time
  is a snapshot that goes stale — which is the reasoning this section already
  applies to the roster itself (*"a stored candidate list would be a second
  representation of a fact the provider owns"*). If the figure should ever prefer
  a served window to a declared one, the shape is a read-time query over the
  model cache (`config_edit::brazen::model_cache_at`, already on disk inside the
  workspace's wall), never a field a write seeds.

**What brazen publishes.** `bz --list-models --json` returns
`{"models":[{"id":…,"default":…}]}` plus three OPTION-shaped metadata keys beside
them — `context_window`, `max_output_tokens`, `display_name` — each carried only
where that provider's list GET served it (Google serves all three, Anthropic only
`display_name`, OpenAI and Ollama none: brazen's own empty-set rule). The codex
measurement this section once recorded as "and nothing else" measured a row of the
last kind; it was never the shape of the payload. yog reads `id` off it and
nothing more.

Rejected, with reasons:

- **The intersection** — offer only models that brazen serves *and*
  `models.yaml` describes, greying the rest with an explanation. This is the
  reported bug restated politely: the operator still hand-edits a second file
  to use a model his provider already offers. (It is doubly wrong now: the second
  file governs nothing.)
- **A litany-side default for an undeclared model.** Right shape, wrong repo:
  litany is an exact registry pin (§16.7) that yog cannot change — and a
  harness that invents a context window has invented it *silently*, where a
  config file that declares one can be read and corrected. Overtaken anyway:
  litany declines to carry model policy at all.
- **Keeping the `models.yaml` write "in case litany's check returns".** A write
  kept against a hypothetical is a fact with two homes today. If the check
  returns, this section is where it lands.
- **A new yog-owned file for the window.** It would delete a file and add a file,
  and leave the operator's one seat for correcting the denominator behind a path
  nothing points at. The block already has an editor, a typed control and a
  reader; moving it buys a cleaner sentence and nothing else.

**yog still declares no YAML dependency (§9.2), and litany's parser is
private** (its crate exposes only `cmd`). Every edit to either file is therefore
an **anchored line edit over the block form litany's own template authors** —
never a general YAML transform: `roles:` / `models:` at column 0, two-space entry keys,
four-space fields. yog recognizes exactly that shape and **declines loudly**
(ichor, §7.3) on anything else, pointing at the §9.2 / §9.3 raw editors. That
refusal is not a dead end precisely because those raw surfaces already exist —
they are the escape hatch, and the picker is the fast path over the shape litany
itself writes.

**Scope is stated at the point of change — and the change reaches the whole
lineage.** An existing agent still derives its **governing** config commit, the
`config/*` ancestor its branch forked off (§5.1 #17) — but that commit is no
longer the answer, it is the **input** to a second derivation. **Fork settles
only which config *lineage* governs; control resolves from that lineage's
current tip at every step boundary** (upstream bl-403b/bl-e580, operator ruling
2026-09-01; yog's half is bl-e654). So advancing `config/default` *does* change
the conversation on screen — at its next step, with no per-conversation act and
no boundary gesture — and it changes what the next conversation forks off as
well. The picker says exactly that, and the conversation's model line (seated
with the rest of the conversation's settings at the bottom of the surface,
beside the composer — §11) **is** the pair, in the picker's own two dropdowns.
The ruling: the model selection in the conversation window carries both
dropdowns, provider and model, and the whole line becomes `<provider> - <model>`
and nothing else (bl-cd2a, superseding the `model · <model> · frozen at <oid>`
sentence and the `change…` that used to stand in front of it,
bl-a147/bl-9786.)

**The followed commit — the derivation the governing commit feeds.** The
ancestry query is untouched and stays exactly where it was
(`config_edit::branch::governing_config`, §5.1 #17): pure, unmoving, the nearest
`config/*` ancestor of the agent's branch. What consumes it is
`config_edit::branch::follow` — a faithful port of litany's own
`workspace/current_config.rs`, the way the ancestry query is of its
`workspace.rs::governing_config`:

> the governing commit → the `config/*` heads whose history **contains** it →
> their **distinct tips**. Exactly **one** distinct tip is **followed**, and
> that tip is what all control resolves from. That one rule covers the
> freshly-forked case too, where several refs still stand on one commit and the
> distinct-tip count is still one. **Two or more** distinct tips is real
> divergence the derivation must not guess between: the fork commit itself
> resolves, **HELD**, until `litany retarget` settles the lineage. Zero cannot
> occur — the head that contributed the governing commit contains it.

`GoverningConfig` therefore answers the **resolved** commit — its `oid`,
`short_oid` and `files` are the followed-or-held commit's — beside a
`governance` telling which of the two arms produced it:
`Governance::Follows(<lineage name>)`, whose tip is that oid, or
`Governance::Held { diverged_lineages }`. There is no third state and no
bootstrap case: an un-advanced lineage's tip *is* the fork commit, so a brand-new
conversation follows by the same arm every other one does.

**yog derives held-ness itself, and must never read it out of a log.** litany
prints `litany: notice: N diverged config lineages reach [<agent>] …` on its own
stderr at every step, and that line is its **operator channel**, not an
interface. yog has no notice-prefix contract and must not grow one: bl-b95e
deleted the `opslog::notice` marker table on exactly this ground — *a phrase
table over sentences litany is free to reword is not a classifier* — and ruled
that captured content is diagnosis, never a trigger. One fact, one home, and the
home is the git derivation **both sides run**: the same `for-each-ref` over
`config/*` and the same containment walk the §7 snapshot already pays for. A
future reader tempted to scrape `driver.log` for the notice is re-proposing the
thing that rule already killed.

**One label, one wording: `GoverningConfig::label()`.** The two arms render
exactly:

```
policy follows config/<name>, now at <short-oid>
policy held at <short-oid> — <n> diverged config lineages
```

and nothing else says it in other words. That is the same single-wording
discipline the §8.3 auth sentences keep (§7.3): every seat that answers *which
config governs* — the §9.3 browse line, the §11 Config tab, the §5.1 #31 as-of
tab — prints this function's output rather than composing its own sentence out
of the parts.

**The model row's clause is a different sentence and deliberately so.** It is
not a second spelling of the label: the label answers *which config*, and the
model row answers *which model*, which is the question bl-9786's operator was
actually asking. So the row names the **pair** the conversation resolves and
this function names the **commit** — one fact each, neither restating the
other, and the count of diverged lineages rides the label because that is the
surface where a number is the answer and not a distraction.

**What the dropdowns show is what they write — and now also what the
conversation resolves.** They carry the config branch **tip**'s assignment, and
under follow-the-tip that is the open conversation's assignment too: one commit
answering both questions in the ordinary case. The justification the freeze once
needed here is gone with it — a control that displayed the freeze would have
reported the operator's own write back as a no-op — and what stands in its place
is simpler: the tip is shown because the tip is the truth, for the next
conversation and for this one.

**A caption is not enough: the model line states the fact itself (bl-9786, kept
for its own lesson).** The picker's scope sentence is read at the moment of the
*write*; the surprise arrives later, at the moment of the *read*. bl-9786's
incident was a frozen conversation whose model line still said `gpt-5.4` after
the default advanced, read twice as a write that had not landed. Follow-the-tip
dissolves that incident, but not the lesson it paid for — a fact a surface
cannot be relied on to have been told once, in passing, belongs on the row that
is being read. So the model line still carries a clause, and its subject has
inverted with the doctrine: it fires when this conversation's **followed commit
is not this workspace's default lineage tip**, which no longer means *frozen
behind* but **apart** — held on a divergence, or following a different lineage
than the one this workspace runs:

```
[ openai-chatgpt ▾] · [ gpt-5.6-sol ▾]
this conversation resolves openai-chatgpt · gpt-5.4 at 1a2b3c4d, not this workspace's lineage
```

The clause appears **only** while the two oids differ; a conversation that
resolves this workspace's own lineage — the ordinary case, and now the case a
config edit *restores* rather than breaks — is the bare pair and nothing else.
Beside it is **one exit**, not two: *settle this conversation onto this
workspace's config lineage* (`RETARGET_EXIT`, bl-2d19, below).
`NEW_CONVERSATION_EXIT` is **deleted** — there is no freeze to escape, so
discarding a history is no longer a peer of the keeping act, and offering it
beside this clause would advertise the destructive answer to a state that has a
cheap one. **Apartness is derived at render, never stored:** it is the
inequality of the conversation's *followed* oid and the workspace's
config-lineage tip, the latter already in the §7 snapshot (`GitTree::commits`),
so the clause appears and clears the moment the git facts move and no field
records it. It is the **oids** differing, not the models — two lineages that
happen to name `gpt-5.4` are still two lineages, and the line says so with both
oids rather than special-casing the coincidence.

**Reversed: "Rejected: mid-conversation adoption" (bl-e654, overturning the
bl-2d19-era ruling).** This section used to forbid the open conversation picking
up the new config, and the paragraph is kept rather than deleted because the
trade it named was real. **What the freeze bought**, stated honestly: (a)
mid-conversation stability — an agent's policy could not change under it without
a per-agent act; and (b) branch-only policy replay — *which config governed step
N* was answerable from the branch's ancestry alone, at any later date. **What it
cost** is the live incident that ended it: a deployed workspace's roles moved
off an uncredentialed provider row onto a working one, and every existing
conversation went on refusing on the dead row until each was hand-retargeted and
nudged — the operator had no idea their running conversations were pinned, and
nothing on any surface said so. The operator's ruling is that a `litany config`
edit **is** the intended act and its reach is **the point, not the hazard**. Note
what was *not* the freeze's job and is untouched: resolution is per-hop, so a
step already in flight finishes on the commit it started with. Mid-*step*
consistency never depended on the freeze.

**What (b) cost, said plainly and not coded around.** Per-step policy provenance
is no longer derivable from ancestry alone: *which config governed step N* is
now a fact about **when the step ran**. What survives today is that each step
record carries the policy's byte-for-byte **effects** — `request.json` holds the
model id, the soul, the tools and the workflow-derived retry the step actually
ran under (§5.1 #13) — so the question *what did this step do* is answered
exactly, and only *what document said so* is lost. litany filed the fix rather
than leaving it silent (their bl-e4a0: record the resolved config commit in each
step's `meta.json`), and yog states the gap wherever it claims an as-of config
answer (§5.1 #31, STORIES §S10, VISION V1.2) instead of implying a precision it
no longer has.

**The change of lineage: `retarget` (bl-2d19, re-scoped by bl-e654).**
`retarget` is not deprecated and its mechanics are unchanged; what changed is
what it is *for*. It is now two things and neither of them is an exit from a
freeze: **(a)** the change of **lineage** — it re-forks the branch's ancestry
onto the target lineage's head so the follow derivation follows *that* lineage
from then on — and **(b)** the act that **settles a held (diverged) lineage**,
which is the only state that now paints the clause above with no cheaper answer.
It is emphatically no longer how a config edit reaches a running conversation:
that needs no verb at all. The re-fork argument survives verbatim and is why the
verb is safe to keep: a newly minted dispatch commit is derived on top of the
target config commit, the conversation's own post-dispatch commits are replayed
onto that base, and the branch moves to the replayed tip. Nothing is discarded
and nothing is stored.

yog's half is the **gesture and its seat**, and both are decided by the clause
above: the operator reads *policy held at …* and the verb that answers it is the
next thing on that row (bl-a0d4's ruling — the weight question is answered by
giving the sentence a verb, not by more ink). The gesture is the boundary's own
`Action::Retarget { workspace, agent }` (§8.5), so the button, `/retarget` and a
deposit are one implementation; the keyboard path is that line and the §11 focus
floor, and the hover names it (§11 rule 3). It carries **no config name**:
litany defaults to the lineage §9.3 writes, and §9.3 writes exactly one, so the
argument would be a knob with one lawful value — and that one lawful value is
precisely right here, being *the lineage this workspace runs*, which is what a
held or foreign-lineage conversation is being settled onto.

**What yog does not do here.** It writes no ref and lands nothing: `litany
retarget` marks `refs/litany/retarget/<agent>` and the conversation's **own**
executor consumes the mark at its next step boundary (litany's §2.3
single-writer invariant — the user marks, the executor writes). So the receipt
yog paints says *when* it lands rather than that it has, the apart clause is
still true at the moment of the click, and retargeting the lineage a
conversation already follows is litany's own clean no-op, reported on litany's
stderr — a state yog does not model, for the same reason it does not read the
divergence notice there.

**Two seats, one picker — and the second one is the start (bl-824e).** The
operator's request was verbatim *"when starting a new conversation, I should be
able to select the model."* The clean semantic would be a per-conversation pick
that scopes to the conversation being born and leaves the workspace default
alone. **lernie 0.0.3 does not offer it, and this was settled against the crate,
not assumed:** `litany prompt` takes exactly `<repo>` and `<message>` (`cmd/
prompt.rs`), and `prompt::run` resolves
`ConfigSource::ConfigBranch(workspace::DEFAULT_CONFIG_REF)` — the literal
`"config/default"` — internally (`prompt/resolve.rs`). There is no argument, no
env, and no alternate branch a caller can name; a fresh root *always* forks that
branch's head. A start-time pick is therefore **the same write §9.4 always did,
made one gesture before the start instead of after it** — the workspace default
moves, and every conversation started next is born on it.

So the §11 birth-config block wears the same row (the same two dropdowns over
the same state, minus an apart clause it cannot have — it has no conversation to
be apart) and states that plainly in one sentence — *"this
moves the `<ws>` workspace default too"* — where the conversation seat's scope
sentence would have named a conversation the block does not have. A conversation
born here forks that head and **follows that lineage** for the rest of its life,
which is a stronger claim than the block used to make and a simpler one: the
sentence the operator reads at the start is still true at every later step. The
two seats
differ in exactly that sentence and nothing else; `pane` takes the sentence
rather than deriving it, so there is one pane, one write pipeline, and one
authority over the two files. **The rejected alternative is a yog-side
per-conversation config** — forking a throwaway `config/<conv>` branch per
start, or advancing and reverting `config/default` around a spawn. Both make
yog a second authority on a lineage litany owns (§9.3: `litany config` is the
only lawful writer), and the second one is a race with any other start in
flight. The honest answer is the one the substrate supports, said out loud; if
litany ever grows a per-start config argument, this block is where it lands and
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

**A dead assignment is marked where it is read (bl-53be, re-pointed by
bl-d9cb).** The candidate set is a live query, so a dead row is never *offered* —
but the role rows name what each role is **already** on, read from
`providers.yaml`, and that is exactly where a dead one hides. Each row therefore
carries the judgement: the provider row this role dispatches through is not one
brazen's table has, so every step under it dies with `unknown provider`. A
faulted row is glyphed and the reason is painted in ichor under the selection, so
"one usable role out of two" is visible at the point of change instead of at fire.

**It judged the wrong file until bl-d9cb, and missed the defect it was named
for.** The mark used to read the global `models.yaml` and ask two questions of it:
was the role's model declared there at all, and did that declaration's own
`provider:` name a live row. Both are dead at the pin — litany loads no `models:`
table, so an undeclared id refuses nothing, and the declaration's provider is read
by no dispatch — while the *live* pointer went unjudged here: a role sitting on a
row brazen had dropped was **unmarked** whenever its old model entry happened to
name a live one. The wording is `PickError::UnknownProvider`'s own, one home for
the mark and the refusal a gesture later.

The one fact this needs — brazen's rows — is asked **once per open** and
discarded with the surface (§5.3), on the same terms as the roster: it answers
"is what you already have usable?", a question that exists only while the picker
is on screen. It was two facts, and the second one was the file above.

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
  derivation the §8.3 Login rows render, under the row's name. This is a
  second seat at one wording, never a second wording. It
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
(`grammar::is_unknown_row`) the §9.4 pick gate and the §9.4 role marks call.
They cannot disagree. (It was the §9.2 Apply gate's too until bl-3ffa; that gate
read a field with no other consumer and went with it — §9.2.)

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
| brazen (derived, §5.1 #20/#21) | how many provider rows the file ends up routing | a counted line plus the control that focuses §8.3, where the rows themselves are (bl-20cb — the rows used to be re-rendered here, which was one rendering too many) |
| `models.yaml` (§9.2) | `models.<id>.model_id` | scalar field — the wire id, when the entry is filed under an alias |
| | `models.<id>.context_window` | bounded number (1 … 100 000 000) |
| | *declare a new model id* | id field → `declare_model` |
| | ~~`models.<id>.provider`~~ / ~~`capabilities`~~ | **no control since bl-3ffa** — a picker over a field nothing dispatched through (its one reader was the §9.2 gate that judged it) and a list no program in the suite reads. A control over a fact nothing consumes is a setting that cannot matter; neither control KIND died with them, both being `providers.yaml`'s (§9.2) |
| `workflows/*.yaml` (§9.2) | litany's workflow DSL | **raw text** — fallback 2 |
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
2. **`workflows/*.yaml`.** litany's workflow DSL, whose parser is private (its
   crate exposes only `cmd`) and which yog has no reader for. A grammar guessed
   at here would be the same second authority.
3. **A config commit's non-`providers.yaml` paths.** `souls/**` and
   `descriptions/**` are prose; `workflow.yaml` / `manifest.yaml` / `version` are
   litany's own schemas. Prose has no fields, and litany's schemas are litany's.

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
  all git/CLI spawning, the XDG folds (balls/litany/yog paths are pure-XDG on
  both platforms, matching those tools; brazen's config, credentials and model
  cache are **not** folded per-OS at all since the blast-radius ruling — they
  resolve inside the focused workspace's wall, one yog-owned layout identical
  on every target, §16.2 as amended and `config_edit::brazen::BrazenPaths`).
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
    the sweep (§7.2). **Those two evictions are complementary, not
    overlapping** (bl-1015): the sweep evicts for a workspace that *holds* a
    Live/InFlight agent, because only those can die silently; the watcher-driven
    re-derivation evicts for a workspace that holds *none*, because that is the
    only signal a driver has arrived. A streaming `response.json` storm is
    neither, so it stays collapsed on the cache — which is what the cache is
    for.
  - **The target is resolved before it is asked about** (bl-1015): both sides
    canonical, mirroring the procfs backends, because lsof prints the resolved
    name of every fd it finds and on macOS the whole temp tree resolves
    (`/var/folders/…` → `/private/var/…`). And **a target the filesystem does
    not resolve is a definite `Free`, never `Unknown`** — nothing can hold a
    path that is not there, which is exactly what the procfs backends answer for
    one. lsof instead *errors* on an absent path, indistinguishable from lsof
    being broken, so every agent with no inbox directory and every agent with no
    step yet used to read `Unknown`: a "?" badge on its row and a refused §3.6
    delete, on macOS only.
  - `lsof` missing/failing ⇒ `Unknown` ⇒ classification degrades to
    framing-only: closed-with-`end` = quiescent, closed-without = stopped,
    open-file undetectable ⇒ rendered with an explicit **uncertainty badge**
    — a bare `?` beside the row's state, its words on hover
    (`theme::STATE_UNCERTAIN`) — never a false definite state.
  - **Rejected: flock-acquire probing** (`flock(LOCK_SH|LOCK_NB)` then
    release) — portable and dependency-free but **perturbs the substrate**:
    during yog's transient hold, a `litany message` writer's probe sees the
    lock taken, concludes a driver exists, and strands the deposit until the
    next scan (writer/driver totality, ARCH §2.11). A probe must never affect
    the observed (I8). Also rejected: the libproc crate (a dependency for one
    probe) and hand-rolled FFI (unsafe, untestable on Linux).
- **Coverage mechanics:** there is **no per-OS path fold left to cover** —
  the brazen folds that once carried one now resolve per wall (above), and
  every fold in `src/xdg` reads only its injected `Env`, so Linux tarpaulin
  covers all of them. The `#[cfg(target_os)]` split that remains is the probe
  stack's (`git_tree::probe_stack`), whose macOS half is the pure `lsof -F`
  parser — compiled and tested everywhere, which is what keeps it out of the
  coverage hole. Any future per-OS branch takes the OS as a runtime parameter
  rather than a `cfg`, for that reason.
- **Known upstream limit, documented, not worked around:** `litany stop` is
  itself /proc-based (Linux-only). On macOS yog surfaces the Stop failure
  verbatim in `ops.jsonl`; fixing stop portability is litany's ball.
- **CI:** Linux runs the full gate (fmt, clippy -D warnings, tarpaulin 100%
  pinned 0.35.2). macOS (aarch64) is `cargo build` + `make test`, no tarpaulin
  (Linux sees every line because nothing but the lsof spawn shim is cfg'd out),
  and it lives in a **workflow of its own** (`.github/workflows/macos.yml`,
  bl-0158) on ci.yml's triggers: same visibility, but the release gate reads the
  `CI` workflow's verdict, so macOS reports without holding the pipeline. Moving
  it back under the name `CI` is a decision to let it block a release. That leg
  runs the **suite** on a mac; **producing a distributable mac artifact is a
  separate question and §10.2 is where it is answered** — for the components
  that can be cross-produced from Linux at all, and for the two that cannot.
- **The macOS suite is green, and what it took is recorded here because the
  obvious reading was wrong** (bl-1015). Thirteen tests failed there from its
  first run to 2026-08-14, and the standing hypothesis was two causes: a
  Linux-shaped liveness probe *and* a text layout that came out narrower on
  aarch64. Measured on a `macos-14` runner, every painted galley is
  **byte-for-byte the width it is on Linux** — the acceptance harness runs
  `egui::Context::default()`, so both platforms lay out through egui's own
  embedded faces at the same `pixels_per_point`, and there is no per-platform
  text metric to find. The narrower titles were the *first* cause wearing a
  disguise: the probe answered `Unknown` for agents with no inbox directory, so
  every such row grew a §10 "?" badge in its trailing group, and the title,
  which fills what the trailing group leaves (§11), truncated 14 px earlier.
  One defect, twelve tests. The thirteenth was the FSEvents arming race below.
- **A watcher is not armed when its constructor returns** — on macOS. inotify
  arms inside the syscall; FSEvents starts its stream on another thread, and a
  write that lands first emits *no event at all*, which no downstream timeout
  can recover. Tests therefore **prove** arming (rewrite a probe file until the
  watcher reports it) rather than sleeping a guess at it.
- **The shell the gates are written in is bash 3.2**, because that is what
  macOS ships and — the licence having changed under it — always will. Two of
  its limits had made `scripts/leak-scan.sh` Linux-only: associative arrays
  (bash 4) and, worse, `"${empty[@]}"` under `set -u`, which 3.2 treats as an
  unbound variable, kills the shell on, **and exits 0 doing** — so a tree full
  of findings passed the gate silently. Rules are a `case` and every array
  expansion is guarded `${a[@]+"${a[@]}"}`; a new script is checked with
  `bash -n` under a real 3.2, never against the host's bash.

### 10.1 Deployment: one OCI image per component (bl-223f)

**An OCI image is a tarball of filesystem layers plus a manifest, in the format
podman and docker both read** — the unit an operator installs on a box that
takes images rather than binaries.

Operator ruling 2026-08-30: every component of the four-way split ships one —
yog (the server), litany (the engine), thrall (the foot), lernie (the seat) —
**for deployment sanity and for nothing else**. Not for internal container
semantics: no component uses the container filesystem as a feature, none may
start to, and a container is never a containment claim any component makes on
the wire. The image is the unit of install. That is the whole of the ruling,
and the rest of this section is the shape the two containerized repos landed
against it (litany bl-6467, thrall bl-3586), which the remaining two follow.

**Each repo owns its own `Containerfile` and `make image`; there is no shared
build tooling and no meta-repo.** The components meet at the wire and nowhere
else (REMOTE), and a build system that spanned them would be the first place
they met somewhere else.

Six properties, and each is load-bearing rather than stylistic:

- **The toolchain pin has one home and the image proves it.** The build stage
  is `rust:<pin>-alpine`, and a step inside the build reads `channel` out of
  the repo's own `rust-toolchain.toml` and fails the build if it differs from
  the base image's `rustc`. The `FROM` tag is unavoidably a second statement of
  the pin; this makes the drift a build failure instead of a silent difference
  between what the gate compiles and what ships.
- **The build stage is discarded whole.** Two stages, and only built artifacts
  cross. No compiler, no cargo, no source, no `target/` in the shipped layers.
- **The runtime layer is exactly what the binary EXECS, and that is per crate,
  not per house style.** `FROM scratch` is right only for a binary that execs
  nothing — a static musl build is that binary right up until it forks
  something. litany execs `git` (the harness is git-backed and shells to the
  binary on PATH), `sh` (its `bash` built-in tool), `bz` (the adapter, at the
  pin read from its own `Cargo.toml`), and itself; thrall execs
  operator-configured argv that is not knowable from thrall's repo at all. Both
  therefore ship `alpine`, and both **record the reasoning in the file** —
  because the next person's instinct will be to shrink the layer, and the
  reason not to is not visible from the outside.
- **No state in the image; the XDG dirs are the runtime contract.** Each image
  sets the XDG variables and provisions nothing under them, so the roots are
  mounts. litany's image deliberately does not run `litany prime`: seeding the
  harness root writes files, and writing them into a **layer** puts the one
  state litany owns where a mount cannot replace it and an upgrade cannot see
  it. Nor is there a `VOLUME` instruction anywhere — a `VOLUME` lets an
  unmounted run succeed against an empty anonymous volume, where the refusal
  naming the missing file is the answer an operator can act on.
- **No credentials and no certificates, ever.** thrall's foot-grade certificate
  and brazen's provider auth are mounted or injected. A credential baked into a
  layer is a credential published to everyone who can pull it — the same
  reasoning as the ball-body rule above, one distribution channel over.
- **`make image` pushes nothing, and no repo has a `push` target.** Podman or
  docker, autodetected, podman first (no daemon, no group membership); the tag
  is read out of `Cargo.toml`'s version rather than typed. A push is not
  undoable — a tag can move, but the bytes anyone pulled are theirs — and a
  convenience target for an irreversible act is how the act happens by accident
  (the same reasoning that keeps `publish` guarded, and that item 6 of the
  publication checklist in AGENTS.md was written by). **Where these images
  publish was unanswered when this section was written; it is answered below,
  and the answer did not move the push into `make`.**

#### The registry (operator ruling 2026-08-30)

**`ghcr.io/mudbungie/<component>` — one package per repo, named for the
component.** `ghcr.io/mudbungie/litany`, `/thrall`, `/yog`, `/lernie`. GitHub's
registry and not a fifth account somewhere else, because the repos are already
there: the package inherits the repo's owner, its visibility and its access
control, so there is no second identity to hold, no second credential to
rotate, and no way for a package to outlive the repo that explains it.

Four properties, and each is a refusal of a way this goes wrong:

- **Pushed only from that repo's own release workflow, at tag time.** Not from
  a laptop, not from `make`, not from a second repo's workflow. The publishing
  identity is the repository's own `GITHUB_TOKEN`, which exists only inside
  that workflow run — so "who can publish" is answered by the same access
  control that answers "who can push a tag", and there is no long-lived
  registry credential on any box.
- **The version tag and the manifest digest, both immutable. Never a moving
  `latest`.** A published `latest` is a name whose bytes change under everyone
  who ever wrote it down; a digest is the only tag that means one thing
  forever. `make image` still tags `latest` **locally**, and that is not the
  same act: a local tag is a convenience on one box that nobody else can pull.
- **A version is published once.** The registry is not a place to re-push a
  corrected build under the same tag — that is a moving `latest` wearing a
  version number. A bad image is superseded by the next version, exactly as a
  bad crate release is (AGENTS.md item 6: `cargo publish` is irreversible, and
  so is `podman push`).
- **The image and the crate are the same publication, so they carry the same
  version.** The tag is read out of `Cargo.toml` (it already is), which makes
  the crate version the one home for both and makes a mismatched pair
  impossible to produce by hand.

#### The condition: an image-side disclosure gate (REQUIRED)

The ruling above is **conditional**, and this is the condition. No component
image is pushed — by a workflow or by hand — unless a scan of that image has
passed, and the scan is wired so that the build path runs it: **`make
image-scan` is part of `make image`, or is a prerequisite of whatever target
pushes**, mirroring exactly how the source `leak-scan` sits ahead of the commit
rather than beside it. A gate a person has to remember to run is not a gate;
that is the whole reasoning of the pre-commit hook, one artifact over.

**It is a second gate and not a reuse of the first, because an image is not a
tree.** `make leak-scan` reads the git INDEX (AGENTS.md, "It reads index BLOBS,
not the worktree"), and every input an image has that a commit does not — the
build context as the engine actually receives it, the layers a `RUN` writes,
the base image, the package index, and the image CONFIG — is outside every byte
that gate has ever read. The `include` allowlist in `Cargo.toml` is the
analogue one publication channel over (item 6 of the AGENTS.md checklist), and
its lesson transfers: **the build context is the image's `include` list**, so
each repo's `.containerignore` and its `COPY`-by-name discipline are the thing
the scan is checking has held, and each repo records that as a line of its own
publication checklist.

Three surfaces, because those are the three ways bytes reach an image:

- **The authored filesystem** — what the build ADDED above the pinned base
  digest, isolated by content and not by trust in the `COPY` lines. Each repo
  picks the mechanically simplest isolation its engine gives it and **records
  why in the script**; what is not negotiable is that the isolation be by
  content-addressed comparison against the base, so a file the build did not
  touch is never scanned and a file the build rewrote always is.
- **The distro floor is accounted for, not exempted.** A runtime layer that
  runs `apk add` adds thousands of files that this repo did not write; scanning
  them is noise that gets the gate switched off, and skipping them by path is
  an allowlist. The package manager's own ledger is the authority instead:
  a file **owned by an installed package** is distro content, a symlink whose
  resolved target is distro content is distro content, and everything else
  above the base is this repo's and is scanned. That keeps the enumeration
  derived rather than typed — the same rule as everywhere else in this tree.
- **The image config** — every `Env`, every `Label`, and every `history` entry.
  Build arguments echo into history, and an `ENV` is shipped to everyone who
  pulls whether or not any file holds it.

And the posture the source gate already fixed, carried over unchanged:

- **The rule table is the SAME table**, `scripts/leak-rules.sh`, sourced and
  never copied. Two copies of the rules drift within a week (AGENTS.md says so
  about the task-store gate; it is no less true here).
- **Findings LOCATE, they never reprint.** Truncated as the source gate
  truncates them.
- **Unreadable is rejected, not skipped.** The binaries a build authors are
  knowable — they are the `COPY --from=` destinations, which the Containerfile
  already names, so the expected set is **derived from the Containerfile**
  rather than typed into the scanner. Any OTHER authored file the rules cannot
  read is a refusal.
- **Both directions, and a scan that enumerates nothing fails.** A self-test
  builds a scratch image that layers in a fabricated secret carrying the
  `notreal` marker AND plants one in an `ENV`, and proves the scan catches
  each; the real image must pass clean. A leak gate dies by matching nothing
  and passing everything forever — the same two-direction discipline as
  `line-cap`, `rules-audit` and `leak-scan --self-test`.

**What it cannot promise, stated rather than implied.** It scans one image, on
the box that built it, before the push. It does not read what is already in the
registry, it cannot un-publish a digest, and whoever runs the build can bypass
it exactly as `--no-verify` bypasses the commit hook. That is the same
prevention-is-local, enforcement-is-late split AGENTS.md draws for the task
store; the late half here is that a published image can only be superseded,
never recalled, which is why the gate lives ahead of the push and not after it.

**yog's own image is landed** (bl-10f9), and the deferral it waited out is
worth keeping stated: the image is the **UI-free server**, so containerizing
the windowed binary would have shipped a GL display stack into a layer nothing
in a server can present — it was filed behind bl-7942's severance for that one
reason and no other. The seat's image is still behind the seat's own window
chain (bl-320b) and cites this section for the shape.

Three answers are yog's alone, and each is written into `Containerfile` where
the next person reads it:

- **The runtime layer is `git`, `openssl`, `sh` and yog itself.** yog execs
  more than any other component, and §16.4/§16.7's self-multiplex is what keeps
  the list short: the world's `PATH` head is re-exec shims of this binary, so
  `bl`, `litany`, `bz`, the two balls plugin seams and `tool-control` need no
  host binary at all. What is left is git (the crate's one fork, `git_env`),
  `openssl` (the §1.4 mint the boot performs when a box has none — without it a
  fresh mount comes up with no listener), and `sh` (the `$EDITOR` re-entry a
  §9.3 lineage write performs). `lsof` is absent because §10's macOS liveness
  shim is not compiled into a Linux binary. **Nothing an agent runs is in the
  layer**, which is REMOTE §12's ship-inert posture stated as bytes.
- **`XDG_DATA_HOME` is the only root that mounts**, because it is the world's
  anchor (§16.2). `LITANY_HOME` and `XDG_STATE_HOME` are *derived* onto
  `<yog-data-root>/world/…` and handed to every child, so an operator who
  mounted the three separately would be fighting the nesting and would watch
  the world re-found itself. One mount: the data root.
- **No wire material in a layer** (REMOTE §1.4). The CA, the leaves and the
  `address` file are the operator's, minted on the operator's box by
  `yog wire-certs` or by a boot into the mounted root. An image that arrived
  able to present an identity would be the in-channel bootstrap that must never
  exist; `make image-scan` is what turns that promise into a check.

### 10.2 The macOS artifact: `zig cc` from a Linux container, and what it cannot reach (bl-888d)

§10.1 put the Linux images on a reproducible line. This is the other half of
the same ruling: **the macOS binaries come off that line too, not off a
laptop.** A mac binary produced by hand is one nobody can reproduce, nothing
gates, and no one can say what it was built from.

The shape follows §10.1 exactly and for the same reason — **each repo owns its
own `Containerfile.mac` and `make mac-artifact`; there is no shared build
tooling and no meta-repo.** The components meet at the wire and nowhere else.

#### The toolchain is `zig cc`, and osxcross is refused on Apple's licence

Two toolchains can emit Mach-O from Linux. osxcross drives **Apple's own SDK**,
which the *Xcode and Apple SDKs Agreement* forbids twice over — and either
clause alone settles it:

> **2.7** The grants set forth in this Agreement do not permit You to, and You
> agree not to, install, use or run the Apple Software or Apple Services on any
> non-Apple-branded computer or device, or to enable others to do so. … You
> agree not to rent, lease, lend, upload to or host on any website or server,
> sell, redistribute, or sublicense the Apple Software and Apple Services, in
> whole or in part, or to enable others to do so.

> **2.5** You may not alter the Apple Software or Services in any way in such
> copy, e.g., You are expressly prohibited from separately using the Apple SDKs
> or attempting to run any part of the Apple Software on non-Apple-branded
> hardware.

The first means the SDK may never sit in a repository nor in any layer that is
published. **The second closes the escape hatch the obvious design would have
reached for** — take the SDK path as a build argument, keep it out of the tree,
let the operator supply it — because the builder is not Apple-branded hardware
either. So no repo here holds the SDK at arm's length; the whole arm is
refused, and a `Containerfile` with an SDK input would be inviting an operator
into a term they cannot satisfy on a Linux box.

`zig` acquires nothing from Apple: it ships one darwin stub of its own in its
own distribution and under its own licence. It is pinned by version **and**
sha256; `cargo-zigbuild`, which filters the darwin linker flags `zig cc` will
not take, is pinned exactly and installed `--locked`; and both live in a build
stage that is discarded whole.

**It is a C toolchain, deliberately, and that is not the posture `deny.toml`
holds.** The bans on the `openssl-sys` / `native-tls` / `aws-lc-sys` class exist
to stop a C toolchain arriving **implicitly**, through a dependency edge nobody
reviewed. This one arrives explicitly, in a file that argues for it, pinned, in
a stage nothing ships from — and **no crate gained a dependency**: `Cargo.toml`
is untouched by this everywhere except litany's, where the change was a
*removal* (below). The posture is against the accident, not against the
compiler.

#### The rule the choice imposes: libSystem, and nothing above it

zig ships **exactly one** darwin stub — `lib/libc/darwin/libSystem.tbd` — and
**no framework stubs at all**. Not CoreFoundation, not CoreServices, not
AppKit, not OpenGL, and no `libobjc`. So:

> **A component can be cross-produced from Linux exactly when its crate graph
> links nothing above libSystem.** One that links any Apple framework fails at
> the link step with *"unable to find framework"*, and there is no lawful way
> to supply the frameworks on a Linux builder.

That reads as a packaging constraint and is really §10's own gap list wearing a
different hat: **the components that need a macOS platform API are exactly the
ones whose artifact cannot come off this line.** The watcher, the window and
the `lsof` probe are the three places this design reaches for macOS itself, and
each is a framework edge.

#### Measured, per component — built, not assumed

| Component | Cross-produces? | The edge |
|---|---|---|
| **thrall** (the foot) | **Yes** — landed, thrall bl-e479 | none; `rustls`/`ring` and `serde_json` over std |
| **litany** (the engine) + `bz` | **Yes** — landed, litany bl-c2b9 | had one, removed |
| **lernie** (the seat) | **No** | `libobjc` + OpenGL, AppKit, CoreGraphics, Foundation, CoreFoundation, ApplicationServices, CoreVideo, Carbon |
| **yog** (the server) | **No** | CoreServices, and CoreFoundation |

- **thrall passes and is the reference.** Its artifact is a Mach-O arm64
  executable loading three libraries, all stock `/usr/lib`.
- **litany passed after a subtraction.** Its one framework edge was `chrono`'s
  `clock` feature, which pulls `iana-time-zone`, which links CoreFoundation on
  darwin; the crate uses `Utc` only, so the feature narrowed to `now` and five
  crates left its lockfile with it. Smaller graph and a portable one by the
  same edit. `bz` cross-installs at the pin and needed nothing.
- **The seat cannot, and the reason is what the seat IS.** Every crate in the
  window's graph compiles for the target; the build dies at the last link, on
  `libobjc` and eight frameworks. There is no flag for it and no other Linux
  toolchain to try, because the constraint is Apple's licence rather than zig's
  feature set: a toolchain that succeeded would be one that had acquired the
  SDK. Recorded with its measurement in lernie bl-9380.
- **The server cannot either, and one of its two edges is permanent.** The
  removable one is inherited: the embedded litany 0.0.2 still carries the
  `clock` feature, so a later litany release retires it here for free. The
  other does not go away — `notify` reaches FSEvents through `fsevent-sys`,
  which links **CoreServices**, and a macOS watcher is not something yog can
  decline to have (§7). So the server's mac binary is not a scheduling
  question; it is off this line for good.

#### What an artifact proves, and what it does not

**No mac executes anything here.** A green build is not evidence: a wrong
architecture, a dependency on a dylib no stock mac carries, and a binary macOS
would refuse to start all look identical to a successful `cargo build`. So each
repo carries a `scripts/mac-verify.sh` that **reads the produced Mach-O** — the
header, the architecture, the filetype, `LC_BUILD_VERSION`, the code signature,
and every `LC_LOAD_DYLIB`, each of which must be a stock `/usr/lib` or
`/System/Library` path — and runs its negative direction first, on fabricated
malformed inputs it must refuse, because a checker that has quietly stopped
checking passes everything forever.

- **Proven** — the artifact has the shape of a working mac binary, in every
  respect a file can be read for.
- **Not proven** — that it runs. It has not been observed to.

Two properties ride along and both are read rather than declared:

- **The minimum macOS version is the pinned toolchain's, not a setting.**
  `rustc` asks for one and the pinned zig stamps its own; `cargo-zigbuild`'s
  versioned-darwin target syntax does not survive this zig either. So the floor
  is a property of the pinned pair — **read it off the artifact, where
  `mac-verify` prints it, and never from a document, this one included.**
- **The signature is ad-hoc, which is not notarization.** An arm64 mac refuses
  to start an unsigned binary and the cross-linker's ad-hoc signature satisfies
  exactly that. A copy that arrives over a network still carries a quarantine
  attribute; clearing it, or replacing the signature with a real one, is an act
  on a mac by the operator and is outside what this line can do.

#### The route that is left for the components this cannot reach

**Apple hardware, which for CI means a macOS runner.** That is not a defeat of
the ruling so much as its boundary: this section puts on the container line
every component that can lawfully be there, and names the two that cannot along
with why. Two things such a route must answer and this section does not —
whether a CI provider's mac satisfies §2.2.A's *"Apple-branded computers that
are owned or controlled by You"* is the operator's question, not an
implementer's; and signing and notarization are acts on a mac by whoever
publishes.

It is a different thing from the macOS leg §10 already describes.
`.github/workflows/macos.yml` runs the **suite** on `macos-14` and reports;
producing a distributable **artifact** is a second job with a second set of
questions, and neither implies the other.

#### The push posture, unchanged

`make mac-artifact` pushes nothing and no repo has a target that does — the
same refusal §10.1 makes for images, for the same reason. The build's last
stage is `FROM scratch` carrying the binaries, so `create` + `cp` lifts them
out and the wrapper image is deleted; it is never pushed, which is why
`make image-scan` does not apply to it and is not being skipped. An artifact's
content is compiled from the same tree `make leak-scan` already reads, exactly
as every Linux release binary is.


---

## 11. UI structure (retired — the seat's)

The three information altitudes, the roster and inspector, the keyboard table,
the congeries palette and its application mark, the glyph doctrine and the
discoverability invariant are **retired to git history** (bl-7942). They
described a face, and yog has none: the window is the `lernie` seat crate's,
reached over the §8.5 boundary like every other seat (REMOTE §12).

What survives here is a **reading rule**, because ~200 sentences elsewhere in
this document still describe that face:

- Where this document says *the window*, *the frame*, *a paint*, *a click* or
  *a seat renders*, it is describing **a seat**, on the far side of the wire.
  yog states the fact; how it is shown is not yog's.
- Where it says *the frame derives nothing*, read it as **the strongest form
  of what is now structural**: a seat holds no derivation at all, so the
  question of what it may derive on its own thread does not arise.
- Anything this document says yog **paints** is a fact it **answers**. If no
  §8.5 query or reply carries that fact, the fact is not reachable and that is
  a defect to file, not a face to add.

The vocabulary those sentences depend on and a server still owns is named
where it lives: the badge words in `src/badge.rs` (§12), the row tone in
`nav::convs`, the row key in `transcript::key`. The heading stays so citations
(`§11`, `§11 rule 5`, …) keep resolving, exactly as §15's does.

---

## 12. Module map

Dependency doctrine: serde_json covers every machine contract; small pure
functions (percent-decode, XDG folds, lsof parsing) beat
dependencies; and the existing trait-injection pattern (LockProbe/WriterProbe)
is **the template for every new effect** — lsof runner, bl runner, bz runner,
editor shim, clock. The three embedded substrates are exact-pinned crates.io
releases; **the pin authority is `Cargo.toml`/`Cargo.lock`, never this doc**
(§16.5, §16.7). brazen brings the rustls/`ureq` TLS stack, governed by
AGENTS.md rule 6.

tarpaulin excludes: `src/main.rs` — the process edge, and since bl-7942 the only one. **This table carries no line
counts and no per-file budget** (bl-273c): a count written beside a file is a
second representation of a fact the tree already answers, and it drifts on
every commit that does not come back to retype it — 160 of the 389 numbers
this table used to carry were wrong, across 90 of its 147 numeric rows, and 11
files sat over a `≤250` budget no gate had ever read. **`make line-cap` is the
one definition of the cap** (300 lines, inline tests included, whole tree, on
the index); `make line-cap LINE_CAP=199` is how you ask for today's ≥200
pre-split band. Splitting early, along a real seam, before a file reaches the
wall is still the house preference (bl-52f8 swept the tree to it), but it is a
projection the author makes at design time — not a number this table stores
about a file, and not a second gate (AGENTS.md, "300-line hard cap").
A module's test corpus is its own seam — `X.rs` pairs with `X/tests.rs`, a
subsystem with `<sub>/tests/*.rs` — so **a test module is covered by its
production module's row** and never earns one of its own; a row may describe
its corpus, but no cell may name one.
**Rows stay sorted by module path** — inserts distribute instead of stacking
at subsystem boundaries — and a responsibility cell is one clause plus the
§ citations that own the doctrine; doctrine lives in those sections, never in
this table. **Every rule this table states about itself is machine-checked**,
because a stated-and-unchecked rule is how all three of the above drifted:
`tests/design_module_map.rs` holds both path directions (a production file
under `src/` with no row fails; a row naming a path that does not exist
fails), the sort (a row whose first path falls below its predecessor's fails,
a directory's `mod.rs` sorting as the directory), the corpus rule (a cell
naming a test module fails), the shape (a row with a third cell fails — that
is where the counts lived), and single-entry (a path in two rows fails). So
the paths here are a contract, not typography, and a brace spelling the guard
cannot expand reads as a missing row.
**A row stays short enough to merge** (bl-0012): git merges by line, so a row
that grows to hold a whole subsystem becomes one physical line every
concurrent module addition collides on, unconditionally and unresolvably. The
`src/shell/*` glue was exactly that — one ~16,000-character line — and became
one row per file; keep any row that starts accumulating a subsystem split the
same way. (That family is gone with the window, bl-7942, along with every row
that named one of its files; the rule it taught is not.)

| Module | Responsibility |
|---|---|
| `src/actions/{mod,enabled}.rs` | the action root (ARCH §3.4/§3.5): the composer's and the new-ball form's own rules — whether there is anything to fire at all — with `enabled` the §8.2 predicates over a selection, each refusing exactly what the underlying verb would. Both pure and egui-free; the verbs themselves live in `verbs` (§8.2) |
| `src/actions/drafts.rs` | the composer's draft store, keyed by target (§11, §5.3): one draft per new-conversation-in-workspace / message-to-agent |
| `src/actions/verbs{,/balls,/balls/edit,/bound}.rs` | the §8.2 verb dispatchers + opslog wiring, cut on the table's own seam when the V2 attempt joined it (bl-dc0c): `verbs` the litany family — message, the attempt's `dispatch`, stop, scan, and the §9.4 `retarget` change-of-lineage (bl-2d19, re-scoped bl-e654) — acting on a conversation in a workspace; `balls` the `bl` family, acting on a ball in a project and stamped `--as` its §3.2 claimant to a verb; `balls/edit` **what a create/update carries** (bl-dbde), split at the budget on the seam `balls`' own doc draws: the verbs act on a ball in a project, this is what a ball is made of, and only the second grows every time balls learns a field — the boundary carries these types WHOLE, so the roster, the codec and the executor read one vocabulary rather than three copies bridged by an `of` constructor; `bound` the workspace-bound spawn seam every litany verb takes (§8.2's workspace-bound rider, bl-bf79) — one fold laying the wall and the name, so the family owes no per-verb decision |
| `src/actions/verbs/dispatch.rs` | the dispatch + `ops.jsonl` logging core beneath the short verbs (§8.2, §4.2 as amended): every attempted action leaves a durable ops line, a spawn failure logging a synthetic one, so no error class is un-logged (§7.3) |
| `src/alert/{mod,send}.rs` | §6 as amended (bl-e160) — the strip escalated to the desktop. `mod` is the whole decision, pure: a §8.5 queue row projected to the sentence a notification shows, the per-window baseline of what has already been said, and the two gates (focus, the §4.1 knob) that silence the announcing while the baseline advances regardless. `send` is the one spawn — libnotify's `notify-send` through the bare `git_env::command` constructor (not `Cli`: the desktop is not substrate and must not take the §16.2 world fold), synchronous so a test can drive it, with the window running it off-thread and every failure silent |
| `src/app/{mod,roots,view,boot}.rs` | AppModel — what a *frame* owns (§7.2): the held snapshot, ui-state integration, and — since the act path split it out of the root at this section's budget (bl-4841) — `refresh`, the per-frame duty entire: take the newest derivation, adopt an externally-changed `ui.json`, settle the wire's read and act hand-offs, hold the §6 ack; `roots` the boot-time fold of the composed world and the four derived paths every root read goes through (bl-3f46); the §11 transcript-density knobs and the whole-UI zoom (§4.1); the per-conversation view-model assembly the shell paints, plus the snapshot's staleness and live-cadence reads; `seat` the client/server line itself (REMOTE §9.4, bl-1eb0) — the reads a *seat* makes about its own selection, each one a resolution of the per-instance focus (§13.1) against the published snapshot, so what crosses into paint is a payload the wire could carry and never the engine's `GitTree`. **`focused_conversation` is gone** (REMOTE §9.7, bl-48ae): the whole seat view is no longer derived here at all — the facts a click reads are a fold over the landed `Query::Conversations` forest and the selection's own detail is a standing `Query::Agent`, both spelled at `src/shell/seat.rs`; `ops` the frame's two *writes* to the trail — the operator's ack and the clear verb (§4.2 as amended, bl-c417), here rather than in the excluded shell so the gestures a click makes are covered; and `boot` the model's **founding and its one outbound signal**, split off the root at this section's budget on the seam that root's own doc draws — the root declares what a frame owns, `boot` is how one is brought into being (the [`Deriver`] built and handed back as a pair, the first derivation taken synchronously so the window opens on real content and the §4.1 startup focus has a roster) and `mark_dirty` beside it is the frame's **only** way to reach the worker, so the root keeps carrying no coverable `impl` header |
| `src/app/balls.rs` | the frame's read of the live `bl` projection — the ops tail and the two post-verb dirty marks (§5.1, §7.2); the §3.4/§8.1 start hand-off. **The §3.2/§3.5 read half is gone** (REMOTE §9.7, bl-b4b5): `ws_balls`/`roster_ball_rows`/`bound_ball` and the whole `spend.rs` beside them folded the window's own join and bills on the paint thread; one workspace-addressed `Query::WorkspaceBalls` answers the listing *with* each ball's figure, and the roster's partition and the ▶ Continue row's object are pure selections out of it (`nav::balls`). `targets.rs` emptied out when the ball Move was retired (bl-6c28), and the focus's own name went to `app/view.rs` beside the resolver. What is left of the join here is the §8.5 line context's private read, which is the acts side of bl-adcb's line |
| `src/app/cadence.rs` | the clock's periods (§7.2, bl-3381): the `Cadence` value, its `cadence.yaml` grammar (total parse, shared bounds), and the derived periods (wound grace, late pass, staleness) |
| `src/app/derive{,/pass,/route,/sweeps,/fetch,/liveness,/worker}.rs` | the derivation worker (§7.2): its state — every effect it holds and every cache it keeps warm — beside `pass`, the one pass split off it at this section's budget on the seam that file's own doc named (what is dirty, what is due, what gets published); the §7.1 dirty-root routing table; the two sweeps, reconcile, the fetch cadence and re-deriving one root — the work every sweep ends in, moved beside them at the budget (bl-4b28); **which cached liveness observations are evicted and on which signal** — the sweep's poll for agents that can die silently and the watcher's refresh for agents that can come alive, complements by construction (§10, bl-1015); the thread that drives the pass; and `fetch` the live `bl` projection and the ops tail (§5.1 #2/#4, §4.2), split off `sweeps` at that same budget on the seam that file's doc already listed — there is what the world *is* (enumerate, reconcile, re-derive), here is what yog re-reads out of `bl` and out of `ops.jsonl`, on the clones-root dirtiness or the 15 s floor for the whole world and after one dispatched verb for one project, never per frame |
| `src/app/dirty.rs` | Change→dirty-root mapping, debounce/sweep scheduling over the live `Cadence`, `watch::Mark` provenance (§7.2) |
| `src/app/drift.rs` | the four drift kinds and their `ops.jsonl` fold, the late-pass and stale-snapshot thresholds, and the edge test that makes a permanently-late derivation one event rather than one row a sweep (§7.2, bl-4b28) |
| `src/app/grace.rs` | the §7.3 wound banner's grace window (bl-90bf): the render-layer age gate over the same injected clock, so a wound that heals inside the rising edge's own latency (`Cadence::wound_grace`, re-sized to all four of its legs in bl-18e8) never flashes |
| `src/app/snapshot{,/names,/scope}.rs` | the published derivation the frame renders, its age, the per-conversation branch-growth diff (§7.2), the per-workspace `steps/` fold every spend figure filters (§3.5, bl-9dd4), and the `models.yaml` context windows every fullness figure divides by (§5.1 #35, bl-a48b); `names` is the boundary's addressing read off it in **both** directions (REMOTE §8, bl-f5f6), so the two cannot disagree about what a name means, **plus `addressable`** — which sets that addressing reads at the intake (bl-6c9e, both nouns since bl-3377): the live §3.1 workspace enumeration and the §5.1 #1 project one in place of the derivation's cached copies, `Arc` in and out so an unchanged set is handed straight back, which is what makes a workspace's or a project's birth a barrier for the gesture after it (§8.5); `scope` is the REMOTE §4 narrowing to one client's registered workspaces (bl-8bbc) — **one** filter over every workspace-keyed field, which is what makes an unregistered workspace ABSENT rather than forbidden: the roster simply does not list it and the resolver refuses it in the identical bytes a name nobody founded earns. *One* filter is the whole point and it must stay literally one: the §4.3 `fleet` map got a second predicate of its own on the belief that its key was a leaf, and because the key is the `cadence.yaml` entry — a **path** — that predicate was total, dropping every armed loop from every scoped snapshot while the loop went on acting (bl-8bf6). Every field here reads `keep`, and the fixture is keyed as the worker publishes it |
| `src/attention/{mod,roster}.rs` | the §7.3 attention flag: the ack state machine (incl. `evidence` — the **one** definition of what an acknowledgement writes, read by the window's focus tick and by the §8.5 `seen` action, and naming neither of the two signals no watermark may answer: mail, and the §8.6 park) and `AttentionKind::says`, the one home for each rule **in words** (bl-e160's desktop alert states it where the badges glyph it); the per-conversation roster it is raised against |
| `src/badge.rs` | **what a derived row says in words** (§11, retired): a glyph and the fact it stands for, together, in one home per fact — [`op_badge`] over the ops trail's outcomes, [`tool_result_badge`] over a tool result's one flag, each total over its subject so a new outcome cannot ship wordless. It is what a **server** keeps of the congeries palette (bl-7942): the hues, the visuals, the fonts and the application mark were statements about how a face paints and went with the face, while the WORDS are a derived row's own content and cross the boundary inside the row |
| `src/binding/mod.rs` | names-root enumeration (§3.1), claimant join (§3.2), worktree formula, workspace classification |
| `src/board/{mod,rows,rollup}.rs` | the V4 board (§11, VISION §5 V4): the four columns as balls' ladder crossed with its close-gate predicate, and the whole board built pure over one snapshot — its rows, and (bl-66fb) the facts of any §4.3 loop armed over them, empty in every unarmed world; one row's gate, drones and figure; the epic rollup that crosses workspaces, one slice apiece |
| `src/boundary/{mod,action,address,address/agent,address/workspace,query,codec,codec/start,codec/balls,codec/config,codec/deposit,codec/query,codec/query/inspector,codec/monitor,codec/fleet,codec/control,codec/fork,codec/fan,codec/tools,codec/fields}.rs` | the §8.5 typed surface: the Action and Query enums both frontends construct, each in its own file at the cap — `action` the mutating roster (bl-8746) beside `query` the populating-read one (bl-765d), cut on §8.5's own taxonomy, the seam the help table is already cut along — an enum cannot be split across files, so `action` makes room the only way a roster can: a family whose members every layer beneath already reads as a pair folds to ONE variant over that family's own `Verb` (the monitor's, the fleet's, the routing leg's, and — when `interrupt` arrived at the cap, bl-a33d — the §3.8 fan's), which is a real seam and not a line budget precisely because the seam is already drawn three files down — with `address` — **what a gesture addresses**, two tables (which workspace, which project) that are *queries on* the enums rather than parts of them, split off at §12's cap (bl-dc0c) and widened to both nouns by bl-f5f6, so REMOTE §8's one name→path resolution stands once ahead of each chokepoint's table instead of inside twenty arms; `address` is now the **project** table alone, each other noun having earned its own file for a reason worth stating where it lives: `address/agent` is the **third noun** (bl-49bc), its own file because the rule differs where the two above share one: a workspace and a project resolve over an enumerated set of paths, while a conversation is addressed by **an agent id or the unique stored name a living agent wears** — the vocabulary the `Started` receipt speaks — so it carries that ladder beside the table rather than a third table here, and `address/workspace` is the noun that is also **written** (REMOTE §8.2, bl-4e31): a §8.2 entry's client-side leaf may differ from the name that workspace bears on its host, so the table is *borrowed* rather than read — ONE table, with the read answering through the write, because two exhaustive matches over thirty-odd variants are two representations of one fact and the arm that drifted would send a client's own leaf to a host that never heard of it — and the §9 config family answers through its destination's own row (bl-523f), the one address nested a level down inside `target`, whose omission sent a config act aimed at a renamed remote wall to the LOCAL engine's file; the rewrite through that borrow spends the mapping at the channel boundary and nowhere else; the headless JSON envelope, exhaustive both directions (the VISION §4.8 compile gate), cut per family — `codec/config` the §9 destination, the §16.3 mode and the §9.3 origin (bl-3f46), and `codec/query` every populating read's spelling, so the top-level match is the action roster and chains to each family's own reader before it refuses an unknown op (bl-3746; `config`/`marks` read there too since bl-0164, recognized only in their fieldless shape before falling through to the write), with `codec/query/inspector` split off at the same cap (bl-6233) on the seam the §11 family draws — those six are the only reads addressed at a *conversation* rather than a workspace, so the address they share is written once and chains ahead of the sibling table; `codec/monitor` the VISION §4.9 family (bl-8da1); `codec/fleet` the VISION §4.3 armed loop's two, total where the line is terse — the envelope has no seat, so it names the workspace, the project *and* the cap yog will not guess (bl-66fb); `codec/control` the VISION §4.11 hold answer, whose one field is a verdict and whose `tool_use` id is deliberately not on the wire (bl-765d); `codec/fork` the V2 attempt, whose `skills` list needs the one strict array reader the scalar fields do not (bl-dc0c); `codec/fan` the §3.8 mutating fan's three — spread, retire, and V3.2's deliver (bl-c2bd) — whose shared half is an **optional** obligation: a fan with no ball is the bare project-repo one, and absence is a value there rather than a malformed gesture (bl-8746); `codec/balls` the `bl` family's six envelopes, split out at §12's budget when the fan's third arm arrived (bl-c2bd), on the seam every sibling family file is cut on; `codec/tools` REMOTE §5's tool-host presentation (bl-4e08), which carries no client field on purpose — the identity a set lands under is the intake's, and one on the wire would let a connection write another client's set — and whose *element* spelling is deliberately not here but in `registry::tools`, the same encoder the stored document spends; `codec/deposit` the two **depositing** envelopes (bl-a33d) — a plain send and send-and-interrupt, three identical fields with the op word as the whole difference, said once for the reason `ball` is and named once so the two directions cannot drift; and `codec/fields` the total field readers every family imports, split off at §12's cap on the seam `line/args` already draws one serialization over — the verb roster is one thing, what a field is read as is another, and strictness lives there (bl-66fb) |
| `src/boundary/{answer,answer/agent,answer/balls,answer/chrome,answer/confirm,answer/convs,answer/inspector,answer/queue,dispatch,dispatch/advertise,dispatch/arms,dispatch/deps,dispatch/doors,dispatch/delete_exec,dispatch/enroll,dispatch/resolve,reply,reply/model,reply/agent,reply/balls,reply/cleared,reply/encode,reply/decode,reply/decode/inspector,reply/rows,reply/rows/decode,reply/board,reply/board/decode,reply/search,reply/queue,reply/ws_row,routing,ceiling,interrupt,monitor,fleet,control,control/floor,fan}.rs` | the two §8.5 chokepoints, symmetric in shape (`answer(query, deps, ui, now) -> Result<Reply, String>` beside `dispatch(deps, ui, ts, action) -> Result<Reply, String>`, bl-0164): queries mostly pure snapshot derivations the frame's view-models delegate to, the §9 config family's three read from `Deps`'s world exactly as their writes do and so can refuse as they do; actions routed to their §8 executors; the typed replies and which encoder each spends, `reply/model` the answer enum ITSELF — every variant and the doc saying why that outcome is its own row — cut off `reply.rs` at the §12 pre-split band on `start/model`'s seam (bl-1015): what an answer *is*, beside the modules that say it, leaving `reply.rs` the family's doc, its module roster and its re-exports; `reply/encode` the whole surface's one JSON spelling, split from the type at the budget on the seam the codec is already cut along — a `Reply` is what the boundary answers, that is how the transport says it, and the window never comes there (bl-6233) — and `reply/rows`/`reply/board`/`reply/search`/`reply/queue` each cut off at the §12 budget on the seam of one reply whose row carries derived sub-objects, its own address shape, or a derived list — and since bl-1015 `reply/search` carries that reply's **envelope** too, moved off `encode`'s match so the one place that learns how a search answer is said is the file its rows are already spelled in; `reply/ws_row` is the one listing row the boundary itself **owns** (bl-296f) — a workspace named, classified, §6-rolled-up and §4.1 pin-ranked, whose subject exists only as an answer where every other listed thing is somebody else's type (`ConvRow` is `nav`'s, `JoinRow` is `projects`', `OpRow` is `opslog`'s); the pin **rank** rides it rather than a flag so the §11 tab bar can hoist in pin order without joining the answer back against the engine's own `ui.json`, which is bl-7407's refused shape. `reply/decode` is that spelling read back into the type (REMOTE §9 step 2, bl-7067) — the thin seat's half, strict as the gesture codec's decode is, keyed on `kind` because `ok` is the captured run's own exit verdict rather than the envelope's, with `reply/decode/inspector` split off on the seam `codec/query/inspector` already draws and `reply/rows/decode`/`reply/board/decode` beside the encoders they undo; and the §3.5 spend gate's one seat inside the `Prompt` door, whose refusal is a §4.2 `yog-step` row before it rides back (bl-56d5). `answer/confirm` is the §3.6 unmaking's own derivations — what a delete would destroy, read by the dialog and the dispatch gate alike — split out at the budget on the seam `answer`'s own doc already drew (bl-6233); `answer/inspector` the §11 family's five reads (REMOTE §9 step 1, bl-6233): the transcript with the in-flight tail folded on exactly as the window folds it, the steps whose liveness is read off the snapshot rather than taken as a parameter, the worktree listing whose named path is resolved against that same listing — with `working_dir` beside it since bl-1015, the §3.3 cwd mark read back so a conversation bound to a work target says where its deliverable went instead of answering a listing that holds none of it — and the spine's gather — the two derivations the frame used to do for itself, so the shell's memo now wraps a call here instead of a second copy; `answer/agent` is the family's seventh member (REMOTE §9.4, bl-1eb0) — one conversation as a *seat* sees it, the `Agent` view-model's wire projection: the §11 centre pane's identity line, §6 marks, live class and §8.2 verb gates, every one of them a fold the boundary already owned and none of them spellable before, with `reply/agent` both directions of its own spelling beside `reply/search` and `reply/queue`; `answer/queue` is the §6 decision queue (VISION §5 V5.2, bl-f6fe): the flattened roster the ↓ key and the queue share, the queue itself, and the acknowledgement that answers one row; `dispatch/deps` the environment a gesture executes in, split out at the cap, `dispatch/doors` the §8.1 start family's two `pub` typed doors beside it (bl-9bef) — the other way into the chokepoint, on the seam the module doc already drew, and where the §4.11 confinement refusal and the §3.5 ceiling gate a birth — and `dispatch/delete_exec` the §3.6 unmaking's two executors (bl-765d) — the seam being that every other arm *routes* while those two **gate**, re-deriving their confirmation at fire time and refusing fail-closed; `interrupt` is §8.2's send-and-interrupt (bl-a33d) — the one arm that composes **two** substrate acts, a `litany stop` and the `litany message` whose own driver-start is the trigger, so it is a body rather than a row and leaves the two §4.2 rows those verbs each leave; a stop that could not spawn refuses ahead of the deposit, a stop that ran and was declined does not, and both facts are on the trail rather than in a composite row. `monitor` the VISION §4.9 arm/disarm/flag executors — a `cadence.yaml` entry write and one trail row, bodied out of the chokepoint's table (bl-8da1); `fleet` the VISION §4.3 arm/disarm pair, the same shape one block over — no policy file to seed and no first spawn, which belongs to the loop's next tick (bl-66fb); `fan` the §3.8 mutating fan's three (bl-8746; V3.2's deliver, bl-c2bd) — the only executors that reach a linked crate **in process** rather than spawning, upstream having ruled that the attempt capability may have no `bl` verb, so all three leave `["yog-step",…]` rows instead of argv ones; `control` the VISION §4.11 capability family — the hold answer's `["yog-control","answer",…]` row, the detached `litany advance` that releases it, and the confinement-required birth gate both drone doors run (bl-765d) — with `control/floor` the family's other writer beside it (bl-94b4), VISION §4.9's fifth rung as the `["yog-control","floor",…]` row the same fold reads back; the seam is real and not a line budget, since the answer resolves **one invocation** off a live mark and drives the branch on, while the floor writes **standing policy** over a whole descent and launches nothing. `routing` is REMOTE §5's routing leg, engine side (bl-024b) — the four arms that put an invocation in a tool host's mailbox and bring its capture back, in ONE module because they are one mechanism read from four sides and splitting them by mutate/populate would leave nowhere to state the invariant that binds them: `invoke` queues and answers a handle without waiting, `invocations` is the follow-class read (which blocks the CONNECTION thread and never the deposit consumer, because an in-world caller is refused before the wait rather than parked in it), and `complete`/`capture` are bound to the addressee and the asker respectively, a handle belonging to neither being **absent** rather than forbidden. `dispatch/resolve` is the chokepoint's one address resolution and the §4.1 raise it carries, split off at the cap (bl-4e08) on the seam the module doc already draws — everything left in `dispatch` is the `Action` table, and this is the one thing standing *ahead* of it; `dispatch/arms` is what stands *beside* it (bl-c088): the seven bodies the table's arms call — the three-row `bl` spend, the prepare door as a reply, the two `io::Result`→`Reply` folds, the §9.4 retarget routing (the change of lineage, bl-e654), the gated fork and the §6 acknowledgement — each already a body rather than a row because its arm is a call, so cutting them out leaves `dispatch` its two resolutions and the match and nothing else; `dispatch/advertise` is REMOTE §5's tool-host presentation (bl-4e08), a third gating executor beside `delete_exec`, and the only one whose gate is **who is asking** rather than what was named: it addresses no workspace, so its authorization is the identity the intake carries, and an intake with none refuses in band rather than being dropped. `dispatch/enroll` is REMOTE §1.4's enrollment (bl-f4e3), the fourth: it mints a device's leaf on this box's CA through the ONE `wire::provision` recipe, seats its registration in the workspace the gesture named, answers the material and **shreds the key**, leaving the certificate because that is the guard against a second issue under one name. Its gate is the one it does NOT write — §4.2's foot set is enumerated, an act is outside it, so `answer_as` refuses a foot in band before the chokepoint runs and a check here would be a second authority for one fact. What it refuses itself is material a device could not use: a box with no CA, and an `address` whose port is the `:0` a self-provisioning boot wrote, since only the listener knows what that became and it becomes something else at the next boot. `answer/balls` is the §3.2/§3.5 family taken as **one question at one altitude** (REMOTE §9.7, bl-b4b5): every ball one workspace holds, with its badge, the §5.1 #1 project name its `bl` verbs run in, the claimant they stamp `--as` and its priced figure — distinct from `Balls` by *address* the way `Board` is by altitude, and the figure rides the row because it is a filter over the very `Snapshot::bills` walk the listing is made from. `answer/convs` is the §11 conversation list of one workspace and the two reads that hang off it (bl-c088) — the forest rows, the ball facts each row renders, and the §3.3 mint's occupied name set — one subject rather than three, since all three read the same derived tree and the listing spends the ball read itself; cut on the seam that everything left in `answer` is the `Query` table and the two resolutions ahead of it, while this is a derivation the table calls, beside `answer/chrome`'s. `answer/chrome` is the altitude-0 answer split off the chokepoint at the budget on the roster's own seam: the enumeration, its §6 rollups, the §4.1 pin rank, the §2.2 lineage tip, and — since the snapshot carries its completion as a wall-clock stamp — the §7.2 currency of the derivation itself, which rides here rather than on a question of its own because this is the one read every window makes every frame. `reply/balls` is that listing's row in both directions, beside `reply/agent` for the same reason, spending the board's own figure codec rather than restating it; `reply/cleared` is the composer's draft-clearing predicate, its own file at the budget on a real seam (bl-40ab) — a `Reply` is what the boundary answers, and whether an answer clears the draft that asked for it is a question a *seat* asks about one |
| `src/boundary/config{,/file,/write,/read}.rs` | the §9 family's six executors — the apply/marks/pick gestures (bl-3f46) and the read/read_marks/providers queries beside them (bl-0164), with `file` the `ConfigFile` **destination** split off at the cap (bl-f5f6) on the seam `action`/`query` are cut on — a destination is a datum every seat constructs, its addressing rides with it — the sphere it names is a row of the ONE workspace table (`address/workspace`, bl-523f), so a config gesture is addressed like every other one and the §8.2 channel mapping rewrites it, while the name→path resolution stands once at each chokepoint rather than a second time inside the family — and the pipelines that spend it stay here — cut from the pipelines each destination runs (the §9.1 `bz` gate, the §9.2 hash-guarded write — unjudged since bl-3ffa retired its provider gate, so `Deps::provider_rows` went with its one caller — the §9.3 staged `litany config` commit, the two verdict folds, and `read`'s own `load_snapshot`-minus-the-hash twin) |
| `src/boundary/corpus{,/ledger,/store}.rs` | the **wire conformance corpus**'s generator (REMOTE §3, bl-32cb) — test-gated, shipping nothing: it renders the committed `corpus/` fixture set from the codec's own round-trip surfaces (`codec/tests/surface{,/conversation,/ball}.rs` for the request half, the reply round trip's `surface` for the answer half), so the fixtures every client replays and the fixtures yog proves itself against are ONE list. `ledger` holds `corpus/shapes.json` — per shape, the field signature and the protocol version at which it last moved — which is what makes REMOTE's *a wire-visible change bumps the version* mechanical rather than remembered: the fixtures alone cannot enforce it, because regenerating them erases the diff, so the record remembers what the shapes WERE and `make corpus` refuses a signature that moved under a standing `PROTOCOL`. `store` is the disk half, one renderer feeding both the gate (`corpus::gate`, an ordinary test) and the regeneration, so the two can never disagree about what the corpus should be |
| `src/boundary/{deposit,consume,consumer,sugar}.rs` | the §8.5 deposit transport: the gestures-inbox protocol (claim-by-rename, reply files, and `mint` — the id won by an exclusive reply-slot reservation rather than guessed from a clock and a pid, bl-aa9f); one consumption pass; its thread; the `yog gesture` deposit-and-wait sugar (`sugar/argv.rs`: its one payload — envelope or line — and the context flags a line reads its targets from, `--prepared` among them, which is how the start flow's two steps compose across two processes, bl-44d8) |
| `src/boundary/follow{,/open}.rs` | the **follow lane's engine half** (REMOTE §3, §10; bl-73e7): `Query::Follow` answered as a frame *sequence* — one frame per growth of the conversation's open `response.json`, and no terminator until the stream closes. It is a **cadence, not a second reading**: whether there is a tail at all is `answer::inspector::live_tail`, bl-6233's one describer unmoved, and what this adds is where the bytes come from — the derivation folds the whole file on the worker's schedule, `open` folds the suffix on the writer's, and the two agree by `Stream::absorb`'s contract rather than by coincidence. A response file belongs to one step, so a step advancing is not an accumulator to reset but **this stream ending**, which the framing already spells; the seat then swaps to the committed entry the pull read carries and re-asks. The hold is bounded by *quiet* looks only — writing a frame is what discovers a peer that went away — and the snapshot is read off the cell per look rather than off `Deps`, because a read that deliberately outlives its request cannot be gated on a derivation frozen at connect. The **address** is still resolved once, at connect, under the caller's scope (REMOTE §4): a held read is one request |
| `src/boundary/help{,/table,/table/driving,/table/standing,/table/queries,/table/world,/table/following}.rs` | the §8.5 verb table — every gesture's usage, one-liner and page — and the one text rendering every seat prints; the single source for refusals, `/help`, the codec's verb check and the parity tests. The roster outgrew one file at the cap (bl-dc0c) and is cut along §8.5's own taxonomy — `table` the acts on a conversation or a ball, `table/driving` the six of those whose subject is a conversation **already running** (bl-c088: send, cut off, kill, sweep, prompt again, and the §9.4 `retarget` change-of-lineage, bl-2d19 as re-scoped by bl-e654 — none of the six creates or destroys anything, and each is spelled by a `litany` run), `table/standing` the verbs whose subject is a setting, a standing policy or a record (the §4.9 monitor, the §4.3 loop, the §9 config family, the §4.11 capability answers, the trail's own two — split off at the cap when the exit landed, on the seam where the subject changes), `table/queries` the populating reads, `table/world` the eleven of those aimed at a workspace or the world above it (bl-c088: what exists, what each workspace holds, what its agents changed, the V4 board, and the §9 provider/client/lineage/model tables a workspace resolves — `providers` among them, bl-0164 — leaving `table/queries` the reads aimed at one conversation and the five that name nothing at all), and `table/following` the **follow-class** two (bl-73e7) — the reads whose answer is a *sequence* and whose hold is a connection thread, split on REMOTE §3's own seam rather than a line budget, since each page has to explain the same fact from two sides: an intake that can hold a connection answers many frames and one that cannot answers one, and the second is a true answer of the same question — then read back as one by `help::table()`, a function rather than a const because const slices cannot be concatenated in a const: each split is a line budget and must never become a list an operator can read half of, and each of the two bl-c088 cuts is a **prefix** rejoined where it stood, so six lists render the one roster in the one order |
| `src/boundary/line{,/args,/parse,/queries,/spell,/config,/verbs,/balls,/fork,/fan,/tools}.rs` | the §8.5 **line**: the slash spelling of the boundary — the `/`-marker and its `//` escape, and the higher-order help rule read once above the verb match; the argument grammar (context reads that refuse by name, the `--flag value…` split, and — since bl-06a1 — `goal_or_prefill`, the one home of the §3.3 rule that `/prompt`'s goal is optional and falls to `prepared.goal` whole when nothing is typed: a **default, never a concatenation**, because a composer seat sends the edited whole and a `/prompt` that prepended would fire every composer-typed prefill twice); the reader — the mutating verbs, chaining to `queries`, the populating half in its own file since bl-6233 (it was a function inside `parse` from bl-0164) along §4.8's taxonomy, the same seam the codec and the help table are cut on; and the writer whose exhaustive match is the compile gate; `config` the §9 family's own grammar, reader and writer in one home, whose destination is words and whose tail is the file verbatim (bl-3f46) — and (bl-0164) whose *absent* tail, on a destination other than a lineage, is `/config`'s and `/marks`' own read; `verbs` (bl-3746) the per-verb argument builders the reader calls into — the §4.9 monitor's three and the §4.3 loop's two among them (bl-66fb); `fork` the V2 attempt's, whose flags lead so the goal can be the verbatim tail (bl-dc0c); `fan` the §3.8 mutating fan's three, beside it on the same seam — a family whose gestures read an obligation off the seat rather than a bare tail (bl-8746; `/deliver`'s summary is the verbatim tail after its handle, bl-c2bd); `balls` the `bl` family's own grammar beside them (bl-dbde) — the id-taking verbs' ball, the re-home, and the two authoring verbs whose payload is the family's own vocabulary; `tools` the routing leg's two acts (bl-024b), both of which end in a document taken verbatim to the end of the line, exactly as `/advertise`'s set does |
| `src/boundary/sugar/argv.rs` | `yog gesture`'s own argv (§8.5): the one payload — a JSON envelope **or** a slash line — and the context flags a terminal line states its elided targets with, the terminal holding no selection to read them off |
| `src/budgets/{mod,bills}.rs` | the Usage-event vocabulary, parsed in one place (§5.1 #16) — the fold of every attempt segment beside `last_usage`, the final segment alone (§5.1 #35), and the **one home of the prompt reading** every figure above shares (`prompt_tokens` / `uncached_prompt_tokens` / `total_tokens`, bl-6621: the counters overlap, so a total is a fold and never a sum); the one `steps/` walk every figure shares — per-step fold beside the model that billed it, the conv that owns it and its own `seq`, so a `Scope` applies after the walk and "which step is the latest" is asked in memory (§3.5, bl-9dd4; bl-a48b) |
| `src/bz_host.rs` | the entry to the linked brazen (§16.7): the wall fold, the snapshot brazen reads, the route decision, and the no-wall refusal (§16.2) |
| `src/bz_host/routes.rs` | the six `bz` routes and their seam bundles — `bz`'s own `main.rs`, one layer in |
| `src/bz_host/store.rs` | yog's wall-rooted `CredStore`/`ModelCache` (§16.2): brazen's shim, rooted at a path instead of the process env |
| `src/cli_outbound/{mod,run,detach,chunk}.rs` | what a `Cli` *is* — the physical/logical split parametric binary resolution turns on (§8); `run` the streamed spawn class, cut on the same seam every other shape already sits behind; the detached spawn and its stderr sink (§8.1); the stream-chunk framing |
| `src/cli_outbound/exec.rs` | the `yog exec` escape hatch's spawn shape (§8.4, §16.4): a blocking wait with **inherited** stdio, split from `mod` so the piped `run` family keeps the cap |
| `src/cli_outbound/piped.rs` | the **stdin-piped** spawn shape (REMOTE §5.2, bl-024b): `run_in` with the child's stdin a pipe yog writes and closes rather than `/dev/null`, which is litany's own tool contract (its ARCH §3.3) and therefore what a tool host's own child speaks. It reuses that spawn whole rather than restating it, and the deadline on one is the stream's drop — the SIGTERM-then-SIGKILL cascade `stream` already owns |
| `src/cli_outbound/resolve.rs` | binary resolution (§16.7 W12, the self-multiplex spine): which executable a `Cli` execs and under what leading argv — the one switch point, a per-namespace `self_multiplexed` const |
| `src/cli_outbound/self_exe.rs` | **which file yog itself is**, asked once per process (§16.7's fourth standing fact, bl-f558): the memo every self-multiplexed resolution, world tool shim and §9.3 `$EDITOR` re-entry spends, so one fact has one home. Two rules, both here rather than at the call sites — the reading is taken at the first ask (which every face performs at boot, before an install can replace yog's inode under the live process), and a reading naming no file is no reading, judged by `stat` and never by spelling: no `(deleted)` suffix is stripped or matched, because only the filesystem can tell an unlinked binary from one genuinely installed under that name |
| `src/cli_outbound/stream.rs` | the live-handle half: the running-subprocess handle whose iteration yields chunks (final item always the exit) and whose drop terminates the child — SIGTERM, then SIGKILL after a short grace (§2.9) |
| `src/cli_outbound/streamed.rs` | the **streamed-piped** spawn class (§8, §8.3): both output streams line-buffered off a running child, each line tagged with the stream it came from, plus the terminal exit — the shape `bz --login` renders through (§5.3) |
| `src/cli_outbound/sys.rs` | the crate's confined `unsafe` — `libc::kill` for the drop's best-effort SIGTERM, `set_env` for the world fold a substrate arm stands in (§16.2, bl-81c9), `term_disposition` for the §8.5 SIGTERM catch without which none of the others is ever reached (bl-269a), and `ignore_sigpipe`, the disposition `git_env::exec` puts back after an `exec` returns (bl-3792); the one audited home `rules/unsafe-outside-sys.yml` leaves it (AGENTS.md rule 3) |
| `src/cli_outbound/wrap.rs` | the **confinement wrapper** seam (§8.6): a physical argv prefix standing in front of `program` the way the namespace `prefix` stands behind it, generic here like the standing env — which backend the words name lives in `control/confine`. The logical `binary()`, the ops-log argv and the W9 shim stay wrapper-blind: the trail records the act, and the shim runs *inside* the sandbox, where re-wrapping would nest a second one |
| `src/config_edit/mod.rs` | §9's root: load → edit a RAM draft → Apply = stage → validate → hash-guard → atomic rename, one discipline across every file-editing surface |
| `src/config_edit/apply.rs` | the `--editor-apply` copy — drafted files only (§9.3) |
| `src/config_edit/branch.rs` | config-ref browse, the **governing**-config ancestry query, edit plan (§9.3). It keeps the query and hands it on: the derivation that turns a fork commit into an answer lives one level down |
| `src/config_edit/branch/edit.rs` | the §9.3 edit half — the scripted-`$EDITOR` drive of `litany config`, the only lawful writer of `config/*` (ARCH §2.2), re-entering the yog binary at `config_edit::apply` |
| `src/config_edit/branch/edit/staging.rs` | §9.3 step 1 and §5.2 step 5, split off `edit.rs` at the §12 pre-split band along that module's own numbered flow: the `<nonce>/` dir a drafted file waits in for litany's `$EDITOR` callback (`DraftFile`, `next_nonce`, `stage_files`) and the startup sweep of the ones a crash left behind (`stale_staging` the clock-injected pure decision, `sweep_staging` the best-effort delete). A scratch-dir lifecycle with no subprocess in it; steps 2–4 are an argv, an environment and a spawn, and the dir is all the two halves share |
| `src/config_edit/branch/follow.rs` | the §9.4 **follow** derivation and the answer type it produces (bl-e654). The seam is where a fact stops being ancestry: `branch.rs` answers *which `config/*` commit is this agent's fork point*, a pure walk that never moves, and this file answers *which commit does the conversation therefore resolve* — the config heads containing that fork point, reduced to distinct tips, one of them **followed** and two or more **held**. It is a faithful port of litany's `workspace/current_config.rs` exactly as the ancestry query above is of its `workspace.rs::governing_config`, and for the same reason: yog derives the fact itself from git rather than reading it off litany's operator stderr (§9.4). `GoverningConfig`'s `governance`, and the one `label()` wording every seat prints, are here |
| `src/config_edit/brazen/{mod,effects,paths,providers}.rs` | the §9.1 editor (staged validation, hash guard); `paths` the wall's `BrazenPaths` layout and the two reads that need nothing but it — `credential_presence` and `model_cache_at`, which the §8.3 Login surface, the boundary's `Providers` reply and the §9.4 pick all ask holding no draft — cut off `mod.rs` at the §12 pre-split band on the seam its own prose already drew (*"free of the editor"*); the real BzRunner; the provider-row projection (§8.3) — the three consumed columns, `auth` → `login_blocked` (§8.3), and the rendered row every surface paints |
| `src/config_edit/brazen/providers/capability.rs` | what the `protocol` column says about a yog turn (§9.4), split off `providers.rs` at the §12 pre-split band along the seam it already had: the parent owns the columns, this owns the dialect judgement over one of them. Two reads, both a total match over brazen's own public `ProtocolId` so a new upstream dialect fails to compile rather than being guessed at — `tools_blocked` (bl-3d22), which `plan` refuses on, and `context_caveat` (bl-671d), which nothing refuses on: a dialect that leaves the context size to the server is stated beside a selectable row, with `CONTEXT_REMEDY` as the operator's next move, because yog cannot see what the server chose. `dialect_decline` (bl-5252) is a third **way in** and not a third read: a dead step's own words scanned for the protocol spelling brazen's declines lead with, handed to the same `tools_blocked` match, so §7.3's banner and §9.4's control share one judgement |
| `src/config_edit/draft.rs` | the ONE staged-edit `Draft` both §9.1/§9.2 editors are built from — dirty tracking, revert, the hash guard |
| `src/config_edit/effects.rs` | the production `FileIo` — the thin `std::fs` shell behind every editor's pure view-model, covered against a real tempdir with no fakes |
| `src/config_edit/fault.rs` | **where a config-kind failure is fixed** (§9.1, bl-dd7f): the narrow classifier over a failure's own words — litany's `provider error (config)` wrapper and brazen's `unknown provider` — the row it quotes, read out of the sentence rather than joined from the tree, and the one line the §7.3 banner pairs with a route to the §9.1 editor. §8.3 rule 5's sibling on the other kind of fault, and it re-words nothing: brazen's and litany's sentences stay verbatim above it. **Two ways in since bl-5252**, because brazen's dialect declines carry no config-kind word at all: a marker hit, or the request-shape family reached through `brazen::dialect_decline` — the same `tools_blocked` judgement §9.4's picker refuses on, so no dialect fact is written down here |
| `src/config_edit/form{,/schema}.rs` | the §9.5 typed pane: a setting read from and written back into the draft through the §9.4 grammar, with the shared provider judgement; and the enumeration itself — which settings exist, and the control each gets |
| `src/config_edit/litany_global/mod.rs` | the §9.2 editors — the shared pipeline and nothing else since bl-3ffa retired the provider gate over `models.<id>.provider`, a field whose only reader was the refusal |
| `src/config_edit/pipeline.rs` | the write pipeline every §9 editor shares: the one home for how a draft reaches disk without a torn write or a silent last-writer-wins over a concurrent edit |
| `src/context/mod.rs` | §5.1 #35 — the context-fullness query (the root's latest step's prompt against the window `models.yaml` declares, `None` wherever nothing measured can be said) and the one line §11's settings rows paint from it. Pure over `Snapshot::bills`; the prompt reading's two-provider rule lives in its header and nowhere else |
| `src/control/{mod,wire,classify,bash,lex,rules,rules/table,policy,hold,root,judge,author}.rs` | the §8.6 capability control (VISION §4.11): the consult a `world/tools/` shim runs, and the one sentence a park hands the operator — tool, bounded input summary, class, evidence; litany's two wire shapes; the effect vocabulary and the built-in intrinsic map; the bash ruleset over every program a command runs; the shell lexer that finds them; the grammar one rule is written in, with `rules/table` the shipped ruleset as data — one list, because first-match-wins makes its order the policy; `policy` the per-workspace override that ruleset is the default of — `capability.yaml` at the live config tip, four keys, absence *is* the defaults (bl-765d); `hold` litany's valued hold mark, read one agent at a time by the answer gesture and whole-namespace by the snapshot tick; the writable root and its lexical containment; the class→verdict table folded with the trail's answers and floors; `author` the workflow fixed point that makes a workspace born adjudicated **and born unbounded** — one pass over `workflow.yaml` that authors the `tool_control:` block and strips litany's whole-tree `budgets:` ceiling (bl-56af: §3.5's dollar ceiling is the one that survives, and a template only reaches workspaces born after it); its *drive* is `start::ensure`'s single convergence, shared with §3.7's manifest glob |
| `src/control/confine.rs` | the **OS confinement backend** (§8.6, VISION §4.11 item 8, bl-bca4): the platform switch (Linux is bubblewrap, shelled like §16.7's openssl mint — no crate, no `unsafe`; every other OS an explicit refusal naming itself), the availability probe that runs the exact sandbox shape a wrap spends (derived at each birth, never stored), the birth-gate refusal both drone doors call, and the wrapper argv — the fixed shape plus the derived writable set, unconditional under a `confinement: required` policy so an absent backend fails the spawn loudly rather than falling back bare. The set is four members and each is a derivation: the workspace and the composed world root off the env, the host `/tmp` off the fixed shape, and the **bound project repo** off the §3.2 claimant join `control::root::claimed` already owns (bl-34b1) — so a revived driver, which carries no payload, confines exactly as the fire it resumes did. A member that is not on disk drops out rather than failing the spawn on `bwrap`'s own refusal (§3.5's orphaned project), which can only narrow the set |
| `src/delete/{mod,exec}.rs` | the §3.6 unmake: pure confirmation + plan; the logged runner |
| `src/delete/agent.rs` | the §3.6 one-conversation delete (bl-f17a): the member-scoped gate, the blast-radius arming, the `DeleteReport` census parse, the dry-run and removal spawns |
| `src/elide.rs` | **where to cut a string that will not fit** (QUALITY G1, L4; bl-3aa1) — one rule, *cut where the information is not*. Prose is written front-first and keeps its head, so the eight prose sites (previews, reasons, titles) are correct as they stand and are deliberately NOT routed here — a module claiming every cut while they kept their own would be a false claim. A **machine string** (absolute path, spawned `argv`, ancestry chain) is invariant at the front and distinguishing at the back, and that is `middle`'s case and the only one: the activity rows all opened with the same `/home/<user>/.cache/…/data/yog/` run, over half the row, while the workspace leaf and agent id that told two operations apart were exactly what the old head-keeping cut discarded. Carries a legibility FLOOR a tighter cap is raised to, since `…e` names nothing. The other half of L4 — an **id**, whose distinguishing end is a whole terminal segment rather than a character count — is a floor and not a cut, and keeps its one home in `nav::convs::id_floor` (bl-63a1) |
| `src/engine.rs` | **the one assembly a bare `yog` boots** (VISION §5 V5, bl-f6fe): the §5.2 startup sweep, the §7.1 roots, the model's first synchronous derivation, and the derivation worker + watch bridge + gesture consumer + VISION §4.9 monitor sentry + VISION §4.3 fleet pilot spawned beside it (bl-8da1, bl-66fb — each is a fact of the world, so none rode a face even when there were two). It left `main.rs` precisely because that file is coverage-excluded and carried the assembly twice. Since bl-7942 it hands a face nothing at all: the four channel ends it used to mint for a window — the read path, one per §8.2 entry, the follow lane's and the act path's — went with the window, and what a seat gets is a socket (`src/wire/server.rs`) |
| `src/engine/serve.rs` | **the windowless face, whole** (§8.5, bl-269a): `Engine::serve` — seed the world's §8.4 tool shims, catch the §8.5 stop before the engine exists, boot, park, come back stopped. It left `main.rs` for that file's own standing reason (it is coverage-excluded, so what lives there drifts unwatched), and what had kept it there was that it never returned: a face that parks forever is not a function a test can drive. The stop is what dissolved that, so the whole headless face is now under test and `main.rs`'s arm is one call |
| `src/engine/stop.rs` | **what a SIGTERM means to a running yog** (§8.5, bl-269a): the disposition catch, the process-wide flag, and `Engine::park_until_stopped` — which takes the engine BY VALUE, so returning from the loop and dropping the engine are one act and parking-without-dropping cannot be written. Almost entirely a subtraction: dropping the engine already stops and joins every thread (§7.2) and SIGTERMs every piped child (`Stream`'s drop), so nothing here drains, orchestrates or waits on its own account, and nothing here bounds — `yog.service`'s `TimeoutStopSec` is the bound |
| `src/fan/{mod,cohort,delivery,retention,spread}.rs` | the §3.8 **mutating fan** (VISION §4.10, bl-8746) and its V3 resolution (bl-c2bd): `mod` the family's own `Verb` — the three gestures the boundary carries as one `Action::Fan`, folded there in bl-a33d beside `monitor::Verb` and `fleet::Verb` and for their reasons — the obligation and the one route from a handle back to a live attempt, a thin in-process consumer of balls' attempt capability, which owns every name and path here; `spread` the materializing half, split out at §12's budget when the delivery arm arrived — open N candidates, prove one fan's members share one base, rebind a prepared start once per candidate; `delivery` **Deliver candidate** (VISION V3.2) — balls' one delivery law spent by handle, and `delivered_commit`, the derived acceptance mark read off the target's own `[<handle>]`-tagged history, stored nowhere; `cohort` the membership fold, derived from yog's own fire rows and never from the agent-writable `cd` mark; `retention` the one severable `cadence.yaml` policy that turns a released candidate into a discarded one, absent meaning never discard |
| `src/files_view/{mod,wire}.rs` | agent-worktree bounded walk + file preview (§11 Files); `classify` is the one "what this file is" fold — bytes + true size ⇒ `Text`/`Truncated`/`Binary` — shared by the live walk, the pinned `git show` (§5.1 #31) and the Work tab's patch (#32), so three seats never grow three vocabularies. `wire` is the §8.5 spelling of the tab — since bl-1015 carrying the conversation's `working_dir` when the work lands somewhere the listing does not reach, present exactly when there is somewhere else to name — *and* the one home of `preview_value`/`preview_of`, which the work diff's patch codec reads rather than keeping a second wording of a bounded file (bl-6233, both directions since bl-7067) |
| `src/fleet/{mod,arming,facts}.rs` | the VISION §4.3 **armed loop**, off until armed (bl-66fb): the two-gesture family and the law it holds to (*it spawns and reaps; it never diagnoses*); the `cadence.yaml` `fleet:` block — the project, the cap, the optional lease, and the two fields yog refuses to guess; and the derivation the V4 board renders — cap, count, tick, lease, last act and the §3.5 ceiling asked over this workspace's bills, every one a query |
| `src/fleet/{row,pilot,pilot/act,pilot/plan}.rs` | its acting half: the one ops-row shape a spawn and a reap each leave (§4.2) with a reap's reason stored as the *comparison* and no way to store a diagnosis, plus `last_act` — the board's "last tick", derived; and the level trigger and its thread (§7.2) — at most one move per tick, reaps before spawns, both fired through the boundary's own doors so the ceiling and the confinement gate hold by construction, with `pilot/plan` the decision itself cut off at the budget (bl-b4b5) on the seam the module doc already drew — the thread and the tick above, the pure fold over a published snapshot that says which act, if any, below — and `pilot/act` the doing of it, every act through a boundary door — including the §11 **stillbirth** (bl-ab13), the one reap a lease does not gate, whose evidence is the loop's own spawn row joined by stamp and `cwd` to a detached driver that died, while the thread's own half of that invariant is a `birth` that releases the claim its `prepare` made when the `prompt` door refuses. Its corpus is cut on the same two seams the code is: `pilot/tests` the lease table, `pilot/tests/stillbirth` the other decision table beside it (different evidence, no gate) and `pilot/tests/retaking` the third on the far side of the move (bl-3988: what a tick may take, given what this loop has already given back), and under `pilot/tests/fire` — the effect half, fake substrate and real spawns — `fire` the reap and the thread with `fire/birth` the taking of work, which is the loop's own two moves |
| `src/fork/{mod,choices,composer}.rs` | the V2 **attempt** (bl-dc0c): `mod` what one fork *is* — the `litany dispatch` argv and the skill pins; `choices` what one seat *offers* — the fork points a workspace declares, the roles the config each point **resolves** binds to a model (§5.1 #34), and the world's skill pool, all derived on demand and stored nowhere; `composer` the ×N and the readiness rule, where a cohort is a `Vec`'s length and nothing branches on it; `render` the seat at a pinned notch, every control a reading of the workspace rather than a yog list. Nothing here can touch a project worktree — the rung is read-only by construction (VISION §4.10, bl-2b8c) |
| `src/fs_watcher/mod.rs` | the watched root's watcher: the §7.1 allowlist subset exposed as a drainable stream of coalesced change notifications, pure Rust with no egui dependency |
| `src/fs_watcher/{roots,fold,hub}.rs` | per-root-kind allowlists (§7.1); the raw-event drain, coalesce and desync lead (§7.2); the process's one backend instance and its per-root fan-out (§7.1, bl-908c) |
| `src/git_env.rs` | the ambient-git-env scrub at the spawn boundary; the crate's ONE `Command` constructor (bl-916a; `rules/no-bare-command.yml`), **its one fork** — `spawn`/`output`/`status`, which under `cfg(test)` take the binary-wide spawn lock so no fork lands in a peer's ETXTBSY window (bl-6397) — **and its one `exec`**, which returns only on failure and restores the `SIGPIPE` std reset on the way to `execvp` (bl-3792); all four verbs are `rules/no-bare-fork.yml`. A RETURNING exec leaves a second global mark that no repair reaches (bl-419d): std lends the process an environment copy it then frees, under the env READ lock, so a peer thread's env read can walk freed memory and redden its own spawn with `nul byte found in provided data`. The answer is placement — a returning exec is lawful only where no peer thread exists, which is `main.rs` in production and `tests/exec_return.rs` (one `#[test]`) in the suite, and is why `exec` alone of the four is `pub` |
| `src/git_tree/{mod,model,model/agent}.rs` | module wiring + the platform `cfg` probe stack; the inert view-model types it re-exports, incl. the §5.1 #28a call starts (`Agent::call_start_unix`, `ToolCall::start_unix`) and #28b's `Agent::last_delta`. `model/agent` is the one agent-branch structure on its own file (bl-fb87's pre-split): it carries more documented fields than the rest of the module together, and the §5.1 #9 truncation reading (`Agent::truncated`, §8.2's Nudge gate) landed on it |
| `src/git_tree/addressing.rs` | the **live conversation enumeration** the §8.5 boundary addresses over (bl-49bc): every `agents/*` ref with its stored `name` blob, two facts and nothing else, so it is affordable per gesture where the §7.1 tree walk is not — and asked of disk rather than remembered, for bl-6c9e's reason one noun down (a detached driver writes the branch after the fire has already answered) |
| `src/git_tree/cmd.rs` | the git CLI wrapper and log/diff parsing — no libgit2, and every invocation built by `git_env::command` |
| `src/git_tree/cmd/browse.rs` | the §9.3 config-branch reads and the two ancestry probes beside them (`for_each_ref_config`, `ls_tree{,_long}`, `show_file`, `diff_names`, `merge_base`, `is_ancestor`), split off `cmd.rs` at the §12 pre-split band along the banner that file already carried: the parent is the **doorway** — the scrubbed `Command` and the log/step-commit parsing §7.1 walks — and this is the §9.3 surface's own vocabulary over it. Every call still runs through the parent's two runners, so the `GIT_DIR` scrub cannot be bypassed by a second fork site, and `git_tree`'s re-export list is the only way in |
| `src/git_tree/descent.rs` | hyphenated-descent ordering for the agent view (§7.1): hierarchy lives in the name, and litany's narrow grammar is the authority |
| `src/git_tree/detect.rs` | the §3.3 preview: the operator's payload headline, folded off the goal `enumerate` already reads (`agents/<agent-id>/goal.md`) — never off the assembled request, whose head is the §3.7 instruction frame and not anything anyone said (bl-368d) |
| `src/git_tree/enumerate.rs` | commit-node and agent construction: raw git output bridged into the view-model's shapes, the trunk the config lineage |
| `src/git_tree/failure.rs` | **why the latest model call failed**, in the provider's own words (bl-9b88) — the one home for that sentence, and the row-altitude first clause of it. Two shapes of one fact and the §4.4 framing decides which is read: a `Failed` tail's in-band `error` event, else — for a `Killed` one — the step's own `stderr.log`, which is where an adapter that died before reaching the contract at all (a credential-less provider row) says so. A `Complete` tail pays no syscall. *Refused at the provider rung* (bl-b43b) is a **query** over this sentence, never a flag beside it |
| `src/git_tree/{fd_probe,lsof,terminal}.rs` | the `/proc/*/fd` writer scan; the pure `lsof -F` parser + the cfg(macos) spawn shim (§10); the §4.4 settled-tail classifier — **one walk, two facts** since bl-fb87 (transport `Framing`, semantic `Ending`), because transport completion is not task completion and neither reading is recoverable from the other |
| `src/git_tree/lock_probe.rs` | the executor-lock probe behind the §3.5 `live` classification — the inbox-directory `flock`'s holder is *the* driver |
| `src/git_tree/marks.rs` | the `refs/litany/*` namespaces — four read as oids (the §6 watermark evidence) and `held` read as a **value**, its blob parsed by `control::hold` (§8.6); the closed `AgentMark` set (§6) |
| `src/git_tree/probe.rs` | the tri-state probe traits (§10) |
| `src/git_tree/probe_cache.rs` | the 2 s TTL cache over any liveness probe (§10), wrapping the macOS `lsof` backend before the classifier ever observes through it |
| `src/git_tree/probe_stack.rs` | the platform probe stack held across ticks (§10, §15 Y11), so each tick re-derives through one stack instead of rebuilding probes and discarding the cache |
| `src/git_tree/project.rs` | the **project** repo's five reads (§5.1 #32): name the integration branch, resolve a ref, `--numstat` a range, patch one file, and scan the target's history for a delivery tag. The two ancestry reads §3.9 asks of a project repo are deliberately **not** here (bl-40ab): `cmd`'s `merge_base`/`is_ancestor` already spell them for the §9.3 fold, and a second spelling of one git command is the drift this file exists to prevent. It sits inside `git_tree` because `cmd` is the crate's one `git` doorway — the site that scrubs the inherited `GIT_DIR`/`GIT_INDEX_FILE` (§16) — and a second fork site would be a second place to forget it |
| `src/git_tree/state.rs` | the agent-state classifier (§3.5, §7.1): the four live-view states derived from the executor lock and the latest step's `response.json`, nothing stored — plus, off that same settled read and never a second one, the §5.1 #9 truncation reading the Nudge gate consumes (bl-fb87) and the `failure` sentence beside it (bl-9b88, derived in `failure.rs`) |
| `src/git_tree/streaming{,/wire}.rs` | the live `response.json` fold — **one read, two facts** (§5.1 #10, #28b): the display text every tail seat reads, and the last content delta's kind that splits an open model call into waiting / thinking / inference; with `wire` the fold's own JSON spelling both directions (bl-73e7), beside the type for the reason `transcript::wire` and `rail::wire` are — the shape of a fold is the folding module's vocabulary, and the follow lane's frame body is that fold and nothing else |
| `src/git_tree/tools.rs` | the tool-call view-model over the on-disk `tools/<tool-id>/input.json` / `output.json` records, in-flight being an input with no output beside it |
| `src/inboxview/{mod,wire}.rs` | deposit parsing + the listing's per-file name/bytes (`InboxEntry`, §11 Raw) + the one `✉ from · at` header wording every §5.1 #11 seat shares (bl-929d), its sender resolved through the §3.3 ladder over the roster the caller paints from (bl-b6d0); render, both modes; the tests cut out of `mod` when Raw landed (bl-1ff1). Its headless spelling is `wire`, beside the type for the reason `workdiff::wire` gives — these rows' shape *is* this module's vocabulary — added when the §11 reads gained boundary queries (bl-6233). |
| `src/lib.rs` | module decls, Args, test_support (`src/test_support{,/world,/workspace}.rs` — the fake effects, the hermetic fixture world and its wall, and the real on-disk litany workspace, each its own file at §12's cap) |
| `src/login/{mod,auth}.rs` | §8.3 as amended (§15 M6 Z8): the streamed-piped `bz --login` flow, whose lines render verbatim and whose exit lands ONE outcome row, and the pure auth-shaped step-failure predicate that puts the affordance one click from the failure |
| `src/main.rs` (excl.) | entry, multi-call/namespace dispatch, the window face (its `Engine::boot`, and what a window adds beside the engine) and the one call that is the other one (`Engine::serve`) |
| `src/model_pick/{mod,header,pick,grammar/mod,grammar/fields,grammar/models,grammar/roles,grammar/tools,query}.rs` | the §9.4 picker: `pick` the gesture itself — one operator choice, the three gates it passes and the `providers.yaml` text it produces, cut off `mod.rs` at the §12 pre-split band so the picker's vocabulary (the role and branch it writes, where its dropdown lands and what it had to leave behind, the sentences the surface paints) is not the derivation that consumes it — the **one-file** pure plan (bl-d9cb — `plan` returns the `providers.yaml` text and nothing else, litany having retired the `models:` table the second write fed; `grammar/models` survives for yog's own table and the §5.1 #35 denominator read out of it, which since bl-3ffa is the whole of what it writes) + the provider-row gate and default (bl-bd89) + `role_fault`, the role rows' one judgement over the LIVE pointer in the pick gate's own words, the **two** scope sentences the one pane is handed (conversation vs. birth, bl-824e) and the **two** config lines — the conversation's `GoverningConfig::label()` + the derived **apart** clause with the one exit it earns (bl-9786, bl-2d19, inverted by bl-e654: `ModelRow::apart`/`set_apart`, `RETARGET_EXIT` alone, `NEW_CONVERSATION_EXIT` deleted) and the birth block's pair-and-branch-head line, the anchored block grammars (no YAML dep) — `fields` the generic locate/read/replace every rewrite and the §9.5 pane share — the ONE row judgement (`is_unknown_row`) every §9 gate calls, the protocol-capability gate beside it (`PickError::Incapable`, bl-3d22 — the effective table is carried whole because a row name cannot answer whether the dialect takes tools), and `remedy` the way out of a credential-shaped roster failure (bl-91f1, §9.4): §8.3's own `looks_auth` as the gate, `ProviderRow::login_blocked` as the words, a §11 tab as the destination — no wording and no classifier of its own, and no `Unrouted` state, because the picker named the row in the query it just fired |
| `src/monitor/{mod,arming,verdict,row}.rs` | the VISION §4.9 alignment monitor's data half: the anti-reinvention law stated where it must hold; the `cadence.yaml` `monitor:` block (arming, the model pin, the policy file it names) and the seed that file starts from; the three-valued verdict and the one reading of a model's reply; and the ops row that is audit trail, level-trigger memory and tuning dataset at once — with `latest`/`worst`, the queries that make a standing verdict a derivation rather than a field |
| `src/monitor/flag.rs` | the **flag** (VISION §4.9, bl-7aef): the signal-out verb's own ops row — its pseudo-binary, its writer and its reader — and, since bl-6f2f, the fold that makes it §6 rule 7. Split from `row.rs` on the seam the two assertions already draw: a monitor row is a verdict about a sha, a flag is *"a human should look at this"* and anyone granted the verb may write one. The fold runs at the snapshot's publish, the one place the ops trail and the derived trees are both final |
| `src/monitor/{window,check,sentry}.rs` | its acting half: the evidence one check reads (`goal.md` verbatim + the transcript delta derived from the last-checked sha by `git diff`, tail-clipped — plus every §5.1 #12 compaction marker and its summary, quoted as data in every window because the summary is what litany handed the agent in place of the span: the VISION §4.9 compaction ruling, bl-fde5); the one bounded tool-less call through the embedded brazen adapter (§16.7 W10) behind a `Caller` seam, and the NDJSON read that takes the verdict and the provider's own counters; and the level trigger and its thread (§7.2) — one check per tick, only when a tip moved, retry by re-firing |
| `{src/multiplex.rs,src/multiplex/bl.rs,src/multiplex/litany.rs,src/multiplex/help.rs,src/multiplex/landing.rs,src/multiplex/namespace.rs}` | the §16.7 namespace arms: each embedded crate's verb surface, dispatched from `main.rs` — plus the router's namespace table and its exhaustive `owns_argv` classification (`namespace`, bl-4667 — which arms own their argv and answer `--help` themselves, and which are answered from the command table like `serve`); `help` (bl-52ed): the argv seat's whole command table, the top-level roster rendered from it, every per-command page, and the discovery probe the `bl`/`bz` arms answer world-free (§8.5's every-command-answers-help rule at the argv surface); and `landing` (bl-7e54): the §16.3 repair the `bl` arm converges on the way in, re-deriving a pre-nesting landing's plugin schedule from balls' own seed; and `wire` (bl-b6fa, bl-024b): the two wire CLIENT arms — `yog seat` and `yog tool-host` — which compose the world at the process edge because the certificate and the tool config are facts of *this* machine's data root even when the engine is elsewhere |
| `src/names/mod.rs` | the §3.1 workspace-name validation, and only that. The §3.3 conversation mint left with bl-cd38 (bl-aca4's ruling, consumed at lernie 0.0.8): the wordlist, the injected-`Rng` seam and the bounded wraparound scan are `litany::mint`'s, and `words.txt` is deleted |
| `src/naming/mod.rs` | **wire names** (REMOTE §8, bl-f5f6): how a workspace or a project is addressed when a path may not cross the boundary — a workspace by its §3.1 directory leaf (which §3.2 already makes its `--as` identity, so a foreign one needs no special case), a project by the shortest trailing run of components no other enumerated project shares (§5.1 #1 gives it no name of its own). Nothing stored; the §11 roster label is that same name elided |
| `src/nav/{mod,balls,tabs,convs,convs/naming,convs/row,convs/row/model,convs/census,convs/expand,convs/select,convs/doing,convs/flight,convs/group,convs/titles,menu}.rs` | the §11 altitude-0 view-models. **`tabs` folds an answer** (REMOTE §9.7 class 2, bl-296f): the bar and the attention strip beside it are both built out of the `Query::Workspaces` reply — named, §3.1-classified, §6-rolled-up and §4.1 pin-*ranked* rows — so the seat orders and hoists and derives nothing, and no path appears anywhere in it. A stale pin key dissolves rather than being skipped, ranking no row at the boundary. **`convs/naming` is the §3.3 ladder itself** and `convs/row/model` the row type it titles — both cut at the §12 pre-split band on seams the surface already had: `convs` folds the descent forest into conversations and `naming` says what the fold produced is *called* (the ladder, its `id_floor` terminal-generation floor, and the when-seat `started_at` reads out of the same id through the one stamp grammar), while `convs/row` projects a subtree and `convs/row/model` is the shape it projects into. So: tab bar + `Kind` marks, conversation list + the §3.3 display ladder + the header's derived when-seat (bl-16da, assembled through the shared `ui_state::format_iso8601` — bl-61db) + the §3.5 ball overlay, the §5.1 #28b per-agent `Doing` and the §11 live mark's seat roster (`convs/doing`, bl-b768 — the finest live fact, which `convs/flight` then *folds* into the #28 class rather than re-reading the snapshot; both the roster and the strip below are fields on `AgentView` since bl-296f, so the window reads them off the selection's own answer), the #28 live-activity class, its priority **and the bottom strip's characteristics** — including the per-class elapsed each derives from a §5.1 #28a structural start or honestly omits (bl-9dfb), in `convs/row`'s own `age_label` (its own file because all three §11 seats read it, not just the row — bl-905f), the grouped-by-ball partition, the context-menu seat roster; and `ConvRow::verdict`, the VISION §4.9 standing verdict derived per build from the published ops tail (bl-8da1). **`convs/expand` is the unfold** (bl-fa82): the visible-row flatten over `git_tree::descent_order` given the shell's expanded set — the jsonview `flatten`'s shape, one altitude out — plus the two pure walks the §11 keyboard rides (`step` over the visible rows, `parent_of` read off their depths) and the ancestor chain a jump reveals. `convs/row`'s builder generalized with it: it projects **any** member's subtree slice, so the root-only build is one call per depth-0 subtree rather than the only shape it knew — and since bl-1eb0 it carries the row's own two §8.2 gates, neither derivable from anything else on it. **`convs/select` is the second fold over that same answer** (REMOTE §9.7, bl-48ae): `expand::visible` picks the rows a viewport has open, and this one picks the facts a seat knows about the row the operator has *selected* — the conversation it belongs to, the chain §11 unfolds to keep it visible, what it is called, what is in flight in it, the §3.3/§3.5 ball its header paints (`Selection::ball`, bl-296f — `ConvRow`'s own field, and the one member of this fold with no `AgentView` twin, a second copy being exactly the disagreement the parity test exists to catch) and the §8.2 gates a click reads. Pure over rows, so the frame-synchronous half of the old `AppModel::focused_conversation` costs no ask of its own; its parity with `boundary::answer::agent`'s projection of the same derivation is pinned in that module's tests. **`convs/titles` is the ladder's input narrowed to what a wire carries** (REMOTE §9.4, bl-1eb0): the id→title table every seat resolves a *third party* against, built either from the engine's agent set or from a conversations reply's own rows, so painting somebody's name never requires holding the tree. **`convs/census` is the third fold over that answer** (REMOTE §9.7, bl-b4b5) — `expand` picks the rows a viewport has open, `select` the facts about the row it has picked, and this one *what a conversation contains*: the §3.6 gate's per-root liveness (the §10 uncertainty counting as live, so a seat's copy fails closed exactly as the chokepoint's re-derivation does) and the §3.3 occupied name set the mint may not re-use. Depth is the containment, as it is the parentage in `select`. **`balls` is the same kind of thing at the other noun**: the §11 roster's partition and the ▶ Continue row's own object, selected out of the `Query::WorkspaceBalls` listing rather than derived — so a section, a menu and a spend row cannot be three answers of three ages |
| `src/opslog/{mod,entry,line,rows,exit,origin,live,detached,launch,operator}.rs` | `ops.jsonl` append/tail + the sentinels (§4.2), the ≤4096 capper, `entry` the shape of one line and the three synthetic constructors for the lines no process status ever backed (split at the cap on the seam between the log's policy and a record's shape), OpRow's shape + its human-timestamp leading column (bl-61db) and collapsed summary (bl-0bf9 for the cap, bl-3aa1 for where it cuts — `elide::middle`, so the workspace leaf and agent id that tell two rows apart survive the cut; its tables split to `rows/tests` at the cap on this directory's own seam), `exit` the one reading of the `exit` field — `ExitKind` and the `failed`/`drift`/`exit_label`/`detached` half of OpRow that asks it (bl-afa9, bl-8433), `origin` the §7.3 attribution — which surface an op came from, and the one thing a banner filters on so a failure renders once and on its own seat (bl-48f8) — the §6 retirement projection + activity summary + the `Detached` outcome (bl-8433), the stderr-sink fold (§8.1), `launch` **what a detached launch produced** — the state a `-2` row's failure is derived from, and the gate the ops refresh asks before it folds that sink at all (bl-b95e): the launch's target is not being driven (§3.5) and has not acted since the row's own stamp, both holding vacuously when the target is not on disk at all, with the §7.3 grace window ahead of it and no verdict at all over a workspace that derived no tree. It replaced `notice`, a marker table over sentences litany is free to reword (bl-1296), which could never reach the defect under it — the sink is append-only for the driver's whole life and every sweep re-read its tail, so one unrecognized line held its row red however many turns the driver went on to run; `operator` the two lines the operator writes: the ack watermark every alarm derivation reads past and the clear that ends a trail by logging itself as the next one's first row (bl-c417) |
| `src/projects/mod.rs` | clone enumeration, nested-delivery detection, roster labels (§11 rule 1) |
| `src/projects/balls.rs` | `Entry`→`Ball` projection + the closed-listing parse, status ladder, the §3.5 join table |
| `src/projects/join{.rs,/enumerate.rs}` | the §3.5 join-state table (§3.2, §5.1 #7) — (status, bound?) ⇒ exactly one state, never an ad-hoc branch — with `enumerate` the walk that asks it once per combination: the ball × workspace product enumerated once, bound iff the ball's claimant equals the workspace name — no operator identity, no stored fact |
| `src/projects/runner.rs` | the `bl` effect behind `projects::balls` (§5.1 #2/#4, §16.7 W8): the three ball reads of one project, typed, in-process since W8 and faked in tests |
| `src/rail/{mod,cards,cohort,pin,place,tree,wire}.rs` | the §11 step spine (VISION V1, bl-98da; re-seated into the chat by bl-1802), cut on five seams, all derivation and no paint — the seat is `src/transcript/spine.rs`: `mod` the spine's *shape* — the notch spine over the Steps view's `meta.commit` (§5.1 #29) and the row→notch lookup the chat renders through; `place` where each notch sits in the chat and how far its pin cuts, pairing one sealed model-output entry to each completed step (the ordinal alignment bl-929d and bl-98da both got wrong); `cards` a child's *placement* on it — the shared-commit-prefix fork point, the two edges, the fork label and the streaming tail, all pure over facts the snapshot already carries; `pin` the fold that threads one notch through the inspector (the transcript prefix, the budget as-of); `tree` the only new disk read — the Files tab out of `ls-tree`/`git show` at the pinned commit; `cohort` V2's fan, which is nothing but those cards grouped by the notch they were born at (§5.1 #33); and `wire` the spine's §8.5 spelling both ways, beside the type for the reason `workdiff::wire` gives — notches, seats and cards are this module's own vocabulary (bl-6233, decode bl-7067) |
| `src/registry{,/enroll,/leaf,/mailbox,/mailbox/slots,/peer,/presence,/roster,/tools}.rs` | the REMOTE §4 client registry (bl-8bbc): one **file per registration** under `<yog-state-root>/clients/<client>/workspaces/<name>`, so registering is a write, revocation is a delete and the registered set is a listing — nothing stored that the path already says. Plus the reserved `local` identity every in-world caller (window, deposit inbox) owns a §7 pane document under, and `leaf` — the subject common name read off a peer's certificate by a structural DER walk, because yog links no certificate library and a byte search for the CN object identifier would return the issuer's name, which comes first. REMOTE §5's two facts about a tool host hang off the same directory (bl-4e08) and they are deliberately different KINDS of thing: `tools` is the durable half — `clients/<client>/tools.json`, one document per client rather than one per registration, because a tool set is a fact about a machine and the registration listing already says which workspaces see it — carrying the ONE spelling of an element (name, description, JSON Schema verbatim) that the boundary codec and the file both spend, plus the write-only-when-it-differs store and the two ways a presentation declines; `presence` is the live half — a refcount per identity behind the `state.rs` alias, entered by the listener as an RAII guard so there is no leave verb to forget, and never a file, since presence changes at connection rate; and `roster` joins the three reads at the moment they are asked (listing, presence, advertised set) so the window and a headless seat render identical rows, with the reserved `local` name filtered by the rule that already refuses it rather than by a second case. `mailbox` is REMOTE §5's third fact about a tool host (bl-024b): the vocabulary of a routed invocation — the call, the invocation, the capture, the completion, and the `Verb` that folds the two acts — plus its ONE JSON spelling, spent by the gesture codec, by both replies that carry a capture and by the client-side executor alike; `mailbox/slots` is where in-flight ones live, a queue per client and a slot per invocation behind this leg's one lock (the fourth `rules/locks-outside-state.yml` carve-out, for `presence`'s measured reason), swept an hour after a post so a driver that died mid-invocation costs one entry rather than a leak. It also holds **which identities are parked on a follow-class read** (bl-1462): one reader per client, an RAII claim rather than presence's refcount, because two connections on one machine's queue is the pathology itself where two on its presence is an operator with two seats — the second read is refused in band, and so is an advertisement that would change a serving machine's set out from under it (REMOTE §5.1). The hand-off mark there is a **lease and not a latch** (bl-e658, REMOTE §5.3): a read parks for the whole hold and cannot learn its peer went away, so a slot handed to a read that never answered it is re-queued at that client's next follow-class read — the leg is at-least-once, and the invocation id it is redelivered under is the idempotency key `peer` is REMOTE §4.2's grade (bl-7ff3): the two values a leaf's subject can carry, the closed set of gestures a foot may say (advertise, take its invocations, complete one — never `invoke`, the asking side's), and the `Peer` an intake answers as. It is a value BESIDE the identity rather than a field on it, because `Client` keys the presence map, the mailbox and every registration on disk: a peer that connected under one grade must not be a different key from the same client read off the `clients/` listing. Default-operator is total rather than defaulted — `leaf::grade` answers a `Grade` and not an `Option`, so a certificate minted before the grade existed, or bytes that are no certificate at all, read as operator. `enroll` is REMOTE §1.4's two values (bl-f4e3) — what an operator asks for when a device joins (a workspace to seat it in, the common name its certificate will carry, the grade that certificate may say) and the whole of what the engine answers with (that grade and name, the address clients dial, and the three PEMs). They live here rather than at the boundary because the identity the act mints and the registration it seats are the registry's own two facts, and because a payload type with its own module is the fold `mailbox::Verb` and the monitor's family already take: ONE variant at the boundary, one home for the ruling. The private key exists in the `Enrolled` value and nowhere else on this box — the executor shreds it before the answer leaves, and the certificate stays precisely because its presence is what refuses a second enrollment under one name |
| `src/science/{mod,bound,observed,outcome,wire}.rs` | the §3.9 **attempt science projection** (VISION §4.10 item 7, bl-40ab), cut on four seams and owning no fact of its own: `mod` the join — the row type and the one function that assembles it, over `workdiff::read`'s own row set so *which attempts are there* has one answer, and each row *carrying* the `workdiff::Attempt` it composes rather than restating its identity, refs, OIDs, churn and acceptance mark; `bound` *which* conversation an attempt is bound to — the one binding rule at every N (the last fire whose `--cwd` names this attempt's worktree, by balls' own `attempt_path`/`work_worktree_path` formulas) and the §3.3 name→id resolution; `observed` *what about it* — the fork's own inputs (`goal.md`) beside §5.1 #17's **followed** config commit, which since bl-e654 is a fact about now rather than a frozen one (§3.9), the terminal response and the delivered messages with the compacted bound beside them (how many entries the counter proves deleted, bl-fde5 — the §3.9 statement that both were read over a rewritten record), and the step-record columns as an in-memory filter of `Snapshot::bills`, so the projection makes no second pass over `steps/`; `outcome` the four arms over three git facts — the derived acceptance mark, whether the source ref still resolves, and `merge-base --is-ancestor`, which is the delivery law's own staleness precondition read from outside and this ball's reframe of *"the source advanced after a refusal"* — plus, beside them because it is the same question in the same shape, the **base** commit the two ends departed from (item 7's third OID, balls' `merge-base(target, source)` formula spelled here because asking balls means resuming an attempt, and resuming writes); both ancestry calls are `git_tree::cmd`'s existing ones, never a second spelling; `wire` its §8.5 JSON shape both ways, beside the type for `workdiff::wire`'s reason, with the diff column spelled by that module's own row codec |
| `src/science/{compose,respdiff}.rs` | the §11 **fan group card** — the V3 seat over the projection (VISION V3, bl-77bc), cut on its own seams: `render` the group frame — membership (the rows wearing a handle; none paints nothing, the burden check made structural), the header stating the cohort once with the shared base said only when every member wears it, the compare picks and the V3.3 response-diff paint; `render/column` one candidate's column — the derived mark in words, the steps/wall/tokens figures, the one-line churn, the clipped terminal response (clipped in code, because a galley reports the string that went IN and an egui elision is invisible to every assertion), and the Deliver/Retire affordances; `compose` the four affordances **as composed dispatches** — Judge and Synthesize are V2's fire path with a goal carrying each candidate's exact refs, Deliver is `/deliver <handle> ` awaiting the operator's summary, Retire is `/retire <handle>`, and the operator's Enter is always the fire; `respdiff` V3.3's response comparison — a capped line LCS over the two `response` columns, its own forty lines rather than a dependency because the two responses live in two different conversation repos and no one tree can be asked for the diff |
| `src/scratch.rs` | I3's scratch temp (§2 I3, §5.2, bl-e47c): the one spelling of `.<name>.yog-tmp-<pid>` every write site asks for, the exact predicate that recognizes one back, and the startup sweep — the pure 24 h decision, the best-effort per-directory removal, and `dirs`, the fold naming every destination yog writes a temp into. Name and sweep are one home because a sweep spelled differently from the writer deletes nothing, or something else |
| `src/search/{mod,corpus,excerpt}.rs` | the §8.5 global search: the `Address`/`Field`/`Hit` vocabulary, the answer that carries its own needle (`Found::asked`, the strip's offer predicate, beside `Found::is_empty`, the pane's — bl-648a) and the empty answer's wording, the deterministic rank and bound, and `run` — the one engine all three seats end in; the corpus (the snapshot half free, the conversation half re-read from disk and the half cancellation is checked between); and the matched-line window at char boundaries. Every `Address` field is a wire name — the §3.1 workspace leaf, the §5.1 #1 project name — never an engine path, so a hit is an address a seat asks next (REMOTE §8.1, bl-764a). The seat-side searcher thread left with the window (bl-7942): fan-out over many channels and the union's non-promises are the seat crate's, ruled in REMOTE §8.2 |
| `src/spend/{mod,prices,ceiling}.rs` | the §3.5 join, pure over the worker's pre-walked bills (bl-9dd4) — selection, attribution, the honest-granularity label, and the unpriced remainder, with `of_workspace` the one deliberate fresh walk because a gate compares against now; the price table's parse and its micro-USD arithmetic over the §3.5 three-way partition of the prompt (bl-6621 — the cached slice is priced once, and the tokens priced sum to §5.1 #16's fold exactly); the §3.5 spend ceiling's policy half — the operator's number and the at-or-over comparison against the workspace figure (bl-56d5); the one figure widget every spend seat paints — the board's ball rows and the conversation's settings rows (bl-2e18) — whose attribution clause is independent of the price table, so the honest-granularity label survives deleting the cost column (bl-1765) |
| `src/start/{mod,model,goal,identity,exec,exec/claim,ensure,prompt,run}.rs` | the start flow (§3.4/§8.1): pure plan, `model` the inert shapes it reads and returns (`Payload`/`BallSpec`, `StartInputs`, `Step`) cut off `mod.rs` at the §12 pre-split band — what a start *is* beside the one function that derives it, goal compose, the §3.3 stamp and its inverses, the `bl`-facing gated executors, `exec/claim` the one of them whose *answer* is still judged (bl printed a worktree path; the bl-delivery formula says whether it is the canonical leaf, the `<id>-<claimant>` variant, or a `Drift` logged before it returns), `ensure` the workspace's existence and its policy — `litany new` plus the one convergence that authors §8.6's control block and §3.7's instruction glob onto **the lineage the drone will fork off** (§8.7, `config/default` unless the ball's tags named another) in a single `litany config` drive, outside the create skip; the §9.2 birth-template gate that once sat here is retired (bl-00ee) — the detached fire, whose one argv carries `--name`, `--config`, `--cwd` and every `--pin` |
| `src/start/gate.rs` | the §8.1 provider gate (bl-1fd0): a pure read of brazen's `credential` column answering whether the wall a start aims at can reach a model at all. `WallCredit` folds the table to two booleans and `StartGate` is total over them plus the §8.2 channel — Ready (today's flow, byte for byte), SignIn (the rung, refusing Send only where the wall has neither a credential nor a keyless row left to try) and Unknown (a workspace an entry hosts, whose wall this box reads nothing of — bl-61bf's seam). A keyless row is deliberately not Ready: brazen ships `ollama` and `claude-code` at `not required` on every bare wall, so counting them would make the rung vacuous on exactly the wall it was ruled for |
| `src/start/instructions.rs` + `src/start/instructions/{names,manifest}.rs` | the §3.7 project-instruction freeze (bl-aa8b): the walk from the binding's authority root down to the binding and the ranked `--pin` specs it yields — yog reads no instruction bytes, litany's caller-supplied pinned documents do the loading, validation, snapshot and commit; `names` the severable filename policy (`AGENTS.md` in code, `instructions.yaml` at the live config tip overriding it, an existing file authoritative even when it names nothing); `manifest` the `instructions/**` glob's fixed point, without which a frozen document is a committed file no model ever sees |
| `src/start/lineage.rs` | the §8.7 birth policy (bl-380f): which `config/<name>` a ball's tags select, and the whole of how VISION §4.2's *"skills and model from the ball's tags"* is delivered — a lineage already being the (model, skills) pair, its **existence** is the policy and no table exists. First tag naming a lineage wins (the operator's own tag order is the precedence), no match is `None` and an omitted flag, and severability is `git branch -d config/<tag>`. Resolved once in `prepare` because two acts consume it and must not disagree — §8.6's convergence and the fire's `--config` |
| `src/state.rs` | the crate's lock chokepoint: the dirty hand-off, the snapshot cell, the §8.5 search cell and the §7.2 live-tail cell — the whole inter-thread interface (§7.2, §8.5, AGENTS rule 7). The tail cell is **appended whole below every line that was there before**, and takes the snapshot cell's *alias + free functions* spelling rather than a struct with an `impl` — including leaving the module doc's stale "three residents" line untouched. That is the hazard `rules/locks-outside-state.yml` records as the reason for both its carve-outs: llvm-cov mis-attributes phantom uncovered regions onto this file's `impl` headers when anything above them moves, and an added `impl` block draws one onto itself besides. This is genuine cross-thread hand-off state — what the chokepoint exists to inventory — so it belongs here and the spelling gives way instead of the rule. The watch hub's two singletons are its second declared carve-out (§7.1, `rules/locks-outside-state.yml`) |
| `src/steps_view/{mod,detail,wound,wire,wire/decode}.rs` | the step inspector, incl. the §7.3 wound in both its classes — no response, and the §4.4 output limit (§11 Steps). Both tiers are cut twice, read from write: `mod`+`detail` are the list/drill-in **reads**, `render`+`drill` their **paints**, and `columns` is the §11 column table — header, hover explanation and cell in one home, so no field paints without its name (bl-3ffc). `wire` is the §8.5 spelling of **both** tiers, cut along that same read seam (bl-6233), with `wire/decode` its other direction at the §12 budget (bl-7067) — also the one home of the `BudgetSpend` shape, which the §3.5 board figure spends rather than keeping a second wording of four counters |
| `src/steps_view/orphan.rs` | the **orphaned-tail state** (bl-ace6, widened by bl-abba): the transcript's newest entry is owed an answer and nobody holds the driver lock — the class the §7.3 wound cannot see because the driver that died (an unpaired-tail decline, a lease fault, a crashed launch, an executor killed mid-tool-window) never created a step to hang it on. **Two `Tail` shapes, one state**: delivered mail nobody answers, and a model entry whose `tool_use` no `tool_result` answers. Only the newest entry can be in the second shape — a call answered later has its result committed after it — so the predicate is one readdir plus one file read through `transcript::classify`, never a pairing walk and never the whole record. The tool-window shape is suppressed by a live `refs/litany/held/<id>` park (§8.6): a parked call wears the same shape and is waiting on purpose. Derived per reading from the messages listing plus the already-derived §3.5 liveness, nothing stored; the reason is the tail of `steps/<agent>/driver.log`, litany's binding of every launched driver's stderr (the file yog pinned lernie 0.0.9 for and, until this, never read), read only when the state holds |
| `src/steps_view/records.rs` | **what records a step surface has** (bl-83d6): the drill-in picker's row set, the words each seat carries, and the two capture-log file names the §7.3 banners quote. The row set is **derived, not declared** — the five JSON records litany contracts to write always, plus each of `stderr.log` / `driver.log` the step actually has bytes in, read once by `detail` and never stat'd a second time here. A log renders as a `files_view::Preview` rather than a `Doc`, because nothing parsed it |
| `src/test_support{.rs,/wire.rs,/workspace.rs,/world.rs}` | the test-only scaffolding every test module in this binary shares: the spawn/env serialization locks (AGENTS.md rule 7's sanctioned carve-out), the REMOTE §9.5 wire's key material minted at test runtime by the same `openssl` act an operator performs (a certificate fixture is never committed — bl-b6fa), a real litany workspace on disk for §8.6 control authoring, and the §16.2 fixture world every test that touches a §9 destination or a §16.3 space reads and writes through |
| `src/test_support/chrome.rs` | **the §11 accessories that crossed with bl-296f, asked the way a seat asks them** (REMOTE §9.7): the altitude-0 chrome is a fold over `Query::Workspaces` and `Query::Ops`, and the live mark and in-flight strip are fields on `Query::Agent`'s answer, so `AppModel` holds none of them. `convs.rs`' door for the surfaces beside the list |
| `src/test_support/clock.rs` | the suite's deterministic `Clock` — a shared instant a test advances by hand, plus the *lurching* variant whose every read costs time (the one way to exercise §7.2's late-pass drift without a slow machine). Split from `test_support.rs` at §12's cap on the seam that file already had: the spawn discipline is about forking, this is a value the crate reads |
| `src/test_support/engine.rs` | **one act, run where the ENGINE runs it** (§8.5): the model's own `Deps`, a `ui.json` opened per gesture, and `boundary::dispatch` — the arm the wire's listener and the inbox poll both reach. A test that reached past it would assert against a pipeline no seat can drive |
| `src/test_support/seat.rs` | **the suite's own seat**: the client half of the wire, kept as scaffolding when the seat crate took the shipping one (bl-7942). A listener nothing dials is a listener nothing proves, so the crate keeps one client — deliberately the same code the seat was built from (a re-implementation would prove the re-implementation) — under `test_support` rather than in `wire`, because it is not a face yog ships and must not read as one. Carries `loopback`, the address a local client dials: the port the listener really bound, whatever the `address` file says |
| `src/tool_host.rs` | yog's litany **tool injection** (REMOTE §5, bl-c907) — the object the §16.7 W11 arm hands `Fx::tool_injection`, so an agent sees this workspace's client machines and can drive what they advertise. litany's seam is ONE object carrying both halves (its `docs/DESIGN_TOOL_INJECTION.md`): the definitions prompt assembly and the grant gate read, and the router the executor answers through — so a tool declared and not permitted, or permitted and not declared, is unrepresentable. `tools()` is the `clients` tool always plus the agent's loaded set, read off disk and nothing else, because a prefix that varied with an engine's reachability would put a connectivity-rate fact inside the model's cached context. `route()` is **total** since the seam inverted (bl-fe61, litany 0.0.2): nothing resolves a binary behind it, so it answers the `clients` tool, the six engine acts (`tool_host/engine_act`, bl-dfce widened by bl-77be) and every loaded remote name — and hands everything else to the worktree lane (`tool_host/subject`, REMOTE §5.4), which routes a granted bare name to the workspace's one consenting machine with the conversation's cwd on the invocation, or *renders* the refusal naming the way out — non-zero with the reason on stderr, the shape an absent binary produced. Every answer is in the stdio vocabulary the executor already speaks (product on stdout at 0, reason on stderr at 1), so the model cannot tell a routed tool from a local one. Adjudication (`tool-control`, §8.6) is untouched and still runs first. Since bl-024b a loaded remote name is not just declared and adjudicated but **run where it lives** (`tool_host/remote`): the far machine's own stdout, stderr and exit code pass through verbatim, and only a transport failure is a sentence of yog's own — in band and non-zero, which is the shape a vanished endpoint had to produce anyway |
| `src/tool_host/ask.rs` | the driver's ask (REMOTE §3, bl-c907): the injection runs in a *child* process and presence is engine RAM by ruling, so the roster is fetched through the door REMOTE §3 already reserves for the world's own residents — `Query::Clients` deposited into the gestures inbox, its reply read back and decoded with the one reply codec. No verb and no transport added; the child folds the same `<world>/state/yog` the engine writes because the world hands `XDG_STATE_HOME` down. Every wait is bounded by an injected `Budget` and ends early on the stop flag, which is the router obligation litany states and cannot enforce |
| `src/tool_host/clients.rs` | the `clients` tool (REMOTE §5, §5.2; bl-c907, bl-3455): ONE tool in the stable prefix whose subject is the roster — `list` (who is registered, who is connected right now), `get` (one client's detail and its advertised tools), `load` and `unload` (the two acts that change the declared surface, in `clients/edit`). Loaded tools still surface individually named, so this is a roster surface and never a multiplexer. **Only three of the four need the engine**: `list`, `get` and `load` resolve against the roster and each asks for it in its own arm, while `unload` resolves against this agent's own document and deposits nothing — so a finished host can be dropped on a box whose engine is down, the same reason §5.2 gives for declaring touching nothing but disk. An unregistered identity is **absent** rather than forbidden, the same shape a name nobody seated earns; `tools` is required on a load and optional on an unload, where absent means that client's whole loaded set and `[]` still declines, because an act with no effect should be said rather than answered as success |
| `src/tool_host/clients/edit.rs` | the two `clients` ops that WRITE the agent's set (REMOTE §5.2; bl-c907, bl-3455) — split from `clients.rs` at the per-file budget on the seam its doc already draws: that file is what the model may say and where it is routed, this one is what changes the declared prefix. **Whole or not at all in both directions**: a load resolves every name against what the client advertises *now* and one miss refuses the act (a partial load leaves the model believing it holds a tool it does not), an unload resolves every name against what the document *holds* and one miss refuses the act (a partial unload leaves the model believing it dropped a tool it still declares). The authorities differ and that is the whole asymmetry — the roster needs the engine, a file on this box does not. The client is half of every name, so an unload names one machine's copy of a shared `Bash`; a client this conversation loaded nothing from refuses rather than answering an empty success. Neither defers: the subtraction lands now and is spent at the next assembly, and scheduling such an edit against an already-inevitable cache miss is bl-b6f9's mechanism, not this one |
| `src/tool_host/engine_act.rs` | **the compactor's procedure pair, performed as engine acts** (REMOTE §5.4, bl-dfce): `write_summary` and `mark_for_deletion`. litany injects tool definitions from two sources — the host's injection and the calling role's own procedure — and since the router went total the second source arrives here too. They are not tools on a machine: their subject is the conversation, which the server holds, so the subject-locality invariant sends them nowhere and yog answers them itself. **litany defines what they do and yog restates none of it** — the acts are performed by re-entering the engine's own front door (`<driver_target> tool <name>`, the third hop litany's resolution addressed before the inversion) with the caller identity on the child's environment and the `tool_use` input on its stdin, so the compactor's semantics have one definition and it is upstream's. **Six rows since bl-77be**: the pair plus the conversation-subject worker grants (`dispatch`, `message`, `load_skill`, `cd`), each admitted by the same subject-locality audit and performed by the same re-entry — at the caller's resolved cwd since litany bl-ddaa, the engine's own contract for in-process built-ins. The name set is closed and enumerated here and nowhere else — the worktree lane's last rung (bl-5710) calls this module's `perform` without joining the set, because what separates the two is the ordering, not the mechanism; both waits are bounded (the stop flag, and the patience the tool bound already is), and a failure is the in-band non-zero refusal every other answer on this seam is |
| `src/tool_host/loaded.rs` | the durable loaded set (REMOTE §5, bl-c907) — `<yog-state-root>/loaded/<workspace>/<agent>.json`, durable because a driver is not: each step is a fresh process, so a set held in RAM would last one turn. The definition is **frozen at the load act** rather than re-read at assembly (REMOTE §5's "definitions frozen in the prefix, presence answered at invocation"), and the presented name is the client's identity, an underscore, the advertised name — always, never only when ambiguous, so a name a model learned cannot change under it because another machine advertised something. Union by presented name, and no inheritance. **Two writers and they are symmetric** (bl-3455): `add` is the load act's, `remove` the unload act's, and neither resolves a name — each takes entries the act already resolved, so the whole-or-not-at-all rule lives once, in the act, and this module only seals a set it was handed. The last unload leaves an empty array rather than a special case, because a document that is absent, unreadable or empty already reads as the same nothing a fresh agent reads |
| `src/tool_host/remote.rs` | the driver's end of the routing leg (REMOTE §5, §9 step 7; bl-024b): what a loaded remote name does when the model calls it — an `invoke` gesture that queues the call at the engine and answers a handle, then a `capture` poll until the far machine answers, both through the same deposit door the roster read uses (no verb and no transport added). TWO gestures rather than one because the engine's intake is one thread for the world: a gesture that waited for a tool would stop every other deposit converging. The far machine's own stdout, stderr and exit code pass through verbatim, so the model cannot tell a routed tool from a local one; only a transport failure is a sentence of yog's own, in band and non-zero. **The deadline is the visible refusal** — nothing checks whether the machine is connected, because a tool host holds a connection only while it is waiting and is therefore absent for the whole time it is busy |
| `src/tool_host/render.rs` | the dated observation (REMOTE §5, bl-c907): every reply opens with the instant it was read at, because presence is true only then and a line that did not say when would be a claim about now. Text rather than JSON — the reader is a model and the result envelope carries bytes — and each rendering is a list of lines joined once |
| `src/tool_host/subject.rs` | **the worktree lane** (REMOTE §5.4, bl-77be): what a granted, unqualified name does when the model calls it — a workspace-subject attempt, routed to the ONE registered client that both advertises the name and consents to workspace-cwd execution (`subject_cwd`, REMOTE §5.1/§5.2), with the conversation's resolved working directory on the invocation (`RoutedCall::cwd`, litany bl-ddaa). Zero consenting advertisers refuses in band naming the operator's config edit; more than one is a config ambiguity refused naming every claimant, because one adjudication stands for one execution on one machine. **Since bl-5710 the lane is a ladder with a last rung**: where no machine consents, the worktree names the engine itself implements are performed at the engine's own front door, by the same `engine_act::perform` re-entry, so a default install with nothing enrolled can read and write in its own worktree. **That set is a derivation, not a list** (bl-e654): `subject::performs` is `litany::cmd::BUILTIN_TOOLS` — exported by the engine since 0.0.5, upstream bl-4cbb — **minus** `engine_act::NAMES`, so yog states the *partition* (every builtin that is not an engine act is a worktree name) and restates none of the names. What the partition leaves today is `apply_patch`, `bash` and `read_file`; those three literals live in the test that audits the partition, which is where a fact yog does not own belongs — an upstream builtin added or renamed changes the lane by changing the engine's own constant, and reddens one test rather than passing silently against a stale copy (operator ruling 2026-08-31: *ship some basic tools — a default install must be able to write a file*). The ordering is the whole of the distinction from an engine act: an engine act never consults the roster, a worktree name always does, so an enrolled and consenting machine still wins. Every other name keeps its refusal. The selection is a pure function over the roster, so every rung and every refusal arm is provable without an engine. **The sentences live one level down** (`subject/refusal.rs`, bl-68e1) |
| `src/tool_host/subject/refusal.rs` | **what a landing on a refusing rung says** (bl-68e1), split from the lane because it answers a different question than `verdict` does: which rung, versus what the model reads. Two zero-consent shapes (nothing advertises the name; machines advertise it and none consents) and the config ambiguity, each naming the operator's edit by key, file and box. Both zero-consent shapes close on `NOT_A_REMEDY`, the one home of the sentence that the loaded lane is **not** a way to do this work — a loaded invocation carries no directory (REMOTE §5's locality-rides-in-the-name), so it runs in the far process's inherited directory, and nothing this conversation can read would show what it wrote there. The old sentences offered that load first, as the remedy the model could take unaided, and a drive took it: the whole deliverable landed in the foot's inherited directory, every self-check was made in the same wrong place so none could fail, and the conversation reported success over an empty bound directory |
| `src/transcript/{mod,read,compaction,parse,wire,wire/decode}.rs` | the **committed** transcript's enumeration — `read` the directory walk itself, split off at the budget when the follow lane made the two clocks load-bearing (bl-73e7): what a transcript *is* and what it projects is `mod`, and the disk read that produces one is its own file. The live tail is still the caller's fold (§7.2 bl-54f7), and `with_live` **replaces** a live entry rather than appending beside one, so the pull answer's tail and the follow lane's newer one reconcile by newest-wins and cannot paint the answer twice (over the seat's own accumulation since bl-3655, a follow frame being an append rather than the whole answer — §7.2), forgiving parsers, the §11 row vocabulary (classes, tones, roles — bl-3acb — and the auto-state rule incl. the in-flight input), `compaction` — the one reader of what the compactor **deleted** and of the `summary/` prose that replaced it (bl-7bd2), a query over the `NNN` counter splicing a virtual marker into every hole, cut out of `mod` because it is the only part of the enumeration that reads a directory `messages/` is not and the only one whose ruling is about what the bytes CANNOT say (no summary-to-span link exists, so none is guessed) — and — cut at that seam when the §3.3 sender label landed (bl-2335) — the entry→rows projection with the labels and roles, over `project/build`'s row constructor and preview/body split (bl-54f7's own cut: *what an entry becomes* vs *what a row is made of*), with `project/compacted` the one arm of that match which projects a **hole** rather than something somebody said or a tool answered — split off at this section's budget on that same seam (bl-3b22), and the row altitude of `compaction`'s own enumeration, so the two halves of one subject sit one per file rather than one per subsystem, then the §11 turn rollup over it (bl-1f21: the turn boundary, the aggregate line, what a shut turn omits), with `turns/counts` split off at the cap on the seam between deciding *where a turn is* and saying *what it contained*, the folding render with the role stripe at every row's edge and the bl-7654 payload rule stated at the row rather than inherited (the expanded body and the Raw view wrap; the chrome and an abridged preview truncate; only an abridged preview fades, at `theme::tone_solidity`), cut at this section's budget into the scrolling list and where the spine's rules fall in it, over `render/row` — the chrome line, the toggle, the inline preview and the expanded body, which is the very seam the projection above is cut along (bl-3b22), and `spine` — the step spine drawn *through* the chat (bl-1802): the clickable operable-commit rule that pins, and the cards and cohorts born at it, which is the whole of what the retired `history-rail` side panel was; and `wire` the chat's §8.5 spelling, with `wire/decode` its other direction at the §12 budget (bl-7067) — every entry class stays distinguishable on the wire as it does on screen, each row carrying the bytes it was read from because a headless seat has no Raw toggle to reach them with (bl-6233) |
| `src/transcript/key.rs` | a transcript row's stable identity — `tx/<entry filename>#<block index>` — the one spelling of the address a seat and this engine both name a row by. The row projection that minted it went with the window (bl-7942); the §7.3 step spine's placements still tell a seat which row each rule is drawn above, and say so in this vocabulary |
| `src/ui_state/{mod,json,doc,knobs,fields,clock}.rs` | the UI-state schema — **two** documents since REMOTE §7's per-seat split (bl-8bbc): the shared `ui.json` of world facts (`seen`, `pinned`, `identity_last_used`) and the per-client pane document of glass facts (`panels`, `collapsed`, the knobs), read through one handle so which file owns a key is stated once, at that key's accessor. `doc` is the file mechanics spent twice — forgiving load, echo-hash, atomic write-through. Also the seen API, the knobs + `zoom` + the §6 escalation's `notify_unfocused` (§4.1); the per-field accessors derived from it; and `clock`, split off at the budget — the crate's **one** injected time seam (§7.2's `Clock`/`SystemClock`) beside the one calendar routine both directions of its rendering ride on (`format_iso8601`/`iso8601_extended`/`epoch_from_iso8601`, Hinnant's civil-day math, no `chrono`/`time` dep — bl-61db), together because keeping them apart is what would spread that freedom over two files; none of it reads a document |
| `src/ui_state/panels.rs` | the §4.1 `panels` object: the `Panel` enum (key, default, floor, ceiling + the one clamp — one home per boundary) and its forgiving read / snapped write |
| `src/ui_state/{prices,ceiling}.rs` | the §4.1 `prices` object: the §3.5 price table's one read, forgiving and setter-free; the §4.1 `ceiling` number beside it, read the same way — absent is no gate (bl-56d5) |
| `src/ui_state/prune.rs` | the §3.6 prunes: a deleted workspace's keys; a deleted conversation subtree's `seen` watermarks (bl-f17a) |
| `src/watch/mod.rs` | WatchSet reconcile and the ingest bridge thread (§7). The wake-the-face effect it used to carry beside them went with the face (bl-7942): there is nothing in this process to wake, and a seat asks on `wire::ASK_PERIOD` |
| `src/wire.rs` | the REMOTE §9.5 client/server wire (bl-b6fa) — the module root's one function is bringing a booting engine's listener up, and since bl-ae05 it **founds its own material first**: absence stopped being the off switch the day REMOTE §1.2's window became a client of this listener, so an unprovisioned box mints a loopback trust root rather than painting nothing. Half-provisioned material the mint cannot heal still refuses, because silently degrading to no encryption is the one failure the split excludes — and since bl-dc14 every refusal here is a *returned sentence*, said on stderr by the boot: a bind another process beat it to, or a mint this box cannot perform, leaves the engine running without a wire — every deposit still converges, and only a seat is shut out |
| `src/wire/frame.rs` | the framing bl-b6fa decided and REMOTE §3/§10 record: a big-endian `u32` length then that many bytes of JSON, a zero-length frame ending a reply stream, and one bound on what a peer may make a reader allocate. Every answer is a stream, so a follow-class read is the general path with more frames rather than a second form |
| `src/wire/hello.rs` | the wire's **version preface** (REMOTE §3, §12; bl-a670): each end writes one frame stating its protocol version before it reads the peer's, so neither waits on the other and a skew is nameable from whichever side notices. A mismatch is fail-closed — the engine refuses in band on the connection the peer opened, the seat refuses to its caller as the one `Err(String)` every transport failure already arrives as, and both say the same sentence naming BOTH versions, because with four separately installed components (REMOTE §12) the refusal IS the upgrade prompt. No negotiation, no capability probe, no compat shim, and no change to the request frame: the preface rides beside the gesture envelope, so the frame the wire carries is still byte for byte the frame the `gestures/` inbox carries. ALPN would have cost no frame and refused inside rustls, but a TLS alert cannot name a version — which is the requirement |
| `src/wire/intake.rs` | the wire's half of the ONE intake (REMOTE §3): a request frame handed straight to the deposit consumer's own context, so the listener reaches the same codec and the same `dispatch`/`answer` the inbox does — which is why the wire can add no verb |
| `src/wire/material.rs` | where the operator's out-of-channel key material lives and what its three states mean (REMOTE §1.4): absent is off, partial is a refusal naming every gap and the remedy, whole is the anchors, this role's leaf and the one address. It sits BESIDE the world subtree, never inside it — a reseed must not be a revocation |
| `src/wire/provision.rs` | **the mint** (REMOTE §1.4, §8; bl-ae05) — the crate's ONE `openssl` recipe, shelled through `git_env::command` and spent by the engine's boot and by `yog wire-certs` alike (`scripts/wire-certs.sh` is retired: an installed binary has no repository to find a script in). Still out-of-channel by ruling — the trigger moved, the act did not, and yog links no certificate library. Since bl-64a7 the same recipe also issues ONE extra client leaf under a stated common name (`issue`, REMOTE §8.2) — the host half of provisioning an entry on a visiting box, and an act over the CA already here rather than a second recipe; it refuses an identity the registry would refuse (§4.1's own rule, spent once), a directory with no `ca.key`, and a pair already under that name. Since bl-7ff3 that stated leaf carries a **grade** (REMOTE §4.2): `WIRE_FOOT` puts `OU=foot` in the subject, presence-shaped like `FORCE` so there is no word to mistype into a demotion, and unstated is operator — the mint is the only thing entitled to write a grade, because the operator's own CA is. Idempotent by construction: it mints only what is missing and only what it CAN mint, so a box holding an operator's anchor with no CA key beside it is left alone rather than having its trust root replaced. Self-provisioning writes `127.0.0.1:0` — loopback, kernel-chosen port (bl-dc14: a process-global default port made two instances contend, against I0) — which is the whole of what distinguishes loopback-only from wider listening: the address is one fact with one home, and only an operator ever writes a host that is not loopback (or, via `yog wire-certs`, its stated default port) into it. Loopback always rides the server leaf's SAN besides, because the window is a client of `127.0.0.1` unconditionally |
| `src/wire/provision/openssl.rs` | the `openssl` half of that mint (bl-ae05): the two invocations (a self-signed CA, then a CSR and its signature per leaf), and the two X.509 facts yog decides — the subject alternative name and the extended key usage. Split from `provision.rs` at the seam the §12 pre-split band names: which artifacts a box NEEDS is a question about the box, what one `openssl` run SAYS is a question about X.509. The server leaf's SAN says which kind of name a seat verifies against — an IP literal is an IP identity, anything else a DNS one — and always carries loopback beside it, because the window is a client of `127.0.0.1` unconditionally. The tool is named once and `run` takes it as a parameter, so both of its failure paths are testable without uninstalling anything. Since bl-64a7 the issuance body is `issue(dir, name, cn, san, eku)` with two callers: `leaf`, which derives all four from a `Role`, and `stated_leaf`, which takes the common name outright and files the pair under it — `Role::Client`'s facts with the operator's name where the role's would be |
| `src/wire/provision/verb.rs` | `yog wire-certs`, the operator's explicit act over that mint (bl-ae05): a server another machine dials by name, and a rotation. `WIRE_DIR`/`WIRE_HOST`/`WIRE_PORT`/`FORCE` are the interface the retired script had, read at the process edge (`main.rs`) and folded here so the whole decision is one pure function. It refuses to overwrite: a rotation distrusts every certificate already issued, so it is `FORCE=1` and never implicit. `WIRE_LEAF` is the fifth reading (bl-64a7, REMOTE §8.2) and folds to `Act::Leaf` rather than a field on the mint — issuing one extra client leaf is an act over a trust root that already EXISTS, so the rotation guard standing in front of a mint would be exactly backwards in front of it, and an enum leaves no state where both are half-true |
| `src/wire/server.rs` | the engine's synchronous mTLS listener (REMOTE §4, §8): a non-blocking accept loop so `Drop` stops it, a blocking thread per connection, and the `Answerer` seam the intake fills. An unauthenticated peer fails inside the handshake and never reaches the boundary |
| `src/wire/tls.rs` | the two rustls configurations, built from that material with the `ring` provider named outright rather than read from a process-global default that panics when there is none (AGENTS.md rule 4): the server requires a client certificate the operator CA issued, the client requires the same of the server and presents its own |
| `src/workdiff/{mod,candidates,plan,read,wire}.rs` | the §5.1 #32 **project work-diff** (VISION §4.10, bl-3746), cut on six seams: `mod` the vocabulary an answer is said in — the three distinct states a change can take, never one silent empty listing — with `read` the pure git read that says it: resolve each attempt's two ends, count the churn, read one file's patch (a patch pick is addressed by ball **and** handle since bl-c2bd, because a fan's candidates all wear the obligation's ball); `candidates` the §3.8 fan's rows (VISION V3.2–V3.3, bl-c2bd) — one row per cohort member at the ruled `work/<id>..attempt/<handle>` range, each wearing `delivered_commit`'s derived acceptance mark, the obligation read from the same last-claim rule the §8.6 writable root spends (`control::root::claimed`); `plan` the pure half — which attempts a workspace holds and balls' own delivery-target rule re-derived over the snapshot's balls, plus the numstat parse; `render` the §11 Work tab; `wire` its §8.5 JSON shape, written and read back beside the type whose vocabulary it spells (the reply roster still names the codec; bl-7067 added the decode half). The tests are the S11 rung — the pure derivation, the read against a real project repo, and the paint — plus V3's candidate rows against a real fan |
| `src/world/{mod,seed,marks,marks/write,hatch,tools,seat}.rs` | the composed world (§16.2): env + overrides, `litany prime` seeding, the §16.3 **agent balls space** (the `YOG_MARKS` fold, balls' two home directories per space, and the one `tasks_branch` read — bl-e47b — with `marks/write` the one act that points a space at a branch, authoring balls' layer-2 config in full and handing back what landed rather than what was asked), the §8.4 hatches, the §16.4 shim roster (the §8.6 control shim and, since bl-3ff4, `yog` itself among them), and `seat` — which seat may open a window, the guard that keeps that `yog` shim from becoming an agent's way to paint on the operator's desktop. `template.rs` — the §9.2 gate over the workspace-birth template — is deleted with the gate (bl-00ee): yog reads that file nowhere now |
| `src/world/wall.rs` | the per-workspace wall (§16.2, §3.1): the `YOG_WALL` layer, its layout and its read lens |
| `src/xdg/{mod,substrate}.rs` | env folds: yog roots, the wall (§16.2), percent-decode, and — in `substrate`, split at the cap on the seam between yog's own roots and where another tool keeps its things — the balls layout (delegated to `balls::layout::Xdg`, over the §16.3 space's two home directories) and the litany roots behind `LITANY_HOME`. brazen's ambient per-OS fold was deleted with the sharing it served |
| `tests/brazen_claude_code_decline.rs` | the dialect-decline pin (bl-5252, §8.3 rule 6): drives the LINKED brazen with the request a yog turn is — one user message and the unconditional `clients` tool — and takes the sentence its `claude_code` encoder declines with, before any transport. Three legs: the decline names no config fault (so litany's `Config` wrapper cannot carry it, which is why the marker table missed the family); yog's classifier routes it, wrapped exactly as litany's `AdapterError` renders it, to the §9.1 editor; and the same turn through a tool-carrying row reaches the wire, so what is classified is the dialect and never the request. A classifier keyed on another crate's words that nothing measures outlives the words |
| `tests/brazen_ollama_context.rs` | the Ollama context pin (bl-671d, §9.4): drives the LINKED brazen with a capturing `Transport` — no network, no server — and asserts the three facts the §9.4 caveat and its remedy rest on. A yog turn reaches an `ollama_chat` row with the output cap and **no** `options.num_ctx`, so the server's own default governs; a row's `body_defaults` `options` beside that typed cap is dropped whole and silently; and clearing the typed cap lets an explicit `num_ctx`/`num_predict` pair through. A caveat about upstream behaviour that nothing measures outlives the behaviour, so this fails the day brazen bl-f19d lands and names what to delete |
| `tests/design_citations.rs` | the citation guard: every cited `§N`/`§N.M` resolves to a DESIGN heading (the header's retirement doctrine, machine-checked); its `strings` half is the other direction (bl-cdd2) — a `§` belongs in a comment and never in a string the operator reads — split off at the cap on that seam, carrying the scanner only it uses |
| `tests/design_module_map.rs` | the module-map guard (bl-9f72, widened by bl-273c to every rule §12 states about itself): both path directions, the sort, the no-test-module rule, the two-cell row shape and single-entry — brace lists expanded, test corpora excluded from the file sweep per the rule above. The guard is the mechanism; the prose rule alone had already failed three times over |
| `tests/integration/support/{mod,recorder,world,payload,clock}.rs` | the story harness (STORIES "Test harness"): the fake-`bl` runner and the one-agent workspace, the argv/env recorder script and its read-back parser, the **multi-agent** workspace builder (goal stamps, `refs/litany/*` marks incl. the hold blob, dated commits, settled step framing), and the on-disk payload writers (`messages/`, `steps/`, `inbox/`, the balls clone dir) it composes — plus `clock`, the harness's own hand-driven `Clock` (bl-9006): `AppModel::boot` takes an `Arc<dyn Clock>` exactly so a test can supply one, and until INV-1 reddened under a nine-tarpaulin gate every beat in this crate had booted on the system clock and measured the machine instead of yog. Split at those three seams when the Z10–Z14 fake halves landed (bl-3b24) |

Testing per the house pattern: real-git tempdir fixtures (extended with a
balls-clone-layout and yog-ball-root fixture builder), argv-recorder scripts
(written bare — the binary-wide SPAWN_LOCK sits at the one fork, not at the
write, bl-6397), fake `/proc` and fake `lsof` output
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
*shape* is here. Bash, no repo deps, six tiers — a front door, a seat, a
fixture, a read tier, a verdict, and the story beats — cut so that no tier
knows the tier above it.

| File | Responsibility |
|---|---|
| `scripts/drive/drive.sh` | the front door: the live-world refusal (a two-directional path-prefix test against `$XDG_DATA_HOME` — a run wipes its world before it starts), the `target/release` PATH prefix that makes the drive prove the build in hand, one scratch world per run verb under a stamped evidence root **outside the checkout** (a world nests `git init` fixtures and a path-mirroring delivery territory; inside the repo those come within a fixture's reach), and the log skeleton at the tail — which is written **whatever happened**, including to a run that never reached a beat (bl-d0a0). Two things make that true and both were absent: one `stages.tsv` row per verb it drives, verb and exit code, so a stage that dies before its first assertion is still named in the report; and a guard around the generator, because a report's own failure may never replace the run's. It was a bare command under `set -e`, so a seat that never came up ended the front door on the GENERATOR's complaint, with a zero-byte report and the seat's error scrolled past. The Makefile's `drive` family is a one-line wrapper over it |
| `scripts/drive/preflight.sh` | the host contract, named in full and at once: the seven binaries the scripts actually call (`Xvfb`, `xdotool`, `ffmpeg` for capture, `ffprobe` for `locate.sh`'s read of a shot's size, `python3`, `git`, the `yog` under drive), the two world-seed files every run verb copies, and — since §16.2 moved brazen inside the wall — the **wire** tier (bl-49c6, demoted to advisory by bl-00ee): per provider row the seeded birth template names, whether a NEWBORN wall's table ships it (asked of the binary under drive through an empty `YOG_WALL`, never a copy of brazen's defaults kept here to drift) and whether the host credential `seed_wall` copies into the wall exists. Advisory both, because birth no longer gates on either — only a beat that SPENDS does |
| `scripts/drive/harness.sh` | the tier every run shares, sourced by `stories.sh` — its MECHANISM half since bl-7547 (the assertion helpers it used to carry are `predicates.sh` below): the two waiting primitives (`await`, `until_landed` — nothing waits on a clock, and `until_landed` takes a **no-op-on-miss gesture and a MONOTONE predicate**: it re-fires, so an equality on a quantity the gesture adds to is destroyed by its own retry loop, bl-0e44), `one_name_one_definition` (the guard over the one flat sourced namespace every `beats_*.sh` lands in — a duplicate top-level beat name silently deletes the earlier stage and leaves no verdict row to say so, bl-0e44), the per-run seat pair, and the verdict in both halves — the printf PASS/FAIL line and the `verdicts.jsonl` row beside it, which is also where the binary under drive is resolved and recorded (bl-d1af). It sources the three tiers of its own: `predicates.sh`, `wall.sh` and `gesture.sh` |
| `scripts/drive/headless.sh` | the engine tier (split out of `beats_headless.sh` at the cap, bl-7547): `boot_headless`, which backgrounds a bare `yog` and pairs a kill trap with the boot, and the two reply predicates every seatless beat is judged by — `reply_is`, which reads the TAIL of `gestures.jsonl` as JSON and evaluates a python expression over it (a `grep -q '"ready"'` is true of any reply mentioning the word, and the tail is what makes the answer THIS gesture's, bl-f16e), and `row`, the board-row sub-expression whose `+[{}]` makes a MISSING row a false predicate rather than a traceback. A tier and not a run verb: five sourced files spend it |
| `scripts/drive/predicates.sh` | the harness's **READ tier** (split out of `harness.sh` at the cap, bl-7547): every true/false question a beat asks of the world on disk, including the ones that name a conversation by **identity** rather than by a count or a rank (`stopped`, `other_root`, `seen_kind`), since yog's list has ranked nothing since bl-cad5, and the file predicates two runners share, `file_has` and `md5of` (which answers `absent:<path>` rather than the empty string that made two absences compare EQUAL, bl-f16e). Three disciplines hold across all of them and each was learned by a beat that passed while proving nothing: MONOTONE never an equality, an EMPTY SUBJECT refused rather than interpolated, and structured facts read as DATA rather than grepped |
| `scripts/drive/wall.sh` | the §16.2 WALL FIXTURE in its **three degrees** (bl-9e10 split `seed_wall` into `seed_wall_config` + `seed_wall_credential`, because nothing laid / the row table alone / both are three different first turns and the last is the only one every other verb wants), sourced by `harness.sh` and spent by `stories.sh`'s `seed` and `beats_s5.sh`'s `wall_config`: `BOOTSTRAP_WS`, `wall_dir`, `seed_wall`. The one tier that LAYS state instead of reading it, which is why it is neither in `harness.sh` nor in `predicates.sh` — those are waiting, seat and verdict on one side and reading on the other, and a fixture that copies a host credential into a scratch world is none of them (split at the cap, bl-f16e). A wall is keyed by a NAME, and for every world this harness drives that name is §3.1's bootstrap constant `home`, so the wall goes down with the world seed *before* the launch rather than chasing a mint the first model call has already outrun (bl-49c6, bl-1851) |
| `scripts/drive/stories.sh` | the STEERING doctrine that aims every beat in the family (§11 keys where the subject is the window, the §8.5 line where it is not, a coordinate only for a view, and never a pinned one — `locate.sh` above); the world seed — litany's roster and birth template **and the bootstrap sphere's wall**, both halves of one fact in one place (bl-1851) — the `gesture` transport, and the verb dispatch |
| `scripts/drive/beats_headless.sh` | the run verb, `run_headless` — since bl-7942 the ONLY one, every other having driven a window (it was already the only one that claimed no X display and spent nothing on the wire, bl-bb20). The whole run is `yog gesture` lines against a real world. It carries S2-T1's prepared binding, S14-T8 (its own premise) and S14-T5, and reaches for every other rung as a stage: `s13_board`, `s11_workdiff`, `s19_adjudicator`, then the armed `s18_admiral` LAST because from `/fleet` on a real loop is moving the board every beat above reads, then `s10_historian` over the drone it minted and `s13_schedule` once `/disband` has put the world back. The file's head records the whole scope decision, rung by rung, including every rung ruled OUT and why |
| `scripts/drive/beats_s18.sh` | **S18 Admiral's ARMED half** (VISION §4.3, bl-faca), fired from `run_headless` last and ending on `/disband` — from the moment `/fleet` writes its entry a real loop is claiming real balls in the run's world, so no beat that reads the board may come after it. It drives the TRAJECTORY, which is the half no fixture snapshot holds: a tick claims the board's top ready row through real `bl` and mints a conversation through the real start flow; `lease_min: 0` (the operator's own knob at its floor) makes the next tick the deadline, so the claim comes back with the comparison — never a diagnosis — on the trail; and a later tick does NOT retake that ball (bl-3988's given-back law, the wedge the rung exists to catch). Armed cap arithmetic is the loop's alone because the fixture gives back the hand-bound S13 ball first, and **nothing spends**: the fixture removes the wall's sign-in (§16.2), so every drone declines at the wall before a request is made |
| `scripts/drive/beats_s10.sh` | **S10 Historian through the headless spellings** (bl-faca), over the drone `beats_s18.sh` just minted — one fixture, both rungs. bl-bb20 ruled the rung out because these surfaces had no headless spelling; bl-6233 gave every one a query and a line and bl-13f9 put the window on the same door. Six reads, each asserted against content only disk could carry: the composed goal's ball id in the transcript's first turn, a settled `failed` step whose `tokens.total` is 0 — the run's own no-wire proof, on the surface a spend would show — the step drill-in's request record, the worktree listing with `goal.md` previewed, the spine notch naming its `tx/` cut, the freeze's lineage/oid/files, and a deposit followed to where it lands (delivered turn, empty inbox) |
| `scripts/drive/beats_s11.sh` | **S11 Auditor's headless rung** (bl-bb20, its own file since bl-7547), fired from `run_headless` after the fixture `/assign` cut a real worktree and before `beats_s19.sh` reads the work it lays. The only rung in the seatless family whose subject is GIT: a real `bl claim` worktree off `main`, a real commit landing in it, `Query::WorkDiff` as a pure git read of `target..source`. Asserted by IDENTITY at every field — the file's name and its ±, both refs, both oids present and different — because a reply that merely has the right SHAPE is what an empty diff also has |
| `scripts/drive/beats_s13.sh` | **S13 Boardwalker, both halves**, fired from `run_headless` (bl-7547 brought the read half here from inside the run verb; it was `beats_s13w.sh`, the write half alone). The READ half `s13_board` (bl-bb20) goes early, before the armed rung starts moving the board: the four columns over STORED balls facts, awaited rather than read once because yog's join lands after `bl claim` writes, with `blocked` coming from balls' own resolution of a live edge. The WRITE half `s13_schedule` (bl-dbde) goes last, after `/disband`: one `/create` carrying all four scheduling facts, the board read back for the priority, the parent and the block, `bl show` as the witness for the tag no reply carries, and the clearing half (`--no-priority --no-parent --no-needs`), without which a remote coordinator could add a mis-wired blocker and never unwire it |
| `scripts/drive/beats_s19.sh` | **S19 Adjudicator's real-substrate half** (VISION V3, bl-77bc), fired from `run_headless` after `s11_workdiff` laid real work on `work/<ball>`: a real fan spread answers two rebound starts bound to two real attempt worktrees whose branches share the target's own tip; real work in one candidate delivers by handle and the target's history carries the `[handle]`-tagged squash; the stale sibling's delivery is refused by balls' own law with the target tip unmoved; retirement releases the worktree and keeps the ref; and `/science` answers the claim row over the real store. **No model call anywhere** — cohort membership on the read surfaces needs real fire rows, a fire is a `litany prompt`, and that join is the in-crate tests' half over fixture trails (the file's head carries the scope decision) |
| `scripts/drive/cleanroom.sh` | the §16.7 W14 standing done-bar: a room whose only substrate on `PATH` is one `yog`, asserted rather than assumed — and, since §16.2 moved brazen inside the wall, asserted in the same two directions for brazen state: the room lays no ambient `$XDG_CONFIG_HOME/brazen` and refuses to run if one is there (bl-49c6) — then handed to `stories.sh` unchanged |
| `scripts/drive/logskel.sh` | the report generator: sha, host tuple, load, the driven binary, the stage table read from `drive.sh`'s `stages.tsv` (exit code beside verdict-row count, so a stage that reported nothing is visible) and the beat table emitted from `verdicts.jsonl` — nothing re-asked of the generator's own PATH, which resolves the *installed* yog and not the build under drive (bl-d1af) — with every judgement left as an explicit hand-finish marker. A drive log starts generated and is finished by hand; the house style (evidence quoted, not summarized) is the operator's half. **A verdict-less run is reported, not refused** (bl-d0a0): zero beats is a finding — "NO VERDICTS PRODUCED", and the stage table says which stage — not an empty table and never an exit code, which is how the generator came to be the loudest thing about a run it knew nothing about |

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

`litany prompt` is spawned detached (§8.1), so a prompt-time failure surfaces
from disk rather than from a pipe yog holds. This is the
robustness-over-immediacy trade: yog's death must never kill a running loop,
and the short piped steps (`bl claim`, `litany new`) still surface their errors
directly in `ops.jsonl`.

**Deferred to disk, not discarded (bl-a649).** stdin and stdout stay null;
**stderr goes to a per-spawn sink file** (§4.2 / §5.2:
`$XDG_STATE_HOME/yog/detached/<ts>-<workspace>.err`) — without it, a driver
that launched and then died was byte-identical to a clean launch. The sink is
a *file*, not a pipe: yog holds no fd, so the §8.1 lifetime guarantee is
untouched — the child outlives yog and keeps writing to an inode nobody must
be alive to drain. The ops row's `stderr` is folded in from that file **at
read time**, on the ops sweep (§7.2), and a capture the notice classifier does
not recognize makes the row a rendered failure — the existing §7.3 machinery.
Only the *immediacy* is still deferred: the death is visible on the next sweep,
not at fire.

**A non-empty capture was the wrong test (bl-1296, structurally fixed
bl-b95e).** The sink is a *transport* and says nothing about what it carries;
what it meant was decided one layer up by "the driver said anything at all",
and litany's contract makes this channel an **operator-notice** one as much as
a dying one — declines, superseded compaction landings, accepted-crash-class
launch notes, a §6 budget stop, every one on a path that returns `Ok(())`.
Since the sink is append-only for the driver's whole life and the fold re-read
its tail on every sweep, one benign line held the newest row of its origin in
the §7.3 banner and the ⚠ chip until it was acked. bl-1296 answered with a
marker table over the sentences litany prints — knowingly fragile, and unable
to reach the defect above, which is about *time* rather than words.

bl-b95e moved the decision instead, to exactly where the orphaned-mail
paragraph below already puts it for `driver.log`: **the fold is gated on
state**, and the tail is diagnosis. `opslog::launch::stillborn` reads the
launch's own target out of the row's argv — the `--name` a `prompt` minted, the
agent id an `advance` was handed, the tokens living in `opslog::launch` so the
spawn and the reading cannot drift — and answers *stillborn* when no matching
agent is being driven (§3.5) and none has acted since the row's stamp. Both
halves hold vacuously when the target is not on disk at all, which is this
sink's own class: a driver that died before writing a branch. Ahead of it stand
the §7.3 grace window and §10's rule against a false definite — a workspace
that derived no tree is no verdict, never an accusation. The marker table, the
fifth `OpOutcome` and its badge are gone with it; a driver that filed a notice
and carried on is an ordinary handoff, because nothing reads its sink. The cost
is that a healthy launch's row has no stderr to expand — a capture log is
diagnosis, read where something is wrong, and these lines keep their durable
home in `driver.log` (§13.3's own record seat).

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
sink; a turn continued by `litany message` is driven by a child litany launched,
so no `-2` row exists to fold anything into. Meanwhile the adapter's own words
were already sitting inside the step yog was reading — `stderr.log`, the third
file of the record (litany ARCH §2.3). So the conversation surface owes *both*
halves for this class, and the wound states the reason instead of pointing
across at a trail that may hold nothing. The ops sink is unchanged and still
answers for the spawns yog itself fires.

**And one class had no step to hang either half on (bl-ace6).** The wound is a
fact *about a step*; a driver that dies **at the boundary** — an unpaired-tail
decline, a lease fault, a launch that crashed before its model call — has
delivered the mail and created nothing else. The deposit's own ops row says
`exit 0` (the deposit succeeded; the failure was a grandchild litany launched),
the steps tree is unchanged, and the transcript simply stops — the exact shape
an operator reads as "the chat stopped working". The **orphaned-mail state**
(`steps_view::orphan`) is that class's vocabulary: the newest transcript entry
is a delivered `NNN-<sender>.md` while nobody holds the agent's lock — a pair
that is derivable, stored nowhere, and on a healthy branch exists only for the
relaunch gap, because delivery only ever happens under the driver's own lock
(litany §2.11). It banners at Altitude 1 through the wound's own grace window — sized
against the rising edge's real latency, and shortened from the other end by the
send marking its own workspace dirty (both bl-18e8, §7.3's row) — and its
sentence carries the tail of `steps/<agent>/driver.log` — where litany
has bound every launched driver's stderr since 0.0.9 (its bl-55f9), the file
yog pinned that release for and, until this, never read. The log is append-only
across launches, so its *content* is never the trigger — a stale line from a
healed crash must not alarm — only the diagnosis. The badge vocabulary and
attention's rest-not-wound rule are deliberately untouched (bl-d816 tracks
whether they ever should not be).

**And the class had a third member with no paint at all (bl-abba).** An agent
whose executor died *inside* a tool window leaves its assistant entry committed
with `tool_use` blocks nobody answered, no hold mark, and its lock free. Neither
banner above fires, both for good reasons: the wound is *unanswered on disk*
(`response.json` empty and `meta.json` absent) while here the model call
returned and settled, and the orphan wants a `.md` on the tail while here it is
an assistant `.json`. So the conversation read as an ordinary idle one that
simply chose to stop — the one member of the class that looks like nothing is
wrong. It is now a second **`Tail` shape of the same state**, not a third
banner: same predicate pair (a tail owed an answer, nobody driving), same
`driver.log` reason, same seat, same grace window, same sentence-in-one-home
rule — so the wire pays for it the way the wound already did (bl-fb87), the
`orphaned` boolean becoming a class token because a `(bool, Option<reason>)`
pair stops being a bijection at the third arm. Two things are its own: only the
newest entry can be in this shape (a call answered later has its result
committed *after* it), so the predicate is one file read through the
transcript's own classifier rather than a pairing walk; and a live
`refs/litany/held/<agent-id>` park suppresses it (§8.6), because a parked call
wears exactly this shape and is waiting on purpose. **The sentence carries the
remedy**, which is the one place this shape differs from the other two: the
state is transient and self-healing — the next drive boundary settles the
window with an in-band `is_error` `tool_result` per unanswered id before
delivery (litany ARCH §6, its bl-4187, consumed in bl-4c1f) — but nobody
deposits into an agent that looks finished, so on an unattended box the window
before that deposit has no upper bound. Naming the gesture is what closes it.

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
  yog hold makes a `litany message` writer skip launching a driver and strand
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
  acknowledgement (no litany ack verb exists; marks are level-triggered) and
  violates the governing requirement.
- **Ops-log-free design** — gate/close/scan output would be RAM-only and
  non-convergent; the current stderr-and-drop hole would persist.
- **The word "session" in code or UI** — litany bans it for cause; the
  concept dissolves (start = claim+new+prompt, the unit is the workspace, the
  list is workspace enumeration).
- **Daemon/socket/IPC between instances** — disk is the bus (every
  substrate's religion). *Scope narrowed by the client/server split (bl-b9a2,
  `docs/REMOTE.md`): a seat may reach the §8.5 boundary over mTLS channels —
  since bl-aaec one per workspace held elsewhere (REMOTE §8.2), each a client
  transport to one engine. The wire is never the bus between two engines;
  instance coordination stays disk-only.*
- **Linking litany/brazen crates *as a blanket rule*** — *superseded by
  §16.5.* The original stance (CLI + disk is the whole contract; brazen's
  library API is explicitly unstable) is now the *phase-1* posture only; the
  phase-2 end state embeds `balls`/`brazen`/`litany` as **exact-pinned** crates.
  Instability is answered by exact-pinning (brazen's own README posture), and
  process semantics stay non-negotiable regardless of linking — drivers are
  processes holding flocks; plugin dispatch stays subprocess (§16.5).
- **A reproduction hatch at the wound** (a "re-run this prompt with stderr
  attached" button beside a §7.3 no-response step, running `yog exec litany
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
- **yog aping litany's seeding** — the nested `LITANY_HOME` is seeded by
  litany's own bootstrap verb (**`litany prime`**, landed upstream as
  bl-6d83: `LITANY_HOME=<dir> litany prime`, seed-if-absent, idempotent,
  silent on success; `models.yaml` at the home root is the seeded marker),
  never by yog reproducing litany's seed logic; a second seeder drifts from
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
`bl`/`litany`/`bz` state. It inverts: yog composes its own nested world
(§16.2), and the substrate state yog drives is *yog's*, under yog's data root.
Playing on top of the user's *direct* tool usage stays possible — an agent's
task branch can be pointed at the project's shared store branch at launch
(§16.3's launch clause) — so **compatibility with an ambient workflow is the user's decision,
not a structural given.** The world encapsulates litany, balls, and brazen
state so completely that yog and the human's own shell never collide unless the
user chooses overlap.

**Rejected:** yog as a pure ambient overlay (no nested state) — the coordination
point would be the user's live working tree and clones, so every yog action
would perturb the user's own `bl`/`litany` work; encapsulation is what makes yog
safe to run beside a working human.

### 16.2 The composed world environment

yog reads the ambient environment once (`xdg::Env::from_env`), computes its
data-root anchor `$XDG_DATA_HOME/yog`, and composes **one** world `Env` — the
ambient snapshot plus a fixed override set — used both to derive every substrate
path yog reads *and* to spawn every child. Overridden, to nest:

| Var | World value | Nests |
|---|---|---|
| `LITANY_HOME` | `<yog-data-root>/world/litany` | litany config **and** data (the `Env::litany_home` collapse) |
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

**The engine's leaf and marks moved at the REMOTE §12 fence, and an old world
is migrated by hand (bl-9905).** The engine crate renamed `lernie` → `litany`
with no compatibility shim (upstream bl-2f58), taking three durable-state
surfaces with it: the variable `LERNIE_HOME` → `LITANY_HOME`, the harness roots
it collapses (here, the world leaf `world/lernie` → `world/litany`), and the
in-workspace mark namespace `refs/lernie/*` → `refs/litany/*` that §8.6's
adjudication and §3.5's liveness read. A world founded before the fence is
therefore invisible to the pinned engine — nothing is lost, but nothing looks
where it sits. **The migration is one operator paste, rooted at yog's own data
root:**

```sh
world="${XDG_DATA_HOME:-$HOME/.local/share}/yog/world"
mv "$world/lernie" "$world/litany"
for repo in "$world"/litany/workspaces/*/repo.git; do
  git -C "$repo" for-each-ref --format='%(refname) %(objectname)' refs/lernie/ |
  while read -r ref sha; do
    git -C "$repo" update-ref "refs/litany/${ref#refs/lernie/}" "$sha"
    git -C "$repo" update-ref -d "$ref"
  done
done
```

**yog does not run it, and does not silently proceed without it.** Not at boot,
because the two halves — the leaf and the refs — are one act, and performing
the cheap half in code while the paste owns the expensive half would be two
representations of one migration; because the population is enumerable and
one-time inside precisely the pre-stability fence that exists for a break like
this; and because a permanent boot path that serves a transient fact fails the
severability test. What yog owes instead is a refusal: `world::seed`'s
`ensure_seeded` is the one place that would *found* an engine home, and a
`world/lernie` present with no `world/litany` makes it refuse
(`SeedError::Unmigrated`) naming this recipe, rather than priming a rival empty
home beside the operator's conversations. That is §5's use-is-attempt — the
refusal lands on the first gesture that would do harm — and it is not a
bootstrap branch: a fresh world has no such directory and takes the general
path with empty inputs, and a migrated world is seeded and never reaches the
probe.

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
(litany's home, balls' store layout) stays nested as before.

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
first `litany prompt` fire together, §8.1), so a fixture laid "after the mint" is
always later than the first model call: every scratch-world drive run's first
turn died `unknown provider` for two days while the beat asserting its reply read
as a wire outage.

**One var carries it: `YOG_WALL`**, layered onto the world's override set for
a workspace-bound read or spawn, never part of the world set itself (the world
is workspace-free — it is a pure function of the anchor). Everything else is
derived from it, so there is one fact and no second var to drift. Setting it
once at the edge that knows the workspace is enough for the whole descendant
tree: the fired `litany` loop inherits it, litany hands its own environment to
every tool subprocess (litany ARCH §3.3), and a bare `bz` in an agent's bash
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
either way), and the world's own editors — litany's global config, the yog
clock's `cadence.yaml` — deliberately stay out of that bundle, since one file
with one draft must not become one draft per sphere.

**The world is a value AND a place (bl-81c9).** `world::compose` makes the world
an `Env`, which is enough for every yog fold and for every child yog *spawns*
(the override set rides the `Command`, W2). It is not enough for the §16.7
namespaces, whose whole point is that the substrate runs **in yog's own
process**: the linked balls takes its two homes from the `Edge` the arm hands it,
but its plugin chain spawns `bl-delivery`, `bl-tracker` and `git` as children
that fold `$XDG_STATE_HOME` out of their own environment, and the linked litany
resolves its harness root from `getenv("LITANY_HOME")` with no injection seam at
all. So a substrate arm **stands the process in the world** before the embedded
crate runs — one `set_env` of the same override set, at the process edge
(`world::inhabit`) — and every read, spawn and descendant follows from it.

Until this ruling a bare `yog bl` / `yog litany` at an ambient shell drove the
operator's OWN landing and harness while advertising the nested one: `yog bl
prime` founded its clone under the ambient `$XDG_STATE_HOME`, `yog bl claim` cut
its worktree in the ambient plugin territory, `yog litany prime` seeded the
ambient `LITANY_HOME`, and `yog exec bl …` was the only spelling that meant what
both spellings say. **One command spelling must not address two universes**, and
`yog exec` is a hatch for a human's shell (§8.4), never the repair for yog's own
verbs.

The fold is **idempotent for the same reason the override set is** (above): every
value is a pure function of the anchor, which the world never overrides, and the
`PATH` prepend recognizes its own entry — so the spawned case, where the parent
folded already, re-derives an identical set and stacks nothing. That dissolves
"already composed" rather than testing for it, and because the set is exactly
the §16.2 three, the fold displaces neither an agent's space (`YOG_MARKS`, §16.3)
nor a workspace's wall (`YOG_WALL`) — both ride one layer in, and the `bl` arm
folds the space in **after** this set for exactly that reason (§16.3's
worktree ruling: balls' plugin children read `$XDG_STATE_HOME`, so a space has to
be a place there too). Two arms fold
nothing, by one test — *does this arm's substrate read the process env?* A
**discovery probe** (`--help`/`-h`/`--version`/`-V`/`--skill`, §8.5's narrow
form) does not, and asking what a verb does must not depend on a world existing;
`bz` does not either, because since the blast-radius ruling brazen's state is
per-wall rather than per-XDG, so the world's three vars name nothing it reads.

**I1/I2 hold against the nested disk:** every §5.1 derivation resolves through
the world `Env`, so "disk is the app" means yog's world; two instances compose
the identical world from the identical ambient env and converge exactly as
before. **Severability widens (§3.1):** one `rm -rf $XDG_DATA_HOME/yog` erases
the entire world — nested litany home, nested balls state, and yog's own
artifacts — and leaves the *ambient* substrates untouched.

The `LITANY_HOME` seed is written by litany's own bootstrap verb
(`litany prime`, upstream bl-6d83), never by yog — yog composes the env and
calls the verb; it never apes litany's seeding (§14).

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

**A space is a value AND a place too, and it owns its worktrees (bl-c21d).** The
definition above is exact where it says *worktrees*: a space IS balls' state
home, and balls folds its clone bundle, its plugin territories (`bl-delivery`'s
`work/<id>` code worktrees, `bl-tracker`'s push mirror) and its attempts tree off
that one fact. Supplying the fact through the `Edge` alone supplies it to the
*linked* balls and to nothing balls **spawns** — `bl-delivery` is a real
subprocess that rebuilds its own `balls::layout::Xdg` from `$XDG_STATE_HOME` in
its process env and holds no `Edge` at all — so an own space kept its store and
left its worktrees in whatever state home the process happened to stand in (the
world's since §16.2's place ruling, the operator's ambient territory before it).
So the `bl` arm stands the process in the **space** as well as in the world: one
more `set_env` of `XDG_STATE_HOME`, layered on the §16.2 override set exactly as
`YOG_MARKS` and `YOG_WALL` layer onto it for a spawn, one var deep and last write
wins. It is **unconditional, and that dissolves the case rather than branching on
it**: an absent `YOG_MARKS` resolves the world's own space, whose state home is
the value the world fold just wrote, so the no-marks path re-writes one identical
value and nothing needs to ask "own or not".

**The rejected answer was the other one this defect offered** — *worktrees are
deliberately world-wide, one code territory per project whoever claims, and this
section drops the word.* It cannot be said in balls' terms: balls has ONE state
home, so "clones in the space, territories in the world" is yog injecting two
different values for one fact forever, and every further fold off that home lands
on whichever side its reader happens to sit. balls' attempts tree
(`balls/attempts/<invocation-path>/<handle>`, the private worktree its own
attempt capability cuts — the one §3.8's fan consumes in process) already
demonstrates the split it would institutionalize: balls computes it **in-process
from the `Edge`**, so it would land in the space while the `work/<id>` worktree
beside it landed in the
world, two sibling territories of one `balls/` state dir straddling two roots for
no reason anybody chose. It would also falsify this section's own severability
clause — *delete the space and the policy is gone* — because an agent's code
worktrees would outlive its space, and §3.6's unmaking (which takes `<wall>/marks`
with the wall) would strand them under the world.

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
| subagents inherit | `YOG_MARKS` rides the spawn, litany hands its environment to every tool subprocess (litany ARCH §3.3), and a bare `bl` is the world's shim re-entering yog. Zero mechanism of its own |
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
only as the plane on which litany's worker agents drive tools by bash — an
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
yog seeds `<yog-data-root>/world/tools/{bl,litany,bz}` — re-exec shims of the yog
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
`serve`, the three namespaces) and exit `64`, the usage class the wall-less
`bz` already refuses with. It is keyed on the **environment, not the shim**, so
it holds however yog was reached — including an agent that finds the operator's
installed yog on the ambient `PATH`, which is the very drift the roster entry
exists to end. The operator is never caught by it: `yog env` prints
`LITANY_HOME`, `XDG_STATE_HOME` and `PATH` — plus `YOG_WALL` when, and only
when, it is asked for a workspace (`yog env --ws <name>`, §16.2's headless
sign-in) — and never `YOG_NAME` (§8.4), so a human who ran
`eval "$(yog env)"` carries no window seat; and every yog-spawned child
names a namespace verb, so none reaches the window seat at all.

**Version coherence is structural, not gated.** The phase-1 capability gate
(per-verb `--help` probes, a toolchain pane, dispatch-layer refusals) could
only *detect* skew, and only some of it — a `litany` linking a different
brazen than the `bz` beside it was capable on every probe and fatal on every
dispatch. The end state **dissolves** the class instead: one `Cargo.lock`
names one balls, one brazen and one litany, and every process image in every
chain is the yog binary those pins built — a skewed pair is not guarded
against, it is unrepresentable. The lockfile is the version (§16.5); a gate
every tool passes by construction is dead code, so the gate is gone (§16.6).
After-the-fact rendering stays for a different reason — the §7.3 no-response
wound, the §8.1 driver-stderr sink and the §13.3 orphaned-mail banner
(bl-ace6) together surface a dead driver wherever it died — mid-step, in a
yog spawn, or at the boundary of a driver litany launched — and none of
them was ever gate machinery. (Until bl-ace6 this sentence claimed the first
two alone covered *any* dead driver; the boundary class — no step, no yog
spawn — was invisible, which is the gap that ball closed.)

**The discovery mechanism is bash, not litany's tool slot (W9).** litany
discovers `litany-tool-<name>` under its own data root with a JSON-on-stdin
contract, a schema file, a skill, and a `providers.yaml` grant — a surface
yog could reach only by aping litany's config authoring (the §14 rejection),
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
batteries; no runtime dependence on any system-installed `litany`/`bl`/`bz`
survives, and the machine's local checkouts/installed binaries may differ
freely from yog's linked versions. (The forcing incident: a machine-wide
brazen upgrade against litany's exact pin killed every prompt silently — the
skew class the pin-in-lockfile abolishes.) The upstream seams each embedding
stands on: balls' promoted typed read surface (`reads::Catalog`,
`reads::Entry`; upstream bl-9901) and the plugin-binary entrypoints
(`delivery_bin::run`/`tracker::run`; U-balls-3) — mutations stay behind the
verb surface, the worktree-open → seal(ff-only CAS) → unseal protocol and the
plugin chain never bypassed by a raw file write; brazen's feature-gated
`native-host` exposure of `brazen::native` (the feature is the purity
boundary its upstream test pins); litany's `litany::cmd` API (CLI-parity
CI-enforced) with the injected `Fx::driver_target`/`Fx::adapter_target`
re-entry seams — yog re-execs **itself** as driver and adapter.

**Process semantics are non-negotiable regardless of linking:** drivers are
processes holding flocks, and plugin dispatch stays subprocess. Linking changes
what code yog *calls*, never the concurrency model — a linked litany still runs
drivers as flock-holding processes, and yog re-execing itself as the driver is
exactly that process, not an in-process task.

### 16.6 Phase 1 — binaries (retired)

Phase 1 shelled to host binaries and is complete; its task record (W1–W7) is
retired to git history (bl-43cd). What it built and kept is stated above: the
world env (§16.2), seeding via `litany prime` (§16.2, §14), the store-branch
knob (§16.3), the escape hatches (§8.4). Its capability gate and toolchain
pane (W5) were deleted outright once the embedded crates made every verdict
but `Ok` unreachable — a gate every tool passes by construction is dead code
(§16.4); the skew incidents that shaped W5 are the argument for §16.4's
structural answer, and the Login pane is the surviving half of its pane
(§8.3, §11).

### 16.7 Phase 2 — batteries included (landed)

**The one mechanism: self-multiplex.** Every spawn that today targets a host
binary instead targets **yog's own executable** with a verb-namespace argv —
`yog litany <argv…>`, `yog bl <argv…>`, `yog bz <argv…>` — dispatched by
`main.rs` to the embedded crate exactly as each upstream's own thin bin does
(the `--editor-apply` / `yog env` / `yog exec` multi-call pattern, §8.4,
generalized). The principle from §16.5 holds unchanged: **linking changes what
code yog calls, never the concurrency model.** Every seam that is a process
for a *reason* stays a process — litany's detached drivers and execve
lease-baton hops, balls' per-op change-worktree + ff-only-seal CAS, the
GUI-vs-stdout boundary on mutating bl verbs — only the binary on the other
side becomes yog itself. Seams that are *data-shaped* go in-process: balls
store reads (typed `Catalog` load replaces `bl list/show --json`
spawn-and-parse), brazen config projection. Skew death is structural: cargo
resolves ONE brazen in yog's graph (yog's pin and litany's pin must agree or
the build fails — the lockfile is the parity check), the multiplexed verb
implementations ARE their versions, and litany's runtime `bz --version` guard
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
**Residual host deps, documented and deliberate:** `git` (both yog and litany
shell to it; a battery not worth including) and the platform probes
(`lsof`/`/proc`). `$EDITOR` never escapes: yog already re-enters itself
(§9.3). The `Binary` env overrides (`LITANY_BINARY`/`BL_BINARY`/`BZ_BINARY`)
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
Four facts live only here:

- **litany's `Fx` re-entry targets are spelled as the world's shims**
  (`world/tools/{litany,bz}`), never the bare yog executable: both targets
  are single paths litany spawns verbatim, so the bare exe would drop the
  namespace word and re-enter the GUI. The arms converge the shim roster on
  the way into every verb — one read, no write in the steady state — so the
  first invocation has valid re-entry targets with no ordering dependence on
  a start.
- **The `bl` and `litany` arms fold the world into their own process env** on
  the way in (`world::inhabit`, §16.2's value-and-place ruling), which is what
  makes a bare `yog bl` at an ambient shell the same world `yog exec bl …`
  hands out. The other arms do not: `bz` is per-wall, the plugin arms are
  spawned by a balls that folded already, and `gesture`/`seat`/`tool-host` are
  yog's own code over a composed `Env`.
- **One read is still a subprocess: the closed listing** (`yog bl list -s
  closed --json`, §5.1 #4) — balls' dead-set history walk is not on the
  promoted read surface. **Residual upstream ask, U-balls-2: promote the
  dead-set walk** (`reads::history`); landing it deletes yog's last spawned
  read and its last JSON parse together.
- **Which file yog itself is, is read ONCE per process** (`cli_outbound::
  self_exe`, bl-f558) — a fact of the process, not a question re-asked at every
  resolution. W12 made `current_exe()` the default target of every namespace,
  which handed yog a fact it was reading afresh in four places; a fact with
  several readings drifts, and here it drifts catastrophically. **A live engine
  outlives its own inode:** installing, updating or rebuilding yog moves a new
  file onto yog's pathname (`cp new yog.next && mv -f yog.next yog`, atomic by
  `rename(2)`) while the running process keeps executing the unlinked image,
  and from that instant Linux's `/proc/self/exe` — the whole of `current_exe()`
  there — reads back `<path> (deleted)`, a procfs annotation naming a file that
  does not exist. The §8.6 control is the sharp case because `ensure_control`
  re-resolves on **every** Start, so one Start after an install burned that
  annotation into the adjudicator litany consults before every granted tool
  invocation, and every later tool call failed closed at a boundary whose cause
  was hours in the past. Two rules answer it and both live in that module.
  *(a)* The reading is taken at the **first** ask — which every face performs at
  boot, `main.rs` converging the tool roster before eframe and a `yog <ns> …`
  re-entry resolving before it dispatches — so the process holds the pathname
  it was born from for its whole life. That is the honest answer as well as the
  stable one: the shim must name yog's install path, and the install path is
  exactly what the install did not change. *(b)* A reading that names no file
  is not a reading, judged by `stat` and never by spelling — no `(deleted)`
  suffix is stripped and none is matched, because a binary genuinely installed
  under a name ending in ` (deleted)` is a real target while an unlinked one is
  not, and only the filesystem can tell those apart; the same one test covers
  the platforms that report the original path with no annotation at all (macOS
  `_NSGetExecutablePath`). What is left after both is a pathname deleted
  outright rather than replaced — a stale target, honestly named, healed by the
  next install. **The durable half is `world::tools::ensure_shim`, which
  refuses to write a shim whose target is not an absolute path.** When the
  self-exe reading is unusable, resolution falls back to the bare PATH *name*
  of the tool, and persisting that here is a catastrophe rather than a
  degradation: the world's `PATH` is fronted by the tools dir itself (§16.2),
  so `exec 'bl' "$@"` re-resolves to the shim and spins, and where a host
  binary answers the name it silently runs the operator's installed tool
  instead of yog's (§16.4's (b), bl-d1af's defect class). A convergent
  artifact's honest answer to *"I do not know what to write"* is to leave the
  last good file alone and say so — `yog env`/`yog exec` warn on stderr, a
  Start fails with the reason.
