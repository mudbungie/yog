+++
title = "agent names are one word: mint from a single wordlist, retry the live set on collision"
created = 1785731178
updated = 1785731191
claimant = "name-cutter"
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator ruling 2026-08-03, verbatim: 'I'd like to cut down to just one-word names. Let's just keep a hash map of what's live, and discard/retry if we find a collision.'

The mint is yog policy (bl-50f3 ruling: yog mints, lernie stores). src/names/mod.rs mint(rng, occupied) is a pure function over an injected RNG plus a per-workspace occupied set — the collision machinery the operator describes already exists as that set. The change:

1. Names become a SINGLE word from one wordlist (today's two-word compound shape goes). Keep the wordlist large enough that live-set collisions stay rare in a normal workspace.
2. On collision with the occupied set, discard and re-sample. Bound the retry (e.g. give up and error loudly if the wordlist is exhausted against the occupied set — an unbounded loop over a small free pool is a hang). The pure-function shape (injected RNG, deterministic under test) stays.
3. Composer preview (I7) unchanged mechanically — it previews whatever mint returns.
4. If DESIGN 3.3 records the two-word shape, amend it to one word.

Independent of the lernie pin: lernie 0.0.4's creation-time uniqueness check is a second guard, not this mechanism. Verify all paths against the tree before editing.