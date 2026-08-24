+++
title = "decompose 1/10 — src/shell/acceptance/: ten drives at 250-291 lines, on the surface the REMOTE §8.2 window chain must add coverage to next"
created = 1787546558
updated = 1787547050
claimant = "Thimble"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["decompose"]
+++
First of ten waves sequenced by bl-b278, which took the 300 wall and the ≥295 band. Each wave is one subsystem, so no two waves touch one file and they land in any order.

**This one is first because the §8.2 window chain (bl-028a, bl-670c, bl-f29c) adds acceptance coverage here next**, and a drive already near the wall is where an author finishing something else reaches for the shave.

    src/shell/acceptance/screen.rs        291
    src/shell/acceptance/settings.rs      287
    src/shell/acceptance/name_column.rs   275
    src/shell/acceptance/walls.rs         268
    src/shell/acceptance/fixture/world.rs 266
    src/shell/acceptance/naming.rs        264
    src/shell/acceptance/unfold/drive.rs  263
    src/shell/acceptance/unfold/mod.rs    260
    src/shell/acceptance/legible/mod.rs   255
    src/shell/acceptance/focus.rs         251

The seam this directory already runs on is one file per claim a drive makes,
and bl-b278 used it twice: `elision.rs` gave up L4's *where* beat to
`activity_tail.rs`, and `fixture.rs` gave up its assembly to
`fixture/build.rs`. `fixture/world.rs` and `screen.rs` are the harness
half and will want the same cut.

Coverage note: `src/shell/*` is a tarpaulin exclude, so these files carry no
coverage obligation of their own — the obligation is that the drives still run.

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