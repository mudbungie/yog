+++
title = "a model pick refused with 'use the Config editors': the sentence names no verb a seat can open, and a model written as a block value in providers.yaml cannot be picked at all — the pick should select the block's key and keep its options, and every refusal that names a remedy should name it as the seat's slash verb (/config)"
created = 1791951827
updated = 1791951930
claimant = "LanePick"
priority = 1
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
## Defect 1 — a tagged local model could not be picked

Cause, read from the code rather than the sentence: `model_pick::pick::is_plain_id` refused every id containing `:`, `#` or whitespace. An Ollama tag is `<name>:<tag>` (`qwen2.5:7b-instruct`), so every tagged local model was refused. The sentence said "yog writes it as a block value" meaning the four-space `    model: <id>` line, not a per-model mapping with options — there is no model-keyed block in providers.yaml.

Reframe: the question was never "is this a plain key", it is "does this id stand as a plain YAML scalar on the `model:` line, read back as itself by the anchored grammar and by litany's YAML parser". A colon with a character after it is inside a plain scalar; the grammar's reader splits once at the field name's colon. So the id is written as it stands. What truly cannot stand plain: blank, whitespace (the only way `: ` or ` #` arise), a leading YAML indicator (`- ? : , [ ] { } # & * ! | > ' " % @` and backtick), a trailing `:`. One predicate, no branch.

Tests (`src/model_pick/tests/plan.rs`): a tagged id (`qwen2.5:7b-instruct`, `hf.co/org/repo:Q4_K_M`, `a#b`) is written and read back whole; an id no row lists is picked; the narrowed refusal set (blank, whitespace, `gpt: 5`, `gpt #5`, `gpt:`, `-x`, `#x`, `@cf/x`, quoted, flow) is refused and names `/config`.

## Defect 2 — remedies name the seat's verb

| Site | Before | After |
|---|---|---|
| `model_pick/pick.rs` NotAnId | `` `{model}` is not a plain model id — yog writes it as a block value in providers.yaml; use the Config editors for anything else `` | `` `{model}` cannot be written as a plain model id — it is blank, holds whitespace, opens with a YAML indicator or ends in `:`; write the role's `model:` line yourself with /config `` |
| `model_pick/pick.rs` UnknownProvider | `… pick a live row here, or add the row to brazen's own config` | `… pick a live row here, or add the row with /config brazen` |
| `model_pick/grammar/mod.rs` (3 refusals) | `… yog edits only the block form litany writes — use the raw editor` | `… yog edits only the block form litany writes — edit it with /config` |
| `model_pick/mod.rs` strand note | `… add or rename the row in config.toml to keep it` | `… add or rename the row with /config brazen to keep it` |
| `config_edit/form.rs` provider fault | `… pick a live row, or add the row in the brazen config editor` | `… pick a live row, or add the row with /config brazen` |
| `config_edit/fault.rs` dialect remedy | `… give this role a tool-carrying row in the model picker, or author one in brazen's config.toml` | `… give this role a tool-carrying row with /model, or author one with /config brazen` |
| `config_edit/fault.rs` named row | `no provider row named {row} — add or rename it in brazen's config.toml, then …` | `no provider row named {row} — add or rename it with /config brazen, then …` |
| `config_edit/fault.rs` no row | `… the row a role names has to exist in brazen's config.toml` | `… the row a role names has to exist in brazen's table — add it with /config brazen` |
| `boundary/fleet.rs`, `boundary/monitor.rs` | `cadence.yaml: fleet/monitor: carries an inline value; edit it by hand and try again` | `… edit it with /config cadence and try again` |
| `attention/kind.rs` Refused | `was refused at the provider — sign a provider in on this workspace` | `… sign a provider in on this workspace with /login` |
| `attention/kind.rs` Truncated | `… raise the role's max_output_tokens in providers.yaml, or ask for less in one step` | `… raise the role's max_output_tokens with /config, or ask for less in one step` |
| `steps_view/wound.rs` CUT_OFF | `Raise the role's max_output_tokens in providers.yaml, or …` | `Raise the role's max_output_tokens with /config, or …` |
| `boundary/help/table/standing.rs` /fleet detail | `Add a lease_min to its cadence.yaml entry` | ``Add a lease_min to its entry with `/config cadence` `` |

Left as is, on purpose: `boundary/dispatch/enroll.rs` "remove it by hand" and the provision/help "carry … by hand" sentences — no seat verb performs those acts on the engine's disk or another box.

Docs: REMOTE §9.25 (new) states the rule — a remedy is named by the seat's slash verb, never a pane, editor or file; a file or key may be named as WHAT to change, never as the way in. No shape moved, PROTOCOL unchanged; the corpus regenerated two string values (`reply/config` fault, `reply/help` fleet detail). DESIGN §9.4's NotAnId bullet amended. `tests/design_citations.rs` registers 9.25 as a foreign key.