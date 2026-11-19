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
# with `GH_TOKEN` set to the JOB token (`actions: write`; the release token is
# refused a workflow dispatch, HTTP 403 — bl-37d8), `PAT` set to the
# repository's `RELEASE_PLZ_TOKEN`, and the checkout made with that same token
# so git pushes with it. The caller has already decided there is something to land.
#
#   1. The named paths committed on `<branch-prefix>/<utc-stamp>` and pushed.
#   2. `ci.yml` dispatched on that branch and watched. That workflow is the one
#      definition of green (`make ci` is `make check`); nothing here restates a
#      step of it.
#   3. On a pass, `main` fast-forwarded to the branch. The branch is deleted
#      whichever way this ends: the run's log is the record, the branch was
#      only its seat.
#   4. Nothing: the release follows by itself. The push in (3) is made with
#      the release token, so it is an ordinary push to `main` — CI runs,
#      `release-plz.yml` opens the release PR, `release-automerge.yml` merges
#      it.
#
# WITHOUT THE RELEASE TOKEN IT REFUSES, before it writes anything (bl-3102).
# A push by the default `GITHUB_TOKEN` triggers no workflow, and here nothing
# can stand in for it: `release-plz.yml` opens a release PR only on a `push`
# (its dispatch is a binaries backfill that never publishes) and
# `release-automerge.yml` merges nothing without the token. A landing made
# without it would be a change on `main` no release ever follows — so there is
# one path, and the lack of it is loud.
#
# NEITHER REFUSAL TOUCHES `main`, AND ONLY ONE IS RED. A red `ci.yml` means the
# change broke something real and is a hand's: exit 1. A refused fast-forward
# means `main` moved while the gate ran, and the caller's next run begins again
# from the new `main` with no hand involved — so it says so and exits 0 (bl-1f1b,
# operator ruling): a run that heals itself must not open an alarm. That is
# forgiven only when `main` is SEEN to have moved; a push refused while main's
# tip is still inside the branch (permissions, a protection rule) is exit 1.
set -euo pipefail

[ $# -ge 3 ] || { echo "usage: $0 <branch-prefix> <commit-message> <path>..." >&2; exit 2; }
prefix=$1 message=$2
shift 2
[ -n "${PAT:-}" ] || {
  echo "RELEASE_PLZ_TOKEN is not set: a push without it triggers no CI and no release on main; landing nothing" >&2
  exit 1
}

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
git push origin "$branch:main" && exit 0
# Refused. Forgiven only when main really moved: its tip still inside the
# branch means nothing landed underneath, and the refusal is something else.
git fetch -q origin main
if git merge-base --is-ancestor FETCH_HEAD "$branch"; then
  echo "the push to main was refused and main did not move: this is a hand's" >&2
  exit 1
fi
echo "main moved while the gate ran: nothing landed; the next run begins from the new main" >&2
