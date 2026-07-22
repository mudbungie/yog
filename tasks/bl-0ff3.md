+++
title = "world: stop nesting BRAZEN_CONFIG in phase 1 — share the ambient brazen config"
created = 1784696431
updated = 1784696431
priority = 4
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
## Problem (proven live 2026-07-21)
The composed world env overrides BRAZEN_CONFIG to <data-root>/world/brazen/config.toml — a file nothing ever creates. bz then falls back to built-in defaults (anthropic), the machine's only credential is codex OAuth, and every in-app conversation dies at step 001 with kind:auth ('no credential for this provider'). See <workspace> step 001. A live 'lernie prompt' through the world WITHOUT the override (ambient ~/.config/brazen/config.toml, codex/gpt-5.4) went green end-to-end.

## Fix (DESIGN §16.2 amendment included in this ball)
Phase 1 spawns the one host bz binary — there is no version skew for the nested config to protect against, and the provider rows are credential-adjacent (oauth endpoints/client ids for the credentials that are already deliberately shared). Move BRAZEN_CONFIG from the override set to the 'left ambient' table: yog no longer sets it; Env::brazen_config_path resolves ambient ($BRAZEN_CONFIG else $XDG_CONFIG_HOME/brazen/config.toml). Consequences to wire + document:
- src/world/mod.rs: drop the override from the composed env + Layout::brazen_config (and its seed/hatch mentions).
- fs_watcher BrazenConfig root + config_edit/brazen.rs now watch/edit the ambient file — that IS the intent (one bz, one config); say so in §9.1's margin.
- DESIGN §16.2: BRAZEN_CONFIG row moves tables with reasoning + a phase-2 note (re-nest only if the embedded-crate phase reintroduces schema skew).
- Tests: world/tests.rs, xdg tests unchanged (ambient resolution already covered).
100% coverage, make check green, 300-line caps hold.