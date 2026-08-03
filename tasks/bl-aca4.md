+++
title = "name is an exposed dispatch parameter; omission auto-mints a one-word name — rule the mint's one home and amend the naming contract"
created = 1785736966
updated = 1785736966
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator ruling 2026-08-03, verbatim: 'agent names should be a part of the dispatch command. It's pretty simple, already tells the depth. It also simplifies the whole name thing: yog just generates a one-word name for you if you don't provide it yourself. It will help keep subagents' identities and tasks clear. If a dispatch command omits it, we can provide it dynamically, randomly, but it's an exposed parameter.'

Standing facts (verify against pinned lernie 0.0.5 and the tree):
- lernie dispatch (tool + CLI) already ACCEPTS name (0.0.4/bl-c8ed; the tool test shows {role, goal, name}); creation refuses a living-name collision; the fact stores beside goal.md; yog passes --name at fire (bl-08f2).
- The GAP: omission yields a nameless child — the display ladder falls through to payload line/id (the raw-id complaint, bl-df72). The operator rules: omission auto-mints a one-word name, and the parameter is prominently exposed so dispatching models keep child identities clear.
- The mint (one word, wordlist, collision-retry, bl-d12f) currently lives in yog src/names — but the name fact, uniqueness enforcement, and every dispatch path live in lernie, and lernie must now mint on omission.

The design ball's job (bl-50f3 pattern — deliverable is the ruling + DESIGN §3.3 amendment + implementation balls):
1. Rule the mint's one home. The strong candidate: the mint mechanism (wordlist + draw + collision-retry) moves INTO lernie beside the uniqueness check; yog consumes it THROUGH the lernie crate it already links (the yog-lernie multiplex proves the linkage) for its composer preview — preview timing/I7 stays yog policy, mechanism has one implementation, zero drift. Attack this: does the composer preview survive (preview draws from the same function with the same occupied set)? Does the lost-race re-mint story still hold? Record why any loser lost. This AMENDS bl-50f3's 'yog stays the minter' — nothing set in stone; fix the doc, record the supersession.
2. Dispatch tool description teaches the parameter (identity clarity for subagent trees — the operator's stated purpose); omission mints silently and validly.
3. File implementation balls: lernie-side (mint-on-omission + mint home + tool description; needs a lernie release), yog-side consume (drop src/names' local wordlist in favor of the crate call, or whatever the ruling says; gated as prose on the lernie release, bl-5134 pattern).
Depth note from the ruling ('already tells the depth'): dispatch implies parent→child, so depth stays derivable from the tree — no depth field anywhere.