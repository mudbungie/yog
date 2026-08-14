+++
title = "sweep the drive beats for vacuous assertions: empty-variable and generic-string greps pass in runs where the gesture never happened"
created = 1786513798
updated = 1786683583
claimant = "Ingot"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["testing"]
+++
Filed by Alkaloid 2026-08-11 from concrete instances found by Dowel (bl-afa7, bl-2d45) and the parallel Rust-side finding in bl-bc06.

## The pattern

Two independent harnesses in this repo were asserting **vacuously** — passing green in runs where the thing under test provably never happened. Found in one session, by two agents who were not looking for it:

**Shell drive beats** (Dowel, while repairing bl-afa7 and bl-2d45):
- An unset `$minted` turned a beat's check into `grep -q '""'` — which matches anything, so two beats passed with no minted sphere at all.
- An ack watermark checked with `grep '"seen"'` passed **in the very runs where the stop never happened** — the generic key was present regardless of the gesture.

**Rust paint assertions** (Ferrule, bl-bc06): `egui::Galley::text()` returns the INPUT string, so every paint-layer assertion was blind to elision. All 1815 tests passed on the fix, proving none covered truncation. That half is tracked in **bl-36c3**; this ball is the shell-side half.

## Why these are one bug class, not two

Both are assertions whose predicate is satisfied by something other than the behaviour under test — an empty expansion, a key that is always present, a string that was never laid out. A beat like this **can never go red**, so it reads as coverage while providing none. It is worse than a missing test: a missing test is visible in a coverage report.

## Scope

Sweep `scripts/drive/` and `tests/integration/stories_*.rs` for assertions that cannot fail:

1. **Greps on interpolated variables** that are empty or unset on the failure path — quote-and-check, or assert the variable is non-empty first. `grep -q "$x"` with empty `$x` is an unconditional pass.
2. **Greps on generic keys** (`"seen"`, `"ok"`, a bare field name) present in the output regardless of outcome — assert on the identity that distinguishes this run (conversation id, agent id, the argv the gesture actually carried), which is what bl-2d45's repair does.
3. **Beats that have never failed** in the log history. Per the standing prior, a never-red beat is a suspect, not a success.

## The standard for each repair

**Prove the beat bites**: revert the fix (or run against a tree where the gesture does not happen) and show the beat goes red. Dowel's bl-afa7 repair met this — two real drives, three FAILs and two vacuous PASSes became five real PASSes. An assertion not shown to fail is not evidence.

## Relationship to other balls

- **bl-36c3** — same sweep, paint layer. Sibling, not duplicate.
- **bl-bb20** — S2 and S9-S18 have ZERO real-substrate beats. That is *absent* coverage; this ball is *fake* coverage. Do them separately; fake coverage is the more dangerous of the two because it reads as done.

Verify all cited paths against HEAD first; ball bodies drift.

---

The canonical instance, for whoever runs the sweep: an assertion can be vacuous by measuring a DIFFERENT EVENT, not merely by being loose.

`beats_s3s4s6.sh`'s S6 ack beat was `await seen_recorded "$ui"`, i.e. `grep -q '"seen"' ui.json`. Every run of that world selects a conversation at the S3 rung, ~40 beats earlier, and a selection IS an acknowledgement (§6, `focus_agent` -> `record_seen`) — so the `seen` key is in ui.json long before the stop this beat is about. The beat therefore reported PASS in the same two runs where its own predecessor reported `S6 stop: lernie stop dispatched FAIL - no stop verb`: nothing was stopped, nothing new was acknowledged, and the assertion could not tell, because it was reading a fact written by an unrelated gesture.

Reproduced live twice in this session (2026-08-12 drives at 20260812T032053Z and T033533Z) before the repair, and both are in the run scroll: FAIL and PASS adjacent, on the same evidence.

The repair shape that generalizes: assert on the IDENTITY of the thing the gesture was about, not on the presence of a shape. `seen_kind $ui $flight` (harness.sh) asks whether the watermark exists under THAT agent id; `stopped $flight` asks whether the stop argv names THAT conversation, where `verb_ge stop 1` counted rows and a stop of the wrong conversation satisfied it equally. Both predicates now live in harness.sh so the next beat reaches for them instead of a grep.

Sweep heuristic worth applying: any predicate whose subject is a bare shape (`'"seen"'`, `'"balls"'`) or an interpolated variable that can be empty is suspect — grep the beats for `grep -q` with no id in the pattern, and for `"$var"` patterns whose var is set by a `find` that can return nothing (bl-afa7's `$minted` was the empty-string half of this same class).

---

## Mechanize the heuristic (Alkaloid)

Dowel's sweep heuristic is stated in a form a machine can check, so it should not stay a habit:

- any `grep -q` whose pattern **carries no id**
- any interpolated variable set by a `find` (or any command) that **can return nothing**

Both are greppable over `scripts/drive/`. Consider landing the sweep as a **check**, not just a one-time cleanup — the repo's own discipline is that policy lives in an enforced rule (`rules/*.yml`, the clippy manifest, `make line-cap`), not in reviewer memory. A shell-side audit step invoked from `make lint` would keep this class from regrowing the moment attention moves on, and this ball's whole finding is that the class is invisible precisely because nothing looks for it.

Two design constraints if it is mechanized:

1. **It must not be defeatable by an inline exception.** Rule 5's reasoning (no `#[allow]` in prod; suppressions live in one reviewable manifest) applies equally here.
2. **A pattern with no id is not always wrong** — some beats legitimately assert on a shape. The check should demand the assertion name what distinguishes this run, or carry a one-line justification in a single reviewable place. Prefer the first.

If mechanizing turns out to cost more than it saves, say so on the ball and land the sweep alone — but make that a decision with a recorded reason, not an omission.

---

## Where to look: drift and vacuity are neighbours (Dowel, via Alkaloid)

Empirical result from repairing three stale beats (bl-d1af, bl-afa7, bl-2d45), 3 for 3:

> A red drive beat after a quiet period was the beat that drifted, three times out of three — **and each drifted beat had a vacuous neighbour keeping it company.**

That co-occurrence is a search strategy, not a coincidence. The reason is mechanical: a beat that drifts stops producing the state its *neighbours* assert on. A neighbour that keeps passing anyway is proving it never depended on that state. The red beat is the alarm; the green one beside it is the actual defect, and it is the one nobody would ever look at.

**So: when repairing any red beat, audit the beats immediately around it in the same run before closing.** Do not stop at the one that failed. Concretely, in this cluster:

- bl-afa7's `w` drift sat beside two beats whose `$minted` was empty, making `grep -q '""'` match nearly any row.
- bl-2d45's dead ↓ sat beside an ack beat reading a watermark S3's selection had written ~40 beats earlier.

Neither vacuous beat was in any ball. Both were found only because someone was already standing there.

**Result of the repair, as the standard to hold this ball's sweep to:** `run-s3s4s6` went from 4 FAIL + 2 vacuous PASS to 18/18 real PASS — the first fully green run in that cluster.

---

The limit case of this ball's pattern, for the sweep: **a beat that never RAN**.

Vacuity so far has meant an assertion that passes without measuring its subject. One rung worse is an assertion that is not evaluated at all and leaves no trace. bl-2d45's delivery gave a new function the name of an existing one in a file that is sourced into a flat namespace; bash's later-wins rule deleted three S6-T1 beats from every `run_s7`, and no verdict row, no drive log and no PASS/FAIL count could show it — the run reported ALL BEATS PASS for the beats that survived.

So the sweep needs a second question beside 'does this assertion measure its subject?': **'did every beat this ladder claims to have run actually run?'** The answer is not in `verdicts.jsonl`; it has to come from the source. Fixed structurally in bl-0e44 (`stories.sh` refuses a duplicate top-level function name), but the general lesson stands for any harness where registration is by name: absence of a failure is not evidence of a test.

---

## Sweep landed (Ingot). Six vacuous assertions repaired in scripts/drive/, every one mutation-proved; the Rust half swept and found CLEAN.

### The shell instances, and the world each one passed in

1. **beats_s3res.sh S3-T4** — 'the refused ball is still claimed'. `refused=$(grep failed-close-row | grep -o 'bl-[0-9a-f]*' | head -1)` is empty when the beat ABOVE it found no failed close row, and `ball_listed` then ran `grep -q ""` over the listing, which matches every non-empty stream. The beat passed precisely when its own predecessor failed — bl-afa7's $minted shape, verbatim, one file over.

2. **beats_s8.sh S8-T4** — 'balls' landing branch is refused, not written' was spelled `gesture … || pass "…"`. **No fail arm at all**: the one outcome it exists to catch — the unlawful branch ACCEPTED — short-circuits the `||` and emits nothing. Not a false pass but a DELETED ROW, the same blindness as bl-0e44's duplicate function name reached by a different road. It was the only such beat in the harness; a label-symmetry scan over all 69 beat labels now reports none.

3. **beats_s6.sh S6-T1** — 'mail: no watermark exists to silence it' was `! grep -q '"mail"' $ui`. A missing ui.json makes grep exit 2, the `!` turns that into PASS. So did an ack that landed on the wrong conversation, or on none. Now bracketed: `seen_kind $ui $agent && ! grep -q '"mail"' $ui` — THIS agent was acknowledged, and mail still survived it.

4. **md5of (beats_s5.sh, spent by beats_s8.sh)** returned the EMPTY STRING for a missing file, and every caller compares two of them to claim 'this file did not move'. **Two absences compared EQUAL.** S8-T1's 'the ambient world's own seed never moved' therefore passed on any host with no ambient yog, and S8-T4's ui.json hash on any run that had not written one. Now `absent:<path>`, plus an explicit existence requirement on both beats.

5. **harness.sh no_dead_step** was `! find steps -name response.json -size 0`, true of a world with NO STEPS AT ALL — so 'no step died before its first token' passed in every run where the S1 message never reached a driver. Now: some step wrote a response.json AND none is zero-byte.

6. **beats_s7.sh seen_agent** — `grep -q "\"$2\"" ui.json` — was dead code left behind when bl-2d45 moved that read to `seen_kind`. Deleted rather than kept: an unreached vacuous predicate is the next beat's first temptation.

Plus the class fix Dowel asked for: **the empty-subject guard lives in the PREDICATE, not the call site** — `stopped`, `seen_kind`, `ball_listed`, `closed_claims` and `prompt_cwd_is_worktree` all refuse an empty id now, so the trap cannot be re-armed by the next beat that copies a line.

### Proof that each bites (the ball's standard)

A mutation harness builds, per repair, the world in which the gesture NEVER HAPPENED and evaluates the old spelling against the new. Verbatim:

    ball_listed, no refused close row (id empty)          old=PASS  new=FAIL
    ball_listed, the ball really is listed                old=PASS  new=PASS
    no_dead_step, the message never reached a driver      old=PASS  new=FAIL
    no_dead_step, a driver replied                        old=PASS  new=PASS
    no_dead_step, a driver died before its first token    old=FAIL  new=FAIL
    stopped, the conversation could not be named          old=PASS  new=FAIL
    stopped, the right conversation was stopped           old=PASS  new=PASS
    S6-T1 mail, ui.json does not exist                    old=PASS  new=FAIL
    S6-T1 mail, the ack landed on the WRONG conversation  old=PASS  new=FAIL
    S6-T1 mail, acked and mail survived                   old=PASS  new=PASS
    S6-T1 mail, mail WAS seen-gated                       old=FAIL  new=FAIL
    md5of, neither file exists (ambient seed beat)        old=PASS  new=FAIL
    md5of, both exist and match                           old=PASS  new=PASS
    prompt_cwd_is_worktree, no ball id to name            old=PASS  new=FAIL

Every repair goes PASS->FAIL on the vacuous world and holds its verdict on the honest ones. The pass-only beat (#2) is structural, so it is proved by the label scan instead: asymmetric labels went 1 -> 0.

### tests/integration/stories_*.rs — swept, CLEAN, and this is a result not a null

The Rust shapes are `.all()` over a possibly-empty collection (vacuously true) and a `contains()` on a generic needle. All twelve `.all()` sites are bracketed by an `assert_eq!` on the collection's exact contents or length, or by a `.find().unwrap()`, within a few lines — s4_t7/s4_t3/s1_t3/s0_t5 by contents, s4_t2/s4_t5/s8_t3 by length, s4_t4/s4_t6/s6_t4/s5_t6 by an unwrapping find. The only `contains()` needles under 8 chars are "install", "close", "claim" and "='", each specific to its subject. No repair was warranted. (The paint-layer half stays bl-36c3's.)

### Mechanization — DECIDED, and split, not skipped

Alkaloid asked for a recorded decision. The checks belong in **bl-70b8**, whose entire subject is landing the mechanical checks for the seven shapes and which already names 'a grep -q no-id lint' among them. Landing a second, differently-shaped shell audit here would be the two-homes-for-one-fact failure the repo's own discipline forbids. What this ball contributes to that work is the two rules proved cheap and total by running them: **(a)** the pass/fail LABEL SYMMETRY scan — 69 labels, one violation, zero false positives, and it needs no allowlist because a beat with only one arm is never right; **(b)** the no-id `grep -q` lint, whose honest exceptions turned out to be exactly the fixed literal strings ('"balls"', 'yogdrive-marker-A', 'tasks_branch = …'), so the rule wants to be 'the pattern is a literal or it names an id', not 'the pattern names an id'.

### Forced structural change, recorded because it was not asked for

The repairs put harness.sh at 304 lines and beats_s5.sh at 302. `make line-cap` answers that with 'split along a real seam (DESIGN §12) — do not shave lines', so:

- **scripts/drive/wall.sh is new** (67 lines): `BOOTSTRAP_WS`, `wall_dir`, `seed_wall`, moved out of harness.sh unchanged. The seam is real and it makes harness.sh's own header true again — that file says it is 'the assertion helpers, the two waiting primitives, the per-run seat, and the verdict in BOTH its halves', and a fixture that copies a host credential into a scratch world is none of those. It was parked there only because bl-49c6 had nowhere else to put it.
- **md5of and file_has moved INTO harness.sh** from beats_s5.sh, where beats_s8.sh and beats_s6.sh were already reaching across for them. Same rule that moved `seen_kind` (bl-2d45): a predicate two runners assert on has one home. That is also why md5of's vacuity spanned two files.
- harness.sh 263, wall.sh 67, beats_s5.sh 299. DESIGN §12.2 updated: the harness row, a new wall.sh row, every line count, and the tier count 'four tiers' -> 'five tiers — a front door, a seat, a fixture, a verdict, and the story beats'. The five prose references to 'harness.sh's seed_wall' (preflight.sh, cleanroom.sh, yogdrive.sh, beats_s5.sh, STORIES.md x2) were repointed. `one_name_one_definition` now also scans wall.sh — mutation-tested: a duplicate `seed_wall` in beats_s8.sh makes stories.sh refuse.

### Doctrine

STORIES.md gains the three shapes and the both-arms corollary beside the existing `until_landed` doctrine, so the next beat author reads the rule where the rules live.

### Found, not fixed

Every drive script now sits within a handful of lines of the 300 cap (beats_s5.sh 299, stories.sh 297, beats_s3s4s6.sh 274). The next ball to touch any of them pays a split it did not budget for, as this one did. Worth a ball of its own; not filed, because the right split depends on what that ball is adding.
