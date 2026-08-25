#!/bin/bash
# beats_s13.sh — **S13 Boardwalker, both halves**, fired from `run_headless`
# (beats_headless.sh): the READ half early, before the armed rung starts moving
# the board, and the WRITE half last, after `/disband` has put the world back.
# Sourced by stories.sh; not an entry point of its own. It was `beats_s13w.sh`,
# the write half alone, until bl-7547 brought the read half here from inside
# the run verb — one file per rung, which is what every other stage tier in
# this directory already is.
#
# The READ half (`s13_board`, bl-bb20) proves yog READS balls' priority, parent
# and blocker graph. The WRITE half (`s13_schedule`, bl-dbde) proves a
# teleoperator can WRITE them. Before bl-dbde the
# control boundary carried a title and a body and nothing else, so a remote
# coordinator could arm a fleet and had no way to schedule the queue it drained
# — the facts the §11 board orders on and `fleet::pilot::plan::spawn` selects by
# were readable and unwritable.
#
# ONLY THE REAL SUBSTRATE CAN ANSWER HERE. `blocked` is balls' own resolution of
# a live edge against the live set, not a stored field yog could echo back; the
# priority is what re-sorts the ready column; and the tag rides no reply at all,
# so its witness is `bl show` in the world — the store, asked directly.

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

# The board's own row predicate under `await`: `/board` is a pure read, so
# re-asking on a miss is a no-op and the wait is monotone (harness.sh).
sched_says() { gesture "$1" /board ; reply_is "$2"; }

s13_schedule() {
  data=$1 ; ready=$2 ; proj_name=$3 ; ws=$4
  # ONE gesture carrying all four facts — priority, a tag, the parent pointer
  # and a blocker on a live ball. The reply's stdout is `bl create`'s new id,
  # read back rather than guessed.
  gesture "$data" \
    "/create scheduled by the wire --priority 7 --tag hot --parent $ready --needs $ready" \
    --ws "$ws" --project "$proj_name" --as "$BOOTSTRAP_WS" || true
  if reply_is 'd["ok"] and d["exit"]==0'; then
    pass "S13-T9 schedule: one wire gesture states priority, tag, parent and a blocker"
  else
    fail "S13-T9 schedule: one wire gesture states priority, tag, parent and a blocker" \
      "create refused"
    return 0
  fi
  new=$(python3 -c "
import json
line=[l for l in open('$out/gestures.jsonl').read().split('\n') if l.strip()][-1]
print(json.loads(line).get('stdout','').strip())")
  if [ -z "$new" ]; then
    fail "S13-T9 schedule: one wire gesture states priority, tag, parent and a blocker" \
      "no id on stdout"
    return 0
  fi
  # The board is the observation: the priority is the number that was typed, the
  # parent is the pointer that was named, and the column is balls' verdict on
  # the edge — a ball blocked by a ball that is still open.
  await sched_says "$data" \
    '('"$(row "$new")"'.get("priority")==7 and '"$(row "$new")"'.get("parent")=="'"$ready"'"
      and '"$(row "$new")"'.get("column")=="blocked")' \
    && pass "S13-T9 schedule: the board reads back the priority, the parent and the block" \
    || fail "S13-T9 schedule: the board reads back the priority, the parent and the block" \
      "the row is not scheduled as stated"
  # The tag reaches no reply, so the store is the witness.
  if in_world "$data" bl show "$new" | grep -E '^ *tags ' | grep -q 'hot'; then
    pass "S13-T9 schedule: the tag lands in balls' own store"
  else
    fail "S13-T9 schedule: the tag lands in balls' own store" "no tag on the ball"
  fi
  # The clearing half, which is the other direction of the same four facts: a
  # remote seat that can only ever ADD a blocker cannot fix a mis-wired one.
  gesture "$data" "/update $new --no-priority --no-parent --no-needs $ready" \
    --ws "$ws" --project "$proj_name" --as "$BOOTSTRAP_WS" || true
  await sched_says "$data" \
    '('"$(row "$new")"'.get("priority")!=7 and not '"$(row "$new")"'.get("parent")
      and '"$(row "$new")"'.get("column")=="ready")' \
    && pass "S13-T9 schedule: clearing the three unblocks the ball and drops its rank" \
    || fail "S13-T9 schedule: clearing the three unblocks the ball and drops its rank" \
      "the row is still scheduled"
}
