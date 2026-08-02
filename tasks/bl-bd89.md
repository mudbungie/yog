+++
title = "model selector: picking gpt-5.4 writes a models.yaml row pointing at provider 'codex', which brazen does not have"
created = 1785644944
updated = 1785645177
claimant = "model-fixer"
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator report 2026-08-01: model selector "not working". Selecting/opening the picker in a live workspace painted:

> changes config/default for the whole <workspace> workspace — it governs the NEXT conversation started here; this one stays frozen at ae27cdc7
> writes models.yaml first (capabilities and context window are declared defaults — brazen publishes neither), then providers.yaml through `lernie config`
> ⚠ gpt-5.4 names provider row `codex`, which brazen's table does not have — repoint it in the Config editors (§9.2), or pick a live model here
> ⚠ unknown provider `codex`
> or run by hand: bz --list-models --provider codex --json

## Root cause (the original premise was wrong)

The `gpt-5.4 → codex` mapping is NOT a table in yog. It is the operator's own world data:
`$XDG_DATA_HOME/yog/world/lernie/models.yaml` declares `gpt-5.4: {provider: codex}` and
`world/lernie/template/providers.yaml` puts both roles on `codex`; both were laid down by `lernie prime`.
brazen's ambient `~/.config/brazen/config.toml` (edited 2026-07-31) names that row `openai-chatgpt` —
brazen 0.0.4 and 0.0.5 only ever ship `openai-chatgpt` as the built-in. Measured:
`bz --list-models --provider codex --json` → exit 78, stderr "unknown provider `codex`".

So the picker never OFFERED gpt-5.4 — that line was the role's CURRENT model, marked faulted.
The picker took its roster query row from the role's existing providers.yaml assignment, so a role
stranded on a dead row got a failing roster and ZERO candidate buttons. Nothing was clickable,
therefore nothing was written: ops.jsonl carries no `lernie config <ws> default` op for any pick
attempt (only the two fired by §8.1's tools grant right after `lernie new`). The write path was
never broken; it was never reached.

## The shape built (operator's redesign)

Two dropdowns, both sourced from brazen, plus an explicit Set button:

- **provider dropdown** — brazen's own `--list-providers` table. Defaults to the role's row while
  brazen has it, brazen's first row once brazen does not (`model_pick::default_row`), saying so.
  Last entry `add a provider…` ROUTES to the §9.1 brazen config.toml editor (the one place a row is
  authored) rather than growing a second form over the same file.
- **model dropdown** — that row's live `bz --list-models --provider X --json` roster. Last entry
  `custom model id…` reveals a free-entry field. A failed roster still offers the custom entry, so
  an unlistable provider is not a dead end either.
- **Set button** — selection chooses, the button commits, labelled with the whole pair.

Three refusals, all invariants rather than warnings (the warning already existed and was the dead end):

1. Nothing listed is unroutable — every candidate is a model brazen listed for a row brazen has.
2. `plan` refuses a row brazen lacks (`PickError::UnknownProvider`) before either file is touched.
   The §9.2 Apply gate could not cover this: it only runs when the models.yaml half needs writing.
3. `plan` refuses an id the block grammar cannot hold (`PickError::NotAnId`) — blank, or carrying
   whitespace / `:` / `#`. Only the custom entry can produce one.

Fourth hole found and closed: `declare_model` used to SKIP an already-declared id. Repointing a role
to openai-chatgpt while models.yaml still said `gpt-5.4 → codex` would have written a config lernie
hard-refuses (`models.<m>.provider` must equal `roles.<r>.provider`). It now repoints that single
`provider:` line, preserving the operator's capabilities/context_window.

## bl-662f (new-conversation failures)

Not caused by a picker write — the picker could write nothing. Same stale data one level up: the
world's `template/providers.yaml` births every new workspace on `provider: codex`, so the first
dispatch dies on `unknown provider`. This fix prevents the class going forward for an existing
workspace (a pick now always lands a routable pair, and repairs a stale models.yaml row as a side
effect) but does NOT rewrite the world template — that file is reachable from no yog surface
(§9.2 edits models.yaml + workflows/* only). Residual worth its own ball.

Files: src/model_pick/{mod,grammar/models}.rs, src/model_pick/tests/{plan,grammar}.rs,
src/shell/model_pick/{mod,select,write}.rs (select.rs new), src/shell/workspace.rs, docs/DESIGN.md §9.4 + §12.