+++
title = "the models.yaml half of a pick is inert: lernie retired the models: table, so yog writes a file nothing reads and owns the only reader of context_window"
created = 1786846408
updated = 1786936893
claimant = "Inert-d9cb"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["design", "config", "model-pick"]
+++
DESIGN §9.4's whole premise is that a role assignment and a model declaration are "two halves of one fact", enforced by lernie's cross-check: *"lernie's cross-check (`config::cross::check_roles_against_models`) refuses to load any config whose `roles.<r>.model` is not declared in the global `models.yaml`, and refuses one whose declared `provider` differs from that model's."*

That check does not exist at the pinned lernie. Verified in the pinned source, `config/cross/mod.rs`, verbatim:

> There is no roles-against-models check any more (bl-35e2): the global `models.yaml` carries no `models:` table, a role's `providers.yaml` assignment is the single home of its (provider row, model id) pointer, and id validity is brazen's fact caught at the first live model call (ARCH §4.2).

And `config/models.rs`: the harness-root `models.yaml` deserializes to one optional `adapter:` field; *"A leftover `models:` block in an operator's file is ignored on parse (serde's default for unknown keys), so existing installs load unchanged."* The per-workspace `providers.yaml` goes further and REFUSES a legacy `models:` block outright.

Consequences to work through, none of them urgent and all of them a design question rather than a bug fix:

1. The picker's first write is inert. `declare_model` composes a `models:` entry with `provider`/`model_id`/`capabilities`/`context_window`, the boundary writes it first "normatively", and the harness ignores every byte. The ordering rule it justifies ("a role naming an undeclared model bricks every step in the workspace") is no longer true either.
2. `context_window` therefore has exactly one reader: yog's own fullness figure. That is the state bl-671d hit from the other side — a declared 200000 default beside an Ollama turn the server truncates at its own default. If the number is only ever yog's denominator, its home may not be a lernie config file at all.
3. `DEFAULT_CONTEXT_WINDOW` and the two "declared default vs served" comment variants exist to make that entry honest. Whatever replaces the entry has to keep the honesty distinction (bl-848f) or lose the feature.
4. The §9.2 `models.yaml` editor and its provider gate are the same question one surface over.

The subtraction is the interesting shape: if the pointer's single home is `providers.yaml`, the picker is ONE write, not two ordered ones, and a whole ordering invariant plus a grammar half dissolve. Attack that before adding anything.