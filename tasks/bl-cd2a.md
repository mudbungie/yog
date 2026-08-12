+++
title = "the conversation's model row becomes the selection itself: two live dropdowns, <provider> · <model>, and nothing else on the line"
created = 1786511991
updated = 1786511992
claimant = "Disavow"
priority = 1
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator, 2026-08-11, verbatim: *"in the harmonics chat, it looks like this: `model · openai-chatgpt · gpt-5.6-sol · frozen at a06b0902`. Right there, I want essentially that whole line changed to: `<provider> - <model>`. That's it. Leave the budget stuff, that's there for other reasons."* — and, from the same intake: *"the model selection in the yog conversation window should have both dropdowns: provider and model. … those both show up when the pane is expanded. I mean when it's not."*

The ruling: the bottom settings row IS the selection. Not a sentence plus a `change…` that reveals the choice — the two dropdowns live on the row.

## What changes

1. **The row is `<provider> · <model>`, two combo boxes and nothing else.** Gone from the line: the `model ·` prefix, `frozen at <short-oid>`, the `change…` button.
2. **The combos show and write the workspace default (config branch tip)** for the `worker` role — the pair a pick actually advances. Showing the *frozen* pair instead would make every write look like a no-op: the tip moves, the freeze cannot.
3. **The freeze moves to hover** (it was `ui.weak()` text before, so this loses no weight): the row's hover states what this conversation is frozen on and that a pick governs the NEXT conversation.
4. **Drift keeps its clause and its exit** (bl-9786), conditional as today: when the governing oid and the tip differ, a weak clause names the frozen pair and the `new conversation uses the current config` affordance sits beside it. An undrifted conversation — the common case — is exactly the bare pair the operator asked for.
5. **The pane (`m`) keeps only what a row cannot hold**: the role strip, the fault line, the custom-id field, the status receipt, the `add a provider…` route. Its two dropdowns are NOT duplicated — the row's combos ARE the picker's pair control, re-scoped by the role strip while the pane is open. One pair of combos in the app, one state, one write path.
6. **The roster fires lazily**, when the model popup opens rather than on picker-open: a row that is always on screen must not fire a provider query on sight.

Both seats move together (§11 bottom settings, bl-2e18/bl-824e): the birth block's line gets the same row, minus the drift clause a not-yet-started conversation cannot have.

## Where

- `src/model_pick/header.rs` — `frozen_header`/`birth_header` (sentence composers) become the row's view-model: the pair to show, the hover, the optional drift clause. Tested there; `src/model_pick/tests/header.rs` follows.
- `src/shell/model_pick/lines.rs` — paints the row (keeps both memos: the derivation is several `git show` spawns).
- `src/shell/model_pick/select.rs` — the two combos lose their leading labels and the model combo fires the roster from inside its popup.
- `src/shell/model_pick/mod.rs` — `pane` becomes the extras block.
- DESIGN §9.4 and §11 — the model line's definition and its sample; amended in the same commit, not left to drift.