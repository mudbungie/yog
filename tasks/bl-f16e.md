+++
title = "sweep the drive beats for vacuous assertions: empty-variable and generic-string greps pass in runs where the gesture never happened"
created = 1786513798
updated = 1786514667
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
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
