+++
title = "the codec emits a conformance corpus: one canonical fixture set every client replays, so an implementation miss fails a fixture instead of shipping"
created = 1788068701
updated = 1788068791
claimant = "OrderNotary"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
The N-implementations hazard, ruled 2026-08-30: yog plus several clients each implement the wire vocabulary, and the failure mode is one of them being a quiet miss. A shared types crate was weighed and declined — it protects only same-language consumers (the Kotlin client cannot link it) and couples release cadence for one protected consumer. The general mechanism instead: the codec, which is compile-gated against the real boundary and whose tests already exercise every shape, gains a generator that emits a canonical fixture corpus — every act, every reply, every variant, edge cases, stamped with the wire protocol version — committed or published as a build artifact. Every client (the seat, the foot where its small surface warrants, the android app) replays the corpus against its own encode/decode: decode everything, round-trip what it emits. Regenerating the corpus after a boundary change diffs loudly, which also enforces the companion REMOTE rule this ball adds: any change to a wire-visible shape bumps the protocol version. Deliverable: the generator + corpus in this repo, the REMOTE amendment, and the corpus's consumption contract stated where clients will find it.