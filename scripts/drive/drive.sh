#!/bin/bash
# drive.sh — the one front door to the real-substrate harness (bl-56d5).
#
# The scripts beside it are primitives with positional arguments and no defaults:
# to drive the ladder you had to know that each run verb wants its OWN scratch
# world, that the driven `yog` resolves from `PATH` so a worktree build must be
# prefixed onto it, and where to put the evidence. That is three facts to get
# right before the first beat, and getting any of them wrong is silent — which is why the ladder went undriven for eleven
# days while 177 commits landed. This file knows all four, so the Makefile's
# `drive` family is a one-line wrapper over it and a fresh agent needs one
# command per script family.
#
# It WRAPS, never replaces: `stories.sh <verb> <data> <out>` and
# `cleanroom.sh <bin> <root> <out> [verb]` keep working exactly as they were.
#
# Usage:
#   drive.sh preflight              name every missing host prerequisite at once
#   drive.sh seed [dir]             lay a scratch world, print its path
#   drive.sh ladder [verb ...]      run the ladder (default: every run verb)
#   drive.sh cleanroom [verb]       the §16.7 W14 batteries proof, in the room
#   drive.sh log [dir]              re-emit a run's log skeleton (default: newest)
#
# Evidence root: $DRIVE_ROOT (default `$XDG_CACHE_HOME/yog-drive`, outside the
# checkout — see below), one stamped directory per invocation, one subdirectory
# per run verb.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../.." && pwd)
tdir=${CARGO_TARGET_DIR:-target}
case $tdir in /*) : ;; *) tdir="$repo/$tdir" ;; esac
# The evidence root lives OUTSIDE the checkout, deliberately. A scratch world is
# not inert data: it holds `git init` project fixtures and a whole nested balls
# delivery territory that MIRRORS the project's own path. Nested inside the
# repo, that mirror reproduces the checkout's path underneath itself — and a
# fixture reaching for a delivery worktree by name then walks `git` up out of the
# scratch tree and into the real repo. That is not hypothetical: the first drive
# through this file committed an agent's entire staged worktree onto its own
# branch as "yogdrive work" (fixed at the fixture too, in beats_s3res.sh — the
# two together are belt and braces, and the class deserves both).
root=${DRIVE_ROOT:-${XDG_CACHE_HOME:-$HOME/.cache}/yog-drive}
live=${XDG_DATA_HOME:-$HOME/.local/share}

# --- never the live world ---------------------------------------------------
# Every run verb opens with `rm -rf "$data"` — that is what makes a run
# repeatable, and it is also why a scratch root that overlapped the operator's
# own `$XDG_DATA_HOME` would delete their workspaces, conversations and world
# without a prompt. The check is a two-directional path-prefix test rather than
# an equality test, because containment either way is the same accident.
# The live world is the operator's engine's (bl-6260);
# nothing in this family ever is.
refuse() {
  echo "drive.sh: refusing to drive the LIVE world." >&2
  echo "  scratch: $1" >&2
  echo "  live:    $live" >&2
  echo "A run wipes its world before it starts, so the two may never overlap." >&2
  echo "Point DRIVE_ROOT somewhere else; the live world is the engine's." >&2
  exit 1
}
guard() {
  case "$live/" in "$1"/*) refuse "$1" ;; esac
  case "$1/" in "$live"/*) refuse "$1" ;; esac
}

# The build under drive. yogdrive.sh launches `yog` from PATH, so the drive
# proves the binary in hand only if this checkout's release build is ahead of
# whatever is installed — the prefix IS the proof, and its absence is loud.
use_release() {
  [ -x "$tdir/release/yog" ] || {
    echo "drive.sh: no release binary at $tdir/release/yog." >&2
    echo "  build it first: make release   (make drive does this for you)" >&2
    exit 1
  }
  PATH="$tdir/release:$PATH" ; export PATH
}

stamp() { date -u +%Y%m%dT%H%M%SZ; }

# EVERY STAGE THIS FILE DRIVES LEAVES A ROW, whatever happened to it — verb and
# exit code, appended before the next stage starts (bl-d0a0). A stage that dies
# before its first beat writes no verdict, so until this it left no trace at all:
# an engine that never came up, a duplicate beat definition refused at source
# time, a preflight tool that answered and then quit. The report reads these rows, so
# "which stage failed" is in the document rather than in a scrolled-past
# terminal. It is not a second copy of the verdicts — it is the only record of a
# run that produced none, and the only record of the process's own exit code.
stage_row() { printf '%s\t%s\n' "$2" "$3" >>"$1/stages.tsv"; }

# The skeleton is emitted from the run's own verdict rows, so it exists whether
# the ladder was green or red — a red run is exactly when a report gets written.
#
# **And a report's own failure may never replace the run's** (bl-d0a0). This was
# a bare command under `set -e`: the generator refused a root with no
# `verdicts.jsonl`, so a run that died before its first beat took drive.sh out
# HERE — leaving a zero-byte `drive-log.md`, swallowing the "N of the driven
# verbs reported failing beats" line, and ending on the generator's complaint
# instead of on the failure that actually happened. The generator now
# reports a verdict-less run instead of refusing it, and this guard keeps the
# primary failure and the exit code below whatever else it does. (The run that
# found this had died claiming an X seat; there is no seat to fail at now, but
# an engine that will not boot fails in exactly the same place.)
skeleton() {
  "$here/logskel.sh" "$1" >"$1/drive-log.md" \
    || echo "drive.sh: the log skeleton failed; the run's own verdict stands" >&2
  echo "log skeleton: $1/drive-log.md"
  echo "  hand-finish it IN PLACE (QUALITY.md §3 step 6): the log stays with its"
  echo "  evidence, outside the checkout — what comes back is the balls it files"
}

# --- the verbs --------------------------------------------------------------
seed_world() {
  d=${1:-$root/seed-$(stamp)}
  guard "$d"
  mkdir -p "$d"
  "$here/stories.sh" seed "$d"
  echo "$d"
}

ladder() {
  runs=${*:-}
  # One verb since bl-7942: every windowed run went with the window, and what
  # is left claims no display and spends nothing on the wire. The list stays a
  # list rather than collapsing to a call, because the ladder's shape — one
  # world per verb, one verdict row per verb — is what a second verb would slot
  # into, and this file should not have to be rewritten to grow one.
  [ -n "$runs" ] || runs="run-headless"
  # PATH first, THEN preflight: the preflight's `yog` row must name the binary
  # that is about to be driven, not whatever happens to be installed.
  use_release
  "$here/preflight.sh"
  base="$root/$(stamp)"
  bad=0
  for v in $runs; do
    d="$base/$v"
    guard "$d/data"
    mkdir -p "$d/out"
    echo
    echo "=== $v   world: $d/data   evidence: $d/out"
    rc=0
    "$here/stories.sh" "$v" "$d/data" "$d/out" || rc=$?
    [ "$rc" = 0 ] || bad=$((bad + 1))
    stage_row "$base" "$v" "$rc"
  done
  echo
  skeleton "$base"
  [ "$bad" = 0 ] || {
    echo "drive.sh: $bad of the driven verbs reported failing beats" >&2
    exit 1
  }
}

cleanroom() {
  verb=${1:-run-headless}
  use_release
  "$here/preflight.sh"
  base="$root/$(stamp)/cleanroom-$verb"
  guard "$base/room"
  mkdir -p "$base/out"
  bad=0
  "$here/cleanroom.sh" "$tdir/release/yog" "$base/room" "$base/out" "$verb" || bad=$?
  stage_row "$base" "cleanroom-$verb" "$bad"
  echo
  skeleton "$base"
  [ "$bad" = 0 ] || { echo "drive.sh: the clean room reported failing beats" >&2; exit 1; }
}

log_it() {
  d=${1:-}
  [ -n "$d" ] || d=$(ls -dt "$root"/*/ 2>/dev/null | head -1)
  [ -n "$d" ] || { echo "drive.sh: nothing driven under $root yet" >&2; exit 1; }
  exec "$here/logskel.sh" "$d"
}

case ${1:-} in
preflight) shift; exec "$here/preflight.sh" "$@" ;;
seed)      shift; seed_world "${1:-}" ;;
ladder)    shift; ladder "$@" ;;
cleanroom) shift; cleanroom "$@" ;;
log)       shift; log_it "${1:-}" ;;
# Usage is read back out of this file's own header, keyed on the verb lines'
# content rather than on their line numbers — a line range silently prints the
# wrong thing the first time a comment above it grows.
*) echo "usage:" >&2; sed -n 's/^#   \(drive\.sh .*\)/  \1/p' "$0" >&2; exit 1 ;;
esac
