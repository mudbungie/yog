+++
title = "two loopback socket beats fail under heavy full-suite concurrency: a refused preface reads as a closed connection"
created = 1788484554
updated = 1788484690
claimant = "Spellbind-V"
priority = 4
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["flake", "wire"]
+++
Sighted while verifying bl-98ce's fix: 80 full-suite lib runs at 4-way concurrency on a 16-core box. Three of the five failing runs were these two, and neither is the claim-lock family bl-98ce fixed (that one went to zero in the same 80 runs).

- `wire::server::tests::protocol::a_skewed_peer_is_refused_and_never_reaches_the_answerer`, twice, `src/wire/server/tests/protocol.rs:73`:

      assertion `left == right` failed: the preface, then the refusal
        left: 0
       right: 2

  The beat expects two frames back and read none — so the connection gave EOF where the refusal was due.

- `test_support::seat::tests::a_dead_address_refuses_naming_itself`, once, `src/test_support/seat/tests.rs:79`: `send: Connection reset by peer (os error 104)`.

Both are loopback accept/write races rather than assertions about yog's own state, and both read as *nothing arrived* rather than *the wrong thing arrived* — the same shape as a count assertion that can honestly answer zero, which is the shape that lands a permanent FAIL verdict in the merge queue. The question for each is whether the beat can hand off on an observable fact instead of on a read that a reset can empty; a widened deadline is not the answer.

Filed as a sighting with the evidence, not investigated.