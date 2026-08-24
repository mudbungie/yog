+++
title = "macOS-only: multiplex landing repair test fails with NotFound on the balls clone path — /var vs /private/var canonicalization is the suspect"
created = 1787546576
updated = 1787546576
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["flake", "macos"]
+++
## The failure

`multiplex::landing::tests::the_repair_is_idempotent` fails on the macOS CI leg
(run 32672624965, on the release-v0.0.5 PR; 2670 passed, this 1 failed):

    panicked at src/multiplex/landing/tests/mod.rs:85:48:
    first: Custom { kind: NotFound, error: "git status
      (<tmp>/yog/world/state/balls/clones/%2F<percent-encoded tmp path>%2Fproj/config):
      No such file or directory" }

Linux is green on the same tree, and macOS is reported-but-not-gating
(`.github/workflows/macos.yml`), so nothing is blocked — but a leg that stays
red rots into being ignored, which is how the last two macOS breaks (bl-1015)
got to 14 tests deep before anyone looked.

## The suspect

The balls clone path is keyed on the **percent-encoded literal invocation
path**. On macOS, `$TMPDIR` lives under `/var/folders/…`, and `/var` is a
symlink to `/private/var` — so any step that canonicalizes (git itself does,
`std::fs::canonicalize` does) produces `/private/var/…` while a step that
percent-encodes the uncanonicalized path produces `%2Fvar%2F…`. Two spellings
of one directory ⇒ the store is founded under one key and looked up under the
other ⇒ NotFound. Linux has no such symlink, which is exactly the observed
split.

Check where the test (or the landing repair it drives) derives the clone key
versus where the fixture creates it; the fix is to canonicalize at ONE place
before encoding — or in the test fixture, to canonicalize the tmp dir before
handing it to anything, which is what several other tests in this tree already
do for the same reason. Verify against the tree; this body reasons from the
log, not from the code.