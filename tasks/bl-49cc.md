+++
title = "decompose 5/10 — src/shell/ outside acceptance: eleven window files at 250-294 lines"
created = 1787546571
updated = 1787546571
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["decompose"]
+++
Fifth of ten waves sequenced by bl-b278. The window's own painting code, kept apart from wave 1's drives so the two never touch one file.

    src/shell/ram.rs                        294
    src/shell/verb_row.rs                   292
    src/shell/delete.rs                     283
    src/shell/input_bar.rs                  280
    src/shell/delete_agent.rs               279
    src/shell/mod.rs                        274
    src/shell/config_edit/form_ui.rs        272
    src/shell/model_pick/select.rs          270
    src/shell/inbox_queue.rs                267
    src/shell/config_edit/branch_pane.rs    266
    src/shell/start_rows.rs                 250

`src/shell/*` is a tarpaulin exclude, so the coverage floor does not reach
these — which is exactly why they drift long, and why the DESIGN §12 rows are
the only record a split leaves. Add one per new module.

Check `bl list` first: bl-e64e was mid-flight in the Files-tab shell surfaces
when this was filed.

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