#!/usr/bin/env bash
# protocol-gate.sh — a yog release that RAISES the wire protocol may not
# publish before its consumers can speak it (bl-bca2).
#
#   scripts/protocol-gate.sh hello        yog's own hello file, one path
#   scripts/protocol-gate.sh roster       the consumers, `repo<TAB>hello` rows
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
# `hello.rs` files (yog at its last release tag, yog at the pull request's
# head, and each consumer's `main`) and hands this script paths. The roster
# below is the one home for WHERE each consumer keeps its vendored constant;
# the workflow reads it rather than restating it.

set -euo pipefail

# yog's own. The number's one home, and the file the two yog readings below
# are taken from at two different commits.
HELLO='src/wire/hello.rs'

# The consumers, and where each vendors the constant. Tab-separated so the
# workflow can read it with `while IFS=$'\t' read`.
#
# All three keep it as a plain `const PROTOCOL: u32 = N;` in a `hello`-shaped
# module, which is why one regex reads all four files. thrall keeps a SECOND
# copy in `src/corpus.rs` that its own tests hold equal to this one; the wire
# constant is the one that decides a handshake, so it is the one read here.
ROSTER=$(
  printf '%s\t%s\n' \
    'mudbungie/thrall'      'src/channel/hello.rs' \
    'mudbungie/lernie'      'src/channel/hello.rs' \
    'mudbungie/yog-android' 'src/hello.rs'
)

# The declaration, anchored at the start of a line so a doc comment quoting it
# — thrall's `src/corpus.rs` quotes the line verbatim — is not mistaken for it.
# `pub`, `pub(crate)` and a bare `const` all read.
DECL='^[[:space:]]*(pub[[:space:]]*(\([^)]*\))?[[:space:]]+)?const[[:space:]]+PROTOCOL[[:space:]]*:[[:space:]]*u32[[:space:]]*=[[:space:]]*([0-9]+)[[:space:]]*;.*$'

# The integer FILE states, or nothing and a non-zero status. No pipe: `sed |
# head` would kill the writer with SIGPIPE and `pipefail` would then report the
# read as failed exactly when it succeeded (scripts/beat-audit.sh, shape C).
protocol_of() {
  local file=$1 hits
  [ -r "$file" ] || return 1
  hits=$(sed -nE "s/$DECL/\\3/p" "$file")
  [ -n "$hits" ] || return 1
  printf '%s\n' "${hits%%$'\n'*}"
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
    echo "hold: no PROTOCOL declaration in the release tree's $candidate"
    return 3
  fi

  if pub=$(protocol_of "$published"); then
    if [ "$pub" = "$cand" ]; then
      echo "merge: PROTOCOL stands at $cand — this release moves no wire version"
      return 0
    fi
    moved="PROTOCOL $pub -> $cand"
  else
    # No readable baseline (no previous release, or the constant moved house
    # in the released tree). Judge it AS a bump: that is the fail-closed
    # reading, and unlike an outright hold it is undone by the consumers
    # catching up rather than by a hand.
    moved="PROTOCOL $cand, with no readable baseline to compare it against"
  fi

  for spec in "$@"; do
    name=${spec%%=*}
    file=${spec#*=}
    if ! n=$(protocol_of "$file"); then
      lag+=("$name: no PROTOCOL declaration read")
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
fixture() {
  printf '//! /// pub const PROTOCOL: u32 = 999;\n%s\n' "$2" >"$1"
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

  fixture "$d/pub15" 'pub const PROTOCOL: u32 = 15;'
  fixture "$d/pub16" 'pub const PROTOCOL: u32 = 16;'
  fixture "$d/crate16" '    pub(crate) const PROTOCOL: u32 = 16;'
  fixture "$d/bare16" 'const PROTOCOL: u32 = 16;'
  fixture "$d/quoted" '/// reads `pub const PROTOCOL: u32 = 15`, copied below.'

  # A bump, and a consumer still on the published number: held, and the line
  # names the repo and what it speaks. This is the 2026-09-06 tree.
  expect 3 'hold: PROTOCOL 15 -> 16, and lernie speaks 15' \
    "$d/pub15" "$d/pub16" "thrall=$d/pub16" "lernie=$d/pub15"
  # Two behind: both named, in roster order, in one line.
  expect 3 'hold: PROTOCOL 15 -> 16, and thrall speaks 15; lernie speaks 15' \
    "$d/pub15" "$d/pub16" "thrall=$d/pub15" "lernie=$d/pub15"
  # The same bump once they have landed it — however each spells the constant.
  expect 0 'merge: PROTOCOL 15 -> 16, and every consumer main carries 16' \
    "$d/pub15" "$d/pub16" "thrall=$d/crate16" "lernie=$d/bare16"
  # No bump: unaffected, even with every consumer behind.
  expect 0 'merge: PROTOCOL stands at 15 — this release moves no wire version' \
    "$d/pub15" "$d/pub15" "thrall=$d/pub15" "lernie=$d/quoted"
  # Fail closed on each unreadable input, and never merge on one.
  expect 3 'hold: PROTOCOL 16, with no readable baseline to compare it against, and lernie speaks 15' \
    "$d/absent" "$d/pub16" "lernie=$d/pub15"
  expect 3 "hold: no PROTOCOL declaration in the release tree's $d/quoted" \
    "$d/pub15" "$d/quoted" "lernie=$d/pub16"
  expect 3 'hold: PROTOCOL 15 -> 16, and lernie: no PROTOCOL declaration read' \
    "$d/pub15" "$d/pub16" "lernie=$d/absent"
  expect 3 'hold: no consumer was named, so nothing was checked' \
    "$d/pub15" "$d/pub16"

  # The roster is data the workflow spends, so it is checked as data: three
  # rows, each naming a repository and a path. A roster that enumerated
  # nothing would merge every bump unchecked.
  local rows
  rows=$(printf '%s\n' "$ROSTER" | grep -c '^mudbungie/[a-z-]*	src/.*hello\.rs$' || true)
  if [ "$rows" != 3 ] || [ "$HELLO" != 'src/wire/hello.rs' ]; then
    echo "protocol-gate self-test: the roster is not three consumer rows plus yog's own" >&2
    exit 1
  fi
  echo "protocol-gate: self-test OK — 8 verdicts, both directions, over a 3-consumer roster" >&2
}

case ${1:-} in
--self-test) self_test ;;
hello) printf '%s\n' "$HELLO" ;;
roster) printf '%s\n' "$ROSTER" ;;
read) [ "$#" = 2 ] || { echo "usage: protocol-gate.sh read FILE" >&2; exit 2; }
      protocol_of "$2" || { echo "protocol-gate: no PROTOCOL declaration in $2" >&2; exit 1; } ;;
judge) shift
       [ "$#" -ge 2 ] || { echo "usage: protocol-gate.sh judge PUBLISHED CANDIDATE NAME=FILE..." >&2; exit 2; }
       judge "$@" ;;
*) echo "usage: protocol-gate.sh {hello|roster|read FILE|judge ...|--self-test}" >&2; exit 2 ;;
esac
