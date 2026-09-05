+++
title = "pin the litany release that carries brazen =0.0.10, and brazen =0.0.10 with it: one brazen in the graph"
created = 1788580600
updated = 1788581591
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
The litany half landed in two balls, not one: litany bl-3fe6 pinned brazen =0.0.10, and litany bl-a825 moved it on to =0.0.12 after 0.0.11 (an auto-merge job, no crate change) and 0.0.12 (brazen bl-5053, per-row shape capability: Protocol::shapes() answers tools and multi_turn, and bz --list-providers gains a shapes column reading '-' on claude_code) published while the first was in the gate. So this ball's title version is stale: the one-brazen invariant now means =0.0.12, or whatever is newest at the moment of the edit if litany moves again first.

Still blocked on litany's release. litany main carries both bumps but crates.io is at 0.0.8; the release PR proposes 0.0.9 and litany's own merge-release-pr job holds it until CHANGELOG.md on main names that version — the documented unblock is landing 'make promote-changelog VERSION=0.0.9' on litany main through the normal task-worktree flow, after which the chain is hands-off.

Two findings for whoever takes this. yog's transcript Usage is an open BTreeMap<String, u64> (src/transcript/mod.rs), so brazen's additive input_total_tokens and context_window ride through the codec with no edit — that is the additive-counter proof this ball asks for, and it holds by construction rather than by fixture. And yog's own BudgetSpend (src/budgets/mod.rs) still folds max(input, cache_read + cache_write) + output, the approximation litany just retired in favour of the served counter; adopting input_total_tokens there is a separate change, and it sits beside bl-9c8a rather than inside this pin bump.
