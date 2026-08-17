+++
title = "the models: entry collapses to the id and its context_window: two columns with no reader and the §9.2 gate over one of them are gone — write less, read the same"
created = 1786937632
updated = 1786938201
claimant = "Columns3ffa"
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["design", "config", "subtraction"]
+++
bl-d9cb established that lernie 0.0.10 reads no `models:` table (its bl-35e2 — `config/models.rs` deserializes the whole file to one optional `adapter:` field) and left the block standing as yog's own table for the one fact still read out of it: `context_window`, the §5.1 #35 fullness denominator, via `grammar::context_windows`. That was the scoped subtraction and it landed. This is the loose end DESIGN §9.2 now names in as many words.

Two of the entry's four fields have no consumer at all, and one of them still has a gate over it.

**`provider:`** is read by `grammar::declared`, and `declared` is read only by `grammar::unknown_rows`, which is read only by the §9.2 Apply gate (`config_edit::lernie_global::Editor::apply`) — which judges the field. Nothing dispatches through it: the role's own `roles.<r>.provider` in `providers.yaml` is the whole pointer, and the §9.4 role marks were re-pointed onto it in bl-d9cb. So the gate refuses a draft on the strength of a field whose only reader is the refusal. `context_windows` does not consult it — the map keys on `model_id` (falling back to the entry key) and never on the row.

**`capabilities:`** has no reader in either program. `model_entry` writes `capabilities: []` and the §9.5 form offers a List control over it; nothing else in yog or lernie touches the word.

The §9.2 provider gate's own justification (bl-53be) was a shipped `models.yaml` offering two Claude models on an uncredentialed row — real then, when a role's model resolved through that declaration. It does not resolve through it now.

What to attack, in the order that keeps the operator's seat working:

1. Does the entry need `provider:` at all? If it goes, `declared`, `DeclaredModel`, `unknown_rows`, the §9.2 Apply gate, its `Rejected { unknown }` arm, the Apply hover that promises the refusal, and the schema's Provider control go with it — and the block collapses to `<model-id>: { context_window }`, which is exactly what its one reader wants. Note it also removes one of `is_unknown_row`'s four sites; DESIGN §9.2's two non-special-cases (the gate runs over every file; an empty table gates nothing) then need re-siting or deleting.
2. `capabilities: []` is pure ceremony either way.
3. Whatever survives, the §9.5 typed rows must keep a control over `context_window` — that is the operator's only way to correct the denominator, and bl-d9cb deliberately kept it rather than moving the fact to a new file.
4. Weigh against it: the block is lernie's shape, and a file the operator may already have hand-written to it. Writing less does not stop the reader tolerating the old shape (the grammar is anchored line reads, so a legacy entry keeps parsing) — say so in DESIGN if the answer is "write less, read the same".

Verify every premise against the tree before editing; bl-d9cb found DESIGN §9.4 asserting a lernie cross-check that had been retired upstream.