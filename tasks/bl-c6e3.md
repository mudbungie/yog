+++
title = "brazen bl-6feb: providers reply is now the routing TABLE; add the sign-in CATALOG read (bare bz --login) and stop refusing unnamed built-ins"
created = 1792728816
updated = 1792729112
claimant = "Gelded-yog"
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["integration"]
+++
## Upstream change (brazen bl-6feb, specs/config.md §3.2)
`bz --list-providers` now prints only the routing TABLE: rows the operator named in config + built-in presets a credential resolves for (stored/ambient/inline). An untouched preset (anthropic/openai/mistral/openai-responses with no key; ollama/claude-code unless named) is absent. `credential = missing` appears only on a row the operator named. Zero-config routing walks the table; an EMPTY table is exit 78 'no provider resolved: …' (was 77 on anthropic). Explicit `--provider X` still resolves any preset (the catalog). NEW: `bz --login` with no `--provider` prints the sign-in catalog — every row carrying an oauth block, preset or not — in the --list-providers shapes (text / --json {providers:[…]}), exit 0, no network.

## yog work
1. src/config_edit/brazen/effects.rs:79 `provider_rows(capture(["--list-providers","--json"]))` — now the table. Pass it through as the `providers` reply (the picker's list); seats delete their own credential filters (lernie bl-526d).
2. Add a CATALOG read for the login pane: capture `["--login","--json"]` (no --provider) → same row shape → a new op or a field on `providers`. Bump PROTOCOL; re-vendor corpus into lernie + yog-android.
3. src/model_pick/grammar/rows.rs:30-36 `is_unknown_row`: it treats the listing as 'built-ins included' and would now refuse a `model` write naming an unnamed preset (e.g. openai with no key yet). Explicit naming resolves the catalog upstream — decide: accept catalog names (resolve against catalog = providers ∪ login catalog ∪ known preset names), or refuse with 'sign in / add a key first'.
4. src/monitor/check.rs:116-119 passes --provider only when watch.provider is set; otherwise prefix routing can now hit 78 on an empty table. Surface that 78 text or always pass --provider.
5. A role-less workspace reaching zero-config bz routing now gets 78 'no provider resolved' instead of 77 — check litany/engine sentences (bl-8155 overlaps).
6. help text (corpus/answers/help.json:387 'brazen's provider rows … built-ins included') goes stale.