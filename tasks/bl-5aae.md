+++
title = "decompose 8/10 — the paint tier: src/theme/, src/ui_state/, src/paint_probe.rs, src/inspector/, src/rail/, src/steps_view/, ten files at 250-294"
created = 1787546581
updated = 1787546581
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["decompose"]
+++
Eighth of ten waves sequenced by bl-b278. What the window is painted WITH, as against wave 5's window itself.

    src/theme/tests/mod.rs           294
    src/paint_probe.rs               293
    src/ui_state/mod.rs              287
    src/ui_state/tests.rs            280
    src/inspector/mod.rs             279
    src/theme/badges.rs              268
    src/steps_view/tests/vm.rs       256
    src/rail/tests/build.rs          254
    src/steps_view/tests/render.rs   250
    src/theme/mod.rs                 250

`src/paint_probe.rs` wants care rather than speed: it is the one lawful paint
walk (`rules/no-hand-rolled-paint-walk.yml`), and bl-bc06 found 1815 tests
passing while it covered no truncation at all. A split there must leave the
rule's own `ignores` list still naming exactly the walk.

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