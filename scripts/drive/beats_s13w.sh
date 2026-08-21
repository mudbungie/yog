#!/bin/bash
# beats_s13w.sh — S13's WRITE half (bl-dbde): the four scheduling facts a
# seatless seat states over the wire, and the board order they produce. Sourced
# by stories.sh; not an entry point of its own.
#
# The board rung beside this one proves yog READS balls' priority, parent and
# blocker graph. This proves a teleoperator can WRITE them. Before bl-dbde the
# control boundary carried a title and a body and nothing else, so a remote
# coordinator could arm a fleet and had no way to schedule the queue it drained
# — the facts the §11 board orders on and `fleet::pilot::plan::spawn` selects by
# were readable and unwritable.
#
# ONLY THE REAL SUBSTRATE CAN ANSWER HERE. `blocked` is balls' own resolution of
# a live edge against the live set, not a stored field yog could echo back; the
# priority is what re-sorts the ready column; and the tag rides no reply at all,
# so its witness is `bl show` in the world — the store, asked directly.

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
