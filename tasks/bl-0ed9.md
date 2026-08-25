+++
title = "the follow-lane growth test assumes write cadence outlives read latency: under load all three deltas coalesce into one frame and said.len() > 1 fails"
created = 1787622719
updated = 1787622719
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
## The flake

`wire::lane::tests::engine::the_engine_holds_one_connection_across_every_growth_of_the_tail` (moved by bl-eae9 into the split lane corpus; formerly src/wire/lane/tests/engine.rs:220) asserts `said.len() > 1` — that a follow stream emits more than one frame across three writes spaced 40ms apart.

Three sightings in one day, all under gate load (two or more concurrent tarpaulin runs), never in isolation:

- killed one bl-5aae close attempt; passed 3/3 standalone immediately after
- failed a bl-7547 close attempt twice; passed 5/5 in isolation
- (first sighting was during the same day's bl-5aae work)

## The cause, as diagnosed at the third sighting

Under load the reader thread is scheduled late enough that all three 40ms-spaced deltas land before its first read, so the stream legitimately answers ONE frame carrying the whole tail. The engine is behaving correctly — newest-wins coalescing is the follow lane's design. The assertion encodes a scheduling assumption ("a 40ms gap is enough for a read to interleave"), not a contract.

## The shape of a fix

The contract worth proving is that the stream delivers growth incrementally when reads DO interleave — so either gate each write on the reader having consumed the previous frame (a handshake, making the interleave a fact rather than a race), or assert on the CONTENT reaching the glass (the final tail is complete, every frame is a prefix-extension) rather than on the frame count. Do not widen the sleep — that just moves the threshold.