+++
title = "Wordlist curation + bl identity charset check (gates Z1)"
created = 1784523881
updated = 1784524026
claimant = "Budgetary"
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Diligence for Z1 (bl-9769). Both questions answered empirically; wordlist curated and landed.

## Finding 1 — bl and hyphenated identities: YES, and bl validates nothing at all

Probed against a throwaway repo (bl prime / create / claim / show --json) with
`--as` values `a/b`, `..`, `with space`, `unknown`, `UPPER-case`, `x'y`,
`<workspace>`. **Every one was accepted and round-tripped byte-identical into
the claimant field.** bl applies no charset validation, no length limit, no
normalization. Hyphens are safe; so is everything else, which is the real
finding — the burden of path-safety is entirely yog's.

**Correction to this ball's premise:** the `<id>-<claimant>` bl-delivery
worktree-path variant **does not exist** in the installed bl-delivery. The
observed path is `<id>` only:

    /home/u/.local/state/balls/plugins/bl-delivery/<mirrored-project-path>/bl-3c16

No `<id>-<claimant>` form appears in `bl conf` or in the binary's strings, and
the delivery branch is `work/<id>`. So the claimant does NOT currently ride into
any path. Path-safety is still required — but because of §3.1, not bl: the name
IS the workspace dir leaf (`$XDG_DATA_HOME/yog/workspaces/<name>/`). Same
constraint, different justification; if bl later adds the variant, yog is
already compliant.

## Finding 2 — the `unknown` / `$USER` collision is mostly structural

bl's `--as` fallbacks are `$USER`, then the literal `unknown` (`bl claim
--skill`). A minted name is always two hyphenated words, so it can **never** be
equal to a single-token identity like `unknown` or a typical `$USER`. The
residual risk is narrow and real: a **hyphenated** human identity — `jean-luc`,
`mary-jane`, a hyphenated login. That is exactly why no single word may be a
plausible human name; excluding the literal `unknown` is belt-and-braces.

## Deliverable — src/names/words.txt (landed here, 7395 words)

- **Source:** EFF's Long Wordlist (`eff_large_wordlist.txt`, 7776 words),
  published 2016-07-18 with "Deep Dive: EFF's New Wordlists for Random
  Passphrases". https://www.eff.org/files/2016/07/18/eff_large_wordlist.txt
- **License:** CC BY 4.0 International, per https://www.eff.org/copyright
  (fetched and read, 2026-07-19 — *not* the CC BY 3.0 US the list is often cited
  under). Attribution to the Electronic Frontier Foundation lives in the file's
  header so it rides into the binary via `include_str!`.
- **Curation, upstream minus:** (1) the 4 already-hyphenated entries
  (`drop-down`, `felt-tip`, `t-shirt`, `yo-yo`) — they would yield a
  three-segment name; (2) 365 words that are a given name or surname,
  case-folded, per the first-names + surnames corpora at
  github.com/dominictarr/random-name (used only as a filter, never
  redistributed, so its terms do not ride into the artifact); (3) the literal
  `unknown` plus 11 common system/service usernames.
- **Verified:** 7395 words, all `^[a-z]{3,9}$`, sorted, unique, no `unknown`.
  54,678,630 ordered pairs. A 48-name adversarial probe (amber, rose, ivy,
  hazel, jean, luc, mary, jane, luna, willow, …) leaves exactly one survivor,
  `crimson` — not a plausible human identity.

Constraints restated in bl-9769 as five numbered items mod.rs must not weaken.

## Also amended

DESIGN §3.1: the illustrative path `~/.local/share/yog/workspaces/<workspace>/`
was **not mintable** — `velvet` is filtered as a surname and `marmot` was never
in the EFF list. Changed to `cobalt-gecko` (both present) and added the wordlist
invariant to the "The name" bullet, so the architecture authority states the
property rather than leaving it only in a data-file header.