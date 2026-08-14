+++
title = "branch cleanup runs only where nothing leaks: trap speculate-gate's delete, and widen the prune to every branch with no open PR"
created = 1786686041
updated = 1786686041
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Follows bl-7066, which wrote the branch model down. This makes the machine keep
it. Mark ruled 2026-08-13 on both halves after the alternative — a `close.post`
plugin — was rejected: close fires only where the owner survived, which is
precisely where nothing leaks.

**Why not close.post.** The branches that strand are the ones whose owner never
reaches close: a killed `speculate-gate`, an abandoned probe, an `unclaim`, a
dead agent. On the happy path it is redundant (`speculate-gate` deletes its own
branch; the pusher is alive and the rule now tells them to). And it could not
find the garbage anyway — Anvil's probe was `anvil-measure-1015`, not
`work/bl-1015`, so a ball-id-keyed hook has nothing to match, and making it
match would mean mandating a naming convention to prop up the hook.

**Half 1 — `scripts/speculate-gate` does not trap its own delete.** It pushes at
line 30 and deletes at lines 46 and 61, both `|| true`, and the only trap is
`trap 'rm -rf "$verdicts"' EXIT`. Anything between the push and the delete —
SIGINT, SIGTERM, a dropped network, an `exit 1` path added later — strands
`speculation/<sha>` on the PUBLIC remote. The script's own header already
concedes it:

    #      sweep by hand: git push origin --delete speculation/<sha>);

Trap the delete on EXIT INT TERM HUP and drop the two hand-rolled calls.

**One trap that must be handled with it: line 64 is `exec bl-speculate check`.**
`exec` REPLACES the shell, so an EXIT trap never runs on the success path — the
one path that matters most. It has to become a plain call so the trap fires,
with the exit status still the check's. SIGKILL remains untrappable; say so
rather than imply the hole is closed.

**Half 2 — `prune-release-branches` has the right test and the wrong glob.** Its
own comment argues the principle:

    The repo's "Automatically delete head branches" setting only fires on
    MERGE, which covers the happy path and nothing else. This job covers the
    rest: it runs immediately after the PR job, so the branch backing the
    CURRENT release PR is already open, and "has no open PR" is therefore an
    exact test for "superseded or orphaned". No allowlist, no age heuristic,
    no state to keep — the open-PR set is the single source of truth.

It then globs `refs/heads/release-plz-*` only.

That test generalizes exactly, for a reason worth stating in the job: `ci.yml`
triggers on `push: branches: [main]` and `pull_request` — NOT on pushes to
arbitrary branches. A probe branch with no PR therefore receives no CI at all,
so it cannot be a live probe; it is debris by construction.

Widen it to delete every remote branch that is not `main`, not `balls/tasks`,
not `speculation/**`, and backs no open PR. The `speculation/**` exclusion is
load-bearing and must be commented: `speculate.yml` runs on
`push: branches: 'speculation/**'`, so a LIVE speculation branch has no PR and
would otherwise be swept out from under a running gate.

Keep "no age heuristic". A false delete costs a re-push and nothing else — the
work lives in the claim worktree, never on the branch.

Done when a stranded branch with no open PR is deleted by the next push to main,
and killing `speculate-gate` with SIGTERM mid-run leaves no `speculation/*`
behind.