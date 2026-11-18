#!/usr/bin/env bash
# **How an unattended change reaches `main`** — the one definition, called by
# every workflow that writes there without a ball: `advisories.yml` (the day's
# lockfile mend) and `ingest.yml` (the litany/brazen pin move, bl-38dc). It was
# advisories.yml's own step until a second caller arrived; two copies of the
# steps that write to `main` would have drifted within a week.
#
#   scripts/land-through-gate.sh <branch-prefix> <commit-message> <path>...
#
# Run on a runner, in a FULL-history checkout (a shallow one has the branch's
# parent beyond its boundary and every fast-forward is refused "fetch first"),
# with `GH_TOKEN` set and `PAT` set to the repository's `RELEASE_PLZ_TOKEN` or
# empty. The caller has already decided there is something to land.
#
#   1. The named paths committed on `<branch-prefix>/<utc-stamp>` and pushed.
#   2. `ci.yml` dispatched on that branch and watched. That workflow is the one
#      definition of green (`make ci` is `make check`); nothing here restates a
#      step of it.
#   3. On a pass, `main` fast-forwarded to the branch. The branch is deleted
#      whichever way this ends: the run's log is the record, the branch was
#      only its seat.
#   4. The release re-judged. A push by the default `GITHUB_TOKEN` triggers no
#      workflow, so `release-plz.yml` is dispatched; where `PAT` is set the
#      push was made with it, triggered the chain itself, and no dispatch is
#      sent.
#
# BOTH FAILURES LEAVE `main` UNTOUCHED. A red `ci.yml` means the change broke
# something real and is a hand's. A refused fast-forward means `main` moved
# while the gate ran; the caller's next run begins again from the new `main`.
set -euo pipefail

[ $# -ge 3 ] || { echo "usage: $0 <branch-prefix> <commit-message> <path>..." >&2; exit 2; }
prefix=$1 message=$2
shift 2

branch="$prefix/$(date -u +%Y%m%d-%H%M)"
git config user.name "github-actions[bot]"
git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
git switch -c "$branch"
git commit -qm "$message" -- "$@"
git push -u origin "$branch"
# The branch was only the run's seat: gone whichever way this ends.
trap 'git push origin --delete "$branch" || true' EXIT
gh workflow run ci.yml --ref "$branch"
# A dispatch answers before its run exists; find the run by its branch.
run=""
for _ in $(seq 1 30); do
  run=$(gh run list --workflow ci.yml --branch "$branch" --event workflow_dispatch --limit 1 --json databaseId --jq '.[0].databaseId // empty')
  [ -n "$run" ] && break
  sleep 5
done
[ -n "$run" ] || { echo "ci.yml never started on $branch" >&2; exit 1; }
gh run watch "$run" --exit-status
git push origin "$branch:main" || {
  echo "main moved while the gate ran: nothing landed; the next run begins from the new main" >&2
  exit 1
}
if [ -z "${PAT:-}" ]; then
  gh workflow run release-plz.yml --ref main
fi
