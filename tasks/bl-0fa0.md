+++
title = "text selection: double-click-drag doesn't extend selection by word boundaries — OPERATOR DISCUSSION, do not claim"
created = 1785645008
updated = 1785645625
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Operator report 2026-08-01: standard text-selection idiom — double-click to select a word, keep the button down and drag — should extend the selection word-by-word. In yog it doesn't; the drag reverts to character-level selection. Triple-click-drag → line granularity is the same family.

## Investigation 2026-08-02 (select-fixer) — yog owns none of this; upstream egui does not implement it at any version

**Verdict: findings only, no code written. This is an operator decision (see "Paths" below).**

### Who owns the selection path

Nobody in yog. Every piece of selectable text in the app is a **stock `egui::Label`**, and label text selection lives entirely inside egui:

- Conversation/transcript text: `src/transcript/render.rs` — `ui.label` / `ui.weak` / `ui.monospace` / `ui.colored_label` only.
- Same for `src/jsonview/mod.rs`, `src/inspector/`, `src/steps_view/render.rs`, `src/inboxview/render.rs`, `src/shell/navigator.rs`.
- yog never overrides `style.interaction.selectable_labels` or `multi_widget_text_select` (grep over `src/` returns nothing), so both are at the egui defaults (`true`), i.e. plain stock behaviour including cross-label drag selection.
- yog's only other text surfaces are `egui::TextEdit`s: `src/shell/input_bar.rs:91`, `src/shell/new_ws.rs:65`, `src/shell/workspace.rs:231`, `src/shell/config_edit/mod.rs:126,157,249`.

### What egui 0.29.1 actually does

`egui-0.29.1/src/text_selection/text_cursor_state.rs:99` `TextCursorState::pointer_interaction` handles the three cases as mutually exclusive branches with **no memory of granularity**:

    if response.double_clicked() {
        // Select word:
        let ccursor_range = select_word_at(text, cursor_at_pointer.ccursor);
        ...
    } else if response.triple_clicked() {
        // Select line:
        ...
    } else if response.sense.drag {
        ...
        } else if is_being_dragged {
            // Drag to select text:
            if let Some(mut cursor_range) = self.range(galley) {
                cursor_range.primary = cursor_at_pointer;
                self.set_range(Some(cursor_range));
            }

The drag branch assigns the raw character cursor under the pointer. Nothing records that the gesture began as a double- or triple-click, so the next drag frame silently downgrades the word selection to characters. For labels specifically the drag update is in `label_text_selection.rs:337` (`cursor_for`), which is the same character-granularity assignment: `Some(galley.cursor_from_pos(pointer_pos - galley_pos))`. (`label_text_selection.rs:504` even comments: *"This is where we handle start-of-drag and double-click-to-select. Actual drag-to-select happens elsewhere."*)

**So it is absent, not buggy.** Word-granularity drag has never existed in egui.

### Upstream is not ahead of us — an egui bump would NOT fix this

egui issue **#2550, "Add double-click and drag and triple-click and drag to select multiple words and to select multiple paragraphs"** (opened 2023-01-06) is **still open, with no linked branch or PR**. Confirmed against egui `master`: `pointer_interaction` there is unchanged in this respect — no granularity field, drag still moves `primary` to the raw pointer cursor. Upgrading eframe/egui buys nothing here.

### Can yog intercept it in its own layer? For labels, no.

`LabelSelectionState` is `pub` but every field is private, and its entire public surface is `load` / `store` / `has_selection` / `clear_selection` / `label_text_selection`. There is **no public way to read or set the label selection range**, so yog cannot correct the range after the fact and cannot re-snap it during a drag. There is no hook (`on_end_pass` gives no access either).

The only remaining ways to fix labels:

1. Fork/patch egui (`[patch.crates-io]` → git). **Forbidden by AGENTS.md rule 6**: sources are registry-only, a git dep re-blocks `make publish` (crates.io refuses git deps).
2. Reimplement selectable text in yog: own galley layout, hit-testing, cross-widget selection state, painting, Ctrl+C accumulation, keyboard extension, ScrollArea auto-scroll. That is a ~600-line reimplementation of an egui subsystem (`label_text_selection.rs`) and would regress drag-select-across-rows if done per-widget. Disproportionate to the irritation.

### One partial fix IS cheap — but it's in the wrong place

`TextEditState` exposes `pub cursor: TextCursorState` plus `ccursor_range()` / `set_ccursor_range()` (`egui-0.29.1/src/widgets/text_edit/state.rs:38,73,79`). So for **`TextEdit` widgets only**, yog could: record the anchor word's char range on `response.double_clicked()`, and on each subsequent drag frame union that anchor range with the live range and expand both ends to word boundaries in the `String` buffer (no galley needed — it's pure char-index arithmetic), then `set_ccursor_range`. Fully unit-testable, ~60 lines, 100%-coverable.

That would fix the composer and the config editors — **not the conversation view**, which is where the operator hit it. Shipping word-drag in the edit boxes but not the transcript makes the app inconsistent with itself, which is arguably worse than uniformly missing. Not recommended as a standalone.

### Paths (operator's call)

- **A. Upstream it (recommended).** Small, correct, benefits the transcript and the edit boxes at once, no yog inertia. The change lives in `TextCursorState`: add a granularity (`Char`/`Word`/`Line`) + anchor range, set it on double/triple click, reset on a fresh press, and in the drag branch expand `cursor_at_pointer` to the boundary in the drag direction while pinning `secondary` to the far end of the anchor unit. The label case additionally needs the anchor unit's two ccursors carried in `LabelSelectionState::selection.secondary` so the far end can be recovered when the anchor sits in another widget. Then yog inherits it at the next eframe bump. egui #2550 is the ticket; it has waited 3 years, so this is "file a PR", not "wait".
- **B. Do nothing.** Accept character granularity; close this ball as won't-fix-here.
- **C. TextEdit-only partial** (above) — only if the operator decides the composer alone is worth it.

Explicitly NOT done, per the dispatch and AGENTS.md: no egui fork, no version bump.

## HELD FOR OPERATOR DISCUSSION (2026-08-01) — do not claim, either fleet

Operator verbatim: *"file a ball for this granularity task, but don't pick it up. it's a real discussion and I don't need to distract you with it."*

The decision on the table is the Paths section above — A (upstream egui PR against #2550: granularity memory in TextCursorState), B (won't-fix), C (TextEdit-only partial). Coordinator recommendation on record: A, with B as the free fallback if the PR stalls; C skipped (fixes the composer, not the transcript where the report originated, and becomes deletion debt if A lands). No implementation until the operator closes the discussion.