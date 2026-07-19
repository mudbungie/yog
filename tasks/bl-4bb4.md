+++
title = "B4: owned-signature wave — rules 1/2/9 (lifetimes, pub borrows, pub bounds) + their ast-grep rules"
created = 1784433624
updated = 1784434636
claimant = "filtered"
parent = "bl-97fb"
priority = 1
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"

[[blockers]]
id = "bl-c5d8"
on = "claim"
+++
The big signature wave, per the assessment inventory (16 prod named-lifetime sites, 21 pub borrow-returns, 6 pub generic bounds). Strategy, in order of preference: (a) pub(crate) demotion — the rule boundary is pub; everything not consumed by tests/ integration tests or main-as-lib is demoted to pub(crate) honestly (AppModel getters, draft_mut, render fns, TabData...). Sweep lib.rs's pub mod tree: which items do tests/*.rs actually import? Everything else internal. (b) Owned or index-based view-models where types stay pub: attention — delete Workspace<'a>/RosterEntry<'a>/Focus<'a>/Seen<'a>: fns take borrowed slices as ARGS (legal, elided) and return OWNED keys (Vec<RosterKey{ws:String, agent_id:String}> or index pairs; small strings per row per frame are fine); descent_order -> Vec<DescentRow{depth, index:usize}> owned+Copy (callers .get() back into the slice — indexing_slicing lands in B5, use .get() now); inspector TabData<'a> -> TabData OWNING the six VMs (the shell builds them fresh per frame anyway — move, zero extra clones; update the module doc comment); actions Update<'a> -> Option<String>x3; start Deps<'a> -> owned {bl:Cli, lernie:Cli, state_root:PathBuf}; ui_state derive_startup_focus -> Option<String>. Private fns (descend, bound_of, walk) may keep elided/named lifetimes ONLY if the no-named-lifetimes rule passes — it is repo-wide: rewrite private helpers too (descend's &mut re-borrow: restructure with the entry API or loop; bound_of returns owned). (c) Rule 9: AppModel<C:Clock>/UiState<C:Clock>/TtlCache<P,C> -> Box<dyn Clock> / Box<dyn ...> fields (vtable on cold paths only — acceptable; document); the IntoIterator/Into ergonomic bounds -> concrete Vec parameters. (d) Zero Cow/impl-Trait returns exist; keep it so. Then install rules/no-named-lifetimes.yml, rules/no-pub-borrow-return.yml, rules/no-pub-generic-bounds.yml VERBATIM from the bootstrap doc; extend fixtures; smoke-test; `ast-grep scan` clean on src/. Every refactor keeps behavior: the full test suite is the referee, coverage stays 100%, files stay <=300 (split as needed). This may split into follow-up balls if one close is too big — file subtasks if so, but land the whole wave before B5.