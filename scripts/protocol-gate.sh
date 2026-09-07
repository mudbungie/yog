#!/usr/bin/env bash
# protocol-gate.sh — a yog release that RAISES the wire protocol may not
# publish before its consumers can speak it (bl-bca2).
#
#   scripts/protocol-gate.sh file         the path every repo states it at
#   scripts/protocol-gate.sh roster       the consumer repositories, one a line
#   scripts/protocol-gate.sh read FILE    the PROTOCOL integer that file states
#   scripts/protocol-gate.sh judge PUBLISHED CANDIDATE NAME=FILE...
#   scripts/protocol-gate.sh --self-test  both directions, no network
#
# WHY THIS EXISTS. The wire is fail-closed on a version mismatch and there is
# no negotiation (docs/REMOTE.md §3): an engine and a seat that state different
# integers refuse each other, naming both numbers. That is the right refusal
# and is not what this gate is about. What it is about is the RELEASE WINDOW it
# opens. yog is the only component that mints the number; the seat, the foot
# and the phone each VENDOR a copy of the constant and learn about a bump when
# the engine they dial refuses them. So every yog release that raised PROTOCOL
# published a suite that could not compose, and stayed that way until three
# other repositories caught up. On 2026-09-06 a clean box installing all five
# published crates got engine 15, foot 14, seat 13 — no combination on
# crates.io composed, and the same defect had been filed and correctly closed
# eight times across two repositories, each close stale by the next release.
# The recurrence is the finding: a per-instance re-vendor ball is a fact about
# one afternoon, and the class is that yog publishes the number first.
#
# THE RULE (overseer ruling, bl-bca2). A yog release that raises PROTOCOL does
# not auto-merge until the consumers' mains carry it. A release that moves no
# wire version is unaffected — it publishes on the same terms it always did,
# because it makes nothing worse than the skew already standing.
#
# WHAT IT IS NOT. Not negotiation, not a compat window, not a version list. It
# does not touch the handshake at all; it moves the ORDER in which the four
# components publish, which is the only place the skew is actually decidable.
#
# THIS FILE IS PURE LOGIC AND READS NO NETWORK, which is the whole reason the
# rule is testable: `.github/workflows/release-automerge.yml` fetches the four
# `PROTOCOL` files (yog at its last release tag, yog at the pull request's
# head, and each consumer's `main`) and hands this script paths. The roster
# below is the one home for WHICH repositories are consumers; the workflow
# reads it rather than restating it.
#
# THE NUMBER IS A FILE, NOT A DECLARATION (bl-3e57). Every read here is a fetch
# of one path out of a tree the reader does not build, and a Rust path is not a
# stable address for that: bl-94a5 split yog's `src/wire/hello.rs` into
# `src/wire/hello/version.rs`, leaving the old file re-exporting — invisible to
# a build, fatal to a regex. Every consumer's gate then read the engine's
# constant as ABSENT, which is fail-closed and therefore held, so the bump the
# holds were waiting for would have made them permanent (thrall bl-c618). So
# the number now has one file-shaped home per repository, at the one address a
# module split cannot move: a top-level `PROTOCOL` file, one line, the integer.
# yog's `build.rs` compiles it into the constant; each consumer does the same
# for its vendored copy. There is NO Rust path in this file.

set -euo pipefail

# The path, at the root of every one of the four repositories. One constant for
# all four reads, because one address is the whole point: a per-repository path
# is a per-repository way to rot.
FILE='PROTOCOL'

# The consumers. One repository a line — where each keeps its copy is no longer
# a fact anything needs, so it is no longer a fact anything can get wrong.
ROSTER=$(
  printf '%s\n' \
    'mudbungie/thrall' \
    'mudbungie/lernie' \
    'mudbungie/yog-android'
)

# The integer FILE states, or nothing and a non-zero status. The whole file is
# the number: any second line, any word, any punctuation is not a PROTOCOL file
# and reads as unstated rather than as a number found inside something else.
# Command substitution strips the trailing newline, and a newline is not a
# digit, so the one `case` rejects an empty file and a multi-line one alike.
protocol_of() {
  local file=$1 stated
  [ -r "$file" ] || return 1
  stated=$(<"$file")
  case $stated in
  '' | *[!0-9]*) return 1 ;;
  esac
  printf '%s\n' "$stated"
}

# The verdict, as one line on stdout. Exit 0 merges, 3 holds, 2 is a usage
# fault. Every unreadable input holds: a gate that cannot read its inputs has
# not answered, and answering "merge" there is the one failure this cannot
# recover from — a hold is undone by the next run.
judge() {
  local published=$1 candidate=$2
  shift 2
  local pub cand moved spec name file n
  local -a lag=()

  [ "$#" -gt 0 ] || {
    echo "hold: no consumer was named, so nothing was checked"
    return 3
  }

  if ! cand=$(protocol_of "$candidate"); then
    echo "hold: the release tree's $candidate states no PROTOCOL integer"
    return 3
  fi

  if pub=$(protocol_of "$published"); then
    if [ "$pub" = "$cand" ]; then
      echo "merge: PROTOCOL stands at $cand — this release moves no wire version"
      return 0
    fi
    moved="PROTOCOL $pub -> $cand"
  else
    # No readable baseline (no previous release, or a release predating the
    # `PROTOCOL` file). Judge it AS a bump: that is the fail-closed
    # reading, and unlike an outright hold it is undone by the consumers
    # catching up rather than by a hand.
    moved="PROTOCOL $cand, with no readable baseline to compare it against"
  fi

  for spec in "$@"; do
    name=${spec%%=*}
    file=${spec#*=}
    if ! n=$(protocol_of "$file"); then
      lag+=("$name states no PROTOCOL integer")
    elif [ "$n" != "$cand" ]; then
      lag+=("$name speaks $n")
    fi
  done

  if [ "${#lag[@]}" -gt 0 ]; then
    local list
    list=$(printf '%s; ' "${lag[@]}")
    echo "hold: $moved, and ${list%; }"
    return 3
  fi
  echo "merge: $moved, and every consumer main carries $cand"
}

# --- the self-test ----------------------------------------------------------
# Both directions, because the workflow that spends this cannot run locally and
# a check that has stopped matching passes everything forever. Each case states
# a tree and the one line the gate must answer with.
# A tree's `PROTOCOL` file, written byte for byte — `%b` so a case can state
# its own line endings, which is half of what these fixtures are for.
fixture() {
  printf '%b' "$2" >"$1"
}

expect() {
  local want_rc=$1 want=$2
  shift 2
  local got rc=0
  got=$(judge "$@") || rc=$?
  if [ "$rc" != "$want_rc" ] || [ "$got" != "$want" ]; then
    echo "protocol-gate self-test: judge $*" >&2
    echo "  expected rc $want_rc: $want" >&2
    echo "  answered rc $rc: $got" >&2
    exit 1
  fi
}

self_test() {
  local d
  d=$(mktemp -d)
  # shellcheck disable=SC2064
  trap "rm -rf '$d'" EXIT

  fixture "$d/n15" '15\n'
  fixture "$d/n16" '16\n'
  # A file is the number and nothing else, but the number may be spelled with
  # the whitespace an editor leaves: no trailing newline, and a trailing blank
  # line, are the same 16.
  fixture "$d/tight16" '16'
  fixture "$d/loose16" '16\n\n'
  # What the number's home used to be. A Rust declaration is now exactly as
  # unreadable as prose, which is the point: this file names no Rust path, so a
  # tree that still keeps its number in a module states nothing.
  fixture "$d/decl" 'pub const PROTOCOL: u32 = 16;\n'

  # A bump, and a consumer still on the published number: held, and the line
  # names the repo and what it speaks. This is the 2026-09-06 tree.
  expect 3 'hold: PROTOCOL 15 -> 16, and lernie speaks 15' \
    "$d/n15" "$d/n16" "thrall=$d/n16" "lernie=$d/n15"
  # Two behind: both named, in roster order, in one line.
  expect 3 'hold: PROTOCOL 15 -> 16, and thrall speaks 15; lernie speaks 15' \
    "$d/n15" "$d/n16" "thrall=$d/n15" "lernie=$d/n15"
  # The same bump once they have landed it — however the file is terminated.
  expect 0 'merge: PROTOCOL 15 -> 16, and every consumer main carries 16' \
    "$d/n15" "$d/n16" "thrall=$d/tight16" "lernie=$d/loose16"
  # No bump: unaffected, even with every consumer behind or unreadable.
  expect 0 'merge: PROTOCOL stands at 15 — this release moves no wire version' \
    "$d/n15" "$d/n15" "thrall=$d/n15" "lernie=$d/decl"
  # Fail closed on each unreadable input, and never merge on one.
  expect 3 'hold: PROTOCOL 16, with no readable baseline to compare it against, and lernie speaks 15' \
    "$d/absent" "$d/n16" "lernie=$d/n15"
  expect 3 "hold: the release tree's $d/decl states no PROTOCOL integer" \
    "$d/n15" "$d/decl" "lernie=$d/n16"
  expect 3 'hold: PROTOCOL 15 -> 16, and lernie states no PROTOCOL integer' \
    "$d/n15" "$d/n16" "lernie=$d/absent"
  expect 3 'hold: no consumer was named, so nothing was checked' \
    "$d/n15" "$d/n16"

  # The roster is data the workflow spends, so it is checked as data: three
  # rows, each naming a repository and nothing else. A roster that enumerated
  # nothing would merge every bump unchecked. `FILE` is checked beside it
  # because a path with a directory in it is a path a module split can move,
  # which is the defect this shape exists to end.
  local rows
  rows=$(printf '%s\n' "$ROSTER" | grep -c '^mudbungie/[a-z-]*$' || true)
  if [ "$rows" != 3 ] || [ "$FILE" != 'PROTOCOL' ]; then
    echo "protocol-gate self-test: the roster is not three bare consumer repositories, or the number is not read at the repo root" >&2
    exit 1
  fi
  echo "protocol-gate: self-test OK — 8 verdicts, both directions, over a 3-consumer roster" >&2
}

case ${1:-} in
--self-test) self_test ;;
file) printf '%s\n' "$FILE" ;;
roster) printf '%s\n' "$ROSTER" ;;
read) [ "$#" = 2 ] || { echo "usage: protocol-gate.sh read FILE" >&2; exit 2; }
      protocol_of "$2" || { echo "protocol-gate: $2 states no PROTOCOL integer" >&2; exit 1; } ;;
judge) shift
       [ "$#" -ge 2 ] || { echo "usage: protocol-gate.sh judge PUBLISHED CANDIDATE NAME=FILE..." >&2; exit 2; }
       judge "$@" ;;
*) echo "usage: protocol-gate.sh {file|roster|read FILE|judge ...|--self-test}" >&2; exit 2 ;;
esac
