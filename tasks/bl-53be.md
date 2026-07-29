+++
title = "models.yaml offers two Claude models whose provider row does not exist in brazen — dead entries, no validation"
created = 1785201365
updated = 1785287751
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

---

## OPERATOR RULING 2026-07-28 (verbatim)

> are you talking about just local configs, running on my machine? we should
> _ship_ to git no actual configs. if you just mean configuring them locally?
> sure, you can re-point them.

Two consequences, both scoped to this ball:

**(a) No actual config content ships from git.** Verified against yog's tree:
yog ships NO `models.yaml` — `git ls-files` carries no `.yaml` outside
`.github/workflows`, `rules/*.yml`, `sgconfig.yml`, and yog's `make install`
installs only the binary, `assets/yog.svg`, `assets/yog.desktop`. The concrete
`models.yaml` quoted at the top of this ball is **lernie's** shipped
`install/models.yaml`, laid into yog's nested world by `lernie prime` —
`src/world/seed.rs` (`ensure_seeded`) invokes lernie's own bootstrap verb and
never reproduces its seed logic (DESIGN §14 rejection, "yog aping lernie's
seeding"). So the derived-seed reframe is **already what yog does**: the seed is
lernie's, obtained by running lernie, and yog adds nothing. Nothing to delete,
nothing to template, no new mechanism. Dropping the dead entries from the
*shipped* default is lernie bl-35e2's business, not yog's.

**(b) The live file was re-pointed** (operator-approved, outside the repo).

## The live re-point, exactly as set

File: `$XDG_DATA_HOME/yog/world/lernie/models.yaml`
(`/home/u/.local/share/yog/world/lernie/models.yaml`).

Row evidence (`~/.config/brazen/config.toml`, read-only — not modified):

- `claude-code` — `protocol = "claude_code"`, `auth = "none"`, `exec = "claude"`,
  `unsupported_body_keys = ["max_tokens","temperature","top_p","stop","output"]`.
  A CLI-exec dialect that takes alias model names (`-m sonnet`) and strips
  `stop`; `bz --list-models --provider claude-code` answers "has no models
  listing; pass --model verbatim". Wrong row for entries whose `model_id` is an
  API id and whose `capabilities` claim `stop_sequences`.
- `claude-session-direct` — `base_url = "https://api.anthropic.com"`,
  `protocol = "anthropic_messages"`, `auth = "oauth2"`,
  `ambient = { format = "claude_code", path = "~/.claude/.credentials.json" }`,
  `body_defaults = { max_tokens = 32000 }`, no `unsupported_body_keys`.
  `bz --list-providers` reports it credentialed (`ambient`).

Measured, not assumed: `bz --list-models --provider claude-session-direct`
lists `claude-sonnet-5` verbatim; `claude-haiku-4-5` is not in the listing
(only `claude-haiku-4-5-20251001`), so the bare alias was tested live —
`bz --provider claude-session-direct -m claude-haiku-4-5 "say ok"` → `ok`.
Both ids therefore resolve on that row and no `model_id` needed changing.

The whole edit is two lines:

    claude-sonnet-5:  provider: anthropic  →  provider: claude-session-direct
    claude-haiku-4-5: provider: anthropic  →  provider: claude-session-direct

`gpt-5.4 / provider: codex` untouched. No `google` row was minted (undecided);
the standing note about it is left as it stands. `~/.config/brazen/config.toml`
was not touched.

## What yog builds (the ball's own deliverable)

1. **The §9.2 Apply gate.** `models.yaml`'s `provider:` values are validated
   against brazen's effective provider table (`BzRunner::providers()`, the
   linked-brazen `--list-providers` projection — never a grep of `config.toml`,
   per the 2026-07-27 correction above). A draft naming a row brazen does not
   have is REFUSED, the same posture §9.1 takes with `bz`-rejected TOML.
2. **The picker surfaces it.** The §9.4 picker's candidate set is already a live
   `bz --list-models` query, so a dead `models.yaml` entry is never *offered* —
   what it does do is name the role's CURRENT model, read from `providers.yaml`,
   which is exactly the dead entry. That row is marked and explained.