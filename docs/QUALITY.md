# yog — interface quality

**Scope.** `STORIES.md` owns *does it work*: the acceptance ladder and its
two-half done-bar. This document owns *does it feel good*: the rubric every
rendered surface is judged against, the shot-sheet that samples the surfaces,
the audit protocol that turns the rubric into a verdict, and the currency rule
that says when any verdict — functional or quality — has expired. When this
document and a shipped surface disagree, one of them is amended deliberately,
never coded around.

**Why the audit is screenshot-borne.** Views — focus, selection, scroll, tab,
drafts — are per-instance RAM with no boundary spelling, by design (DESIGN
§8.5). Nothing on disk can confirm what a surface looks like. So quality is
judged from exactly two witnesses: the captured pixels (`yogdrive.sh shot`)
and the in-crate paint layer (`src/paint_probe.rs`, `src/shell/acceptance/`).
A quality criterion phrased against any other evidence is a category error.

---

## §1 The rubric

One verdict per (surface, criterion): **pass**, **violation**, or **n/a**
(criterion has no subject on this surface — n/a must say why). A violation is
a filed ball citing the criterion id and the shot that shows it. The criteria
carry their precedents: the ball history is the rubric's proof that each line
pays rent — every one of these has already been violated and fixed at least
once, or is the standing principle a fix cited.

### Geometry — G

- **G1 Nothing clipped.** No text or control is cut off or rendered off-screen
  at any captured window size; deliberate elision shows an ellipsis and the
  full value is reachable (hover, inspector, or wider panel). Precedent:
  bl-0c71 (bootstrap composer clipped at default size), bl-b491 (empty-balls
  hint truncated mid-command).
- **G2 Panels are bounded and recoverable.** A panel never sizes itself to its
  content's worst case, and never gets stuck: it can always be dragged back.
  Precedent: bl-ac3d (left panel width explodes to fit an absolute path and
  never shrinks), bl-9ad4 (boundaries not draggable).
- **G3 One grid per surface.** Rows in a list share columns and baselines; one
  row is one scan line. A cell that breaks the scan (full prose inline where
  every sibling shows a summary) is a violation. Precedent: bl-0bf9 (a prompt
  op's full goal text inline in the activity list).
- **G4 Resize sanity.** Every surface stays operable at the small capture and
  wastes no dead field at the large one; nothing overlaps at either.

### Honesty — H

- **H1 One fact, one rendering.** A fact paints once per surface. Two
  renderings of one fact is one too many (VISION V1 ruling, bl-1802).
- **H2 Absence is named.** An empty region says what it is and names the paved
  path in full; it is never a bare blank or an unlabeled box. Precedent:
  bl-b2ed (new-ball form: two unlabeled black boxes), bl-b491.
- **H3 Alive things look alive; dead things name their cause.** In-flight work
  shows liveness where the operator is looking (streaming tail, activity
  indication — bl-90b5, bl-905f); a failure renders its cause, never a quiet
  zero (bl-7f2e: driver death read as a quiet step; bl-4895: dead detached
  prompt showed no banner).
- **H3a A failure yog can fix ends in a control.** Naming the cause is the
  floor. Where the remedy is a surface yog already has, the failure carries the
  thing that goes there — a shell command is a fallback beneath it, never the
  whole answer: the §9.4 picker's credential fault was not fixable through
  the yog interface, and had to become so. Precedent: bl-8e34 (an auth-shaped step failure names its row so Login
  is one click), bl-91f1 (the roster's own credential fault, which had only
  `BRAZEN_API_KEY` and a `bz --login` line). The remedy is derived from an
  existing authority or it is not offered — a routed guess is worse than an
  honest command.
- **H4 No capability theater.** No control renders whose verb cannot fire
  (VISION §8). A control that would lie about what it does — a dropdown
  offering a choice that is not real — is theater even when enabled.

### Hand-feel — F

- **F1 Every gesture has a keyboard spelling.** Anything operable is reachable
  by a DESIGN §11 binding, a §8.5 line, or the §11 **focus floor** — Tab walks
  the frame's controls and Space presses the one it reached — so the pointer is
  never the only path, including for a pick no line addresses (the notch pin,
  a step drill-in, a file preview, a provider sign-in). And every control
  **says** which of the three it takes, on its hover (§11 discoverability rule
  3). Both halves are machine-held, not reviewed: `acceptance/floor.rs` drives
  the walk, `acceptance/hover/spelling.rs` fails a hover that names no
  spelling (bl-30e3 closed the last known binding gap, bl-478d the sweep;
  regressions are violations).
- **F2 Focus is where the eye is.** Typing and Enter land on the surface the
  operator is looking at; a draft never bleeds across verbs or targets, and
  nothing fires an unrequested or empty payload. Precedent: bl-a69a (draft
  bleeds across verbs), bl-9acf (raising a workspace opens an empty draft and
  Send fires it), bl-2826 (minting a workspace does not focus it), bl-49cb
  (view stays on the placeholder after send).
- **F3 Overlay discipline.** Escape dismisses; clicks do not pass through to
  surfaces beneath; dismissal loses nothing typed elsewhere. Precedent:
  bl-d921.
- **F4 The frame never blocks.** Every frame paints from snapshots; no
  substrate read, spawn, or wait runs on the paint path (STORIES INV-1 is the
  functional twin; the felt symptom — a stutter or a beachball — is judged
  here).
- **F5 Gesture economy.** The benchmark is the Codex bar (STORIES intro):
  zero-to-reply is launch → type → Enter, and messaging a visible
  conversation is select → type → Enter. A flow that grows a gesture without
  buying a fact violates this; count the gestures in the shot sequence.
- **F6 Text acts like text.** Selection, copy, and word-boundary gestures work
  wherever text renders (bl-0fa0 open: double-click-drag does not extend by
  words).

### Language — L

- **L1 One role language.** User input, model response, ops rows, errors, and
  pending items are tellable apart at a glance, by one visual vocabulary
  shared across transcript, queue, and inspector (bl-3acb).
- **L2 One scale.** Type sizes, paddings, and colors come from the theme's
  roles; a surface inventing its own size or color for an existing role is a
  violation.
- **L3 Legible always.** Contrast holds on every captured shot; no text
  requires zooming the capture to read at 1:1.
- **L4 Ids are tamed.** A machine id (ancestry chain, sha, absolute path)
  never dominates a row a human scans: floor to the terminal segment or
  middle-elide, with the full value one gesture away (bl-63a1, bl-ac3d).

---

## §2 The shot-sheet

The sheet samples every rendered surface the ladder has landed, each reached
by its story rung's own gestures (keyboard or boundary line — a coordinate
only for a view, per the drive harness's steering rule). Per surface: capture
at the harness default window, one small (the smallest the surface is claimed
to support), one large (maximized). The sheet is the audit's coverage
contract: a surface skipped for want of a fixture is named as skipped, never
silently absent.

| Shot | Surface | Rung |
|---|---|---|
| Q-S0 | first-launch bootstrap composer | S0 |
| Q-S1 | populated conversation: transcript, streaming tail | S1 |
| Q-S4 | workspace tabs, conversation board, ball badges, `by ball` toggle | S4 |
| Q-S4b | Ctrl+F world search results — the §11 Search tab (bl-1ca2 retired the overlay) | S4 |
| Q-S5 | login pane; each of the three §9 config editors | S5 |
| Q-S6 | attention strip stirred + activity chip | S6 |
| Q-S7 | inspector, every tab, Raw toggles, budget fold | S7 |
| Q-S10 | step spine through the chat; a pinned notch | S10 |
| Q-S11 | work tab: `target..source` diff, numstat rows | S11 |
| Q-S12 | fork composer from a pinned notch; a ×N fan group | S12 |
| Q-S13 | balls board, four columns, spend column | S13 |
| Q-S18 | the board armed: cap, count, last/next tick, spawn/reap rows | S18 |
| Q-M | new-workspace modal; new-ball form | S4/S13 |

New rung lands → new row lands with it; this table is edited like code.

---

## §3 The audit protocol

What a dispatched model does, start to finish:

0. **Preflight.** `make drive-preflight` — it names *every* missing host
   requirement at once instead of dying ten seconds into a seat claim: the
   tools (Xvfb, xdotool, ffmpeg, python3, git, the `yog` under drive), the two
   world-seed files, and the **wall** — whether a workspace born in a scratch
   world will carry the provider rows its birth template names, asked of the
   binary under drive through an empty wall, plus a host credential to seed
   into it (DESIGN §16.2; bl-49c6). Both are **advisory** since bl-00ee retired
   the §9.2 birth gate: a workspace is born whatever its template names, so a
   missing row or sign-in costs the wire beats and nothing else. An audit that
   starts here does not discover its host one binary — or one fixture — per
   attempt.
1. **Build and isolate.** `make release`; drive the release binary in a
   scratch world (`XDG_DATA_HOME=<scratch>`) on a claimed seat
   (`make drive-seat`, i.e. `scripts/drive/yogdrive.sh seat`). Never the live
   world — the `make drive` family refuses a scratch root that overlaps
   `$XDG_DATA_HOME` in either direction, and `make ux` / `make reload` are the
   live-world verbs.
2. **Populate.** Seed with the drive harness's own verbs — `make drive-seed`
   prints a laid scratch world's path (`stories.sh seed` underneath), then the
   cheapest beats that materialize each sheet row's fixture
   (`make drive DRIVE_RUNS=run-s5s8` needs no wire). A wire-dead world still
   renders most surfaces; name any row skipped and why.
3. **Capture.** The full sheet, three sizes per row, via `yogdrive.sh shot`.
   Shots are evidence, named `<row>-<size>.png`, kept with the run. A beat-borne
   fixture also leaves `verdicts.jsonl` beside them (DESIGN §12.2): one row per
   beat, so what the fixture *established* is machine-readable rather than
   re-read out of the scroll.
4. **Score.** Every (row, criterion) pair gets pass / violation / n/a with the
   shot as witness. Judge the pixels, not the intent.
5. **File.** One ball per violation: title states the symptom, body cites the
   criterion id, the shot, and the reproduction gestures. Search
   `bl list <needle> --all` first — a recurrence of a closed ball is filed as
   a regression naming it.
6. **Record.** One log per run, **beside the run it is about**: `drive.sh`
   writes `<run>/drive-log.md` under the evidence root `$DRIVE_ROOT` (default
   `$XDG_CACHE_HOME/yog-drive`), with the shots, `gestures.jsonl` and
   `verdicts.jsonl` it quotes. It carries the build sha, host tuple, sheet
   coverage (including named skips), the scorecard table and the filed balls.
   **Start it generated, not blank** — `make drive-log` emits the sha, the host
   tuple, the load, the binary the run actually drove (read off its own verdict
   rows, not re-resolved from the PATH of whoever is generating the log — that
   PATH answers with the *installed* yog, bl-d1af) and any beat table the run
   produced; the scorecard and every judgement are written over that skeleton by
   hand. The house style is evidence quoted, not summarized — exactly the half
   no generator can supply.

   **The log does not come back into the checkout.** It used to: one file per
   run under `docs/drive-logs/`, exempt from the home-path rule because a log's
   paths were its evidence. bl-244f burned all eleven and the exemption with
   them — they were the single largest carrier of operator-home paths in this
   repository's history, and they shipped inside the 0.0.1 crate. So the log
   stays where it is written, outside the tree, and **what comes back is step
   5's output: the balls filed.** A verdict worth keeping is a claim in
   `docs/STORIES.md` citing the run by date and verb, not a file.

   **Quote the evidence, not the operator**, even so — a log gets read aloud,
   pasted into a ball, and quoted in a PR. `logskel.sh` folds `$HOME` to `~` in
   every path it emits; fold the same way in every line you add by hand, and
   mind the two other rules a quoted log trips if any of it reaches a commit or
   a ball body: `personal-email` and `quoted-dialogue` (`scripts/leak-rules.sh`
   is the table; AGENTS.md, "What may never enter a ball body", is the rule).
   An absolute path was never the evidence, the verdict was.

The audit is triage, not repair: it files, it does not fix. Fix balls are
dispatched separately and re-audited by the next run.

---

## §4 The currency rule

A verdict — a drive log's PASS table or an audit's scorecard — is a claim
about the sha it names, and about nothing newer. Main *works* and *looks
right* exactly as far as the newest log whose sha is an ancestor of main; a
merge that touches a judged surface voids that surface's rows, functional and
quality alike. The scorecard is never "the state of yog"; it is the state of
one build, and re-establishing it after the surface moves is ordinary work,
filed and dispatched like any other. Cadence is the operator's policy;
expiry is this rule.
