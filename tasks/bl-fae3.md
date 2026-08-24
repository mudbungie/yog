+++
title = "decompose 7/10 — src/git_tree/, src/nav/, src/start/, src/config_edit/, src/model_pick/: twelve files at 253-290 lines"
created = 1787546576
updated = 1787547887
claimant = "Fairlead"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["decompose"]
+++
Seventh of ten waves sequenced by bl-b278. The read-and-configure family: the git-backed derivation, the roster projection, the fire path, and the two config panes' models.

    src/start/tests/prompt.rs                       290
    src/config_edit/brazen/providers/tests.rs       290
    src/nav/convs.rs                                284
    src/start/mod.rs                                284
    src/git_tree/cmd.rs                             283
    src/config_edit/brazen/mod.rs                   283
    src/git_tree/terminal/tests.rs                  282
    src/model_pick/mod.rs                           281
    src/git_tree/tests/state_unit.rs                281
    src/nav/convs/row.rs                            280
    src/git_tree/tests/fixture.rs                   269
    src/config_edit/branch/edit.rs                  262
    src/git_tree/descent.rs                         260
    src/model_pick/tests/grammar.rs                 256
    src/start/exec.rs                               253

bl-b278 cut `git_tree/tests/repo.rs` here already, on the seam between the
tree's skeleton and the text a row wears (`tests/naming.rs`);
`tests/state_unit.rs` and `tests/fixture.rs` are the same shape.

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