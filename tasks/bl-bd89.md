+++
title = "model selector: picking gpt-5.4 writes a models.yaml row pointing at provider 'codex', which brazen does not have"
created = 1785644944
updated = 1785645106
claimant = "model-fixer"
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator report 2026-08-01: model selector 'not working'. Observed UI output when selecting gpt-5.4:

> changes config/default for the whole <workspace> workspace — it governs the NEXT conversation started here; this one stays frozen at ae27cdc7
> writes models.yaml first (capabilities and context window are declared defaults — brazen publishes neither), then providers.yaml through `lernie config`
> ⚠ gpt-5.4 names provider row `codex`, which brazen's table does not have — repoint it in the Config editors (§9.2), or pick a live model here
> ⚠ unknown provider `codex`
> or run by hand: bz --list-models --provider codex --json

Determine: (a) where the gpt-5.4→codex provider mapping comes from and whether it is stale data or a real gap; (b) whether the selector should refuse/hide models whose provider brazen lacks instead of writing a broken config; (c) whether this broken write is what made new-conversation creation fail (see the new-conversation bug ball). Fix so selecting a listed model yields a working config or the model is not offered.

## Environment note (2026-08-01)

All five workspaces including <workspace> were deleted later the same day (operator-authorized wipe, bl-8f17). The broken config write no longer exists on disk; reproduce in a fresh workspace born from the current template. The defect being chased is in yog's code/data, not in the wiped state.