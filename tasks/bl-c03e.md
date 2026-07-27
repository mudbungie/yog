+++
title = "S7-T4 beat still asserts only that Flush dispatched lernie scan; the upstream exit-1 is fixed, so tighten it to exit 0 and correct the comment's wrong diagnosis"
created = 1785133141
updated = 1785133514
claimant = "waxier-c03e"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
The residual of **bl-a942**, whose upstream half is fixed and delivered
(lernie `c816ee8`, task lernie/bl-025b).

## What the beat says today

`scripts/drive/beats_s7.sh`, `s7_inbox()`:

```sh
  # Dispatch, not exit 0, and that is deliberate: on this tuple `lernie scan`
  # itself exits 1 whenever a ROOT agent holds pending mail (it derives a parent
  # branch from the recipient id and a root has none — `fatal: Not a valid object
  # name agents/<truncated>`). yog's job is to dispatch the verb and surface the
  # outcome verbatim, which it does; the upstream failure is filed, not hidden.
  until_landed flush verb_ge scan 1 \
    && pass "S7-T4 inbox: Flush dispatches lernie scan" \
    || fail "S7-T4 inbox: Flush dispatches lernie scan" "no scan verb"
```

## Two things to change

**1. The comment states a diagnosis that is wrong.** A root with pending mail
scans clean — verified against a real workspace, exit 0, `drivers launched: 1`.
The actual trigger is the *sibling* the same fixture lays two dozen lines up in
`lay_forensics()`:

```sh
  git --git-dir="$g" branch "agents/$ag-c0ffee" "$tip"
```

`lernie scan`'s sweep derived that branch's parent address by token arithmetic
over the hyphenated descent — a segment is `<ts>-<short>`, i.e. exactly two
hyphen-free tokens (lernie ARCH §2.3) — so a **one-token** suffix makes three
tokens, the last two get stripped, and the sweep asked git for
`agents/20260727T031141Z`, a ref that never existed. Git's 128 aborted the whole
pass *before the flush ran*. The mail was a bystander; the branch did not need
any.

lernie now intersects the derived parent address with the `agents/*` registry
(the same intersection its flush already applied to the inbox listing), so a
branch whose derived parent holds no ref is treated as a root: nothing to
deposit, nothing asked of git. `lernie scan` on this exact fixture now exits 0.

**2. Tighten the assertion**, as bl-a942 specified: add `row_ok
'"lernie","scan"'` (`scripts/drive/stories.sh:68`) alongside the dispatch check,
so the beat asserts the outcome and not merely that the verb was spawned.

## Prerequisite — this will fail against a stale install

yog dispatches the `lernie` on `PATH`, not the one in `~/dev/lernie`. The fix
landed on lernie `main` and is **not** in the 0.0.1 crates.io release. Run `make
install` in `~/dev/lernie` (or otherwise put a build at or past `c816ee8` on
`PATH`) before running the drive, or the tightened beat fails for an
install-drift reason and not a yog one.

## Note, not a defect

yog's fixture comment reads §2.3 as *"hierarchy lives in the id, so a
hyphen-descent branch off the root's tip IS a member"*. lernie's grammar is
narrower: a descent **segment** is two hyphen-free tokens, so `<root>-c0ffee` is
an id lernie would never mint. It stays a legal branch name and lernie no longer
chokes on it, and the S7-T5 member beat passes on it — but a fixture that meant
to be lernie-shaped would use `<root>-<ts>-<short>`. Worth deciding once, since
the same shape feeds yog's own member derivation.

## Done when

The beat asserts exit 0, the stale comment is replaced by the above, and the
drive is run green against an installed lernie at or past `c816ee8`.