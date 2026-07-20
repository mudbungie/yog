+++
title = "Local CICD: install main to $PATH on close (bl-install-main plugin)"
created = 1784518803
updated = 1784518804
claimant = "Floundered"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["build"]
+++
Add a close.post plugin `scripts/bl-install-main` (sibling of bl-push-main = the local half of delivery). On a bl-close landing it compiles the `main` ref and `make install`s yog into $PATH.

Contract mirrors bl-push-main: protocol handshake serves `close`; drains stdin; ALWAYS exits 0; resolves repo from its own on-disk home.

Correctness: build from an EPHEMERAL `git worktree --detach main` (root tree is stale after plumbing delivery and may hold uncommitted work), sharing repo target/ via CARGO_TARGET_DIR. Detach the build (setsid) so close returns at once; log to target/cicd-install.log.

Makefile: make `install` honor CARGO_TARGET_DIR so the shared-target build's binary is found.

Wiring (post-close, done at repo root): bl conf append close.post bl-install-main; bl install --bin bl-install-main=scripts/bl-install-main; then seed one install.