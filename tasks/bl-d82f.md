+++
title = "bump the embedded lernie pin past c816ee8 when the next lernie releases: yog's `lernie scan` still 128s on an out-of-grammar agents/* id"
created = 1785133675
updated = 1785287188
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
yog embeds lernie as a library dep pinned to the crates.io release —
`Cargo.toml`: `lernie = "=0.0.1"` — and every `lernie <verb>` yog spawns runs
through the §16.7 W12 self-multiplex into THAT crate, never the `lernie` on
PATH (the world seeds `world/tools/lernie` as a shim: `exec <yog> lernie "$@"`).

**The pin predates lernie `c816ee8`** (lernie/bl-025b), which hardened the
`lernie scan` sweep: the parent address it derives from an agent id is now
intersected with the `agents/*` registry, so a branch whose derived parent holds
no ref is treated as a root instead of being asked of git.

Measured on the bl-c03e drive world (`agents/<root>-c0ffee` laid by hand, one
token short of a §2.3 descent segment):

```
--- embedded (yog lernie scan <ws>) ---
lernie scan: git ls-tree messages: git ["ls-tree","-r","--name-only",
  "agents/20260727T064848Z","--","messages"] exited with exit status: 128:
  fatal: Not a valid object name agents/20260727T064848Z
exit=1
--- installed (PATH lernie scan <ws>, at/past c816ee8) ---
silent deaths: 0; died deposits swept: 0; drivers launched: 0; inboxes with
no agent branch: 0
exit=0
```

## Exposure is real but narrow

Neither lernie nor yog ever *mints* an out-of-grammar id, so this needs a
hand-laid or hand-deleted ref to trigger. When it does trigger, one bad branch
aborts the WHOLE scan pass — the flush never runs, so pending mail everywhere in
the workspace stays undelivered, and yog surfaces only git's raw 128.

## Not blocking anything today

bl-c03e's S7-T4 beat asserts `lernie scan` exits 0 and is green on the pin,
because world C's fixture now mints a well-formed `<ts>-<short>` child segment
(`CHILD_SEG` in `scripts/drive/beats_s7.sh`) — the shape lernie would actually
produce. That beat is this task's acceptance too: it must stay green across the
bump.

## Done when

`Cargo.toml` pins a published lernie at or past `c816ee8`, `Cargo.lock` is
committed with it, `cargo test` is green, and `scripts/drive/stories.sh run-s7`
is green on a per-run seat.

---

## UPDATE 2026-07-28 (scorched-d82f) — already delivered by bl-4014, criteria met on main

Checked crates.io: `cargo info lernie` reports **0.0.2** as the published
latest. In `~/dev/lernie`, `git merge-base --is-ancestor c816ee8 v0.0.2` exits
0 — **c816ee8 IS included** in the published 0.0.2 release. This qualifies per
this ball's own gate.

But the bump is already done. A separate ball, **bl-4014** ("yog links lernie
=0.0.1 but 0.0.2 is the published latest — bump the registry pin so W11
consumes upstream's current release"), landed this exact change on `main` as
commit `798b75c` and has since been closed (it no longer appears in `bl list`,
only in `bl list --all`). `Cargo.toml` on `main` already reads
`lernie = "=0.0.2"` with an updated comment citing the 2.11 lost-wakeup fix.

I claimed this ball and got a `work/bl-d82f` worktree off current `main` — it
has **zero diff** against `main` (`git diff main -- Cargo.toml` empty,
`git status` clean). There is nothing left to bump.

**cargo test could not be verified green**, but not for a reason related to
lernie or this ball: 66 tests fail identically on plain `main` (verified by
running the same failing test directly on the `main` checkout, no worktree
involved) with `commit refused: AUTHOR email is <t@t.local>, not
<mudbungie@gmail.com>` — this machine's global `core.hooksPath`
(`~/userconf/githooks-global`) enforces commit-identity on every repo,
including the throwaway git fixtures yog's own test suite spins up and commits
into as `t@t.local` (`src/git_tree/tests/git.rs`). This is a pre-existing,
machine-local environment conflict, not a code or lernie-version defect, and
is out of scope for this ball to fix (touches a personal dotfiles hook outside
this repo).

Given zero diff and this pre-existing unrelated test-environment issue, I did
not run `scripts/drive/stories.sh run-s7` (heavy Xvfb e2e, not warranted for a
no-op worktree) and I am **not closing this ball** as done — the acceptance
text asks for a green `cargo test`, which I cannot certify here for reasons
unrelated to the pin. Unclaiming rather than closing; recommend either (a)
closing bl-d82f as a duplicate/no-op now that bl-4014 already satisfied the
substantive requirement, once someone confirms cargo test is expected to be
run in an environment without the conflicting global hook (e.g. CI), or (b)
filing the hook/test-fixture conflict as its own ball if it should be fixed
generally.