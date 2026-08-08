+++
title = "local CICD triggers on the close verb, not on main moving: rebuild+reinstall whenever refs/heads/main changes, however it changed"
created = 1786162403
updated = 1786162413
claimant = "ref-trigger"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Local CICD (`scripts/bl-install-main`, registered `close.post` in
`config/plugins.toml`) rebuilds `main`'s tip, `make install`s it and
`make reload`s a running window. It works — it fired on bl-f908's close and
exited 0.

But its trigger is **the verb, not the fact**. It fires because `bl close`
ran, so `main` moving any other way installs nothing:

- `git pull` / `git merge` on main (another machine's work, or a co-worker's);
- a `git push` received into this repo;
- a hand-repaired main (`git reset`, `git revert`, a squash landed by hand).

In each case `~/.local/bin/yog` silently keeps serving an older tip, which is
exactly the drift already recorded against the other installed binaries.

## The ruling: the trigger is `refs/heads/main` moving

One fact — *main's tip is not what is installed* — should drive one mechanism.
Move the trigger onto the ref:

- a repo hook that fires when `refs/heads/main` is updated, whatever moved it
  (`reference-transaction` on the `committed` state names the ref and both
  oids; `post-merge` alone misses bl's plumbing and a received push);
- **it must be idempotent and cheap when nothing changed**: compare main's tip
  against what is installed before building, so a no-op ref write costs a
  string compare, not a release build. Where "what is installed" is recorded
  is part of the design — a stamp file beside the binary is the obvious
  answer, but check whether the build already leaves one before adding a
  second copy of the fact.
- it must keep `bl-install-main`'s hard-won contract verbatim: always exit 0
  (a non-zero close.post rolls the close back), never block the caller
  (detached via `setsid`, outcome to `target/cicd-install.log`), and build from
  an ephemeral `git worktree --detach main` sharing `CARGO_TARGET_DIR` — never
  from the repo root, which `bl close` leaves stale.

**Then subtract**: if the ref trigger covers the close, the `close.post`
registration of `bl-install-main` is a second path to one outcome and should
go, leaving `bl-push-main` (a genuinely different job: delivery to origin)
alone. Do not leave both firing — the double build is the smell that says the
trigger was in the wrong place.

## The machine's hook chain, read before writing a hook

`core.hooksPath` is global (`~/userconf/githooks-global`), one script symlinked
under every hook name, and its documented second job is: *"Everywhere else, get
out of the way: the repo's own .git/hooks/<name> is exec'd if it is there."* So
a repo-local hook does run — verify that for the specific hook you pick rather
than assuming it, and note that `~/.git/hooks` is not tracked by the repo, so
whatever you add needs an install path (`make install-hooks` already exists —
extend it rather than inventing a second installer).

## Acceptance

Prove it, don't assert it: move `main` by each route (a `bl close`, a plain
`git merge` into main, a `git reset --hard` forward) and show the log rebuilt
exactly once for each, and not at all when main did not move.