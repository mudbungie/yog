+++
title = "the hello states the corpus edition: build.rs compiles corpus/shapes.json's newest stamp into EDITION beside PROTOCOL, both ends write it, the engine reads the peer's"
created = 1788926013
updated = 1788927472
claimant = "Cantaloups-P1"
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["usability-r3"]
+++
Child of bl-e598 (REMOTE 3.2: "Capability discovery replaces the bump for the additive class"). The design needs one wire-visible fact the engine does not state today: the EDITION of the corpus this build was generated from — the newest stamp in `corpus/shapes.json` — so a seat that vendored the ledger can grey a control whose field the engine cannot spell, and render an absent post-floor field as *this engine cannot say* rather than as the default.

What lands:
- `build.rs` reads `corpus/shapes.json` beside `PROTOCOL` and writes `pub const EDITION: u32` = the maximum over every `signature` value (computed, never stored — the record carries no `edition` key on purpose). `serde_json` joins `[build-dependencies]`: no new crate, it is already in the graph. `Cargo.toml`'s `include` allowlist gains `/corpus/shapes.json` (a build input, the loud half of the asymmetry the list is shaped by) and `tests/packaged_files.rs` admits exactly that file.
- `src/wire/hello.rs`: `state` writes `{"protocol": PROTOCOL, "edition": EDITION}`; `stated` reads the peer's `edition` beside its `protocol` (absent reads as the floor, `shapes.json`'s `floor`, which build.rs also emits as `FLOOR`); `admit` still decides on `protocol` alone. The peer's edition is kept where the connection's identity is (registry presence, REMOTE 5) so `reply/clients` can carry it later if a seat wants it — not now.
- A unit test that `EDITION` equals `Ledger::read(shapes.json).edition()` and `FLOOR` its `floor`, so the compiled constants and the committed record cannot disagree; the hello tests gain the additive case: a peer stating 19 with no edition is admitted, a peer stating 19 with a higher edition is admitted.
- The corpus gains nothing: the hello is not a boundary frame (REMOTE 3, "the preface rides beside the gesture envelope"), so `shapes.json` is unchanged and this is not an edition.
- REMOTE 3.2's "Capability discovery" paragraph cites this ball's landing.

The three consumer children (thrall, lernie, yog-android: filed beside this one, tag usability-r3) read this key; they ignore its absence today, so the order between this ball and them is free.