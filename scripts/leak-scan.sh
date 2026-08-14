#!/usr/bin/env bash
# yog leak scan (bl-fd5a, reworked bl-167d) — the disclosure half of the gate.
# The rest of the gate asks whether the tree is well-formed (fmt, clippy, the
# 300-line cap, the ast-grep rules, cargo-deny, coverage); nothing asked
# whether it discloses something. yog exists to drive real agent sessions on a
# real box, so the material it could leak is exactly the material it handles:
# brazen credentials, Claude Code session transcripts, world paths under an
# operator's home, opslog dumps.
#
# The rules live next door in `scripts/leak-rules.sh`; this file is mechanism.
#
#   scripts/leak-scan.sh              scan the whole tracked tree (the gate)
#   scripts/leak-scan.sh FILE...      scan exactly these files (commit-msg)
#   scripts/leak-scan.sh --self-test  prove every rule still fires, and that
#                                     none fires on the clean fixture
#
# THE TREE IT SCANS IS THE ONE IT IS RUN IN, WHICH NEED NOT BE THIS REPO
# (bl-1043). The rule table is resolved from the SCRIPT's own directory and the
# tree from `git rev-parse` in the working directory, so the same mechanism and
# the same table judge yog's index and the balls TASK STORE — a different git
# repo entirely (`<state>/balls/clones/<enc>/tasks`, holding `tasks/*.md`),
# written by `bl`, never reached by this repo's pre-commit hook. Ball bodies
# are prose on a ref that publishes beside the source, and a second copy of the
# rules for them would drift from this one inside a week. Its callers are
# `scripts/yog-leak-gate` (the balls plugin, before the store is pushed) and
# `.github/workflows/store-scan.yml` (the published ref, after).
#
# THE TREE MODE READS INDEX BLOBS, NOT THE WORKTREE. That is the whole of
# bl-167d's headline: this scan used to enumerate `git ls-files` and then hand
# those PATH NAMES to grep, which opens the WORKTREE file — so a leak that was
# `git add`ed and then overwritten with a clean copy on disk was committed
# without the gate ever reading the bytes it was gating. `git checkout-index`
# materializes the index into a scratch directory and the scan reads that, so
# the bytes scanned are the bytes committed. The index rather than the diff,
# for the same reason `make line-cap` reads it: a diff-only gate is a sampling,
# not an invariant, and a file that leaked once and was never touched again
# would never be looked at again.
#
# THE REGRESSION HALF IS `--self-test`, and it is the point of this file. A
# leak gate does not die by being wrong; it dies by silently matching nothing
# after a pattern is edited, and then passing everything forever. So every
# rule owns a fixture (`scripts/leak-fixtures/<rule>.txt`) in which EVERY
# non-comment line must be flagged — line granularity, not file granularity,
# so one dead alternative inside a nine-way `vendor-token` pattern cannot hide
# behind the eight that still work — and must carry `FIXTURE_MARKER`, because
# no regex can tell a real secret from a fabricated one and only the value can
# say so. `rules-audit` (the ast-grep equivalent in the Makefile) only asserts
# its fixture DIRECTORY is flagged; this is the stronger check. The other
# direction is `clean.txt` / `clean-paths.txt`: near-misses that must NOT be
# flagged, because a gate that cries wolf on a fifth of the tree gets
# bypassed, and a bypassed gate is no gate.
#
# NOTHING IS EXEMPT FROM THE TREE SCAN ANY MORE. The scanner and its rule
# table used to be skipped for being made of the patterns; they are scanned,
# and stay clean because no pattern may match its own text (leak-rules.sh
# says how). A rule fixture is scanned by every rule EXCEPT the one it is the
# fixture of — its own rule must flag it, that is its contract — which is a
# structural exemption keyed to the file's own name, not an allowlist: adding
# a file to it means adding a RULE of that name.
#
# WHAT A COMMIT HOOK CANNOT PROMISE. This scans ONE TREE. Old commits, other
# refs, pull-request and release text, Actions logs, build artifacts and
# already-published crate versions are all outside it, and no hook can reach
# them; a gate that implied otherwise would be worse than one that says so.
# They are a RELEASE CHECKLIST instead — AGENTS.md, "Before making the repo or
# a crate version public".
#
# Known limits, stated rather than implied:
#   - IPv6 is matched in full 8-group form only. The compressed `::` forms
#     cannot be told from Rust path syntax (`deadbeef::cafe`) without a false
#     positive rate that would get the whole gate disabled.
#   - A four-part version string (major.minor.patch.build) is
#     indistinguishable from an IPv4 address. If one ever lands, it goes in
#     the rule's EXCEPT list.
#   - Ordinary prose is not detectable. A pasted paragraph of somebody's
#     conversation with no speaker label and no session key reads as writing;
#     `quoted-dialogue` catches the SHAPE transcripts arrive in, which is all
#     a regex can do.

set -euo pipefail

# The table travels with the SCANNER, not with the tree under scan: run in the
# task store there is no `scripts/` to source, and a copy of the rules there
# would be a second definition of what counts as a leak. Resolved BEFORE the
# `cd`, or a relative `$0` would be resolved against the wrong directory.
# `--self-test` and the fixture skip stay tree-relative, because the fixtures
# are tracked files of THIS repo and only this repo's scan has any to skip.
HERE="$(CDPATH= cd -- "$(dirname -- "$(readlink -f "$0" 2>/dev/null || echo "$0")")" && pwd)"

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

FIXTURES="scripts/leak-fixtures"

# shellcheck source=scripts/leak-rules.sh
. "$HERE/leak-rules.sh"

# --- mechanism -------------------------------------------------------------

# Emit one line per finding: "  path:line  [rule]  <first 12 chars of match>".
# Truncated deliberately — the finding must LOCATE the leak, never reprint it
# into a terminal, a CI log or a bug report.
report() {
  awk -F: -v rule="$1" '{
    m = substr($0, length($1) + length($2) + 3)
    if (length(m) > 12) m = substr(m, 1, 12) "..."
    printf "  %s:%s  [%s]  %s\n", $1, $2, rule, m
  }'
}

# scan_rule RULE FILE... -> findings on stdout, empty if clean.
scan_rule() {
  local rule="$1"; shift
  [ "$#" -gt 0 ] || return 0
  local hits PATTERN EXCEPT WHY
  rule_fields "$rule"
  hits="$(grep -HIonE -e "$PATTERN" -- "$@" 2>/dev/null || true)"
  [ -n "$hits" ] || return 0
  if [ -n "$EXCEPT" ]; then
    hits="$(printf '%s\n' "$hits" | grep -vE ":[0-9]+:(${EXCEPT})" || true)"
  fi
  [ -n "$hits" ] || return 0
  printf '%s\n' "$hits" | report "$rule"
}

# scan_paths PATH... -> findings for the path rule.
scan_paths() {
  local p
  for p in "$@"; do
    printf '%s\n' "$p" | grep -qE "$FORBIDDEN_PATH" && printf '  %s  [forbidden-path]\n' "$p"
  done
  return 0
}

# scan_binary FILE... -> findings for content no rule can read. A file grep
# will not read as text is not clean, it is unexamined.
scan_binary() {
  local f
  for f in "$@"; do
    [ -s "$f" ] || continue
    grep -qI '' "$f" 2>/dev/null && continue
    printf '%s\n' "$f" | grep -qE "$BINARY_ALLOWED" && continue
    printf '  %s  [binary-content]\n' "$f"
  done
  return 0
}

# scan [--skip RULE] FILE... -> 0 clean, 1 with findings printed to stderr.
scan() {
  local skip=''
  if [ "${1-}" = --skip ]; then skip="$2"; shift 2; fi
  local rule found='' out PATTERN EXCEPT WHY
  for rule in "${RULES[@]}"; do
    [ "$rule" = "$skip" ] && continue
    out="$(scan_rule "$rule" "$@")"
    rule_fields "$rule"
    [ -n "$out" ] && found+="$out"$'\n'"       $WHY"$'\n'
  done
  for rule in forbidden-path binary-content; do
    [ "$rule" = "$skip" ] && continue
    case "$rule" in
      forbidden-path) out="$(scan_paths "$@")" ;;
      *)              out="$(scan_binary "$@")" ;;
    esac
    rule_fields "$rule"
    [ -n "$out" ] && found+="$out"$'\n'"       $WHY"$'\n'
  done
  if [ -n "$found" ]; then
    echo "error: leak-scan found material that must not be committed:" >&2
    printf '%s' "$found" >&2
    return 1
  fi
  return 0
}

# --- modes -----------------------------------------------------------------

# The index, materialized into SCAN_DIR. Everything the tree mode reads comes
# from here, so "what the gate scanned" and "what the commit contains" are the
# same bytes. Not a command substitution: the trap has to be set in THIS shell
# or the scratch tree is deleted the moment the subshell returns it.
checkout_index() {
  SCAN_DIR="$(mktemp -d "${TMPDIR:-/tmp}/leak-scan.XXXXXXXX")"
  trap 'rm -rf "$SCAN_DIR"' EXIT
  git checkout-index --all --force --prefix="$SCAN_DIR/"
}

scan_tree() {
  local files=() fixtures=() f rc=0
  while IFS= read -r f; do
    case "$f" in "$FIXTURES"/*) fixtures+=("$f"); continue ;; esac
    files+=("$f")
  done < <(git ls-files)
  if [ "${#files[@]}" -eq 0 ]; then
    echo "leak-scan: enumerated 0 tracked files — the scan is broken, not the tree." >&2
    exit 1
  fi
  checkout_index
  cd "$SCAN_DIR"
  scan "${files[@]}" || rc=1
  # Each fixture, judged by every rule but its own (see the header).
  for f in "${fixtures[@]}"; do
    f="${f##*/}"
    scan --skip "${f%.*}" "$FIXTURES/$f" || rc=1
  done
  [ "$rc" -eq 0 ] || exit 1
  echo "leak-scan: $(( ${#files[@]} + ${#fixtures[@]} )) tracked files, no disclosure findings"
}

# --- self-test -------------------------------------------------------------

# Every non-blank, non-'#' line of a rule's fixture must be flagged BY THAT
# RULE and must carry FIXTURE_MARKER; nothing in the clean fixtures may be
# flagged by anything.
fixture_lines() {
  local rule="$1" fixture="$2" hit="$3" ln fails=0 n=0 content
  while IFS= read -r ln; do
    [ -n "$ln" ] || continue
    n=$((n + 1))
    content="$(sed -n "${ln}p" "$fixture")"
    if [ "$rule" = forbidden-path ]; then
      printf '%s\n' "$hit" | grep -qF "$content" || {
        echo "self-test: [$rule] line $ln of $fixture was NOT flagged" >&2; fails=1; }
      continue
    fi
    printf '%s\n' "$hit" | grep -qE ":$ln  \[" || {
      echo "self-test: [$rule] line $ln of $fixture was NOT flagged" >&2; fails=1; }
    printf '%s' "$content" | grep -qi "$FIXTURE_MARKER" || {
      echo "self-test: [$rule] line $ln of $fixture carries no '$FIXTURE_MARKER' marker — a fixture value must be unmistakably fabricated" >&2
      fails=1; }
  done <<<"$(grep -nvE '^(#|$)' "$fixture" | cut -d: -f1)"
  [ "$n" -gt 0 ] || { echo "self-test: $fixture has no cases" >&2; fails=1; }
  return "$fails"
}

self_test() {
  local rule fixture fails=0 hit p
  for rule in "${RULES[@]}" forbidden-path; do
    fixture="$FIXTURES/$rule.txt"
    if [ ! -f "$fixture" ]; then
      echo "self-test: rule '$rule' has no fixture at $fixture" >&2; fails=1; continue
    fi
    if [ "$rule" = forbidden-path ]; then
      hit="$(grep -vE '^(#|$)' "$fixture" | while IFS= read -r p; do scan_paths "$p"; done)"
    else
      hit="$(scan_rule "$rule" "$fixture")"
    fi
    fixture_lines "$rule" "$fixture" "$hit" || fails=1
  done
  # binary-content owns bytes, not lines: its fixture cannot carry a marker or
  # be read at all, so it is capped instead. 512 bytes is far too small to
  # smuggle a dump through the one file the scanner cannot look inside.
  fixture="$FIXTURES/binary-content.bin"
  [ -n "$(scan_binary "$fixture")" ] || {
    echo "self-test: [binary-content] $fixture was NOT flagged" >&2; fails=1; }
  [ "$(wc -c <"$fixture")" -le 512 ] || {
    echo "self-test: $fixture is over the 512-byte cap on unreadable fixtures" >&2; fails=1; }
  [ -z "$(scan_binary assets/yog-16.png)" ] || {
    echo "self-test: a declared derivation (assets/yog-16.png) was flagged binary" >&2; fails=1; }
  # The false-positive direction, and the half that keeps the gate usable: a
  # near-miss for every rule, none of which may be flagged.
  if ! scan "$FIXTURES/clean.txt"; then
    echo "self-test: $FIXTURES/clean.txt was flagged above — a rule is over-broad" >&2
    fails=1
  fi
  while IFS= read -r p; do
    case "$p" in '#'*|'') continue ;; esac
    [ -z "$(scan_paths "$p")" ] || {
      echo "self-test: clean path '$p' was flagged — forbidden-path is over-broad" >&2; fails=1; }
  done <"$FIXTURES/clean-paths.txt"
  [ "$fails" -eq 0 ] || exit 1
  echo "leak-scan: self-test OK — ${#RULES[@]} content rules + forbidden-path + binary-content all live, clean fixtures unflagged"
}

case "${1-}" in
  --self-test) self_test ;;
  '') scan_tree ;;
  *) scan "$@" || exit 1 ;;
esac
