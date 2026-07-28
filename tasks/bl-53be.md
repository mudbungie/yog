+++
title = "models.yaml offers two Claude models whose provider row does not exist in brazen — dead entries, no validation"
created = 1785201365
updated = 1785201365
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["reliability"]
+++
Found during the 2026-07-27 UX-testing session while answering "how do I change
the model".

## The inconsistency

The yog world's `models.yaml`
(`$XDG_DATA_HOME/yog/world/lernie/models.yaml`) defines three models:

    models:
      claude-sonnet-5:
        provider: anthropic          ← no such brazen provider row
      claude-haiku-4-5:
        provider: anthropic          ← no such brazen provider row
      gpt-5.4:
        provider: codex              ← exists

Its own header states the contract:

> `provider:` on each model is a brazen provider-row NAME (§4.1) — endpoints,
> auth, and wire dialects live in brazen's own config
> (`~/.config/brazen/config.toml`) …

Brazen's config on this box has four rows, and `anthropic` is not among them:

    $ grep -n "^name" ~/.config/brazen/config.toml
    5:name       = "codex"
    44:name     = "local"
    57:name = "claude-code"
    66:name = "claude-session-direct"

So both Claude entries are dead. Selecting either would fail at fire, and the
operator has exactly one working model without knowing it.

(Credentials present: `codex.json`, `google.json` under
`~/.local/share/brazen/credentials/` — a `google` row has creds but no row
either, worth a look while you are in here.)

## Why it is yog's problem

yog's §9.2 `lernie_global` editor is a write surface for exactly this file, and
the module header says so plainly:

    //! [`lernie_global`] (§9.2, raw `models.yaml`/`workflows/*.yaml`
    //! with no validator)

"No validator" is the defect. yog already runs `bz` (`RealBzRunner`,
`src/config_edit/brazen`) and already has the brazen config surface (§9.1), so
the provider-row set is reachable. A `models.yaml` whose `provider:` names a
row brazen does not have is checkable at edit time and is currently not
checked.

## Ask

1. Validate `provider:` against brazen's actual rows in the §9.2 editor —
   refuse or warn on Apply, the way §9.1 gates raw TOML through `bz`.
2. Decide what the *shipped* `models.yaml` should say. Either drop the two dead
   entries, or point them at a row that exists (`claude-code` /
   `claude-session-direct`) — check with the operator which, since it is his auth. Note
   `make install` "lays this down only if no models.yaml already exists at the
   config root", so a fix to the shipped default does not repair an existing
   install; say in the ball report whether the live file needs a manual touch.
3. Surface it: an unusable model must not be offered. bl-5426 (the model
   picker) reads this file for its candidate set and depends on this landing
   first.

## Acceptance

An edit that sets `provider:` to a non-existent brazen row is refused (or
loudly warned) by the §9.2 editor, with a test.