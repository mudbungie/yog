+++
title = "the follow-lane engine test is sleep-timed and dies under parallel gate load: said == [] where the tail should have three frames"
created = 1787549117
updated = 1787549117
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