+++
title = "<workspace> storm debris: 179 stale refs/lernie/budget-exhausted refs keep 220 compactor commits alive — cleanup procedure drafted, AWAITING OPERATOR SIGN-OFF"
created = 1785460448
updated = 1785644902
claimant = "Riffle"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["investigation"]
+++
## Status: AWAITING OPERATOR SIGN-OFF. Do not execute any command below.

Carried out of bl-ebbd, which closed against the lernie 0.0.3 fix for the defect
itself. The debris is a separate, destructive act that needs the operator's yes.

## The state has changed since bl-ebbd was filed — read this first

bl-ebbd's body describes 227 branches on `refs/heads/agents/*` and a live
conflict-markered `summary/001.md` at the root branch tip. **Neither is true any
more.** Re-surveyed read-only 2026-07-30 in
`/home/u/.local/share/yog/workspaces/<workspace>/repo.git`:

| observation | command | value |
|---|---|---|
| agent branches | `git for-each-ref 'refs/heads/agents/*' \| wc -l` | **2** — `<agent-id>`, `<agent-id>`. No `3b14deaf` head survives. |
| storm root tip | `git rev-parse agents/<agent-id>` | **unknown revision** — the branch is gone |
| budget-exhausted refs | `git for-each-ref 'refs/lernie/budget-exhausted/*' \| wc -l` | **179** — all under the `3b14deaf` descent |
| conflicted refs | `git for-each-ref 'refs/lernie/conflicted/*' \| wc -l` | **0** |
| compactor dispatch commits still reachable | `git log --all --oneline --grep='^dispatch: compactor' \| wc -l` | **220** — reachable only via the 179 refs above |
| the conflicted summary | `git show 1596809d…:summary/001.md \| grep -c '^<<<<<<<\|^=======\|^>>>>>>>'` | **9** marker lines; the object survives but `for-each-ref --contains 1596809d…` is **empty** — reachable from no ref |
| worktrees | `git worktree list \| wc -l` | 3 (bare + 2) |
| `agents/`, `inbox/`, `steps/` dirs | `ls … \| wc -l` | **2 each**; zero `3b14deaf` entries |

So the branch heads, worktrees, inboxes and step transcripts of the storm are
**already gone**. Who removed them is **unknown** — `repo.git/logs/refs/heads/agents/`
carries logs for the two surviving branches only, so this repo holds no record of the
deletion. It was not `yog delete`: that verb unmakes a whole workspace directory
(`src/delete/mod.rs` — "removal of the workspace directory"), and `<workspace>`
is intact.

What is actually left is: **179 stale `refs/lernie/budget-exhausted/*` refs**, the
220 commits they keep alive, and three directory inodes inflated by the entries they
briefly held (`agents/` and `inbox/` at 49152 bytes, `steps/` at 20480, for 2 entries
each).

## Corroboration worth keeping before anything is deleted

The 179 refs are the evidence that lernie 0.0.2's `max_depth` check **did** fire and
fired **too late** — precisely the gap lernie 0.0.3 closes by moving the gate ahead of
the fork. Depth census of the ref names (levels below the root):

    depth 2:   2 refs
    depth 3:  11 refs
    depth 4:  54 refs
    depth 5: 112 refs

`max_depth: 4` exhausts at `depth > 4`, so the 112 at depth 5 are depth exhaustions;
the 67 shallower ones exhausted on `max_total_tokens: 2000000` / `max_wall_seconds`
across the whole tree. Every one of those branches was minted — branch, worktree and
inbox — before being refused. Deleting the refs destroys this record.

## The procedure

Run from `/home/u/.local/share/yog/workspaces/<workspace>/`.

**Step 0 — archive first (recommended, non-destructive).** Everything below is
unrecoverable without this.

    git -C repo.git bundle create ~/<workspace>-storm-20260728.bundle \
      --all

**Step 1 — delete the 179 budget-exhausted refs.**

    git -C repo.git for-each-ref --format='delete %(refname)' \
      'refs/lernie/budget-exhausted/*' | git -C repo.git update-ref --stdin

Touches: only `refs/lernie/budget-exhausted/*`. Touches no `refs/heads/*`, no
`config/*`, no worktree, no file outside `repo.git`. After this the 220
`dispatch: compactor` commits are unreachable but still on disk.

**Step 2 — drop the now-unreachable objects.**

    git -C repo.git reflog expire --expire=now --all
    git -C repo.git gc --prune=now

Touches: the object store. This is the **unrecoverable** step — it destroys the 220
storm commits and, with them, the last copy of
`1596809d26fa89e2f01cb2c2c22cb4501ff62209:summary/001.md`, the nine-marker-line
triple-conflicted summary that is the only surviving artifact of bl-ebbd's third
sub-claim. Skipping step 2 costs only disk; step 1 alone already unclutters every
`for-each-ref` walk.

**Step 3 — the inflated directory inodes (optional, cosmetic).** `agents/`,
`inbox/` and `steps/` hold 2 entries each but occupy a 48K/48K/20K directory block
each. Only a recreate shrinks them:

    # for each of agents, inbox, steps — with NO driver running in this workspace
    mv agents agents.old && mkdir agents && mv agents.old/* agents/ && rmdir agents.old

Touches: live workspace state. **Racy** — a running driver holds an inbox flock and
writes step files; do this only with the workspace quiet, or not at all. The cost of
not doing it is one wasted `getdents` block per scan. Recommend: **skip**.

## Recommendation

Steps 0 + 1. Hold step 2 until the lernie 0.0.3 fixes have been live-verified against
a real storm-shaped run; hold step 3 indefinitely.