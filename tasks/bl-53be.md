+++
title = "models.yaml offers two Claude models whose provider row does not exist in brazen — dead entries, no validation"
created = 1785201365
updated = 1785218850
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

---

## CORRECTION 2026-07-27 — the evidence above is partly WRONG

I diagnosed the two Claude entries as dead because `provider: anthropic` matched
no row in `~/.config/brazen/config.toml`. That grep was the wrong instrument.
`bz --help` says plainly that `--dump-config` **omits the built-in rows**, and
brazen ships built-ins in `data/defaults.toml`. The authoritative query is
`bz --list-providers`, which reports the EFFECTIVE table:

    codex                  openai_responses      oauth2   stored
    local                  ollama_chat           none     not required
    claude-code            claude_code           none     not required
    claude-session-direct  anthropic_messages    oauth2   ambient
    anthropic              anthropic_messages    api_key  missing
    openai                 openai_chat           bearer   missing
    mistral                openai_chat           bearer   missing
    openai-responses       openai_responses      bearer   missing
    google                 google_generative_ai  api_key  stored
    ollama                 ollama_chat           none     not required

So **`anthropic` exists** — a built-in row, `anthropic_messages` protocol,
api_key auth. The entries are not unresolvable. What they are is
**uncredentialed**: `api_key  missing`. Selecting `claude-sonnet-5` fails at
auth, not at row resolution — a different failure with a different fix.

Also corrected: `google` shows `stored`, so the `google.json` credential I
flagged as an anomaly is an ordinary built-in row with a key. Not a finding.

**`claude-session-direct` is the row that already works** — `anthropic_messages`,
oauth2, credential `ambient`. A Claude model pointed at that row needs no new
credential.

## What actually remains, restated

1. **The yog-side validator.** Still real, still yog's. §9.2's `lernie_global`
   editor writes `models.yaml` with, in its own module doc, "no validator" — so
   a `provider:` naming a row brazen does not have, or one with no credential,
   is accepted silently. That is worth catching at edit time, and yog already
   runs `bz` (`RealBzRunner`) so the effective table is reachable. Validate
   against `bz --list-providers`, not against a grep of `config.toml`.
2. **What the two entries should say** is no longer a yog question. The file is
   shipped from the lernie repo (`install/models.yaml`, committed) and seeded by
   `lernie prime`. See lernie **bl-35e2**, which carries the operator's ruling
   that source should ship protocols and login URLs and no policy at all.

## Standing question for the operator

Unanswered, and now better posed: the working Anthropic row is
`claude-session-direct` (already authenticated). Repointing the two entries at
it would make them live with no new credential. Alternatively `anthropic` needs
an api_key. Or drop them. Operator's call.
