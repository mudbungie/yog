#!/bin/bash
# logskel.sh — emit the SKELETON of a drive log, pre-filled from the run's own
# verdict rows (bl-56d5).
#
# Every drive report used to be typed from a blank page: the sha
# looked up by hand, the host tuple retyped, and the beat table transcribed from
# a scrolled-past terminal — which is both the dullest part of a drive and the
# part most likely to be quietly wrong, because a transcribed table is a claim
# about a run rather than a reading of it. Everything mechanical is emitted
# here from `verdicts.jsonl` and from the host itself; everything that requires
# judgement is left as an explicit HAND-FINISH marker, because a generated
# report is a *start*, never a filed one. The house style (evidence quoted, not
# summarized) is the operator's half and cannot be generated.
#
# Usage: logskel.sh <dir>
#   <dir> is a ladder root (`drive.sh ladder` writes `<stamp>/<verb>/out/`) or a
#   single run's out dir, under the evidence root `$DRIVE_ROOT` (default
#   `$XDG_CACHE_HOME/yog-drive`). Every `verdicts.jsonl` beneath it becomes a
#   section. Writes markdown to stdout, and the log is finished and kept THERE,
#   beside its shots — never committed (QUALITY.md §3 step 6; bl-244f):
#     scripts/drive/logskel.sh "$DRIVE_ROOT/<stamp>" > "$DRIVE_ROOT/<stamp>/drive-log.md"
set -eu
# pipefail because the emission below now ENDS in a pipe (`| home_fold`), and
# without it a python3 that dies mid-table would exit 0 through sed and hand
# back a truncated skeleton as if it were whole. The pipelines that predate it
# are all `|| true`-guarded (`first`, the `bins` count), so this changes
# nothing else.
set -o pipefail
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../.." && pwd)
dir=${1:?usage: logskel.sh <ladder-root-or-out-dir>}
dir=$(cd "$dir" && pwd)

files=$(find "$dir" -name verdicts.jsonl | sort)
# A VERDICT-LESS RUN IS REPORTED, NOT REFUSED (bl-d0a0). This used to exit 1
# before emitting a byte, which made the generator the loudest thing about a
# run it knew nothing about: `drive.sh` redirects into `drive-log.md` and dies
# on the non-zero under `set -e`, so a seat that never came up produced a
# zero-byte report and a complaint about the report in place of the seat's own
# error. Emitting the run is the generator's job and it can always do it — the
# beat table is simply the sentence "no verdicts produced", which is a finding.
# The stderr line stays: it is a diagnostic for this terminal, not the product.
[ -n "$files" ] \
  || echo "logskel: no verdicts.jsonl under $dir — reporting a verdict-less run." >&2

# `git` is read through a scrubbed env: a `GIT_DIR` inherited from a hook
# outranks `-C` and would silently report another repo's sha into this log.
sha=$(env -u GIT_DIR -u GIT_INDEX_FILE git -C "$repo" rev-parse --short HEAD 2>/dev/null || echo unknown)
branch=$(env -u GIT_DIR -u GIT_INDEX_FILE git -C "$repo" rev-parse --abbrev-ref HEAD 2>/dev/null || echo unknown)
if env -u GIT_DIR -u GIT_INDEX_FILE git -C "$repo" diff --quiet HEAD 2>/dev/null; then
  tree="clean tree"
else
  tree="**dirty tree** (the sha alone does not name what was driven)"
fi

first() { "$@" 2>&1 | head -1 || true; }

# The binary under drive is READ BACK OFF THE RUN'S OWN ROWS (`bin`, written by
# harness.sh's `record`), never re-asked of PATH here. Inside `drive.sh ladder`
# the PATH is arranged so `command -v yog` resolves the driven build; run
# standalone — which is how `make drive-log` runs, and QUALITY.md §3 step 6
# REQUIRES an audit to start there ("Start it generated, not blank") — it
# resolves `~/.local/bin/yog`, the operator's live installed binary, a different
# sha, into the one field a drive log exists to pin (bl-d1af; §4: "a verdict is
# a claim about the sha it names, and about nothing newer"). It failed silently
# and plausibly, because the path looked right.
#
# So an unrecorded binary is LOUD in both halves — a marker in the field and a
# line on stderr — rather than a plausible default. Two distinct values are
# louder still: a ladder root whose runs drove different binaries is not one
# build's verdict at all, and the log says so instead of picking one.
# shellcheck disable=SC2086
bins=$(python3 - $files <<'PY'
import json, sys
seen = []
for path in sys.argv[1:]:
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            b = json.loads(line).get("bin") or ""
            if b and b not in seen:
                seen.append(b)
print("\n".join(seen))
PY
)
n=$(printf '%s' "$bins" | grep -c . || true)
if [ "$n" = 1 ]; then
  bin=$bins
  [ -e "$bin" ] \
    && binfield="\`$bin\` (mtime $(date -r "$bin" -u +%Y-%m-%dT%H:%M:%SZ))" \
    || binfield="\`$bin\` (**gone** — the driven binary no longer exists at that path)"
elif [ "$n" = 0 ] && [ -z "$files" ]; then
  # No rows at all, so there is nothing to have recorded a binary — a different
  # statement from rows that failed to (bl-d0a0), and saying the other one here
  # sent a reader looking for a harness defect that is not there.
  binfield="**NONE DRIVEN** — no verdict row exists to name a binary"
elif [ "$n" = 0 ]; then
  echo "logskel: no run under $dir recorded the binary it drove." >&2
  echo "  these rows predate bl-d1af, or were not written by harness.sh." >&2
  binfield="**UNRECORDED** — HAND-FINISH: these verdict rows name no driven binary,
  and this log will not guess one from PATH (bl-d1af)"
else
  echo "logskel: $n DIFFERENT binaries were driven under $dir." >&2
  binfield="**$n BINARIES DRIVEN** — HAND-FINISH: this is not one build's verdict —
$(printf '%s' "$bins" | sed 's/^/  - `/; s/$/`/')"
fi
read -r l1 l5 l15 _ </proc/loadavg

# Everything below is the emitted markdown, and every path in it — the scratch
# world root, the binary that was driven, each `verdicts.jsonl` — is absolute
# and rooted in the operator's home. That is what `scripts/leak-scan.sh`'s
# operator-home rule refuses, so an unfolded skeleton would generate a document
# the gate then refuses to commit (bl-244f burned the drive logs precisely to
# stop making an exception for it). The fold belongs HERE, in the one place
# that writes the text, not in the hands of whoever hand-finishes it: one
# stream, one substitution, before the operator ever sees a real path.
# stderr is deliberately outside it — diagnostics are for this terminal, not
# for the repo.
home_fold() {
  case "${HOME:-}" in
    ''|/) cat ;;
    *) sed "s|${HOME%/}|~|g" ;;
  esac
}

{

cat <<HEAD
# Drive log — HAND-FINISH: what this run set out to prove

- **Date:** $(date -u +%Y-%m-%d)
- **Build driven:** \`$sha\` on \`$branch\`, $tree — $binfield
- **Wire:** HAND-FINISH — which model, and whether it was up
- **Seat:** isolated Xvfb, claimed per run (\`scripts/drive/yogdrive.sh seat\`),
  scratch worlds under \`$dir\` (never \`~/.local/share/yog\`)
- **Harness:** \`make drive\` → \`scripts/drive/stories.sh <verb> <data> <out>\`;
  this table is emitted from each run's own \`verdicts.jsonl\`
- **Host tool tuple:** Xvfb \`$(command -v Xvfb || echo absent)\`;
  $(first xdotool --version); $(first ffmpeg -version | cut -d' ' -f1-3);
  $(first python3 --version); $(first git --version)
- **Machine:** $(uname -srm), $(nproc) cpu, load average $l1 / $l5 / $l15
HEAD

# THE STAGES, WITH THEIR OWN EXIT CODES (bl-d0a0) — `drive.sh` appends one row
# per verb it drives, so a stage that died before its first beat is named here
# even though it wrote no verdict anywhere. `wc -l` per stage is the join: a
# non-zero exit with zero rows is a run that never started, and a zero exit
# with zero rows is a harness that reported nothing and said it was fine.
# Absent for a run driven through `stories.sh` directly, which has no front
# door to write it — the section then simply does not appear.
if [ -f "$dir/stages.tsv" ]; then
  printf '\n## Stages driven\n\n| Stage | Exit | Verdict rows |\n|---|---|---|\n'
  while IFS=$'\t' read -r verb rc; do
    rows=0
    if [ -f "$dir/$verb/out/verdicts.jsonl" ]; then
      rows=$(wc -l <"$dir/$verb/out/verdicts.jsonl")
    fi
    if [ "$rc" = 0 ]; then code=0; else code="**$rc**"; fi
    echo "| \`$verb\` | $code | $rows |"
  done <"$dir/stages.tsv"
  echo
fi

# The beat table, read from the rows rather than transcribed from the scroll.
# python3 is already a harness prerequisite (beats_s7.sh) and is the only thing
# here that will not mangle a label carrying a quote, a pipe or an em dash.
# shellcheck disable=SC2086
python3 - $files <<'PY'
import json, os, sys

runs = []
for path in sys.argv[1:]:
    rows = []
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            line = line.strip()
            if line:
                rows.append(json.loads(line))
    if rows:
        runs.append((path, rows))

total = sum(len(r) for _, r in runs)
# A run with no rows at all is REPORTED, and this sentence is the report
# (bl-d0a0): zero beats is not an empty table, it is a finding — the run died
# before its first assertion, so nothing here is a claim about yog and the
# stage table above is where the run's own failure is named.
if not total:
    print("\n## Result summary — NO VERDICTS PRODUCED\n")
    print("**Zero beats ran.** No `verdicts.jsonl` exists anywhere under this")
    print("evidence root, so this log makes no claim about yog at all: the run")
    print("failed before its first assertion. Read the stage table above for")
    print("which stage it was, and the terminal for what it said.\n")
    raise SystemExit(0)
passed = sum(1 for _, r in runs for row in r if row["verdict"] == "PASS")
failed = total - passed
head = f"## Result summary — {total} BEATS: {passed} PASS"
print("\n" + (head if not failed else head + f", {failed} FAIL") + "\n")
if failed:
    print("**A failing beat is a finding, not a footnote** — every FAIL row below")
    print("is either a yog defect to file or a harness defect to fix, and the log")
    print("says which before it is filed.\n")

def cell(s):
    return s.replace("|", "\\|")

for path, rows in runs:
    verb = rows[0].get("run") or "?"
    out = os.path.dirname(path)
    ok = sum(1 for row in rows if row["verdict"] == "PASS")
    beats = "beat" if len(rows) == 1 else "beats"
    print(f"### `stories.sh {verb}` — {len(rows)} {beats}, {ok} PASS, {len(rows)-ok} FAIL\n")
    print("| Beat | Verdict | Evidence |")
    print("|---|---|---|")
    for row in rows:
        shot = os.path.basename(row.get("evidence") or "") or "—"
        note = row.get("detail") or ""
        ev = f"`{shot}`" + (f" — {cell(note)}" if note else "")
        mark = "PASS" if row["verdict"] == "PASS" else "**FAIL**"
        print(f"| {cell(row['label'])} | {mark} | {ev} |")
    print(f"\nEvidence: `{out}` (shots, `gestures.jsonl`, `verdicts.jsonl`)\n")
PY

cat <<'TAIL'
## What the screenshots show (the halves no assertion can reach)

HAND-FINISH. Name the shot, then quote what is in it: evidence quoted, not
summarized, is the house style. Every beat asserting a *negative* is only as
strong as its screenshot, so those are the ones to describe.

## ops.jsonl, in order

HAND-FINISH. Paste the run's ops rows and read the order out loud; the argv
order is itself an assertion.

## What this does not prove

HAND-FINISH. Honest per-row verdicts for what was drivable and not driven,
what needs a fixture the runner does not lay, and what is not drivable yet.
A verdict is a claim about the sha above and nothing newer (QUALITY.md §4).
TAIL

} | home_fold
