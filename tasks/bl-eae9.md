+++
title = "decompose 4/10 — the REMOTE transport: src/wire/, src/tool_host/, src/engine.rs, seven files at 251-289 lines"
created = 1787546570
updated = 1787620559
claimant = "Bollard"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["decompose"]
+++
Fourth of ten waves sequenced by bl-b278. Fourth because REMOTE is where the open chain (bl-028a, bl-670c, bl-f29c, bl-320b, bl-f4e3) still has work, but the surface was under an active claim when this was filed.

    src/wire/lane/tests/mod.rs        289
    src/wire/host/tests.rs            284
    src/tool_host/tests/mod.rs        275
    src/tool_host/clients/tests.rs    275
    src/wire/server/tests.rs          269
    src/wire/asker/tests.rs           253
    src/engine.rs                     251

**Check `bl list` before claiming.** bl-4e31 was mid-flight in `src/wire/**`
when this was filed and had `host/tests.rs` already becoming `host/tests/mod.rs`
plus `host/tests/engine.rs` — take this wave only once that has landed, and
re-measure, because that ball may have dissolved two of these rows on its own.

Six of the seven are corpora and earn no DESIGN §12 row. `src/engine.rs` does,
and its likely seam is boot (the §16.7 mint, the wire ends) against serve.

## The rule this wave runs on

Split along a **real seam** (AGENTS.md, "300-line hard cap": *"Over the cap?
Split along a real seam and add the row to DESIGN §12; never shave lines to
duck the limit"*). A split that leaves two files with no coherent separate
subject is worse than the long file — **if a file has no seam, leave it and
say so in the close note.** That is a finding, not a failure.

Every new production module needs its DESIGN §12 row: `tests/design_module_map.rs`
fails on an absent row, and it checks more than presence — rows stay in sort
order, brace families (`src/app/{mod,roots}.rs`) must expand, and a test module
is covered by its production module's row and never earns one of its own (the
`X.rs` ↔ `X/tests.rs` seam). A submodule of an integration binary under
`tests/` earns no row either (precedent: `tests/multiplex_bl/fixtures.rs`), and
such a submodule needs `#[path = "<binary>/<name>.rs"]` because the binary's
own file is the crate root.

100% coverage is the floor and a split must not lose a test: moved, not
dropped. Verify with `grep -rc '#\[test\]' src tests --include=*.rs` before and
after.

**Verify the census yourself** — `make line-cap LINE_CAP=249` — rather than
trusting the list below. The tree moves under several agents and this body is a
snapshot taken when bl-b278's first wave landed.