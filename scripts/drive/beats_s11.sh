#!/bin/bash
# beats_s11.sh — **S11 Auditor's headless rung**, fired from `run_headless`
# (beats_headless.sh) after the fixture `/assign` has cut a real worktree, and
# before `beats_s19.sh` reads the work it lays. Sourced by stories.sh; not an
# entry point of its own.
#
# The only rung in the seatless family whose subject is GIT. Everything else
# there is balls state or a boundary reply; this is a real `bl claim` worktree
# off `main`, a real commit landing in it, and `Query::WorkDiff` as a pure git
# read of `target..source`. A fake substrate can answer with a diff-shaped
# reply; only a real one can be wrong about what the diff contains.

# A real `bl claim` cut a real worktree off `main`; a commit lands in it; the
# query is a pure git read of `target..source`. Asserted by IDENTITY at every
# field — the file's name and its ±, both refs, both oids present — because a
# reply that merely has the right SHAPE is what an empty diff also has.
s11_workdiff() {
  data=$1 ; ws=$2 ; claim=$3 ; wt=$4
  if [ -z "$wt" ] || [ ! -d "$wt" ]; then
    fail "S11-T4 work-diff: the agent's commit is the ball's diff" "no worktree to commit in"
    return 0
  fi
  printf 'the agent wrote this\n' > "$wt/agent-wrote-this.txt"
  git -C "$wt" add -A
  git -C "$wt" commit -qm "agent work" --no-verify
  gesture "$data" /work-diff --ws "$ws" || true
  reply_is 'd["ok"] and [r for r in d["rows"] if r["ball_id"]=="'"$claim"'"
      and r["source"]=="work/'"$claim"'" and r["target"]=="main"
      and r["source_oid"] and r["target_oid"] and r["source_oid"]!=r["target_oid"]
      and [f for f in r["files"]
           if f["path"]=="agent-wrote-this.txt" and f["added"]==1 and f["removed"]==0]]' \
    && pass "S11-T4 work-diff: the agent's commit is the ball's diff" \
    || fail "S11-T4 work-diff: the agent's commit is the ball's diff" "no matching diff row"
}
