+++
title = "bump the embedded lernie pin past c816ee8 when the next lernie releases: yog's `lernie scan` still 128s on an out-of-grammar agents/* id"
created = 1785133675
updated = 1785287121
claimant = "scorched-d82f"
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