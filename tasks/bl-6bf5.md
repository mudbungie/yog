+++
title = "the spawn lock is not the binary's: multiplex's in-process balls/litany forks copy fixture write fds, and every write-then-exec test is an ETXTBSY victim"
created = 1788414802
updated = 1788414802
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["flake", "tests"]
+++
`git_env`'s module doc claims the spawn lock closes ETXTBSY for the whole
binary — "every fork through one lock, **zero**". It does not, and the doc
itself names why one paragraph later, at `exec`: "the linked balls' own `git`
forks, which take no lock of yog's". The lib suite runs the embedded substrate
**in-process** — `multiplex`'s `bl`/`litany`/`bz` arms are `balls::run` and
`litany::cmd` called directly, and those crates fork on their own account. A
fork copies every open fd, so any peer thread holding a write fd on a fixture
script it just wrote loses the exec that follows: **Text file busy**.

Measured on a 16-core box under gate-shaped contention, one filter over the lib
test binary, 16 workers x 70 iterations each:

- with `multiplex` in the run — **16 ETXTBSY failures** across 14 distinct
  tests, every one of them a "write a fake binary, then exec it" fixture
  (`start::tests::exec::*`, `cli_outbound::tests::run::spawned::*`,
  `cli_outbound::tests::detach::*`, `cli_outbound::piped::tests::*`,
  `boundary::login::tests::*`).
- the same volume with `multiplex` left out and nothing else changed —
  **zero**.

That is the isolation: `multiplex::landing` forks through `git_env::output` and
is therefore locked and innocent; the in-process substrate arms are the party.

**This is the flake class behind bl-b8c8's neighbours.** bl-5510 filed two
load-sensitive tests as wall-clock-wait defects; one of them
(`boundary::login::tests::a_lane_ends_when_the_run_it_was_reading_is_gone`) has
no wait at all, and was reproduced failing on exactly this — `.expect("started")`
on a spawn that answered "Text file busy". It was fixed there by taking the
fixture away from that one beat (its child was incidental), which dissolves one
victim and not the class. The other twelve-odd stand.

**What will not work.** The write side cannot protect itself: the fd is open
across a fork it does not perform, and the forker is in another crate. Locking
the write buys nothing, because the party takes no lock either. A retry belongs
to nobody — the exec is in production code (`cli_outbound::run_streaming`), and
a test hazard must not become a production loop.

**Two shapes that might.** Neither is designed yet; attack both before
committing.

1. **Hold the spawn lock across the call INTO the substrate.** `git_env::spawn`
   already carries `#[cfg(test)] let _guard = spawn_guard()`; the same two lines
   at `multiplex::bl::run` / the litany arm would put the one fork yog cannot
   reach back inside the one lock. The cost is real and must be measured: the
   guard would be held for the whole of `balls::run` — a child waited on under
   the lock, which the module doc explicitly says never to do — and any path
   where the embedded substrate re-enters yog's own `git_env::spawn` on the same
   thread is a self-deadlock.
2. **Take the in-process substrate out of the lib binary.** `tests/multiplex_bl.rs`
   and `tests/multiplex_litany.rs` are already the integration home for exactly
   this, for a neighbouring reason (they scrub their own process env because no
   spawn boundary exists to do it for them). Moving the forking arms there
   leaves the lib suite with no unlocked forker at all. The cost is the 100%
   coverage floor: what moves out must stay measured.

Whichever wins, `git_env`'s module doc has to stop claiming zero — the
measurement it quotes was taken with every fork locked, which is not the
condition the suite actually runs in.