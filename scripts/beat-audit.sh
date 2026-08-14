#!/usr/bin/env bash
# beat-audit.sh — the two mechanical shapes of a DRIVE BEAT that proves nothing
# (bl-70b8). Run over `scripts/drive/`; called by `make lint`.
#
#   scripts/beat-audit.sh              audit the harness
#   scripts/beat-audit.sh --self-test  prove both shapes still fire, and that
#                                      neither fires on the clean fixture
#
# WHY A CHECK AND NOT A HABIT. Both shapes below were documented before they
# recurred. The vacuity heuristic was written onto bl-f16e and a new instance
# appeared the same day; bl-0e44's duplicate-beat-name collision had a written
# lesson and still deleted three beats from every `run_s7`. What closes a class
# in this repo is a check, the way `make line-cap` closes the file-size class —
# so these two are here, and the shapes that are NOT mechanically checkable are
# recorded as decisions on bl-70b8 rather than as prose nobody re-reads.
#
# SHAPE A — A BEAT THAT CAN ONLY EMIT ONE VERDICT. `gesture … || pass "…"` was
# a real beat (`S8-T4 marks: balls' landing branch is refused, not written`):
# the `||` short-circuits on success, so the ONE outcome it existed to catch —
# the unlawful branch accepted — wrote no row at all and deleted the beat from
# the verdict instead of reddening it. A ladder counts the rows it has, never
# the rows it should have had, which is bl-0e44's blindness reached by another
# road. So every label handed to `pass` must also be handed to `fail`, and
# vice versa. NO ALLOWLIST, because a one-armed beat is never right: measured
# over the whole harness it found exactly one violation and zero false
# positives (bl-f16e).
#
# SHAPE B — AN ASSERTION WHOSE SUBJECT CAN BE EMPTY. `grep -q "$id"` with an
# unset `$id` is `grep -q ""`, which matches every non-empty stream, so the
# beat passes unconditionally — and it does so precisely when the beat ABOVE it
# failed to produce the id, which is when you least want a green row. It has
# bitten twice: `$minted` when the mint above it failed (bl-afa7) and
# `$refused` when the failed-close row above it was never written (bl-f16e).
# The check is narrow on purpose: a pattern that is EXACTLY one interpolation
# and nothing else. A pattern with an interpolation embedded in a literal
# (`"\"cwd\":\"[^\"]*/$1\""`) degrades to a literal when the variable is empty
# and is a different question; a bare `"$x"` degrades to nothing at all. The
# repo's answer to shape B is that the guard lives in the PREDICATE — every
# id-taking predicate in `harness.sh` refuses an empty subject — and this check
# is what keeps the next beat from reaching past it.

set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
found=0

# Every label that reaches `pass` must also reach `fail`. Sorted-set difference
# in both directions: a `pass` with no `fail` is the one-armed beat, and a
# `fail` with no `pass` is a beat that can only ever be red — which is a beat
# nobody has run.
audit_arms() {
  dir=$1
  p=$(grep -ho 'pass "[^"]*"' "$dir"/beats_*.sh "$dir"/stories.sh 2>/dev/null \
    | sed 's/^pass "//; s/"$//' | LC_ALL=C sort -u)
  f=$(grep -ho 'fail "[^"]*"' "$dir"/beats_*.sh "$dir"/stories.sh 2>/dev/null \
    | sed 's/^fail "//; s/"$//' | LC_ALL=C sort -u)
  one_armed=$(comm -3 <(printf '%s\n' "$p") <(printf '%s\n' "$f") | sed 's/^\t//')
  [ -z "$one_armed" ] && return 0
  echo "beat-audit: a beat writes BOTH a PASS and a FAIL, or it is not a beat —" >&2
  echo "  these labels reach only one of the two, so the outcome they exist to" >&2
  echo "  catch leaves no verdict row at all:" >&2
  printf '    %s\n' "$one_armed" >&2
  found=1
}

# A `grep -q`/`grep -lq` whose whole pattern is one interpolation. `"$x"`,
# `"${x}"` and `"$(cmd)"` all collapse to the empty pattern when the subject is
# missing, and the empty pattern matches everything.
#
# `-x` IS EXEMPT, AND NOT BY ALLOWLIST — it anchors the pattern to a whole line,
# so an empty pattern matches only an EMPTY LINE rather than every line, and the
# vacuity is gone. `preflight.sh`'s `grep -qx -- "$p"` is the tree's one such
# site and it is correct; a check that flagged it would be crying wolf on the
# one shape that already answers the objection.
audit_subjects() {
  dir=$1
  bare=$(grep -EHn 'grep -[a-zA-Z]*q[a-zA-Z]*( --)? +"(\$\{?[A-Za-z_][A-Za-z_0-9]*\}?|\$\([^)]*\))"' \
    "$dir"/*.sh 2>/dev/null | grep -Ev 'grep -[a-zA-Z]*x' || true)
  [ -z "$bare" ] && return 0
  echo "beat-audit: this pattern is one bare interpolation, so an empty subject" >&2
  echo "  makes it \`grep -q \"\"\` — true of every non-empty stream. Refuse the" >&2
  echo "  empty subject in the predicate, or name what distinguishes this run:" >&2
  printf '    %s\n' "$bare" >&2
  found=1
}

# --- the self-test ----------------------------------------------------------
# A check does not die by being wrong; it dies by silently matching nothing
# after an edit, and then passing everything forever. Both directions: the
# fixture harness must be flagged for BOTH shapes, and the real harness — which
# is the clean fixture — must not be flagged at all.
self_test() {
  d=$(mktemp -d)
  trap 'rm -rf "$d"' EXIT
  cat > "$d/beats_fixture.sh" <<'FIX'
#!/bin/bash
s_fixture() {
  # shape A: the `|| pass` that emits nothing when the gesture SUCCEEDS
  do_it || pass "fixture: the boundary refuses it"
  # shape B: a pattern that is one bare interpolation
  grep -q "$id" "$ops" \
    && pass "fixture: the row names the conversation" \
    || fail "fixture: the row names the conversation" "no row"
}
FIX
  : > "$d/stories.sh"
  out=$( { audit_arms "$d"; audit_subjects "$d"; } 2>&1 || true )
  for want in "fixture: the boundary refuses it" 'grep -q "$id"'; do
    case $out in
      *"$want"*) ;;
      *) echo "beat-audit self-test: the fixture was NOT flagged for [$want]" >&2
         echo "  — a shape has regressed and this check is now decorative" >&2
         exit 1 ;;
    esac
  done
  found=0
  audit_arms "$here/drive" ; audit_subjects "$here/drive"
  [ "$found" = 0 ] || {
    echo "beat-audit self-test: the real harness was flagged; it is the clean" >&2
    echo "  fixture, so either it regressed or a shape now cries wolf" >&2
    exit 1
  }
  echo "beat-audit: self-test OK — both shapes fire on the fixture, neither on the harness" >&2
}

case ${1:-} in
--self-test) self_test ;;
"") audit_arms "$here/drive" ; audit_subjects "$here/drive"
    [ "$found" = 0 ] || exit 1
    n=$(grep -ho 'pass "[^"]*"' "$here"/drive/beats_*.sh "$here"/drive/stories.sh \
      | LC_ALL=C sort -u | wc -l)
    echo "beat-audit: $n beats, every one two-armed; no assertion on a bare subject" >&2 ;;
*) echo "usage: beat-audit.sh [--self-test]" >&2 ; exit 1 ;;
esac
