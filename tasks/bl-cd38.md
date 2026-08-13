+++
title = "BLOCKED on the lernie 0.0.7 publish (lernie bl-404d is unreleased; 0.0.6 satisfies the body's '> 0.0.5' test but exports no mint) — consume lernie's mint: delete the local wordlist+draw, draw preview and fire through the crate"
created = 1785737045
updated = 1786677458
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"

[[blockers]]
id = "bl-aca4"
on = "claim"
+++
Ruled at bl-aca4 (DESIGN §3.3 amendment, landed with that ball): the mint's one home is lernie; yog is a consumer, not a second minter. Lernie-side is lernie-store bl-404d (mint moves in beside `agent_name::require_available`; every creation path mints on omission; tool schema teaches the parameter; exports `mint` + Rng trait + production RNG).

RELEASE ORDERING (prose gate, bl-5134/bl-811a pattern): do NOT claim this until the published lernie crate verifiably carries bl-404d — check crates.io for the release (> 0.0.5) and confirm the exported mint API exists in the pinned source under ~/.cargo/registry before writing a line. The --needs edge only covers the design ball; the crate release is the real gate.

The change:
- Bump `Cargo.toml` `lernie = "=0.0.5"` to the exact release carrying the mint.
- `src/names`: delete the mint half — `words.txt`, `WORDS_TXT`, `Rng`, `SplitMix64`, `mint`/`mint_from`, `MintError` — keep the §3.1 workspace-name validation (`validate`, `normalize`, `DEFAULT_NAME`, `NameError`) untouched. DESIGN §17 budget row already reworded.
- `src/start/identity.rs` (`mint_conversation`, `identity_preview`) and `src/start/prompt.rs` (`execute_prompt`): call lernie's exported mint. RNG type comes from the crate; `src/shell/clock.rs`'s entropy-seed seat stays yog's (one home for "now"), feeding the crate's seedable RNG.
- Everything operator-facing is UNCHANGED and must test so: preview before spawn (I7), seed lives exactly as long as its prediction (bl-28ba acceptance drive `acceptance/mint_seed`), fire still passes `--name` and returns the minted name (the §3.4 focus-claim handle). Yog never omits --name.
- Occupied set stays yog's one derivation (ladder name facts incl. the legacy goal-stamp rung — a lawful superset of lernie's living-names scan; see §3.3 "Preview parity").
- Exhaustion still lands the `["yog-step","mint"]` ops row (§4.2) — map the crate's exhausted error into `StartError` as today.

Verify premises against the tree before editing (ball-bodies-drift rule).

---

GATE UNSATISFIED at 2026-08-11 — verified while working bl-6654 (Girder). DO NOT CLAIM YET.

This ball's written gate reads "check crates.io for the release (> 0.0.5)". The crates.io index now lists lernie 0.0.6, so that phrasing reads as satisfied — but it is NOT satisfied in substance. Published 0.0.6 does not carry bl-404d:

- `~/.cargo/registry/src/index.crates.io-*/lernie-0.0.6/src/` has no `mint` module (`ls src/` = archive, bin, cmd, config, e2e, harness_root.rs, install, install.rs, lib.rs, name.rs, prompt, provider, skill, skill.rs, template, test_support.rs, workspace, workspace.rs) and `find` turns up no `.txt` wordlist. There is no `lernie::mint`, no exported `Rng` trait, no `SplitMix64`, no `MintError` to call.
- bl-404d is written but unreleased: ~/dev/lernie commit 9f74b8b "the mint moves in: every creation path auto-mints a one-word name on omission; the dispatch parameter is taught [bl-404d]", CHANGELOG entry under `[Unreleased]`, no tag contains it, and lernie `main` is 19 commits ahead of `origin/main` (unpushed, so no publish is even in flight).

The real gate is therefore **lernie 0.0.7 published**, and the same release also carries bl-d0b4 (`--cwd`), which is bl-6654's gate. Both balls should take one pin bump to 0.0.7 together rather than racing two bumps of the same line.

Suggested amendment to this body: replace "> 0.0.5" with "a published lernie whose vendored source under ~/.cargo/registry exports `lernie::mint`" — a version-number comparison was the wrong test, since 0.0.6 shipped between the filing and the mechanism.

---

GATE (2026-08-13 publication follow-up, item 3): do NOT consume lernie's current mint corpus unchanged. It is the same EFF-derived 7,395-word list — CC BY 4.0 data inside an MIT-only package, and it mints hostile identities (carnage, chokehold, cruelty, deceit, depraved, despair, evil, hate, humiliate, stench, threaten, traitor, trash, wrath; 'humiliate' and 'wrath' have both been minted as real names in drive evidence). Consuming it moves neither the license nor the reputation problem into lernie — it just duplicates it. lernie bl-b59c replaces that corpus with a clean-room, independently authored neutral allowlist; this ball consumes the mint only after bl-b59c lands.

---

UNGATED 2026-08-13: lernie bl-b59c landed as lernie 4e3208a — the EFF-derived corpus is gone, replaced by a 541-word clean-room list with an approval pin (len + FNV-1a digest) and a semantic-safety test. The consume surface is unchanged: lernie::mint::mint(rng: &dyn Rng, occupied: &HashSet<String>) -> Result<String, MintError>, with lernie::mint::{Rng, SplitMix64, MintError} alongside (SplitMix64::from_entropy() or ::from_seed(u64)); the wordlist stays behind the function per lernie docs/ARCHITECTURE.md 3.4. STILL BLOCKED on a lernie release: 4e3208a is unpublished and lernie 0.0.7 on crates.io still carries the old corpus. When this ball lands, yog's own src/names/words.txt must be DELETED, not merely bypassed — the CC BY 4.0 header rides in yog's copy too and lernie's fix does not reach it.
