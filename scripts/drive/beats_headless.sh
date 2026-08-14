#!/bin/bash
# beats_headless.sh — the SEATLESS run verb, and the graduated rungs it can
# carry. Sourced by stories.sh; not an entry point of its own.
#
# Every other run verb in this family drives a WINDOW: it claims an X display,
# launches yog on it and presses §11 keys. This one claims nothing. `yog
# headless` is "the same engine with no window: worker, watcher, gesture
# consumer" (§8.4), so the whole run is `yog gesture` lines against a real
# world — which makes it the cheapest real-substrate verb there is: **no seat,
# no window, no wire spend, and no model call anywhere in it.** It is also the
# only run verb an operator can drive on a box with no X server at all.
#
# WHY THESE RUNGS AND NOT THE OTHERS (bl-bb20's scope decision, made against
# the ball's instruction to attack the scope before committing). A drive beat
# earns its keep only where the REAL substrate can say something a fake cannot:
# real balls state and its blocker semantics, real git, real lernie's on-disk
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
#   S10 (historian) OUT — blocked, not skipped: the rail, transcript, steps and
#                      files surfaces have NO headless spelling at all
#                      (bl-6233), so a beat could only screenshot them, and a
#                      screenshot proves nothing about a spine.
#   S12 (counterfactualist) OUT — the N>1 fan is bl-8746, still open and
#                      blocked; a beat written against a half-landed mechanism
#                      is a beat that will be rewritten.
#   S15/S16/S17 (warden, releaser) OUT — a real adjudication needs a real agent
#                      really attempting a real tool, then a real decision:
#                      wire spend, nondeterminism, and the response ladder is
#                      bl-02c2, still open.
#   S18 (admiral, armed) OUT — the armed loop is not live. Its unarmed clause,
#                      "S18-T1 unarmed-is-today's-board", is exactly what the
#                      S13 beats below assert on the real substrate.

# --- the seatless world -----------------------------------------------------
# `yog headless` parks until a signal, so it is backgrounded and its pid is the
# run's one piece of teardown. The consumer is not up the instant the process
# is, so the first gesture is the one that waits — `await`, never a sleep.
boot_headless() {
  XDG_DATA_HOME="$1" yog headless >"$2/headless.log" 2>&1 &
  hpid=$!
  # PAIRED WITH THE BOOT, the way `verdict` is paired with `claim_seat`: `set
  # -e` can end this file anywhere between here and the tail, and a parked
  # engine survives to hold a scratch world nobody will look at again. One
  # aborted mutation run left exactly that behind.
  trap 'kill "$hpid" 2>/dev/null || true' EXIT
  await consumer_up "$1"
}
# The probe is TIMED OUT and its reply is thrown away. `yog gesture` waits for a
# consumer with no deadline of its own, so an engine that never boots would hang
# this run rather than redden it — and the probe's own answers must not land in
# `gestures.jsonl`, which every assertion below reads the tail of.
consumer_up() { timeout 2 env XDG_DATA_HOME="$1" yog gesture /workspaces >/dev/null 2>&1; }

# The LAST boundary reply, read as JSON and asked a python expression over `d`.
# One predicate for every assertion in this file, for the reason `seen_kind` is
# python and not a grep: `grep -q '"ready"'` is true of any reply mentioning the
# word anywhere, and the tail is what makes it THIS gesture's answer rather than
# some earlier beat's (bl-f16e). `$1` is the expression; `d` is the reply.
reply_is() {
  python3 - "$out/gestures.jsonl" "$1" <<'PY'
import json,sys
line=[l for l in open(sys.argv[1]).read().split("\n") if l.strip()][-1]
sys.exit(0 if eval(sys.argv[2], {"d": json.loads(line)}) else 1)
PY
}
# The board row for one ball id, as a python sub-expression — and `+[{}]` so a
# MISSING row is a false predicate rather than a traceback. `.get()` throughout
# for the same reason: an assertion must fail, not error.
row() { printf '([r for r in d["rows"] if r["id"]=="%s"]+[{}])[0]' "$1"; }

# --- the run ----------------------------------------------------------------
run_headless() {
  data=$1 ; out=$2
  mkdir -p "$out" ; rm -rf "$data" ; mkdir -p "$data"
  seed "$data"
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
  # leaving the next `yog gesture` to block on a consumer that will never come.
  boot_headless "$data" "$out" || {
    fail "S14-T8 windowless engine: a deposit is consumed and answered" "no engine in 40s"
    verdict "$out" ; return 1
  }
  ws="$data/yog/workspaces/$BOOTSTRAP_WS"
  gesture "$data" /prepare --ws "$ws" --project "$proj"
  reply_is 'd["ok"] and d["prepared"]["workspace"].endswith("/'"$BOOTSTRAP_WS"'") and not d["prepared"].get("binding")' \
    && pass "S14-T8 windowless engine: a deposit is consumed and answered" \
    || fail "S14-T8 windowless engine: a deposit is consumed and answered" "no prepared reply"

  # S2-T1 — the target rung, asserted at the TYPED BINDING and nowhere else.
  # The prepared reply named this `cwd` until bl-6654 landed (642b054f) and made
  # the target a typed parameter handed to lernie rather than a directory folded
  # into a spawn; this beat was written against `cwd` and reddened on the merge
  # that brought it in, which is the beat working. The other half of S2-T1 — the
  # §3.3 preamble the goal carries — is deliberately NOT asserted: it is the
  # goal-prose operative channel that same ball set out to retire, and a beat
  # that pins prose a live ball is deleting is a beat filed red. The binding is
  # what the rung is for. Bracketed by the bare `/prepare` above, whose reply
  # carries no binding at all.
  gesture "$data" "/prepare dir $proj" --ws "$ws" --project "$proj"
  reply_is 'd["ok"] and d["prepared"].get("binding")=="'"$proj"'"' \
    && pass "S2-T1 path-rung: the prepared binding is the named directory" \
    || fail "S2-T1 path-rung: the prepared binding is the named directory" "binding is not the dir"
  # …and the rung's negative clause, which is why it is a rung and not a flag:
  # naming a directory is not a ball, so nothing on the balls side is spawned.
  bl_rows=$(grep -c '"bl"' "$ops" 2>/dev/null) || true
  gesture "$data" "/prepare dir $proj" --ws "$ws" --project "$proj"
  [ "$(grep -c '"bl"' "$ops" 2>/dev/null || true)" = "${bl_rows:-0}" ] \
    && pass "S2-T1 path-rung: preparing a directory spawns no bl" \
    || fail "S2-T1 path-rung: preparing a directory spawns no bl" "a bl verb fired"

  # Bind one ball to the workspace. The reply's stdout is the worktree `bl
  # claim` cut — READ back rather than recomputed, so the S11 beat below reads
  # the path the boundary itself named.
  gesture "$data" "/assign $claim" --ws "$ws" --project "$proj" --as "$BOOTSTRAP_WS"
  reply_is 'd["ok"] and d["exit"]==0' \
    && pass "S13 fixture: the ball is bound to the workspace" \
    || fail "S13 fixture: the ball is bound to the workspace" "assign refused"
  wt=$(reply_is 'True' && python3 -c "
import json,sys
line=[l for l in open('$out/gestures.jsonl').read().split('\n') if l.strip()][-1]
print(json.loads(line).get('stdout','').strip())")

  s13_board "$data" "$ready" "$claim"
  s11_workdiff "$data" "$ws" "$claim" "$wt"

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

# S13 — the board is the operator's four columns over the STORED facts, and the
# real substrate is the whole point of driving it: `ready`/`claimed` come from
# balls' claimant, and `blocked` from balls' own blocker resolution against the
# LIVE set. A fake join can place a row in a column; only a real store can be
# wrong about which column the row belongs in.
s13_board() {
  data=$1 ; ready=$2 ; claim=$3
  # THE JOIN LANDS LATE, so it is awaited and not read once. `bl claim` writes
  # balls' store; yog's own derivation then has to see it before the board can
  # say which workspace holds the ball, and a single read raced that: the first
  # run of this beat passed and the next reddened with `workspace: null` on a row
  # whose `claimant` was already `home`. `/board` is a PURE READ, so re-asking it
  # is a no-op on a miss and the predicate is monotone — exactly `await`'s
  # contract (harness.sh). Every beat below then reads the reply that landed.
  board_says() { gesture "$data" /board ; reply_is "$1"; }
  # S13-T4, the join filter, BOTH halves: the bound row names its workspace and
  # the ready row names none. Either half alone is satisfied by a board that
  # never joined at all.
  await board_says "$(row "$claim")"'.get("workspace") and not '"$(row "$ready")"'.get("workspace")' \
    && pass "S13-T4 board: the join binds the claimed row only" \
    || fail "S13-T4 board: the join binds the claimed row only" "workspace on the wrong rows"
  reply_is "$(row "$ready")"'.get("column")=="ready"' \
    && pass "S13-T2 board: an unclaimed ball sits in ready" \
    || fail "S13-T2 board: an unclaimed ball sits in ready" "$ready is not ready"
  reply_is "$(row "$claim")"'.get("column")=="claimed" and '"$(row "$claim")"'.get("claimant")=="'"$BOOTSTRAP_WS"'"' \
    && pass "S13-T2 board: a bound ball sits in claimed, naming its claimant" \
    || fail "S13-T2 board: a bound ball sits in claimed, naming its claimant" "wrong column or claimant"
  # The blocker is a LIVE ball, so this row is blocked by balls' resolution of a
  # real edge — the one column no stored field carries.
  reply_is '[r for r in d["rows"] if r["column"]=="blocked"] and all(r["id"]!="'"$ready"'" for r in d["rows"] if r["column"]=="blocked")' \
    && pass "S13-T3 board: a live blocker puts its ball in blocked" \
    || fail "S13-T3 board: a live blocker puts its ball in blocked" "nothing blocked"
}


# S11-T4 — the headless work-diff, and the only rung here whose subject is git.
# A real `bl claim` cut a real worktree off `main`; a commit lands in it; the
# query is a pure git read of `target..source`. Asserted by IDENTITY at every
# field — the file's name and its ±, both refs, both oids present — because a
# reply that merely has the right SHAPE is what an empty diff also has.
s11_workdiff() {
  data=$1 ; ws=$2 ; claim=$3 ; wt=$4
  if [ -z "$wt" ] || [ ! -d "$wt" ]; then
    fail "S11-T4 work-diff: the agent's commit is the ball's diff" "no worktree to commit in"
    return 0
  fi
  printf 'the agent wrote this\n' > "$wt/agent-wrote-this.txt"
  git -C "$wt" add -A
  git -C "$wt" commit -qm "agent work" --no-verify
  gesture "$data" /work-diff --ws "$ws"
  reply_is 'd["ok"] and [r for r in d["rows"] if r["ball_id"]=="'"$claim"'"
      and r["source"]=="work/'"$claim"'" and r["target"]=="main"
      and r["source_oid"] and r["target_oid"] and r["source_oid"]!=r["target_oid"]
      and [f for f in r["files"]
           if f["path"]=="agent-wrote-this.txt" and f["added"]==1 and f["removed"]==0]]' \
    && pass "S11-T4 work-diff: the agent's commit is the ball's diff" \
    || fail "S11-T4 work-diff: the agent's commit is the ball's diff" "no matching diff row"
}
