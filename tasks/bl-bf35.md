+++
title = "yog has no deployment recipe for a headless server: no service unit, no auto-update from the registry, and an unconditional restart kills an in-flight turn"
created = 1787460962
updated = 1787460963
claimant = "Nachos"
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
## What is missing

`yog serve` (DESIGN §8.5, REMOTE §8) is the engine with no window, and it is
the face a headless box runs. Nothing in the tree stands it up as a service or
keeps it current. `make install` is the desktop recipe — it builds from a
checkout, seats icons and a `.desktop` entry, and its `reload` target only
relaunches a yog it started from a pidfile. None of that is what a server
wants: a server has no display, no checkout worth keeping, and no operator
sitting in front of it to notice a release.

The publication channel already exists — release-plz publishes to the registry
— so "a new version dropped" is an observable fact with one home: the sparse
index. What is absent is anything that reads it.

## The hazard an obvious implementation walks into

A timer that installs and restarts unconditionally will SIGTERM the engine
while a `lernie` turn is in flight. `Stream`'s drop already does exactly this
to its child (`src/cli_outbound/stream.rs`), and the engine's own state is safe
under it — §4.1 is write-through and there is no `on_exit` (bl-b54e). The agent
is not safe: a turn killed between a tool call and its result leaves an
unpaired tool-use tail, and every later message on that conversation is
refused. That is not a crash the next boot repairs; it is a wedged
conversation.

So the restart must be **deferred until the engine is quiescent**, and
quiescence must be read rather than stored. The engine spawns its substrate
in-process (§16.7) and its agent turns as children, so the service cgroup
holding exactly one process IS the idle predicate — no new field, no new
gesture, no yog-side API.

## Shape

Host-parameterized, severable, committed with no host, address or home path in
it (the leak gate reads this tree, and the deploy artifacts are in it):

- a user-level service unit for `yog serve`, `%h`-relative
- a timer + oneshot that reconciles installed version against the index and
  restarts only when idle
- one Makefile verb to seat all of it on a named host

Update and restart are two independent reconciliations, not one procedure:
replacing the binary is safe while the engine runs (the running inode
survives), so install happens immediately and the restart waits for quiet. The
"a restart is pending" fact is the running exe's inode differing from the
installed one — computed from the kernel, stored nowhere.

Rollback falls out rather than being built: the reconciler tracks the index's
newest non-yanked version, so yanking a bad release is the operator's revert
lever.