+++
title = "the follow-lane engine test is sleep-timed and dies under parallel gate load: said == [] where the tail should have three frames"
created = 1787549117
updated = 1788407808
claimant = "Spellbind-A"
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Seen once during a `bl close` gate on 2026-08-24, with four instrumented tarpaulin suites running on the box at the same time (the suite took 2831 s, roughly 90x its unloaded 32 s):

    ---- wire::lane::tests::engine::the_engine_holds_one_connection_across_every_growth_of_the_tail
    assertion `left == right` failed: the last frame carries the whole tail: []
      left: None
     right: Some("one two three")

It passes in isolation every time. The test spawns a writer thread that appends one `content_delta` per word with a `std::thread::sleep(40ms)` between them, then publishes a settling snapshot, while the main thread sits in `seat.followed(...)` collecting frames. `said` came back **empty** — not short, not stale: the follow read ended before the lane emitted a single frame.

So the beat proves nothing about the engine when the box is loaded: a 40 ms sleep is not a synchronisation primitive, and the one outcome the assertion exists to catch (a frame carrying a partial tail) is indistinguishable from the scheduler simply never running the writer inside the read window.

The fix is to remove the wall clock from the rendezvous, not to lengthen the sleep — the writer and the reader should hand off on something both can observe (the write itself, a channel, the published snapshot), so the beat is a statement about the fold and not about the machine.

Unrelated to bl-fae3, which is a pure file split; recorded there rather than fixed there.

---

Second and third sightings, from bl-0ed9 (now folded here as the duplicate): the same test also fails the OTHER way under load — the reader is scheduled late enough that all three 40ms-spaced deltas land before its first read, so it sees ONE frame carrying the whole tail and said.len() > 1 fails. Killed one bl-5aae close attempt and two bl-7547 attempts, 5/5 green in isolation each time. Both symptoms (zero frames, one coalesced frame) are the same defect this ball names: the sleep is not a rendezvous. The fix shape from 0ed9 agrees with this ball's: hand off on something both threads observe (gate each write on the reader consuming the previous frame), or assert on content (final tail complete, every frame a prefix-extension) rather than frame count. Do not widen the sleep.

---

Premise verified dead against the tree: the named test and its whole module are gone. `src/wire/lane.rs`, `src/wire/lane/tail.rs` and `src/wire/lane/tests/{mod,engine,absent,tail}.rs` were deleted by the server severance (bl-7942, commit 95316f2f) — `git ls-files | grep wire/lane` returns nothing, and the follow reader that survives in yog (`src/boundary/follow/`) is a single-threaded poller whose tests drive `follow.poll()` directly, with no writer thread and no clock. The sleep-timed beat lives in the seat crate now, if anywhere.

Swept the rest of the tree for the same shape (a sleep standing in for a cross-thread rendezvous, with the assertion counting events). One survivor: `registry::mailbox::slots::tests::work_landing_mid_hold_ends_it`, where a writer thread slept 20 ms and then posted while the main thread sat in a 2 s follow-class hold. Same defect, wider window: under the load that produced this ball (a suite at ~90x its unloaded time) the writer wake can land after the hold expires and the read answers empty.

Fixed the way this ball prescribes — the writer now waits on `Mailbox::serving(client)`, the reader claim the read publishes on its way into the hold, so the post is ordered after the park by a fact both threads observe rather than by the clock. The writer keeps a 10 s escape deadline and posts anyway, so a read that never parked fails the assertion instead of hanging the suite. The assertion is unchanged in content and now says what it means: the parked read answered with the work rather than expiring empty.

Every other bounded wait in the tree is already the right shape: `watch::tests::wait_until`, `app::tests::worker::wait_until`, the consumer/engine deposit loops and the presence-release loop all sleep INSIDE a deadline-bounded poll on an observable condition, which load can only slow, never falsify. `engine::stop::tests` asserts a LOWER bound on elapsed time, which load also cannot break.
