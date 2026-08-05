+++
title = "the pre-commit gate omits make lint, so clippy errors land on main"
created = 1785891988
updated = 1785891997
claimant = "gate-mender"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
VERIFIED 2026-08-04 against .git/hooks/pre-commit and the Makefile, not inferred.

## The hole

`make check` is `fmt-check -> lint -> coverage` (Makefile:170). The pre-commit hook — the gate `bl close` actually runs — enforces its own list of five things (hook header lines 4-18): mainline-commit refusal, `make fmt-check`, `make line-cap`, `make rules-audit`, and tarpaulin `--fail-under 100`. **`make lint` is not among them.** So clippy (and cargo-deny) never run at close, and a clippy error delivers to main clean.

## It already happened

bl-3746 (647bda3) landed a `clippy::semicolon_if_nothing_returned` error in `src/inspector/mod.rs:198`. main failed `make lint` from that commit until bl-f6fe (d05c361) fixed the semicolon — that agent could not otherwise pass its own gate, so it absorbed an unrelated repair to make progress. The next agent inherits a failure it did not write, which is exactly the failure mode the hook's own comment for item 2 records for fmt: "Without this step a fmt-drifted close still passed the gate and landed on main, so the NEXT `make check` failed on someone else's inherited hunk (bl-92e2) — one gate, one definition".

That reasoning was applied to fmt and not to lint. Same hole, same shape, one rung over.

## Wanted

Close the gap the way the hook already closed it for fmt: the gate reuses `make check`'s definition rather than restating a subset of it, so the three seats (hook, `make check`, CI `make ci`) cannot drift apart again. Decide deliberately where lint sits in the order — it is slower than fmt/line-cap/rules-audit and much faster than tarpaulin, so before coverage is the fail-fast placement.

Verify the current Makefile and hook before editing; do not trust this body's line numbers.