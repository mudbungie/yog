#!/bin/bash
# harness.sh — the tier every story run shares, sourced by stories.sh: the two
# waiting primitives, the one-name-one-definition guard, the per-run seat, and
# the verdict in BOTH its halves — the human PASS/FAIL line, and the
# machine-keyable row written beside it. It is the MECHANISM half; the
# assertion helpers those wait on are `predicates.sh`, sourced below.
#
# It is a SEAM, not a shard. stories.sh carries the S0/S1 beats and the STEERING
# doctrine that aims them; nothing below is about S0/S1 — every `run_*` verb in
# every sourced `beats_*.sh` calls `pass`/`fail`/`await`/`until_landed`/
# `claim_seat`/`verdict`, so their one home is here rather than in whichever
# runner happened to be written first. The cut was forced and is the right one:
# stories.sh sat at the repo's 300-line cap, so the verdict could not grow its
# second half where it lived (bl-56d5).
#
# Sourced, never executed. It reads two variables from its caller — `$out`, the
# run's evidence directory, and `$ops`, the world's ops.jsonl — and it defines
# no CLI verb of its own.

# The READ tier — every true/false question a beat asks of the world on disk
# (`verb_count`, `row_ok`, `seen_kind`, `stopped`, `md5of`, …). Its own file
# (bl-7547): this one is how a beat waits, what a verdict is and where the seat
# lives; that one is the vocabulary those wait on.
fails=0
. "$(dirname "${BASH_SOURCE[0]}")/predicates.sh"
# The §16.2 wall fixture — `BOOTSTRAP_WS`, `wall_dir`, `seed_wall` — is the one
# tier that LAYS state instead of reading it, so it is its own file (bl-f16e).
# Sourced here rather than from stories.sh because everything else this file
# defines is reached the same way, and `beats_s5.sh` spends `wall_dir` too.
. "$(dirname "${BASH_SOURCE[0]}")/wall.sh"
# The §8.5 boundary transport and the engine it is aimed at — `launch_engine`,
# `engine_alive`, `gesture` (bl-5cf7). Reached the same way and for the same
# reason: `gesture` is a WAITING PRIMITIVE like the two above it, spent by four
# beats files, and its deadline watches a fact (which yog is up) that every run
# verb records at its own launch.
. "$(dirname "${BASH_SOURCE[0]}")/gesture.sh"

# --- the verdict, machine-keyable (bl-56d5) ---------------------------------
# Every PASS/FAIL line also lands as ONE JSONL row in `$out/verdicts.jsonl`,
# beside the `gestures.jsonl` the boundary transport writes. The printf lines
# stay exactly as they were: a human reads the run scroll, a tool reads the
# rows, and neither is a summary of the other because both are emitted from the
# same call. Before this, a run's whole verdict was ~50 printf lines plus one
# exit code — nothing a report generator, a re-baseline diff or the QUALITY §3
# scorecard could key on without re-parsing prose.
#
# The LABEL stays the single source of the beat's name; `beat` is only that
# label slugged, so there is no second name to drift out of step with the line
# the operator reads. A consumer that needs exactness reads `label`, which is
# verbatim; `beat` is the convenience key (two labels differing only in
# punctuation would slug alike — none do, and `label` is the tiebreak).
#
# `evidence` is the newest screenshot in the run directory at the instant the
# verdict was taken — the frame the beat had driven to when it was judged. That
# is a measurement, not a claim: a beat that shot nothing inherits the previous
# frame, which is why the field names a file rather than asserting it proves
# anything. The shots stay what STORIES.md says they are, visual confirmation.
beat_id() {
  printf '%s' "$1" | tr 'A-Z' 'a-z' \
    | sed -e 's/[^a-z0-9][^a-z0-9]*/-/g' -e 's/^-//' -e 's/-$//'
}
json_str() { printf '%s' "$1" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g'; }
newest_shot() { ls -t "$out"/*.png 2>/dev/null | head -1; }

# `bin` is the BINARY THIS RUN DROVE, resolved inside the run and carried on
# every row. yogdrive.sh launches plain `yog` from PATH and `drive.sh ladder`
# prefixes this checkout's `target/release` onto it, so the run's own PATH is
# the only place the answer exists — a reader that re-asks `command -v yog`
# afterwards resolves the operator's INSTALLED yog, a different sha, into the
# one field a drive log exists to pin (bl-d1af; QUALITY.md §4, "a verdict is a
# claim about the sha it names"). Resolved once and memoized: PATH does not
# move mid-run, and an unresolvable `yog` is left EMPTY rather than filled with
# a plausible path, so logskel.sh can say so out loud.
drive_bin=""
driven_binary() {
  [ -n "$drive_bin" ] || drive_bin=$(command -v yog 2>/dev/null) || drive_bin=""
  printf '%s' "$drive_bin"
}
record() {
  [ -n "${out:-}" ] && [ -d "$out" ] || return 0
  printf '{"run":"%s","beat":"%s","label":"%s","verdict":"%s","detail":"%s","evidence":"%s","bin":"%s","at":"%s"}\n' \
    "$(json_str "${drive_run:-unknown}")" "$(beat_id "$1")" "$(json_str "$1")" \
    "$2" "$(json_str "${3:-}")" "$(json_str "$(newest_shot)")" \
    "$(json_str "$(driven_binary)")" \
    "$(date -u +%Y-%m-%dT%H:%M:%SZ)" >>"$out/verdicts.jsonl"
}

say() { printf '%-46s %s\n' "$1" "$2"; }
pass() { say "$1" "PASS"; record "$1" PASS ""; }
fail() { say "$1" "FAIL — $2"; fails=$((fails + 1)); record "$1" FAIL "$2"; }

# EVERY assertion that waits on the substrate waits HERE, not on a sleep: poll a
# predicate for up to ~40 s. A fixed sleep is the wrong wait for work whose
# latency belongs to the model or to a spawned tool, and it fails in both
# directions — 2026-07-26 alone, an 8 s guess went red against an agent that
# spent four tool round trips before answering, and a 5 s one went red against a
# `lernie stop` whose row landed the moment it finished reaping a *streaming*
# driver. Sleeps that remain gate only screenshots, never a verdict.
await() { for _ in $(seq 1 40); do "$@" && return 0; sleep 1; done; return 1; }

# The same rule for the *input* side: a gesture is only known to have landed when
# the substrate says so. `until_landed <gesture-fn> <predicate...>` fires the
# gesture, waits ~8 s for the predicate, and re-fires up to five times. A blind
# click at a fixed moment is the fixed-sleep gamble moved from time into space —
# yog paints the balls section only after its first project scan and a relaunched
# window only after its first frame, so under load (this runner was hardened at a
# load average of 80) the target is not there yet and the click lands on blank
# panel, silently. Every gesture passed here is a no-op when it misses, and the
# predicate is re-read before each retry, so a landed gesture never re-fires.
#
# TWO REQUIREMENTS ON THE ARGUMENTS, and the second one cost a whole re-baseline
# to learn (bl-0e44). (1) The GESTURE must be a no-op when it misses — the line
# above. (2) The PREDICATE must be MONOTONE: once true it stays true, so `>=` and
# set-membership, never `=`. A retry is not free of consequence for a gesture
# that is not idempotent — `phantom` starts a whole conversation each time — so
# an equality on a quantity the gesture ADDS to is destroyed by its own retry
# loop, and the beat then fails five times about a gesture that succeeded five
# times. `verb_ge`, `agents_ge`, `row_ok`, `stopped` and `seen_kind` are all
# monotone by construction; there is no equality predicate in this file, and a
# beat that wants exactness asserts it OUTSIDE this loop, where nothing re-fires.
until_landed() {
  g=$1 ; shift
  for _ in 1 2 3 4 5; do
    "$g"
    for _ in 1 2 3 4 5 6 7 8; do "$@" && return 0; sleep 1; done
  done
  return 1
}

# --- one name, one definition ----------------------------------------------
# Every `beats_*.sh` is sourced into ONE flat namespace, and bash lets a later
# `f() { … }` silently replace an earlier one. A second `s6_attention` added to
# `beats_s6.sh` therefore DELETED the S6-T1 stage `run_s7` calls: three beats
# stopped running, and nothing in the verdict could say so, because a beat that
# never ran leaves no row and the run reported ALL BEATS PASS for what was left
# (bl-0e44). So the check is STRUCTURAL and fires before any verb runs —
# stories.sh calls it at the one instant the namespace is complete, right after
# the last source. Column-anchored on purpose: a nested helper (`stop_it`,
# `phantom`, `bare_start`) is indented and local to its beat, and those names
# repeat legitimately. `$1` is the script directory.
one_name_one_definition() {
  dupes=$(grep -h '^[a-z0-9_]*() {' "$1"/harness.sh "$1"/predicates.sh \
      "$1"/wall.sh "$1"/gesture.sh "$1"/headless.sh \
      "$1"/stories.sh "$1"/beats_*.sh \
    | sed 's/() {.*//' | sort | uniq -d)
  [ -z "$dupes" ] && return 0
  echo "drive: one beat function, two definitions — the later silently wins," >&2
  echo "  so whatever the earlier one asserted stops running and leaves no trace:" >&2
  printf '    %s\n' $dupes >&2
  exit 1
}

# --- the seat: claimed per RUN, torn down with the verdict ------------------
# Every run opens with `claim_seat` and closes with `verdict "$out"` — the ONLY
# two places a display is named. A hardcoded seat was a singleton: two drives at
# once stole each other's window focus mid-typing and payloads landed
# doubled/truncated (bl-4132). `YOG_SEAT` is exported here, so every `$drive`
# call and every sourced beats_* file inherits this run's display.
claim_seat() { YOG_SEAT=$("$drive" seat); export YOG_SEAT; echo "seat: $YOG_SEAT"; }
# The one tail every run shares: drop the seat, then the verdict. Pairing them
# means a run cannot report PASS and leak its X server.
verdict() {
  # A seatless run is not a special case, it is this one with an empty seat
  # (bl-bb20's `run_headless` claims no display at all, and `yogdrive.sh`
  # refuses outright without `YOG_SEAT`). Nothing else in the tail moves.
  [ -z "${YOG_SEAT:-}" ] || "$drive" unseat
  echo "---"
  echo "screenshots: $1"
  echo "verdicts:    $1/verdicts.jsonl"
  [ "$fails" = 0 ] && echo "ALL BEATS PASS" || { echo "$fails BEAT(S) FAILED"; return 1; }
}
