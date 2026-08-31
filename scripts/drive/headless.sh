#!/bin/bash
# headless.sh — how a run boots an engine and reads what it said. Sourced by
# stories.sh; not an entry point, and it writes no verdict row of its own.
#
# It is a tier for the reason harness.sh and gesture.sh are: five sourced files
# spend `reply_is` and `row` (`beats_headless.sh`, `beats_s13.sh`,
# `beats_s19.sh`, `beats_s10.sh`, `beats_s18.sh`), so their one home is beside
# the other shared primitives rather than inside whichever run verb was written
# first. Split out of beats_headless.sh at the repo's 300-line cap (bl-7547),
# and the seam is the same one wall.sh was cut on: that file is a RUN VERB and
# a scope decision, this is the vocabulary the verb and its stages speak.
#
# It was the SEATLESS tier — the one run verb that claimed no X display while
# every other verb drove a window. Since bl-7942 a bare `yog` IS the engine and
# there is no other kind of run, so what was the exception is the whole harness.

# `yog` parks until a signal, so it is backgrounded and its pid is the run's one
# piece of teardown. The consumer is not up the instant the process is, so the
# first gesture is the one that waits — `await`, never a sleep.
boot_headless() {
  XDG_DATA_HOME="$1" yog >"$2/headless.log" 2>&1 &
  # THIS run's engine, recorded where it is booted — what every `gesture` below
  # watches alongside its reply, so an engine that dies mid-run reddens the next
  # gesture instead of waiting out its deadline (bl-5cf7).
  engine_pid=$!
  # PAIRED WITH THE BOOT: `set -e` can end this file anywhere between here and
  # the tail, and a parked engine survives to hold a scratch world nobody will
  # look at again. One aborted mutation run left exactly that behind.
  trap 'kill "$engine_pid" 2>/dev/null || true' EXIT
  await consumer_up "$1"
}
# The probe is TIMED OUT and its reply is thrown away — 2 s, its own budget,
# because this one is asked BEFORE there is an engine to watch and the answer it
# wants is "not yet". `yog gesture`'s own budget is 60 s (`multiplex.rs`), so
# without this the boot probe would spend a minute per attempt learning nothing;
# the shared `gesture` helper's deadline is `gesture.sh`'s. Its answers must also
# not land in `gestures.jsonl`, which every assertion below reads the tail of.
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

