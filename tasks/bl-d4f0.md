+++
title = "make run tracks main: launch the installed binary and relaunch it when a merge lands"
created = 1786508786
updated = 1786508787
claimant = "waltzing"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
`make run` is `cargo run --quiet` — a debug build of the WORKING TREE, compiled once at launch. It never sees a merge: nothing rebuilds it and nothing restarts it when refs/heads/main moves. The CICD half that does track main (.githooks/reference-transaction -> scripts/install-main -> make install -> make reload) can only relaunch a pidfile-recorded instance, and `run` records no pid.

Operator ask: `make run` should launch the LIVE version of main and close/recompile/relaunch as new merges hit.

Shape: `run` becomes the door to a small supervisor (scripts/run-main), keeping the machinery out of the Makefile the way `drive` does. The supervisor launches $(INSTALL_BIN)/yog, watches $(INSTALL_STAMP) — the ONE record of which commit the installed binary was built from — and relaunches the window when it changes. It also dispatches scripts/install-main when main's tip differs from the stamp, so the verb is self-sufficient when this clone's hooks are not armed (the dispatch is the same idempotent convergence the hook uses; it no-ops when nothing is stale).

It must NOT write YOG_PIDFILE: `make reload` kills that pid and launches a DETACHED replacement, which beside a foreground supervisor is two windows. run owns its own relaunch.