#!/bin/bash
# gesture.sh — the ENGINE UNDER DRIVE and the §8.5 boundary gestures aimed at
# it. Sourced by harness.sh (beside wall.sh, and for the same reason: it is a
# tier of its own, not a shard of the assertion helpers). It defines no beat and
# no verb — one launch wrapper, one liveness predicate, one transport.
#
# THE CONTROL BOUNDARY (DESIGN §8.5). One gesture across the boundary, in the
# line spelling: the same `Gesture` the window's click-glue constructs,
# deposited into the running yog's gestures inbox and run by its own consumer
# thread — one surface, one dispatch, no second implementation. It needs NO
# SEAT: a boundary gesture is named, not aimed, so there is no display, no
# window id and nothing to measure; and the terminal holds no selection, so a
# line states its targets outright (`--ws / --agent / --project / --as`), which
# is what those flags are for.
#
# It is also its OWN RECEIPT — `yog gesture` waits for the reply file and exits
# on its verdict — which is why no `until_landed` wraps one: that primitive
# exists because a click has no reply and may have hit blank panel, and a
# deposit cannot miss. The reply JSON lands beside the screenshots as the audit
# half; the SCREENSHOT after is the other half, and proves what it never could —
# that the window converged on a gesture it did not itself fire.

# --- the engine under drive -------------------------------------------------
# WHICH yog this run's gestures are aimed at, recorded where it is launched and
# nowhere else (bl-5cf7). Every run verb launches exactly one engine at a time —
# `launch_engine` for a windowed one, `boot_headless` for the seatless one — and
# each sets this on the way through, so there is no second place a run says which
# process it is driving and no way to launch without saying it. `engine_wid` is
# the same launch's window (empty for the headless engine, which has none).
engine_pid=""
engine_wid=""
launch_engine() { read -r engine_pid engine_wid < <("$drive" launch "$1"); }
# An EMPTY pid is not "no claim", it is a refusal: a gesture with no engine
# recorded is a gesture nothing can answer, and reporting that in milliseconds
# beats waiting out a deadline to say the same thing (the empty-subject
# discipline every predicate in predicates.sh holds, bl-f16e).
engine_alive() { [ -n "$engine_pid" ] && kill -0 "$engine_pid" 2>/dev/null; }

# --- the transport, deadlined ------------------------------------------------
# `gesture <data-root> <gesture...>` — fire one line across the boundary and
# append its reply to `$out/gestures.jsonl`, which every assertion here reads
# the tail of. Exit is the reply's verdict, exactly as `yog gesture`'s is.
#
# EVERY WAIT HAS A DEADLINE AND EXITS EARLY ON THE TERMINAL STATE (bl-5cf7).
# Two things bound this call, and each answers what the other cannot:
#
#   1. `timeout GESTURE_DEADLINE` — the HARD wall, and it is the tool's rather
#      than this loop's on purpose: whatever the watch below does or fails to
#      do, the call cannot outlive it, so a bug in a shell poll can never
#      restore the unbounded wait this ball is about. The budget sits inside
#      `yog gesture`'s own (60 s — `multiplex.rs`'s `WAITS`×`POLL`, whose
#      timeout note goes to STDERR and would land in `gestures.jsonl` as a line
#      that is not JSON), so ours fires first, the audit file stays one JSON
#      object per line, and the drive owns the verdict. If that budget ever
#      moves, this stays under it.
#   2. THE ENGINE'S OWN LIVENESS, watched alongside the reply — the wall that
#      fires in milliseconds instead of seconds. A yog that dies mid-run cannot
#      answer any deposit, ever, so spending a whole deadline on each of a run's
#      remaining gestures is time spent proving nothing: a dead engine is an
#      immediate red. That is what bounds the case this was written for —
#      `await board_says` (beats_s13.sh) wraps a gesture in a 40-iteration
#      poll, so at `yog gesture`'s own 60 s ONE beat cost forty minutes of a
#      dead engine's silence before it could go red.
#
# A gesture that hits either wall APPENDS ITS OWN REFUSAL ROW and returns 124
# (the shell's timeout convention, and `yog gesture`'s own). The row is not
# decoration: `reply_is` reads the LAST line of `gestures.jsonl`, so an
# abandoned gesture that wrote nothing would leave the previous beat's reply
# standing as this beat's answer — the stale-tail vacuity `seen_kind` and
# `md5of` were each hardened against (bl-f16e), reached by another road. It is
# spelled as the boundary's own refusal shape (`ok:false` + `error`) because
# that is what every assertion in this harness already reads.
#
# The two walls are told apart by the child's status and nothing else: `timeout`
# exits 124 when it fires and 143 when the watch below TERMs it, and neither is
# a verdict `yog gesture` itself can return (0 ok, 1 not-ok, 2 never-deposited).
GESTURE_DEADLINE=${GESTURE_DEADLINE:-45}
gesture() {
  d=$1 ; shift
  timeout "$GESTURE_DEADLINE" env XDG_DATA_HOME="$d" yog gesture "$@" \
    >>"$out/gestures.jsonl" 2>&1 &
  g=$! ; rc=0
  # The watch is a JOB BESIDE the gesture, not a poll around it: this shell
  # `wait`s on the gesture itself, so a landed reply returns the instant it
  # lands, and the only thing polled is the engine. A loop that polled the
  # CHILD instead had to decide when the child was finished, and every wrong
  # answer to that question is a wait it then blocks out in full.
  ( while engine_alive; do sleep 0.5; done; kill "$g" 2>/dev/null ) &
  w=$!
  wait "$g" || rc=$?
  kill "$w" 2>/dev/null || true ; wait "$w" 2>/dev/null || true
  case $rc in
  124) why="no reply in ${GESTURE_DEADLINE}s" ;;
  143) why="the engine (pid ${engine_pid:-unrecorded}) is gone" ;;
  *)   return "$rc" ;;
  esac
  printf '{"ok":false,"error":"drive: gesture abandoned — %s"}\n' "$why" >>"$out/gestures.jsonl"
  echo "drive: gesture abandoned — $why" >&2
  return 124
}
