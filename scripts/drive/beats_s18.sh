#!/bin/bash
# beats_s18.sh — S18 Admiral's ARMED half (VISION §4.3, bl-faca), fired from
# `run_headless`. Sourced by stories.sh; not an entry point of its own.
#
# bl-bb20 ruled this rung OUT with one sentence — "the armed loop is not live" —
# and that stopped being true when bl-66fb landed it. What the in-crate tests
# prove is one pass over a fixture snapshot: `plan` picks the right move. What
# only the real substrate can say is whether a whole TRAJECTORY converges — a
# tick claims a real ball through real `bl`, mints a real conversation through
# the real start flow, a later tick gives that claim back, and a later one still
# does NOT take it again. Those are ticks of one level-triggered loop over the
# state each previous tick left, and a fixture cannot be wrong about them.
#
# **NOTHING HERE SPENDS.** The fixture takes the workspace's sign-in away before
# arming (§16.2: the wall is where a credential lives, and brazen declines at
# the wall before a request is made), so every drone this loop mints dies at its
# first model call — `{"type":"error","kind":"auth"}` in the step's own
# `response.json`, and `tokens.total` 0 on the §11 Steps surface, which
# `beats_s10.sh` asserts. A dead drone is not a compromise here: a drone that
# never answers IS the failed trajectory this rung exists to drive.
#
# The conversation this stage mints is also the S10 historian's subject, handed
# on in `FLEET_AGENT` — one fixture, both rungs, and the read surfaces are then
# read over bytes the real loop really wrote.

# The armed workspace's key in `cadence.yaml` — the workspace PATH, as
# `fleet::arming` spells it (`ui.json`'s §4.1 watermarks use the same key).
fleet_key() { printf '%s/yog/workspaces/%s' "$1" "$2"; }
# One `["yog-fleet",<verb>,<ball>,…]` row on the trail. Never a bare
# interpolation: an empty ball id would make this true of any loop row at all,
# the empty-subject trap every id-taking predicate in predicates.sh is guarded
# against (bl-f16e).
fleet_did() { [ -n "$2" ] && grep -q "\"yog-fleet\",\"$1\",\"$2\"" "$ops" 2>/dev/null; }
fleet_count() { c=$(grep -c "\"yog-fleet\",\"$1\",\"$2\"" "$ops" 2>/dev/null) || true; echo "${c:-0}"; }
# The board, re-asked. `/board` is a pure read, so re-asking is a no-op on a
# miss and every predicate below it is monotone — `await`'s contract exactly.
fleet_board() { gesture "$1" /board ; reply_is "$2"; }
# The conversation a spawn row names (`argv[3]`), read back off the trail rather
# than recomputed — the loop's own record of what it minted.
fleet_conversation() {
  python3 - "$ops" "$1" <<'PY'
import json,sys
for line in open(sys.argv[1]):
    d=json.loads(line)
    if d["argv"][:3]==["yog-fleet","spawn",sys.argv[2]]:
        print(d["argv"][3])
PY
}

# The board's top ready row in this project — the one `plan::spawn` will take,
# derived from the same list and the same order the loop reads rather than
# named here. Empty when the board offers none, which the beat above it is for.
fleet_top_ready() {
  python3 - "$out/gestures.jsonl" "$1" <<'PYTOP'
import json,sys
line=[l for l in open(sys.argv[1]).read().split("\n") if l.strip()][-1]
for r in json.loads(line).get("rows",[]):
    if r.get("column")=="ready" and r.get("project")==sys.argv[2]:
        print(r["id"]); break
PYTOP
}

# The stage. `<data> <ws> <project-name> <bound-ball>` — the bound ball is the
# one `run_headless` assigned by hand, which the fixture gives back so the cap
# governs the loop's own work and nothing else.
# The drone this stage minted, and the ball it was minted on — handed to the
# S10 historian, which reads the conversation the loop really wrote.
FLEET_AGENT="" ; FLEET_BALL=""
s18_admiral() {
  data=$1 ; ws=$2 ; proj_name=$3 ; bound=$4
  key=$(fleet_key "$data" "$ws")
  cadence="$data/yog/world/state/yog/cadence.yaml"
  # THE BRACKET, and it has to come first: every fact below is "the board grew
  # this", and a board that always carried it would satisfy them all vacuously.
  # `fleet` is absent — not empty — when nothing is armed (`reply/encode.rs`).
  gesture "$data" /board || true
  reply_is 'not d.get("fleet")' \
    && pass "S18-T1 unarmed: the board carries no loop facts" \
    || fail "S18-T1 unarmed: the board carries no loop facts" "fleet facts before arming"
  # The wall's sign-in, taken away — the whole of what keeps this stage off the
  # wire, asserted rather than trusted because everything below spawns drones.
  rm -rf "$(wall_dir "$data" "$ws")/credentials"
  [ ! -d "$(wall_dir "$data" "$ws")/credentials" ] \
    && pass "S18 fixture: the wall has no sign-in, so no drone can spend" \
    || fail "S18 fixture: the wall has no sign-in, so no drone can spend" "credentials still there"

  # THE SLOT IS THE LOOP'S OWN. The cap counts every ball this workspace holds,
  # and `run_headless` bound one by hand for S13 — so an armed cap of 1 would be
  # full before the first tick and the loop would correctly do nothing, which no
  # beat below could tell from a loop that is broken. Giving that ball back is
  # one gesture and makes the arithmetic below the loop's alone.
  gesture "$data" "/release $bound" --ws "$ws" --project "$proj_name" --as "$ws" || true
  reply_is 'd["ok"] and d["exit"]==0' \
    && pass "S18 fixture: the hand-bound ball is given back" \
    || fail "S18 fixture: the hand-bound ball is given back" "release refused"
  # The ball the loop is ABOUT to take, read off the board in the order the loop
  # itself reads it — so "the top ready ball" is derived per run and is never a
  # ball this script picked.
  await fleet_board "$data" '[r for r in d["rows"] if r["column"]=="ready"]' \
    && pass "S18 fixture: the board offers ready work" \
    || fail "S18 fixture: the board offers ready work" "nothing ready to take"
  ball=$(fleet_top_ready "$proj_name") ; FLEET_BALL=$ball

  # ARMING IS A GESTURE (V4 item 2) and its whole configuration is one
  # `cadence.yaml` entry. Both halves: the reply, and the file that now says so.
  gesture "$data" "/fleet 1" --ws "$ws" --project "$proj_name" || true
  { reply_is 'd["ok"] and d["armed"]' && grep -q "^  $key:" "$cadence"; } \
    && pass "S18-T2 arm: one gesture writes the workspace's fleet entry" \
    || fail "S18-T2 arm: one gesture writes the workspace's fleet entry" "no entry under fleet:"

  # A TICK CLAIMS AND SPAWNS. `await` because the tick is the world's clock and
  # not this script's, and a trail row never unwrites. The row names the ball,
  # so a loop that spawned on the wrong one reddens here rather than passing on
  # a count.
  await fleet_did spawn "$ball" \
    && pass "S18-T3 spawn: a tick takes the top ready ball" \
    || fail "S18-T3 spawn: a tick takes the top ready ball" "no spawn row for $ball"
  FLEET_AGENT=$(fleet_conversation "$ball")
  # …and the loop RENDERS AS FACTS: the cap it was armed at, the count it is
  # holding, and the tick period — off the same board every other seat reads.
  # `room` false is the cap comparison spelled once.
  await fleet_board "$data" '[f for f in d.get("fleet",[]) if f["cap"]==1
      and f["count"]==1 and not f["room"] and f["tick_secs"]>0]' \
    && pass "S18-T3 facts: cap, count and tick render on the board" \
    || fail "S18-T3 facts: cap, count and tick render on the board" "no full drone on the facts"
  # The claim is REAL BALLS STATE in the workspace's own name — the half a
  # loop that only wrote its own row would fail.
  r=$(row "$ball")
  reply_is "($r.get(\"column\")==\"claimed\" and $r.get(\"claimant\")==\"$ws\")" \
    && pass "S18-T3 spawn: the claim is balls state, in the workspace's name" \
    || fail "S18-T3 spawn: the claim is balls state, in the workspace's name" "ball not claimed here"

  s18_trajectory "$data" "$ws" "$ball" "$key" "$cadence"
}

# THE FAILED TRAJECTORY, the half no fixture can hold: the drone this loop
# minted will never answer, so the claim it holds has to come back — and the
# ball it came back from must not be taken again (bl-3988's given-back law;
# without it a loop with one ready ball and a dying drone repeats that ball
# forever, which is the wedge this rung is for).
#
# `lease_min: 0` is the operator's own knob at its floor: absence means never
# reap, and any number means "quiet longer than this". Zero makes the very next
# tick the deadline, so a trajectory that would otherwise take the operator's
# thirty minutes takes one tick — the same comparison, not a different one.
s18_trajectory() {
  data=$1 ; ws=$2 ; ball=$3 ; key=$4 ; cadence=$5
  python3 - "$cadence" "$key" <<'PY'
import sys
path, key = sys.argv[1], sys.argv[2]
out = []
for line in open(path).read().split("\n"):
    out.append(line)
    if line == f"  {key}:":
        out.append("    lease_min: 0")
open(path, "w").write("\n".join(out))
PY
  grep -q 'lease_min: 0' "$cadence" \
    && pass "S18 fixture: the armed entry carries a lease" \
    || fail "S18 fixture: the armed entry carries a lease" "no lease_min written"
  await fleet_did reap "$ball" \
    && pass "S18-T4 reap: a quiet drone's claim comes back" \
    || fail "S18-T4 reap: a quiet drone's claim comes back" "no reap row for $ball"
  # The reason is the COMPARISON itself and never a diagnosis (§4.3, verbatim).
  # The row's stdout is where the loop wrote it.
  grep -q '"stdout":"lease expired' \
    <<<"$(grep "\"yog-fleet\",\"reap\",\"$ball\"" "$ops" 2>/dev/null)" \
    && pass "S18-T4 reap: the reason on the trail is the comparison" \
    || fail "S18-T4 reap: the reason on the trail is the comparison" "no comparison in the row"
  # The release is real: the board's own row is ready again and the loop has
  # room. Both, because either alone is true of a board that never re-derived.
  r=$(row "$ball")
  await fleet_board "$data" "($r.get(\"column\")==\"ready\"
      and [f for f in d.get(\"fleet\",[]) if f[\"count\"]==0 and f[\"room\"]])" \
    && pass "S18-T4 reap: the ball is ready again and the slot is free" \
    || fail "S18-T4 reap: the ball is ready again and the slot is free" "still held"
  # AND IT DOES NOT TAKE IT BACK. A negative, so it is bracketed by the two
  # positives above and given real time to be wrong in: several tick periods of
  # a loop with room, a ready ball in its project, and one act of its own
  # saying it gave that ball back.
  spawns=$(fleet_count spawn "$ball")
  sleep 8
  [ "$(fleet_count spawn "$ball")" = "$spawns" ] \
    && pass "S18-T4 no-repeat: a ball the loop gave back is not retaken" \
    || fail "S18-T4 no-repeat: a ball the loop gave back is not retaken" "spawned again"

  # SEVERABILITY, the same shape §16.3's marks knob has: disarming deletes the
  # entry, and the facts go with it — back to the frame this stage opened on.
  gesture "$data" /disband --ws "$ws" || true
  { reply_is 'd["ok"]' && ! grep -q "^  $key:" "$cadence"; } \
    && pass "S18-T5 disband: the entry is gone, not emptied" \
    || fail "S18-T5 disband: the entry is gone, not emptied" "entry survived"
  await fleet_board "$data" 'not d.get("fleet")' \
    && pass "S18-T5 disband: the board carries no loop facts again" \
    || fail "S18-T5 disband: the board carries no loop facts again" "facts survived"
}
