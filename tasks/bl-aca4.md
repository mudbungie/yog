+++
title = "name is an exposed dispatch parameter; omission auto-mints a one-word name — rule the mint's one home and amend the naming contract"
created = 1785736966
updated = 1785737051
claimant = "mint-arch"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
RULED AND RECORDED (mint-arch, 2026-08-02). DESIGN §3.3 amended in this ball's worktree; implementation balls filed.

The ruling: the mint's one home is LERNIE. The mechanism (wordlist + RNG-start wraparound draw + bounded collision retry) moves in beside the uniqueness check it races (`agent_name::require_available`); both creation pre-flights (`prompt::run`, `child_dispatch` — the dispatch tool shells into the latter) settle the name there: supplied -> validated, absent -> minted against the same living-names scan. No fork ends nameless; unnamed becomes readable-but-not-creatable. `name` stays a prominently exposed parameter and the dispatch tool schema teaches it (identity clarity in subagent trees — the operator's stated purpose). Yog deletes its local mint and draws the same function through the crate it already links: preview timing (I7), seed lifetime (bl-28ba), and mint-at-fire --name passing stay yog policy over lernie mechanism — the fire's returned name is still the §3.4 focus-claim handle, so yog never omits the parameter. Preview parity: yog's occupied set (ladder facts incl. legacy rung) is a lawful SUPERSET of lernie's scan — yog may avoid a word lernie would allow, never predict one lernie would refuse; discrepancy dies with the legacy rung. Lost race unchanged: preview predicts, fire's fresh mint is truth, `require_available` remains the uniqueness gate.

Supersession recorded honestly in §3.3: bl-50f3's storage half (name blob is the one durable home) stands; "yog stays the minter" is amended — it held only while yog's fire was the only minting path, and mint-on-omission on every lernie creation path makes a yog-resident mint either an inverted dependency or a second list.

Losers, and why: (1) lernie fallback-mints with its own list — two lists/two draws are two representations of one behavior, must drift, curation lands twice; (2) name required, every dispatcher supplies — against the ruling verbatim, multiplies minters, a failure apiece where one default dissolves the class; (3) lernie mints always and yog stops passing --name — kills the preview or I7, and the fire returns nothing to hold the focus claim by.

Depth note ("already tells the depth"): dispatch implies parent->child; depth stays derivable from the descent-branch shape — no depth field anywhere.

Filed: lernie store bl-404d (mint moves in + mint-on-omission on both pre-flights + tool description + exported mint/Rng API; needs a lernie release). Yog store bl-cd38 (consume: delete local mint half of src/names, rewire identity/fire/preview through the crate; --needs bl-aca4, plus the prose crate-release gate — do not claim until the published crate carries bl-404d).

Original operator ruling and standing facts preserved below.

---

Operator ruling 2026-08-03, verbatim: 'agent names should be a part of the dispatch command. It's pretty simple, already tells the depth. It also simplifies the whole name thing: yog just generates a one-word name for you if you don't provide it yourself. It will help keep subagents' identities and tasks clear. If a dispatch command omits it, we can provide it dynamically, randomly, but it's an exposed parameter.'

Standing facts (verified against pinned lernie 0.0.5 and the tree): lernie dispatch (tool + CLI) accepts name; creation refuses a living-name collision; the fact stores beside goal.md; yog passes --name at fire. The gap: omission yields a nameless child and the display ladder falls to payload line/id (bl-df72). The mint (one word, wordlist, collision-retry, bl-d12f) lived in yog src/names.