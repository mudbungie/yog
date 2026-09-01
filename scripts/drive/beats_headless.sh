#!/bin/bash
# beats_headless.sh — the SEATLESS run verb, and the graduated rungs it can
# carry. Sourced by stories.sh; not an entry point of its own.
#
# Every other run verb in this family drives a WINDOW: it claims an X display,
# launches yog on it and presses §11 keys. This one claims nothing. `yog
# serve` is "the same engine with no window: worker, watcher, gesture
# consumer" (§8.4), so the whole run is `yog gesture` lines against a real
# world — which makes it the cheapest real-substrate verb there is: **no seat,
# no window, no wire spend, and no model call anywhere in it.** It is also the
# only run verb an operator can drive on a box with no X server at all.
#
# WHY THESE RUNGS AND NOT THE OTHERS (bl-bb20's scope decision, made against
# the ball's instruction to attack the scope before committing). A drive beat
# earns its keep only where the REAL substrate can say something a fake cannot:
# real balls state and its blocker semantics, real git, real litany's on-disk
# shapes, the real window, the real wire. Where a rung's claim is a derivation
# over yog's own structures, the in-crate test proves it exactly and a beat
# would re-run the same logic through a slower door.
#
#   S13 (board)   IN — the columns are a derivation over REAL balls state, and
#                      `blocked` is real balls' blocker resolution, not yog's.
#   S11 (auditor) IN — the diff is REAL git over the REAL worktree `bl claim`
#                      cut. The fake cannot cut a worktree.
#   S2  (director) IN — the target rung's whole subject is the cwd a spawn will
#                      run in, and the shipped binary is the only thing that
#                      composes it.
#   S14 (teleop)  PART — T8 (a windowless engine answers) is this verb's own
#                      premise, and T5 (an answer aimed at nothing refuses) is
#                      free. T1-T4 need a LIVE conversation waiting on the
#                      operator, which needs a model call; the world that
#                      already has one is `run_s3s4s6`, so the attention queue
#                      belongs there and not here.
#   S9  (settler) OUT — already driven: `cleanroom.sh` IS this rung's drive
#                      (STORIES.md says so outright), and S8-T2 already asserts
#                      the nested-clone half. T3's `--as $YOG_NAME` stamp is
#                      only ever set for an AGENT's tool subprocess (`verbs/
#                      bound.rs`), never for `yog exec` — `world/seat.rs` says
#                      the hatches carry no `YOG_NAME` on purpose — so driving
#                      it needs a real agent running a real tool: wire spend and
#                      agent nondeterminism for a fold `multiplex::bl` tests
#                      exactly.
#   S10 (historian) IN — since bl-faca. It was "blocked, not skipped: the rail,
#                      transcript, steps and files surfaces have NO headless
#                      spelling at all", and bl-6233 gave every one of them a
#                      query and a line — so the block expired and the rung is
#                      driven over the fleet's own drone (`beats_s10.sh`).
#   S12 (counterfactualist) OUT — its ×N is a read-only fork off a pinned
#                      notch, which needs a LIVE conversation with history:
#                      a model call, which this verb refuses to spend.
#   S19 (adjudicator) IN (PART) — the fan's spread, the delivery law, the
#                      staleness refusal and the retirement are real balls +
#                      real git with no model call anywhere (`beats_s19.sh`).
#                      What stays OUT is cohort membership on the read
#                      surfaces: membership is derived from real FIRE rows and
#                      a fire is a detached `litany prompt` — a model call —
#                      so that join is the in-crate tests' half, over fixture
#                      trails.
#   S15/S16/S17 (warden, releaser) OUT — a real adjudication needs a real agent
#                      really attempting a real tool, then a real decision:
#                      wire spend, nondeterminism, and the response ladder is
#                      bl-02c2, still open.
#   S18 (admiral, armed) IN — since bl-faca. It was "the armed loop is not
#                      live", and bl-66fb landed it. A whole TRAJECTORY is the
#                      part no fixture holds: a tick claims and spawns, a later
#                      one gives the claim back, a later one still does not
#                      retake it (`beats_s18.sh`). The unarmed clause is still
#                      the S13 beats below, and is now this rung's bracket.

# --- the project fixture ----------------------------------------------------
# A real git project primed INTO the world, carrying one ready ball; prints the
# ball id. The S11 diff is real git over the worktree `bl claim` cuts and the
# S13 columns are real balls' own blocker resolution, so neither rung has
# anything to read until this has run.
#
# IT LIVES HERE BECAUSE THIS FILE IS ITS ONE CALLER (bl-9397). It was defined in
# `beats_s3s4s6.sh` and shared with the windowed runs; bl-7942 deleted every
# windowed beat file and left this call standing, so the sole surviving run verb
# died at `seed_balls: command not found` before its first beat and the whole
# ladder answered with no verdicts at all. A fixture with one consumer belongs
# in it.
#
# It does NOT re-seed the world: `run_headless` lays the world seed itself, one
# line before it writes the cadence this run is timed against, and a fixture
# saying that a second time is the same fact in two places.
seed_balls() {
  data=$1 ; p="$data/proj"
  mkdir -p "$p"
  git -C "$p" init -q -b main
  git -C "$p" config user.email drive@yog.invalid
  git -C "$p" config user.name yogdrive
  : > "$p/README.md" ; git -C "$p" add -A ; git -C "$p" commit -qm init
  in_world "$data" bl prime --as yogdrive >/dev/null 2>&1
  # The body is the goal's payload verbatim (§3.3), and a fire binds the agent's
  # working directory to the ball worktree (bl-6654) — which reads as an
  # invitation to work the repo, so the body forbids tools outright: these beats
  # are about yog's surfaces, not about an agent doing a job, and every tool
  # round trip is wire spend.
  in_world "$data" bl create "drive ball" \
    --body "Respond with exactly this text and nothing else: Ball wire OK. Run no commands and no tools." \
    --as yogdrive
}

# --- the run ----------------------------------------------------------------
run_headless() {
  data=$1 ; out=$2
  mkdir -p "$out" ; rm -rf "$data" ; mkdir -p "$data"
  seed "$data"
  # THE CLOCK, SEEDED FAST (bl-faca). The §4.3 loop ticks at the watcher's FULL
  # sweep, so the armed rung's trajectory — spawn, then reap, then the tick that
  # must NOT retake — costs three of them. At the shipped 15 s that is a minute
  # of a seatless run waiting; at the floor the §9.5 control itself allows
  # (`cadence::FULL_SWEEP_BOUNDS`) it is six seconds, and the periods are what
  # this rung is timed against, never a fact it asserts.
  mkdir -p "$data/yog/world/state/yog"
  printf 'cadence:\n  watcher:\n    debounce_ms: 100\n    cheap_sweep_ms: 1000\n    full_sweep_ms: 2000\n' \
    > "$data/yog/world/state/yog/cadence.yaml"
  ops="$data/yog/world/state/yog/ops.jsonl"
  proj="$data/proj"
  # A real project with a real balls store: three balls whose STORED facts put
  # them in three different columns, and the third's blocker is a live ball, so
  # `blocked` is balls' own resolution and not a field yog could have read.
  seed_balls "$data" >/dev/null
  ready=$(in_world "$data" bl create "board ready" --body "no tools" --as yogdrive)
  claim=$(in_world "$data" bl create "board claimed" --body "no tools" --as yogdrive)
  in_world "$data" bl create "board blocked" --body "no tools" \
    --needs "$ready" --as yogdrive >/dev/null
  # S14-T8 — the premise, asserted rather than assumed: an engine with no face
  # booted, consumed a deposit and answered it. Every beat below rides on it, so
  # a boot that never answered ends the run HERE with a verdict row rather than
  # leaving every gesture below to spend its deadline on a consumer that will
  # never come.
  boot_headless "$data" "$out" || {
    fail "S14-T8 windowless engine: a deposit is consumed and answered" "no engine in 40s"
    verdict "$out" ; return 1
  }
  # `|| true` ON EVERY BARE FIRE BELOW, and it says something: in this file the
  # REPLY is the verdict — each gesture is followed by the `reply_is` that judges
  # it — so the fire's own exit is not an assertion, and under `set -eu` an
  # unguarded one ENDS THE RUN where a beat should have gone red. That was
  # academic while a gesture only ever failed by being refused; since bl-5cf7 it
  # also fails when the engine dies, which is precisely when the remaining rows
  # are worth writing: each one now reddens on its own refusal row in
  # milliseconds and the run still reaches `verdict`. Where the exit IS the
  # claim — the S14-T5 refusal below — it is spelled `if gesture …` and both
  # arms are written out.
  #
  # The wire spells NAMES, never paths (REMOTE §8, bl-f5f6): `--ws` is the
  # workspace's own name and `--project` the repo's, and the engine resolves
  # each at the dispatch chokepoint.
  ws="$BOOTSTRAP_WS"
  proj_name=$(basename "$proj")
  gesture "$data" /prepare --ws "$ws" --project "$proj_name" || true
  reply_is 'd["ok"] and d["prepared"]["workspace"]=="'"$BOOTSTRAP_WS"'" and not d["prepared"].get("binding")' \
    && pass "S14-T8 windowless engine: a deposit is consumed and answered" \
    || fail "S14-T8 windowless engine: a deposit is consumed and answered" "no prepared reply"

  # S2-T1 — the target rung, asserted at the TYPED BINDING and nowhere else.
  # The prepared reply named this `cwd` until bl-6654 landed (642b054f) and made
  # the target a typed parameter handed to litany rather than a directory folded
  # into a spawn; this beat was written against `cwd` and reddened on the merge
  # that brought it in, which is the beat working. The other half of S2-T1 — the
  # §3.3 preamble the goal carries — is deliberately NOT asserted: it is the
  # goal-prose operative channel that same ball set out to retire, and a beat
  # that pins prose a live ball is deleting is a beat filed red. The binding is
  # what the rung is for. Bracketed by the bare `/prepare` above, whose reply
  # carries no binding at all.
  gesture "$data" "/prepare dir $proj" --ws "$ws" --project "$proj_name" || true
  reply_is 'd["ok"] and d["prepared"].get("binding")=="'"$proj"'"' \
    && pass "S2-T1 path-rung: the prepared binding is the named directory" \
    || fail "S2-T1 path-rung: the prepared binding is the named directory" "binding is not the dir"
  # …and the rung's negative clause, which is why it is a rung and not a flag:
  # naming a directory is not a ball, so nothing on the balls side is spawned.
  bl_rows=$(grep -c '"bl"' "$ops" 2>/dev/null) || true
  gesture "$data" "/prepare dir $proj" --ws "$ws" --project "$proj_name" || true
  [ "$(grep -c '"bl"' "$ops" 2>/dev/null || true)" = "${bl_rows:-0}" ] \
    && pass "S2-T1 path-rung: preparing a directory spawns no bl" \
    || fail "S2-T1 path-rung: preparing a directory spawns no bl" "a bl verb fired"

  # Bind one ball to the workspace. The reply's stdout is the worktree `bl
  # claim` cut — READ back rather than recomputed, so the S11 beat below reads
  # the path the boundary itself named.
  gesture "$data" "/assign $claim" --ws "$ws" --project "$proj_name" --as "$BOOTSTRAP_WS" || true
  reply_is 'd["ok"] and d["exit"]==0' \
    && pass "S13 fixture: the ball is bound to the workspace" \
    || fail "S13 fixture: the ball is bound to the workspace" "assign refused"
  wt=$(reply_is 'True' && python3 -c "
import json,sys
line=[l for l in open('$out/gestures.jsonl').read().split('\n') if l.strip()][-1]
print(json.loads(line).get('stdout','').strip())")

  s13_board "$data" "$ready" "$claim"
  s11_workdiff "$data" "$ws" "$claim" "$wt"
  s19_adjudicator "$data" "$ws" "$claim" "$proj"
  # THE ARMED RUNG GOES LAST, and the order is load-bearing (bl-faca): from the
  # moment `/fleet` writes its entry a real loop is claiming real balls in this
  # world, so every beat above would be asserting against a board another
  # thread is moving. It ends on `/disband`, which puts the world back.
  #
  # S10 rides its drone: the loop mints the only conversation this seatless run
  # has, on a wall the fixture strips of its sign-in, so the historian's six
  # surfaces have real bytes to read and nothing was spent making them.
  s18_admiral "$data" "$ws" "$proj_name" "$claim"
  s10_historian "$data" "$ws" "$FLEET_AGENT" "$FLEET_BALL"
  # S13's write half goes after the armed rung, for the reason the armed rung
  # goes after everything else: it moves the board, and a loop still claiming
  # would race the row it files. `/disband` above has put the world back.
  s13_schedule "$data" "$ready" "$proj_name" "$ws"

  # S14-T5 — an answer aimed at nothing refuses. A refusal beat spells BOTH
  # arms: as `gesture … || pass` it could only ever emit a PASS row, and the one
  # outcome it exists to catch — a `/seen` that silently acknowledges an agent
  # that does not exist — would delete the beat instead of reddening it
  # (bl-f16e).
  if gesture "$data" /seen --ws "$ws" --agent no-such-conversation; then
    fail "S14-T5 an answer aimed at nothing refuses" "the boundary accepted it"
  else
    reply_is 'not d["ok"] and "no-such-conversation" in d["error"]' \
      && pass "S14-T5 an answer aimed at nothing refuses" \
      || fail "S14-T5 an answer aimed at nothing refuses" "refused without naming the agent"
  fi

  verdict "$out"
}
