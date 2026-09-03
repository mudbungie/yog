+++
title = "fan's lib beats fork inside balls: Attempt::open/resume/deliver leave the ETXTBSY class open after bl-6bf5"
created = 1788415404
updated = 1788416313
claimant = "Spellbind-P"
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["flake", "tests"]
+++
`fan`'s beats drive balls' attempt machinery in-process — `Attempt::open`,
`Attempt::resume`, `Attempt::deliver`, `Project::at(..).target(..)` — and every
one of those forks `git` inside balls, outside `git_env`'s `cfg(test)` spawn
lock. That makes the lib test binary an unlocked forker again for the whole
time `fan::` is running, so any peer thread's write fd on a fixture script it
has just written is copied into a balls child and the exec that follows is
**Text file busy**.

Measured on a 16-core box, one filter over the lib test binary
(`fan::` plus the five fixture-exec families:
`start::tests::exec`, `cli_outbound::tests::run::spawned`,
`cli_outbound::tests::detach`, `cli_outbound::piped::tests`,
`boundary::login::tests`), 16 workers x 40 and again x 70 iterations each:
**2 ETXTBSY failures** each time. The same victims with no substrate beat in
the filter at all: **0**.

This is the residual bl-6bf5 left standing, and it is the harder half.
bl-6bf5 took shape 2 (move the in-process substrate out of the lib binary) and
that worked for `multiplex::landing`, whose beats only needed three functions
made `pub`. It does not transfer here:

- `fan`'s beats are unit tests of `pub(crate)` internals, and `tests/*.rs` can
  reach only the `pub` surface.
- The fixture (`src/fan/tests/world.rs`) leans on other `#[cfg(test)]`
  modules — `crate::git_tree::tests::git`, `crate::workdiff::tests` — which do
  not exist in an integration build at all.
- It is 38 beats over five files, not nine over one.

So attack the shape before moving anything. Candidates, none designed:

1. **Widen the `pub` surface `fan` needs and lift the `#[cfg(test)]` fixtures
   with it.** Honest but large, and it publishes internals for a test's sake.
2. **De-fork the victim side.** The exposure is a write fd this process holds.
   A fixture written by a CHILD (`sh -c 'cat > path'`, itself a locked fork)
   is never open for writing in the parent, so no peer fork can copy it. That
   closes every victim at once regardless of who forks — but it is a write-side
   contract, which `git_env`'s module doc argues against on the grounds that
   the victim's own care cannot save it. Here it can, because the fd never
   exists. Weigh that reversal explicitly.
3. **Ask upstream for a lock seam.** balls could expose a fork hook the way
   litany exposed its injection seam; that fixes it for every embedder and
   costs yog nothing but a release.

Do not take "hold the spawn lock across the call into balls": `git_env`'s
module doc forbids waiting on a child under the lock, and `Attempt::open`
waits on several.

---

Shape 2 taken by ruling, and the shape's own objection is answered rather than
waived. The ball recorded it as "a write-side contract, which git_env's module
doc argues against on the grounds that the victim's own care cannot save it."
That argument was against a write-side BRACKET — a lock only excludes forks
that agree to take it, and the linked substrate's forks never did. It does not
reach a write-side RELOCATION: `sh -c 'cat > "$1" && chmod 755 "$1"'` leaves no
descriptor in this process, so there is nothing for any fork, in any crate, to
copy. Candidate 1 (widen `pub` for `fan`) publishes internals for a test's sake
and closes one victim; candidate 3 (an upstream fork hook) closes nothing here
until a release. Shape 2 closes every victim at once and subtracts a lock.

Prior art decided it: `tests/integration/support/mod.rs` already wrote its
fixtures from a child, for the reason that proves the argument — yog linked as
a library is not `cfg(test)`, so the spawn lock was compiled out of the one
binary that needed it, and ~1 run in 8 failed until the fd moved. That helper
is now the shared `tests/support/write_exec.rs`, `#[path]`-included by the
binaries that write a fixture; `src/test_support/fixture.rs` is the lib half
(`write_exec`, plus narrow `read_only`/`writable` for the one non-exec mode
fixture, neither taking a mode).

Converted every site in the tree: 25 in `src`, 8 in `tests`. Structural, not a
convention: `rules/no-hand-chmod.yml` refuses `set_permissions`/`from_mode`/
`set_mode` anywhere in `src` outside `test_support/**` and three named
production files (`world/tools.rs`'s W9 shims, and the 0600/0700 narrowing in
`wire/provision.rs` and `bz_host/store.rs`) — with violation 15 in
`rules/fixtures/violations.rs` so the negative half of `make rules-audit`
proves it bites. A mode READ is untouched. `rules-audit` scans `src` only, so
`tests/` was swept by hand and that file says so.

THE MEASUREMENT (the ball's real question). bl-6bf5's recipe, unchanged: one
filter over the lib test binary — `fan::` plus `start::tests::exec`,
`cli_outbound::tests::run::spawned`, `cli_outbound::tests::detach`,
`cli_outbound::piped::tests`, `boundary::login::tests` — 16 workers x 70
iterations on a 16-core box, three runs each side, 3,360 test-binary runs per
side, counting "Text file busy":

  lock in place:   0 / 0 / 0
  guard removed:   0 / 0 / 0

Baseline for both: the same filter cost 2 before this change. So the lock was
closing nothing once the descriptor was gone, and it is DELETED — `SPAWN_LOCK`,
`spawn_guard`, and every mention (git_env and test_support module docs,
`rules/no-bare-fork.yml`'s rationale, `rules/locks-outside-state.yml`'s
carve-out, CLAUDE.md rule 7 and its bare-fork paragraph, DESIGN section 12's
git_env row and its testing note, and eight stale in-tree comments that still
described holding it). The fork chokepoint itself stays: one place to reason
about what a child inherits, and `exec`'s SIGPIPE/environ contracts are its
own.

Wider evidence, beyond the recipe: the whole lib suite lockless, 8 workers x 4
iterations (32 full runs, 2,333 tests each) — 0 ETXTBSY. One unrelated failure
in those 32, `test_support::seat::tests::a_dead_address_refuses_naming_itself`,
"Connection reset by peer": it binds port 0, drops the listener and calls the
port "certainly dead", which stops being true when 32 copies of the suite are
binding ports on one box. No fork, no exec, no fixture — an artifact of the
stress, not of this change, and not reachable at CI's concurrency.

bl-6bf5's placement rule ("the lib test binary drives no embedded substrate
in-process") is no longer load-bearing for ETXTBSY. The `tests/multiplex_*.rs`
split keeps its other reason, which is `git_env::INHERITED`: a binary running
its subject in-process must scrub its own process env. Both doc sites now say
that and not the old thing.
