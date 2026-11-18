#!/bin/sh
# **A red unattended run reaches a person, once** (bl-3102). Every scheduled
# workflow ends with one of two steps:
#
#   scripts/alarm.sh red      # `if: failure()`
#   scripts/alarm.sh green    # `if: success()`
#
# A schedule that fails tells nobody: `advisories.yml` was red every day for a
# week before anyone looked, and `ingest.yml` fails once an HOUR for as long as
# an upstream release needs a hand. So the first red run of a streak opens an
# issue that mentions the repository owner — the mention is what makes GitHub
# send the mail — and every later red run finds it open and says nothing. The
# next green run closes it, which re-arms the alarm for the next streak.
#
# THE OPEN ISSUE IS THE STATE. Its title is exactly `unattended run red:
# <workflow name>`, it is looked up on every run and stored nowhere, so there
# is no counter to drift and closing the issue by hand is a lawful way to ask
# to be told again. No label: `gh issue create` fails on one that does not
# exist, and an alarm that needs provisioning is an alarm that is not there.
#
# It reads only what a runner already exports, and `GH_TOKEN` (the job token,
# `issues: write`).
set -eu

case ${1:-} in
    red|green) ;;
    *) echo "usage: $0 red|green" >&2; exit 2 ;;
esac

# gh reads the repository off this, not off whatever checkout it stands in.
export GH_REPO="$GITHUB_REPOSITORY"
title="unattended run red: $GITHUB_WORKFLOW"
run="$GITHUB_SERVER_URL/$GITHUB_REPOSITORY/actions/runs/$GITHUB_RUN_ID"

# Exact title, compared here: a search is a substring match, and one
# workflow's name may be the head of another's.
tab=$(printf '\t')
open=$(gh issue list --state open --limit 1000 --json number,title \
        --jq '.[] | "\(.number)\t\(.title)"' \
    | while IFS=$tab read -r _n _t; do
        [ "$_t" != "$title" ] || echo "$_n"
    done | head -1)

case "$1:${open:+open}" in
    red:)
        gh issue create --title "$title" --body "@$GITHUB_REPOSITORY_OWNER the unattended workflow \`$GITHUB_WORKFLOW\` went red: $run

This is the first red run of the streak. Later red runs add nothing here, and the next green run closes this issue." ;;
    green:open)
        gh issue close "$open" --comment "$run is green" ;;
esac
