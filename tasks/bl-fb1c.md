+++
title = "the empty-world masthead splits across two alignment axes: the wordmark is left-aligned while the tagline and name prediction beneath it are centred"
created = 1786163347
updated = 1786514129
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
QUALITY.md §1 criterion **G3** ('One grid per surface … one row is one scan line'), aggravated by **G4** at the large capture. Audited sha 4b0e75c, run /home/u/.cache/yog-drive/quality-20260807T214407Z/out.

SYMPTOM. The bootstrap placeholder paints three stacked elements that are meant to read as one masthead — the mark + 'yog' wordmark, the tagline, and the greyed name prediction — but only two of them are centred. The wordmark sits hard against the left edge of the centre panel.

MECHANISM (read, not guessed). `src/shell/bootstrap.rs` wraps all three in `ui.vertical_centered`, but `crate::theme::wordmark` (`src/theme/mark.rs:46`) is itself a `ui.horizontal(…)`. egui centres each direct child widget; a horizontal child claims the full available width, so its contents start at the left. The two `ui.weak(…)` labels below it centre normally.

WITNESS: `Q-S0-default.png` — wordmark at x≈270 (the panel's left edge), tagline centred at x≈705. `Q-S0-max.png` — the same split at 2560px wide puts roughly 900px between the wordmark and the two lines that belong to it. `Q-S0-small.png` shows it at 420x320 too, so it is not a size-specific artifact.

Also visible in the same shots and worth the fixer's eye, though NOT filed separately: the tagline ('the key and the gate') and the name prediction ('will be named growing') are consecutive `ui.weak` labels in the same size and colour with no separation, so they read as one run-on sentence — 'the key and the gate will be named growing'.

REPRODUCTION: launch yog on an empty scratch world (`XDG_DATA_HOME=<scratch>` with only `yog/world/lernie/models.yaml` + `template/providers.yaml` seeded). The placeholder is the first frame; no gesture needed.

TRIAGE ONLY — filed by the first quality audit, not fixed by it.

---

GROUNDWORK (verified 2026-08-11, Ferrule, read-only — NOT claimed, NOT worked).

The ball's mechanism read is CORRECT and still current — rare enough among these audit balls to be worth stating. `src/shell/bootstrap.rs` wraps three children in `ui.vertical_centered`, and `crate::theme::wordmark` (`src/theme/mark.rs:46-52`) is:

    pub fn wordmark(ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            draw(ui, &icon::Tints::rest());
            ui.add_space(3.0);
            ui.heading(egui::RichText::new("yog").color(GATE).strong());
        });
    }

`ui.vertical_centered` centres each child by the width that child REQUESTS. `ui.horizontal` requests the full available width, so its contents begin at the left edge; the two `ui.weak` labels request their own text width and centre normally. Hence one masthead on two axes.

FIX SHAPE: the row needs a known width before it is placed. Measure the heading galley (`egui::TextStyle::Heading.resolve(ui.style())`), add `MARK_PT` (28.0) + the 3.0 gap, and place the row with `allocate_ui_with_layout` at that exact width and `Layout::left_to_right(Align::Center)`. Then `vertical_centered` has a real width to centre and the mark travels with its own word. Keep it inside `theme::wordmark` — the ball notes the wordmark is used as the empty-workspace placeholder's identity seat, so the fix belongs to the wordmark, not to bootstrap's call site.

EVIDENCE NEEDS NO NEW MACHINERY: `paint_probe::collect` already returns `(String, Rect)` per galley — `Painted` — and `painted_of`/`painted_settled` hand back the positioned list. The ball's own witness is x-coordinates (wordmark x≈270 vs tagline x≈705), so the regression test is a direct assertion that the three galleys share a centre within a tolerance. `src/shell/acceptance/name_column.rs` is the precedent for measuring galley positions rather than asserting strings, and its reason is the same: a string assertion would pass on a tree that had deleted the element outright.

SECOND DEFECT NAMED BUT NOT FILED, from the ball body: the tagline ('the key and the gate') and the name prediction ('will be named growing') are consecutive `ui.weak` labels in the same size and colour, so they read as one run-on sentence — 'the key and the gate will be named growing'. Both are `ui.weak` in `bootstrap.rs`'s `vertical_centered` block. Whoever takes this ball is in the file already; fixing the alignment while leaving two lines that misread as one sentence would be a half-delivery.
