+++
title = "B4b: rule 1 named-lifetime elimination + no-named-lifetimes.yml (B4 close 2)"
created = 1784435095
updated = 1784435095
parent = "bl-97fb"
priority = 1
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-4bb4"
on = "claim"
+++
Continuation of B4 (bl-4bb4). B4 was landed as two closes per the amendment's provision ("land it as two closes, do not leave the tree half-refactored"). THIS ball is close 2: rule 1 (no named lifetimes) only.

## What close 1 (bl-4bb4) already landed (do NOT redo)
- Rule 9 (no-pub-generic-bounds): AppModel<C:Clock> and UiState<C:Clock> de-generified to hold `Arc<dyn Clock>` (NOT Box — the Arc shares one injected time source between the ui.json debounce and the sweep Schedule; Box<dyn Clock> cannot express that sharing without Clone-on-trait. Surfaced adaptation, documented in each struct doc). Schedule<C:Clock> also de-generified. The 3 ergonomic IntoIterator/Into bounds (Schedule::mark, DirtySet::mark_all, Env::from_pairs) demoted to pub(crate); from_pairs also #[cfg(test)] (test-only constructor). TtlCache<P,C:Clock> left generic — it is pub(super), so the rule never sees it (demotion over churn).
- Rule 2 (no-pub-borrow-return): all 13 pub borrow-returning getters demoted to pub(crate) (AppModel::{workspaces,focus,ops_rows,state_root,yog_data_root}, Cli::binary, EditPlan::argv, brazen/lernie_global {draft,draft_mut}, InspectorTab::label). The 6 that are test-only readers (workspaces, yog_data_root, argv, brazen::draft, lernie_global::draft, from_pairs) are also #[cfg(test)] so they are not dead in the non-test lib build.
- Installed rules/no-pub-borrow-return.yml and rules/no-pub-generic-bounds.yml; fixtures violations 7-9 added; `make rules-audit` green both directions.
- Named lifetimes were NOT touched. The tree compiles, 565 tests pass, coverage 100%, all files <=300. This ball's tree is coherent, not half-refactored.

## What THIS ball must do: rule 1 (no-named-lifetimes)
Eliminate every named lifetime token repo-wide (src/ incl. inline tests). Current inventory: `ast-grep scan --rule <the rule below> src` lists ~75 lifetime tokens across attention/mod.rs, inspector/mod.rs (+tests), git_tree/descent.rs, ui_state/mod.rs, start/exec.rs, actions/verbs.rs, projects/join.rs (+tests), and several *tests.rs. Re-derive with the rule; go by the scan.

Strategy (from the B4 body, step b/c): attention — delete Workspace<'a>/RosterEntry<'a>/Focus<'a>/Seen<'a>: fns take borrowed slices as ARGS (elided) and return OWNED keys (RosterKey{ws:String, agent_id:String} or index pairs). Seen<'a> type alias -> `pub(crate) type SeenFn = dyn Fn(SeenKind,&str,&str,&str)->bool;` used as `&SeenFn` (elided). descent_order -> Vec<DescentRow{depth, index:usize}> owned+Copy (callers .get() back into the slice). inspector TabData<'a> -> owned TabData (shell builds the 6 VMs fresh per frame; move them in; update the module doc). ui_state::derive_startup_focus -> Option<String>; ui_state::descend: elide by taking `key: String` (one ref input -> output lifetime elides). start Deps<'a> -> owned {bl:Cli, lernie:Cli, state_root:PathBuf}. actions Update<'a> -> Option<String> x3. projects::join::bound_of returns owned. attention/nav/shell/inspector consume these — attention module can be demoted pub(crate) if not test/main-imported. Rewrite inline-test lifetimes too (the rule is repo-wide, no test carve-out).

## The verified rule (install as rules/no-named-lifetimes.yml)
ast-grep 0.44.1 was verified via --debug-query=ast. A named lifetime is node kind `lifetime` (text like `'a`); `'static` and `'_` are also `lifetime` nodes (text `'static`/`'_`). Rust's regex crate has NO lookahead, so exclude them with a `not` combinator. This rule was authored and confirmed to fire on named lifetimes and NOT on 'static/'_:
```
id: no-named-lifetimes
language: rust
severity: error
message: "no named lifetimes; take borrowed args (elided) and return owned (see rules/no-named-lifetimes.yml)"
rule:
  kind: lifetime
  not:
    regex: "^'(static|_)$"
```
Add a named-lifetime fixture to rules/fixtures/violations.rs (violation 10), e.g. `pub struct Held<'a> { r: &'a str }`. Verify `ast-grep scan --rule rules/no-named-lifetimes.yml rules/fixtures` fires on it and `... src` is clean after the refactor.

## ast-grep 0.44.1 gotchas learned in close 1 (reuse)
- A relational rule (`has`/`inside`) matches NOTHING on its own — it needs an atomic anchor (`kind:`) as a sibling, or wrap in `all` with a kind. `not`+`regex` under a `kind` anchor is fine (rule 1 above).
- `field: visibility_modifier` is INVALID (visibility_modifier is a node kind, not a field) — use `kind: visibility_modifier`. Plain `pub` = visibility_modifier text `pub`; `pub(crate)` carries a `crate` child, so `regex: "^pub$"` excludes it.
- `<T: Bound>` parses as `type_parameter` with a `bounds: trait_bounds` child (NOT constrained_type_parameter). `--inline-rules` with `^`/`$` regex is quoting-fragile; prefer files.

## Gate
Behavior must not change: 565 tests are the referee; coverage 100% (watch llvm phantom regions on restructured impls/entry-API rewrites — prefer plain-statement forms). All files <=300 (note: src/ui_state/mod.rs and src/cli_outbound/mod.rs are AT 300 after close 1 — the ui_state descend/derive_startup_focus rewrite must not grow ui_state/mod.rs; split if needed). fmt + clippy -D warnings + rules-audit green. Process: `bl claim --as filtered`, edit in the worktree, `bl close --as filtered`.