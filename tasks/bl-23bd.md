+++
title = "the boundary speaks effort and priority: two role-tuning gestures, and the providers rows state the capability — one PROTOCOL bump"
created = 1788321305
updated = 1788321305
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
**The ask (operator, 2026-09-01).** The phone's controls row gains an effort selector (how much reasoning the worker's model calls request) and a priority checkbox (spend the provider's fast-token lane), each shown only when the selected provider states the capability. yog's half: a way to SET the two facts, and the capability readable on the rows the phone already reads.

**Where the facts live (settled upstream).** Both are role-assignment fields in the workspace's `config/default:providers.yaml` — litany's ARCH §4.3: `effort: low|medium|high` (LANDED upstream, litany bl-acba, ships in the release after 0.0.5) and `priority: true` (litany bl-f587, gated on brazen bl-fbb0's service_tier knob). Follow-the-tip means a write reaches every following conversation at its next step — no forwarding, same as `/model` today.

**GATES, stated plainly.** litany's serde ignores unknown role keys silently, so a write before the engine's litany reads the field is the bl-d9cb trap (the first write reached nothing that reads it). The `/effort` half needs the litany pin ≥ the release carrying `effort:`; the `/priority` half additionally needs litany bl-f587 published and pinned. Land effort first and priority behind its pin if the releases stagger. Also: **bl-92d3 first or together** — Action (295) and Reply (297) rest at the 300 wall, and this adds two Action variants.

**The gestures — two sibling ops, NOT a widened `/model`.** REMOTE §3: a new op is not a bump; widening `request/model`'s shape is. And a toggle must not force restating provider+model. Grammar, fixed word counts like the family:

- `/effort <role> <low|medium|high|off>` → `Action::SetEffort { workspace, role, level }` — `off` REMOVES the field (absent = none requested; no third spelling).
- `/priority <role> <on|off>` → `Action::SetPriority { workspace, role, on }` — `off` removes the line (absent and false are one fact upstream).

Both flow the `/model` path verbatim: `boundary/config.rs` handler → a pure `model_pick::plan`-class rewrite → `boundary::config::write::commit` driving `litany config` (litany stays the only lawful writer of `config/*`). The grammar (`src/model_pick/grammar/fields.rs`) needs the missing primitives: `set_field` requires the line to exist — add an upsert (insert `    <name>: <value>` into an existing role entry) and a remove; `set_entry`'s whole-entry replacement is the fallback shape if a line-primitive fights the anchored grammar. Closed vocab validated at parse (a stray level refuses with usage); no capability gate on the write — "no surface may refuse on the strength of a question that went unanswered", and the config field is always lawful.

Per the compile gate (DESIGN §8.5 "every new gesture lands as a variant first"): action.rs → codec both directions → line parse/spell → dispatch → the three address tables → help/table/standing.rs → codec surface lists. While in `standing.rs`: the `/model` detail text is stale (still describes the retired models.yaml write) — fix it.

**The read — capability rides the providers rows.** `reply/providers` rows `{name, fact, blocked}` gain `effort: bool` and `priority: bool`. Derived per-row in `src/config_edit/brazen/providers/capability.rs` as a total match over `brazen::ProtocolId` (a new dialect fails to compile), beside `tools_blocked` and with its same migration note — the upstream ask is brazen bl-50a5 (columns on --list-providers); when brazen serves them, delete the match, read the column. Today: effort true for every protocol (all six project the reasoning knob); priority true for openai_chat/openai_responses/anthropic_messages once brazen's knob lands — until the brazen pin serves the knob, priority is FALSE for every row (a capability the engine cannot yet exercise must not be advertised). Per-model nuance is deliberately not modeled: a model that rejects the level surfaces as the provider's own refusal in the seat's banner (the §9.4 caveat discipline — state, don't gate).

**ONE protocol bump.** The providers-row widening is the only shape change (new ops are new shapes at the current version, not moves). PROTOCOL 5→6 in `src/wire/hello.rs` with its changelog line; `make corpus`; surface-list entries for both new ops and the widened reply (populated + empty/absent variants per the rule); a REMOTE §9.13 subsection. Batch NOTHING else shape-changing later — clients re-vendor per bump, and the number just walked 2→5 in one cycle.

**Deliberately out.** No current-assignment read: the wire states no workspace assignment today (§13.2's "a control shows only what this device did" rests on that), and a seat wanting the file's truth has the existing ReadConfig read. No per-conversation override; no new reply kind (gestures answer with the captured litany run, `Reply::Outcome`, like `/model`).

DESIGN amendments land with the change: §8.5 config-family bullets for the two gestures, §9.4 a tuning note, §5.1 if the capability derivation earns an inventory row, module map for any new file.