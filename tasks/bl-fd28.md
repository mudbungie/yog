+++
title = "fan's lib beats fork inside balls: Attempt::open/resume/deliver leave the ETXTBSY class open after bl-6bf5"
created = 1788415404
updated = 1788415543
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